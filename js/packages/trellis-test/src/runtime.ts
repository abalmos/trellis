import type { ConsumerInfo } from "@nats-io/jetstream";
import { Kvm } from "@nats-io/kv";
import { connect, credsAuthenticator } from "@nats-io/transport-deno";
import {
  type ClientAuthContinuation,
  type ClientAuthRequiredContext,
  type ClientOpts,
  createAuth,
  TrellisClient,
} from "@qlever-llc/trellis";
import {
  CONTRACT as trellisJobsContract,
  CONTRACT_DIGEST as trellisJobsContractDigest,
} from "@qlever-llc/trellis/sdk/jobs/manifest";
import { recordTrellisDuration as recordOpenTelemetryDuration } from "@qlever-llc/trellis/telemetry";
import { dirname, join } from "@std/path";

import { TrellisTestAdminAutomation } from "./admin_client.ts";
import type { LocalNatsBootstrapManifest } from "./nats_bootstrap.ts";
import { aliasCaseScopedRuntime } from "./integration/names.ts";
import {
  removeStaleMarkedDirectories,
  writeTrellisTestOwnerMarker,
} from "./cleanup.ts";
import {
  buildControlPlaneConfig,
  generateSessionSeed,
  reserveLocalPort,
  writeTrellisConfig,
} from "./control_plane_config.ts";
import { TrellisControlPlaneSqlite } from "./control_plane_sqlite.ts";
import {
  startTrellisTestEventCapture,
  type TrellisTestEventAction,
  TrellisTestEventCapture,
  type TrellisTestEventCaptureOptions,
  type TrellisTestEventSourceContract,
} from "./event_capture.ts";
import { NatsTestContainer } from "./nats_container.ts";
import {
  recordTrellisTestDuration,
  recordTrellisTestProcessStart,
} from "./integration/metrics.ts";

function recordTrellisDuration(
  name: Parameters<typeof recordOpenTelemetryDuration>[0],
  durationMs: number,
  attributes?: Parameters<typeof recordOpenTelemetryDuration>[2],
): void {
  recordOpenTelemetryDuration(name, durationMs, attributes);
  void recordTrellisTestDuration(name, durationMs, attributes);
}
import type {
  JetStreamAckObserver,
  NatsMessageObserver,
} from "./nats_container.ts";
import { sqliteMemoryUrl as sqliteMemoryUrlHelper } from "./temp.ts";
import {
  startTrellisProcess,
  type TrellisProcessHandle,
} from "./trellis_process.ts";
import type {
  TrellisTestAuthorityPlanClassification,
  TrellisTestClientAuth,
  TrellisTestClientContract,
  TrellisTestClientKey,
  TrellisTestConnectedClient,
  TrellisTestContractApproval,
  TrellisTestContractLike,
  TrellisTestControlPlane,
  TrellisTestRawAuthConnectionPresence,
  TrellisTestRawStateEntry,
  TrellisTestRuntimeStartOptions,
  TrellisTestServiceKey,
  WaitForOptions,
} from "./types.ts";
import { waitFor as waitForHelper } from "./wait.ts";

type ConnectedClient = { connection: { close(): Promise<void> } };
type EventCapture = { stop(): Promise<void> };
type ServiceProcessHandle = { stop(): Promise<void> };

type RuntimeTimeouts = {
  startupMs: number;
  reconciliationMs: number;
  waitForMs: number;
  shutdownMs: number;
};

