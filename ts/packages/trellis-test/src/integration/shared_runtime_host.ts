import { TransportError } from "@qlever-llc/trellis/errors";
import { dirname, join } from "@std/path";

import {
  removeStaleMarkedDirectories,
  writeTrellisTestOwnerMarker,
} from "../cleanup.ts";
import { NatsTestContainer } from "../nats_container.ts";
import { TrellisTestRuntime } from "../runtime.ts";
import { readTrellisTestMetrics, TRELLIS_TEST_METRICS_ENV } from "./metrics.ts";
import { integrationSlug } from "./names.ts";
import {
  TRELLIS_TEST_SHARED_RUNTIME_ENV,
  type TrellisIntegrationRuntimeAssignment,
  type TrellisIntegrationSharedRuntimeManifest,
} from "./shared_runtime_protocol.ts";
import type { TrellisIntegrationRuntimeOptions } from "./types.ts";
import { startTestOidcProvider } from "./oidc_provider.ts";

const WORKDIR_PREFIX = "trellis-test-pool-";
const WORKDIR_OWNER_MARKER = ".trellis-test-owner";
const SHARED_TENANT = "shared";

/** Shared control-plane host that assigns isolated runtimes to concurrent cases. */
export type TrellisIntegrationSharedRuntimeHost = {
  /** Path to the private manifest passed to worker processes. */
  readonly manifestPath: string;
  /** Environment variables workers need to attach to the host. */
  readonly env: Record<string, string>;
  /** Reads process-start metrics before the host workdir is removed. */
  metrics?(): Promise<readonly Record<string, unknown>[]>;
  /** Returns recent shared control-plane output for failed runs. */
  output?(): string;
  /** Stops shared Trellis, NATS, and their temporary workdirs. */
  stop(): Promise<void>;
};