async function startServiceProcess(args: {
  command: {
    cmd: string;
    args: readonly string[];
    env?: Record<string, string>;
    cwd?: string;
  };
  trellisUrl: string;
  key: TrellisTestServiceKey;
  jobsDbPath: string;
  mode: "owner" | "rpc-only";
}): Promise<ServiceProcessHandle> {
  const child = new Deno.Command(args.command.cmd, {
    args: Array.from(args.command.args),
    cwd: args.command.cwd,
    env: {
      ...args.command.env,
      TRELLIS_URL: args.trellisUrl,
      SESSION_KEY_SEED_BASE64URL: args.key.sessionSeed,
      PROVISIONED_IDENTITY_SEED_BASE64URL: args.key.seed,
      TRELLIS_DEPLOYMENT_ID: args.key.deploymentId,
      TRELLIS_INSTANCE_ID: args.key.instanceId,
      TRELLIS_PARTICIPANT_ID: args.key.participantId,
      TRELLIS_PARTICIPANT_DIGEST: args.key.participantArtifactDigest,
      TRELLIS_PARTICIPANT_NEEDS_DIGEST: args.key.participantNeedsDigest,
      TRELLIS_AUTHORIZATION_CONTEXT_FILE: `${args.jobsDbPath}.context.json`,
      TRELLIS_JOBS_DB_PATH: args.jobsDbPath,
      TRELLIS_JOBS_MODE: args.mode,
      TRELLIS_TIMEOUT_MS: "30000",
      NO_COLOR: "1",
    },
    stdin: "null",
    stdout: "inherit",
    stderr: "inherit",
  }).spawn();
  await recordTrellisTestProcessStart("jobs", String(child.pid));
  const status = child.status;
  let exited: Deno.CommandStatus | undefined;
  status.then((value) => exited = value);
  await waitForHelper(async () => {
    if (exited !== undefined) {
      throw new Error(
        `Jobs owner exited during startup with status ${exited.code}`,
      );
    }
    try {
      await Deno.stat(args.jobsDbPath);
      return true;
    } catch (error) {
      if (error instanceof Deno.errors.NotFound) return false;
      throw error;
    }
  }, { timeoutMs: 120_000 });
  let stopped = false;
  return {
    async stop() {
      if (stopped) return;
      stopped = true;
      try {
        child.kill("SIGTERM");
      } catch {
        return;
      }
      const timeout = new Promise<undefined>((resolve) =>
        setTimeout(() => resolve(undefined), 5_000)
      );
      const exited = await Promise.race([status, timeout]);
      if (exited === undefined) {
        try {
          child.kill("SIGKILL");
        } catch {
          // already exited
        }
        await status.catch(() => undefined);
      }
    },
  };
}

const WORKDIR_PREFIX = "trellis-test-";
const WORKDIR_OWNER_MARKER = ".trellis-test-owner";

/** Runs an isolated Trellis control plane and NATS server for integration tests. */
export class TrellisTestRuntime implements AsyncDisposable {
  readonly trellisUrl: string;
  readonly natsUrl: string;
  readonly workdir: string;
  readonly deployments: {
    create(args: { id?: string; mutableDev?: boolean }): Promise<void>;
    reconcile(deployment: string): Promise<void>;
    waitReady(deployment: string): Promise<void>;
  };
  readonly contracts: {
    approve(
      args: {
        deployment?: string;
        contract: TrellisTestContractLike;
        allowPlanClassifications?:
          readonly TrellisTestAuthorityPlanClassification[];
      },
    ): Promise<TrellisTestContractApproval>;
  };
  readonly services: {
    createInstance(args: {
      deployment?: string;
      name: string;
      contract: TrellisTestContractLike;
      sessionKeySeed?: string;
    }): Promise<TrellisTestServiceKey>;
    provisionInstanceOnly(args: {
      deployment?: string;
      sessionKeySeed?: string;
    }): Promise<{ seed: string; sessionKey: string }>;
  };
  readonly authority: {
    readonly plans: {
      list(args: {
        deploymentId?: string;
        state?: "pending" | "accepted" | "rejected" | "superseded" | "expired";
        limit?: number;
        cursor?: string;
      }): Promise<{ entries: unknown[]; nextCursor: string | null }>;
      reject(
        args: { planId: string; reason?: string },
      ): Promise<unknown>;
    };
    acceptUpdate(
      args: { planId: string; expectedDesiredVersion?: number },
    ): Promise<unknown>;
    acceptMigration(args: {
      planId: string;
      acknowledgement: string;
      expectedDesiredVersion?: number;
    }): Promise<unknown>;
  };
  readonly controlPlane?: TrellisTestControlPlane;
  #controlPlane: TrellisProcessHandle | undefined;
  #jobsAdmin: ServiceProcessHandle | undefined;
  #nats: NatsTestContainer;
  #admin: TrellisTestAdminAutomation;
  #configPath: string | undefined;
  #trellisOptions: TrellisTestRuntimeStartOptions["trellis"] | undefined;
  #keepWorkdir: boolean;
  #ownsWorkdir: boolean;
  #deployment: string;
  #timeouts: RuntimeTimeouts;
  #clients = new Set<ConnectedClient>();
  #captures = new Set<EventCapture>();
  #stopped = false;