/** Starts one NATS server and one Trellis runtime for ordinary test cases. */
export async function startTrellisIntegrationSharedRuntimeHost(args: {
  readonly runtime: TrellisIntegrationRuntimeOptions;
  readonly assignments: readonly {
    id: string;
    namespacePrefix?: string;
  }[];
}): Promise<TrellisIntegrationSharedRuntimeHost> {
  const workdir = await Deno.makeTempDir({ prefix: WORKDIR_PREFIX });
  const metricsPath = join(workdir, "metrics.jsonl");
  const previousMetricsPath = Deno.env.get(TRELLIS_TEST_METRICS_ENV);
  Deno.env.set(TRELLIS_TEST_METRICS_ENV, metricsPath);
  await writeTrellisTestOwnerMarker(workdir, WORKDIR_OWNER_MARKER);
  await removeStaleMarkedDirectories({
    parent: dirname(workdir),
    prefix: WORKDIR_PREFIX,
    markerName: WORKDIR_OWNER_MARKER,
  });

  const runId = integrationSlug(crypto.randomUUID()).slice(0, 12);
  const hostDeployment = `it-${runId}-host`;
  const adminPassword = `trellis-test-${crypto.randomUUID()}`;
  const adminRpcToken = crypto.randomUUID();
  const assignments: Record<string, TrellisIntegrationRuntimeAssignment> = {};
  const tenantIds = [SHARED_TENANT];
  for (const [index, assignment] of args.assignments.entries()) {
    const tenantId = `case-${index}`;
    tenantIds.push(tenantId);
    assignments[assignment.id] = {
      mode: "isolated-process",
      namespace: `it-${runId}-${assignment.namespacePrefix ?? "case"}-${
        integrationSlug(assignment.id)
      }`,
      tenantId,
    };
  }

  let nats: NatsTestContainer | undefined;
  let runtime: TrellisTestRuntime | undefined;
  const oidc = await startTestOidcProvider({ roles: ["direct"] });
  const adminRpcAbort = new AbortController();
  let adminRpcFinished: Promise<void> | undefined;
  const persistRetainedOutput = async () => {
    if (args.runtime.keepWorkdir === true) {
      await Deno.writeTextFile(
        join(workdir, "trellis-output.log"),
        runtime?.controlPlaneOutput() ?? "",
      );
      console.log(
        JSON.stringify({ event: "integration-workdir", path: workdir }),
      );
    }
  };
  try {
    nats = await NatsTestContainer.start(workdir, {
      startupMs: args.runtime.timeouts?.startupMs,
      tenantIds,
    });
    runtime = await TrellisTestRuntime.start({
      ...args.runtime,
      oauthProviders: {
        ...args.runtime.oauthProviders,
        "test-oidc": {
          type: "oidc",
          issuer: oidc.issuer,
          clientId: "trellis-test-client",
          displayName: "Test OIDC",
          roleClaims: ["/roles"],
        },
        "other-oidc": {
          type: "oidc",
          issuer: oidc.issuer,
          clientId: "trellis-test-client",
          displayName: "Other OIDC",
          roleClaims: ["/roles"],
        },
      },
      timeouts: {
        ...args.runtime.timeouts,
        reconciliationMs: args.runtime.timeouts?.reconciliationMs ?? 120_000,
      },
      adminPassword,
      deployment: hostDeployment,
      nats: {
        workdir,
        natsUrl: nats.natsUrl,
        websocketUrl: nats.websocketUrl,
        manifest: nats.manifests[SHARED_TENANT],
      },
    });
    await runtime.resetAcceptedIntegrationAuthorities();
    await runtime.deployments.create({ id: hostDeployment });
    const adminRpcServer = Deno.serve({
      hostname: "127.0.0.1",
      port: 0,
      signal: adminRpcAbort.signal,
      onListen: () => undefined,
    }, async (request) => {
      if (
        request.method !== "POST" ||
        request.headers.get("authorization") !== `Bearer ${adminRpcToken}`
      ) {
        return Response.json({ ok: false, error: "unauthorized" }, {
          status: 401,
        });
      }
      try {
        const body: unknown = await request.json();
        if (
          typeof body !== "object" || body === null ||
          !("method" in body) || typeof body.method !== "string" ||
          !("input" in body)
        ) {
          return Response.json({ ok: false, error: "invalid request" }, {
            status: 400,
          });
        }
        let output: unknown;
        if (body.method === "completeClientAuth") {
          if (
            typeof body.input !== "object" || body.input === null ||
            !("loginUrl" in body.input) ||
            typeof body.input.loginUrl !== "string" ||
            !("sessionKey" in body.input) ||
            typeof body.input.sessionKey !== "string" ||
            !("mode" in body.input) ||
            (body.input.mode !== "browser" && body.input.mode !== "session_key")
          ) {
            return Response.json({ ok: false, error: "invalid auth context" }, {
              status: 400,
            });
          }
          const authInput = {
            loginUrl: body.input.loginUrl,
            sessionKey: body.input.sessionKey,
            mode: body.input.mode,
          } as const;
          output = await runtime?.completeClientAuth(authInput);
        } else if (body.method === "testOidcSetClaims") {
          if (
            typeof body.input !== "object" || body.input === null ||
            !("origin" in body.input) ||
            typeof body.input.origin !== "string" ||
            !("claims" in body.input) ||
            typeof body.input.claims !== "object" ||
            body.input.claims === null
          ) {
            return Response.json({ ok: false, error: "invalid OIDC claims" }, {
              status: 400,
            });
          }
          oidc.setClaims(
            Object.fromEntries(Object.entries(body.input.claims)),
            new URL(body.input.origin).origin,
          );
          output = {};
        } else {
          for (let attempt = 1;; attempt += 1) {
            try {
              output = await runtime?.callAdminRpc(body.method, body.input);
              break;
            } catch (error) {
              if (!(error instanceof TransportError) || attempt === 3) {
                throw error;
              }
              await new Promise((resolve) => setTimeout(resolve, 250));
            }
          }
        }
        return Response.json({ ok: true, output });
      } catch (error) {
        return Response.json({
          ok: false,
          error: typeof error === "object" && error !== null &&
              "toSerializable" in error &&
              typeof error.toSerializable === "function"
            ? JSON.stringify(error.toSerializable())
            : error instanceof Error
            ? error.message
            : String(error),
        }, { status: 500 });
      }
    });
    adminRpcFinished = adminRpcServer.finished;

    const manifest: TrellisIntegrationSharedRuntimeManifest = {
      version: 5,
      runId,
      trellisUrl: runtime.trellisUrl,
      natsUrl: nats.natsUrl,
      websocketUrl: nats.websocketUrl,
      workdir,
      controlPlaneSqlitePath: join(
        runtime.workdir,
        "trellis/trellis.sqlite.platform",
      ),
      adminPassword,
      adminRpcUrl: `http://127.0.0.1:${adminRpcServer.addr.port}`,
      adminRpcToken,
      testOidcIssuer: oidc.issuer,
      tenants: { ...nats.manifests },
      assignments,
    };
    const manifestPath = join(workdir, "shared-runtime-manifest.json");
    await Deno.writeTextFile(manifestPath, JSON.stringify(manifest), {
      mode: 0o600,
    });
    return {
      manifestPath,
      env: {
        [TRELLIS_TEST_SHARED_RUNTIME_ENV]: manifestPath,
        [TRELLIS_TEST_METRICS_ENV]: metricsPath,
      },
      metrics: () => readTrellisTestMetrics(metricsPath),
      output: () => runtime?.controlPlaneOutput() ?? "",
      async stop() {
        const errors: unknown[] = [];
        adminRpcAbort.abort();
        await adminRpcFinished?.catch(() => undefined);
        await runtime?.stop().catch((error) => errors.push(error));
        await nats?.stop().catch((error) => errors.push(error));
        await oidc.shutdown().catch((error) => errors.push(error));
        await persistRetainedOutput().catch((error) => errors.push(error));
        if (args.runtime.keepWorkdir !== true) {
          await Deno.remove(workdir, { recursive: true }).catch((error) =>
            errors.push(error)
          );
        }
        restoreMetricsPath(previousMetricsPath);
        if (errors.length > 0) {
          throw new AggregateError(
            errors,
            "failed to stop shared Trellis test host",
          );
        }
      },
    };
  } catch (error) {
    adminRpcAbort.abort();
    await adminRpcFinished?.catch(() => undefined);
    await runtime?.stop().catch(() => undefined);
    await nats?.stop().catch(() => undefined);
    await oidc.shutdown().catch(() => undefined);
    await persistRetainedOutput().catch(() => undefined);
    if (args.runtime.keepWorkdir !== true) {
      await Deno.remove(workdir, { recursive: true }).catch(() => undefined);
    }
    restoreMetricsPath(previousMetricsPath);
    throw error;
  }
}

function restoreMetricsPath(previous: string | undefined): void {
  if (previous === undefined) {
    Deno.env.delete(TRELLIS_TEST_METRICS_ENV);
  } else {
    Deno.env.set(TRELLIS_TEST_METRICS_ENV, previous);
  }
}