  private constructor(args: {
    trellisUrl: string;
    workdir: string;
    deployment: string;
    keepWorkdir: boolean;
    timeouts: RuntimeTimeouts;
    configPath?: string;
    controlPlaneSqlitePath?: string;
    trellisOptions?: TrellisTestRuntimeStartOptions["trellis"];
    nats: NatsTestContainer;
    controlPlane?: TrellisProcessHandle;
    jobsAdmin?: ServiceProcessHandle;
    admin: TrellisTestAdminAutomation;
    ownsWorkdir?: boolean;
  }) {
    this.trellisUrl = args.trellisUrl;
    this.natsUrl = args.nats.natsUrl;
    this.workdir = args.workdir;
    this.#deployment = args.deployment;
    this.#keepWorkdir = args.keepWorkdir;
    this.#ownsWorkdir = args.ownsWorkdir ?? true;
    this.#timeouts = args.timeouts;
    this.#nats = args.nats;
    this.#controlPlane = args.controlPlane;
    this.#jobsAdmin = args.jobsAdmin;
    this.#configPath = args.configPath;
    this.#trellisOptions = args.trellisOptions;
    this.#admin = args.admin;
    if (args.controlPlaneSqlitePath !== undefined) {
      this.controlPlane = {
        sqlite: new TrellisControlPlaneSqlite(args.controlPlaneSqlitePath),
      };
    }
    this.deployments = {
      create: ({ id, mutableDev }) =>
        this.#admin.createDeployment({
          deployment: id ?? this.#deployment,
          mutableDev,
        }),
      reconcile: (deployment) => this.#admin.reconcile(deployment),
      waitReady: (deployment) => this.#admin.waitReady(deployment),
    };
    this.contracts = {
      approve: ({ deployment, contract, allowPlanClassifications }) =>
        this.#admin.approveContract({
          deployment: deployment ?? this.#deployment,
          contract,
          allowPlanClassifications,
        }),
    };
    this.services = {
      createInstance: ({ deployment, contract, sessionKeySeed }) =>
        this.#admin.provisionServiceInstance({
          deployment: deployment ?? this.#deployment,
          contract,
          sessionKeySeed,
        }),
      provisionInstanceOnly: ({ deployment, sessionKeySeed }) =>
        this.#admin.provisionServiceInstanceOnly({
          deployment: deployment ?? this.#deployment,
          sessionKeySeed,
        }),
    };
    this.authority = {
      plans: {
        list: (args) => this.#admin.listAuthorityPlans(args),
        reject: (args) => this.#admin.rejectAuthorityPlan(args),
      },
      acceptUpdate: (args) => this.#admin.acceptAuthorityUpdate(args),
      acceptMigration: (args) => this.#admin.acceptAuthorityMigration(args),
    };
  }

  /** Starts an isolated Trellis test runtime. */
  static async start(
    options: TrellisTestRuntimeStartOptions,
  ): Promise<TrellisTestRuntime> {
    if (options?.trellis?.command === undefined) {
      throw new Error("TrellisTestRuntime.start requires trellis.command");
    }
    const workdir = await Deno.makeTempDir({ prefix: WORKDIR_PREFIX });
    await writeTrellisTestOwnerMarker(workdir, WORKDIR_OWNER_MARKER);
    await removeStaleMarkedDirectories({
      parent: dirname(workdir),
      prefix: WORKDIR_PREFIX,
      markerName: WORKDIR_OWNER_MARKER,
    });
    let nats: NatsTestContainer | undefined;
    let controlPlane: TrellisProcessHandle | undefined;
    let jobsAdmin: ServiceProcessHandle | undefined;
    try {
      const timeouts = {
        startupMs: options.timeouts?.startupMs ?? 30_000,
        reconciliationMs: options.timeouts?.reconciliationMs ?? 5_000,
        waitForMs: options.timeouts?.waitForMs ?? 5_000,
        shutdownMs: options.timeouts?.shutdownMs ?? 5_000,
      };
      await Deno.mkdir(join(workdir, "trellis"), { recursive: true });
      let sharedManifest = typeof options.nats === "object"
        ? structuredClone(options.nats.manifest)
        : undefined;
      if (typeof options.nats === "object" && sharedManifest !== undefined) {
        const sharedNatsDir = join(options.nats.workdir, "nats");
        const localNatsDir = join(workdir, "nats");
        await Deno.mkdir(join(localNatsDir, "creds"), { recursive: true });
        await Deno.mkdir(join(localNatsDir, "secrets"), { recursive: true });
        const copies = [
          [sharedManifest.paths.creds.systemService, "creds/system.creds"],
          [sharedManifest.paths.creds.authService, "creds/auth-auth.creds"],
          [
            sharedManifest.paths.creds.trellisService,
            "creds/trellis-auth.creds",
          ],
          [
            sharedManifest.paths.secrets.authIssuerSigning,
            "secrets/auth-issuer-signing.seed",
          ],
          [
            sharedManifest.paths.secrets.authTargetSigning,
            "secrets/auth-target-signing.seed",
          ],
          [
            sharedManifest.paths.secrets.authCalloutXKey,
            "secrets/auth-sx.seed",
          ],
        ] as const;
        await Promise.all(
          copies.map(([source, target]) =>
            Deno.copyFile(
              join(sharedNatsDir, source),
              join(localNatsDir, target),
            )
          ),
        );
        sharedManifest.paths.creds = {
          systemService: "creds/system.creds",
          authService: "creds/auth-auth.creds",
          trellisService: "creds/trellis-auth.creds",
        };
        sharedManifest.paths.secrets = {
          authIssuerSigning: "secrets/auth-issuer-signing.seed",
          authTargetSigning: "secrets/auth-target-signing.seed",
          authCalloutXKey: "secrets/auth-sx.seed",
        };
      }
      nats = typeof options.nats === "object"
        ? await NatsTestContainer.attach({
          ...options.nats,
          workdir,
          manifest: sharedManifest!,
        })
        : await NatsTestContainer.start(workdir, {
          startupMs: timeouts.startupMs,
        });
      const port = reserveLocalPort();
      const trellisUrl = `http://127.0.0.1:${port}`;
      const config = buildControlPlaneConfig({
        workdir,
        natsUrl: nats.natsUrl,
        websocketUrl: nats.websocketUrl,
        manifest: sharedManifest ?? nats.manifest,
        port,
        oauthProviders: options.oauthProviders,
        failOnceHooks: options.failOnceHooks,
      });
      const configPath = await writeTrellisConfig({ workdir, config });
      const deployment = options.deployment ?? "test";
      const adminPassword = options.adminPassword ??
        `trellis-test-${generateSessionSeed()}`;
      const startedControlPlane = await startTrellisProcess({
        trellisUrl,
        configPath,
        options: options.trellis,
        startupTimeoutMs: timeouts.startupMs,
        shutdownTimeoutMs: timeouts.shutdownMs,
      });
      controlPlane = startedControlPlane;
      const admin = new TrellisTestAdminAutomation({
        trellisUrl: startedControlPlane.trellisUrl,
        adminPassword,
        defaultDeployment: deployment,
        defaultMutableDev: options.trellis.mutableDev ?? true,
        reconciliationMs: timeouts.reconciliationMs,
        autoAccept: options.authority?.autoAccept ?? ["initial", "update"],
        getBootstrapUrl: () =>
          startedControlPlane.waitForBootstrapUrl(timeouts.startupMs),
      });
      if (options.jobsAdmin !== undefined) {
        const key = await admin.registerService({
          deployment: options.jobsAdmin.deployment ?? "trellis-service-jobs",
          contract: {
            CONTRACT: trellisJobsContract,
            CONTRACT_DIGEST: trellisJobsContractDigest,
          },
        });
        const jobsDir = join(workdir, "service-jobs");
        await Deno.mkdir(jobsDir, { recursive: true });
        jobsAdmin = await startServiceProcess({
          command: options.jobsAdmin.command,
          trellisUrl: startedControlPlane.trellisUrl,
          key,
          jobsDbPath: join(jobsDir, "jobs.sqlite"),
          mode: options.jobsAdmin.mode ?? "owner",
        });
      }
      return new TrellisTestRuntime({
        trellisUrl: startedControlPlane.trellisUrl,
        workdir,
        deployment,
        keepWorkdir: options.keepWorkdir ?? false,
        timeouts,
        configPath,
        controlPlaneSqlitePath: config.storage.dbPath,
        trellisOptions: options.trellis,
        nats,
        controlPlane: startedControlPlane,
        jobsAdmin,
        admin,
      });
    } catch (error) {
      await jobsAdmin?.stop().catch(() => undefined);
      await controlPlane?.stop().catch(() => undefined);
      await nats?.stop().catch(() => undefined);
      if (!options.keepWorkdir) {
        await Deno.remove(workdir, { recursive: true }).catch(() => undefined);
      }
      throw error;
    }
  }

  /** Attaches worker-local clients and admin automation to a shared Trellis host. */
  static async attach(args: {
    trellisUrl: string;
    natsUrl: string;
    websocketUrl: string;
    workdir: string;
    manifest: LocalNatsBootstrapManifest;
    adminPassword: string;
    adminRpcProxy: { url: string; token: string };
    deployment: string;
    timeouts?: TrellisTestRuntimeStartOptions["timeouts"];
  }): Promise<TrellisTestRuntime> {
    const timeouts = {
      startupMs: args.timeouts?.startupMs ?? 30_000,
      reconciliationMs: args.timeouts?.reconciliationMs ?? 5_000,
      waitForMs: args.timeouts?.waitForMs ?? 5_000,
      shutdownMs: args.timeouts?.shutdownMs ?? 5_000,
    };
    const nats = await NatsTestContainer.attach({
      workdir: args.workdir,
      natsUrl: args.natsUrl,
      websocketUrl: args.websocketUrl,
      manifest: args.manifest,
    });
    const admin = new TrellisTestAdminAutomation({
      trellisUrl: args.trellisUrl,
      adminPassword: args.adminPassword,
      defaultDeployment: args.deployment,
      defaultMutableDev: true,
      reconciliationMs: timeouts.reconciliationMs,
      autoAccept: ["initial", "update"],
      getBootstrapUrl: () =>
        Promise.reject(new Error("shared host is already bootstrapped")),
      bootstrapComplete: true,
      rpcProxy: args.adminRpcProxy,
    });
    return new TrellisTestRuntime({
      trellisUrl: args.trellisUrl,
      workdir: args.workdir,
      deployment: args.deployment,
      keepWorkdir: true,
      ownsWorkdir: false,
      timeouts,
      nats,
      admin,
    });
  }

  /** @internal Forwards one low-level Auth RPC over the host admin session. */
  callAdminRpc(method: string, input: unknown): Promise<unknown> {
    return this.#admin.callAdminRpc(method, input);
  }

  /** Registers a service contract and creates a service instance key. */
  async registerService(args: {
    name: string;
    contract: TrellisTestContractLike;
    deployment?: string;
    sessionKeySeed?: string;
  }): Promise<TrellisTestServiceKey> {
    return await this.#admin.registerService({
      deployment: args.deployment ?? this.#deployment,
      contract: args.contract,
      sessionKeySeed: args.sessionKeySeed,
    });
  }

  /** Creates app/client session-key material for public `TrellisClient.connect` calls. */
  async registerClient(args: {
    name: string;
    contract: TrellisTestClientContract;
    sessionKeySeed?: string;
  }): Promise<TrellisTestClientKey> {
    const approved = await this.#admin.approveContract({
      deployment: `${this.#deployment}.client.${args.name}`,
      contract: args.contract,
    });
    const seed = args.sessionKeySeed ?? generateSessionSeed();
    const auth = await createAuth({ sessionKeySeed: seed });
    return {
      seed,
      sessionKey: auth.sessionKey,
      participantId: approved.participantId,
      participantArtifactDigest: approved.participantDigest,
      participantNeedsDigest: approved.participantNeedsDigest,
    };
  }

  /**
   * Returns auth options and admin-backed auth continuation for a registered
   * app/client participant. Spread the result into `TrellisClient.connect(...)`.
   */
  clientAuth(key: TrellisTestClientKey): TrellisTestClientAuth {
    return {
      auth: {
        mode: "session_key",
        authorizationContextEphemeral: true,
        sessionKeySeed: key.seed,
        redirectTo: `${this.trellisUrl}/_trellis/test/client-auth`,
      },
      onAuthRequired: (ctx) => this.#admin.completeClientAuth(ctx),
    };
  }

  /**
   * Completes a test app/client auth flow through runtime admin automation.
   * Used by the parallel integration runner coordinator to proxy auth flows
   * from worker processes.
   */
  async completeClientAuth(
    ctx: ClientAuthRequiredContext,
  ): Promise<ClientAuthContinuation> {
    return await this.#admin.completeClientAuth(ctx);
  }

  /** Connects an app/client participant through the public generated client surface. */
  async connectClient<
    TContract extends TrellisTestClientContract,
  >(
    args: ClientOpts & {
      name: string;
      contract: TContract;
      sessionKeySeed?: string;
    },
  ): Promise<TrellisTestConnectedClient<TContract>> {
    const startedAt = performance.now();
    const key = await this.registerClient(args);
    const auth = this.clientAuth(key);
    const client = await TrellisClient.connect({
      ...args,
      trellisUrl: this.trellisUrl,
      participant: {
        id: key.participantId,
        artifactDigest: key.participantArtifactDigest,
        needsDigest: key.participantNeedsDigest,
      },
      auth: auth.auth,
      onAuthRequired: auth.onAuthRequired,
    }).orThrow();
    this.#clients.add(client);
    recordTrellisDuration(
      "trellis.connect.duration",
      performance.now() - startedAt,
      { participantKind: "client", phase: "total" },
    );
    return aliasCaseScopedRuntime(
      args.contract,
      client,
    ) as TrellisTestConnectedClient<TContract>;
  }

  /**
   * Captures live decoded contract events through a synthetic app participant.
   *
   * The capture subscribes with generated event facade listeners in ephemeral mode
   * and uses normal `uses.events.subscribe` authority for the selected source
   * contract events.
   */
  async captureEvents<
    TContract extends TrellisTestEventSourceContract,
    const TEvents extends readonly TrellisTestEventAction[],
  >(
    args: TrellisTestEventCaptureOptions<TContract, TEvents>,
  ): Promise<TrellisTestEventCapture<TEvents[number]>> {
    const capture = await startTrellisTestEventCapture({
      runtime: this,
      options: args,
      onStop: (client, stoppedCapture) => {
        this.#clients.delete(client);
        this.#captures.delete(stoppedCapture);
      },
    });
    this.#captures.add(capture);
    return capture;
  }

  /** Polls until `fn` returns a truthy value. */
  waitFor<T>(
    fn: () =>
      | T
      | null
      | undefined
      | false
      | Promise<T | null | undefined | false>,
    opts?: WaitForOptions,
  ): Promise<T> {
    return waitForHelper(fn, {
      timeoutMs: opts?.timeoutMs ?? this.#timeouts.waitForMs,
      intervalMs: opts?.intervalMs,
    });
  }

  /** Flushes the underlying NATS connection. */
  async flush(): Promise<void> {
    await this.#nats.nc.flush();
  }

  /** Drains the underlying NATS connection. */
  async drain(): Promise<void> {
    await this.#nats.nc.drain();
  }

  /** Lists JetStream consumers on the scratch NATS `trellis` event stream. */
  async listTrellisJetStreamConsumers(): Promise<ConsumerInfo[]> {
    return await this.#nats.listTrellisJetStreamConsumers();
  }

  /** Deletes a JetStream consumer from the scratch NATS server by stream and durable/name. */
  async deleteJetStreamConsumer(
    stream: string,
    name: string,
  ): Promise<boolean> {
    return await this.#nats.deleteJetStreamConsumer(stream, name);
  }

  /** Seeds one raw auth connection-presence KV entry for malformed-entry tests. */
  async seedRawAuthConnectionPresence(
    args: TrellisTestRawAuthConnectionPresence,
  ): Promise<void> {
    const nc = await connect({
      servers: this.#nats.natsUrl,
      authenticator: credsAuthenticator(
        await Deno.readFile(
          join(this.workdir, "nats", "creds", "auth-auth.creds"),
        ),
      ),
    });
    try {
      const kv = await new Kvm(nc).open("trellis_connections");
      await kv.put(args.key, JSON.stringify(args.value));
    } finally {
      await nc.close().catch(() => undefined);
    }
  }

  /** Seeds one raw state KV entry for malformed-entry tests. */
  async seedRawStateEntry(args: TrellisTestRawStateEntry): Promise<void> {
    const nc = await connect({
      servers: this.#nats.natsUrl,
      authenticator: credsAuthenticator(
        await Deno.readFile(
          join(this.workdir, "nats", "creds", "auth-auth.creds"),
        ),
      ),
    });
    try {
      const kv = await new Kvm(nc).open("trellis_state");
      await kv.put(args.key, JSON.stringify(args.value));
    } finally {
      await nc.close().catch(() => undefined);
    }
  }

  /** Observes JetStream ACK reply frames on the scratch NATS server. */
  async startJetStreamAckObserver(
    subject?: string,
  ): Promise<JetStreamAckObserver> {
    return await this.#nats.startJetStreamAckObserver(subject);
  }

  /** Observes raw NATS messages with selected headers on the scratch NATS server. */
  async startNatsMessageObserver(
    subject: string,
    headerNames: readonly string[] = [],
  ): Promise<NatsMessageObserver> {
    return await this.#nats.startNatsMessageObserver(subject, headerNames);
  }

  /** Restarts only the Trellis control-plane process, preserving workdir, SQLite state, and NATS. */
  async restartControlPlane(): Promise<void> {
    if (this.#stopped) {
      throw new Error("Cannot restart a stopped Trellis test runtime");
    }
    if (
      this.#controlPlane === undefined || this.#configPath === undefined ||
      this.#trellisOptions === undefined
    ) {
      throw new Error("Cannot restart an attached Trellis test runtime");
    }

    await this.#admin.prepareForControlPlaneRestart();
    await this.#controlPlane.stop();
    this.#controlPlane = await startTrellisProcess({
      trellisUrl: this.trellisUrl,
      configPath: this.#configPath,
      options: this.#trellisOptions,
      startupTimeoutMs: this.#timeouts.startupMs,
      shutdownTimeoutMs: this.#timeouts.shutdownMs,
    });
  }

  /** Returns a service-owned SQLite path under this runtime workdir. */
  async tempSqlitePath(name = "test.sqlite"): Promise<string> {
    const dir = join(this.workdir, "sqlite");
    await Deno.mkdir(dir, { recursive: true });
    return join(dir, name);
  }

  /** Returns the SQLite in-memory URL used by service-owned tests. */
  sqliteMemoryUrl(): string {
    return sqliteMemoryUrlHelper();
  }

  /** Stops clients, control plane, NATS, and the temp directory. */
  async stop(): Promise<void> {
    if (this.#stopped) return;
    this.#stopped = true;
    const failures: unknown[] = [];
    for (const capture of [...this.#captures]) {
      try {
        await capture.stop();
      } catch (error) {
        failures.push(error);
      }
    }
    for (const client of this.#clients) {
      try {
        await client.connection.close();
      } catch (error) {
        failures.push(error);
      }
    }
    try {
      await this.#admin.close();
    } catch (error) {
      failures.push(error);
    }
    try {
      await this.#jobsAdmin?.stop();
    } catch (error) {
      failures.push(error);
    }
    try {
      await this.#controlPlane?.stop();
    } catch (error) {
      failures.push(error);
    }
    try {
      await this.#nats.stop();
    } catch (error) {
      failures.push(error);
    }
    if (this.#ownsWorkdir && !this.#keepWorkdir) {
      try {
        await Deno.remove(this.workdir, { recursive: true });
      } catch (error) {
        failures.push(error);
      }
    }
    if (failures.length > 0) {
      throw new AggregateError(
        failures,
        `Failed to clean up ${failures.length} Trellis test runtime resource(s)`,
      );
    }
  }

  /** @internal Returns recent control-plane process output for test failures. */
  controlPlaneOutput(): string {
    return this.#controlPlane?.outputTails() ?? "";
  }

  [Symbol.asyncDispose](): Promise<void> {
    return this.stop();
  }
}
