import {
  type CallerRuntime,
  type ClientAuthContinuation,
  type ClientAuthRequiredContext,
  createAuth,
  defineAppContract,
  TrellisClient,
} from "@qlever-llc/trellis";
import {
  type CompiledProtocolArtifacts,
  compileProtocolArtifacts,
} from "@qlever-llc/trellis/contracts";
import { recordTrellisDuration as recordOpenTelemetryDuration } from "@qlever-llc/trellis/telemetry";
import type { Static, TSchema } from "typebox";
import Value from "typebox/value";
import {
  AuthDeploymentAuthorityAcceptMigration,
  AuthDeploymentAuthorityAcceptMigrationRequestSchema,
  AuthDeploymentAuthorityAcceptMigrationResponseSchema,
  AuthDeploymentAuthorityAcceptUpdate,
  AuthDeploymentAuthorityAcceptUpdateRequestSchema,
  AuthDeploymentAuthorityAcceptUpdateResponseSchema,
  AuthDeploymentAuthorityGet,
  AuthDeploymentAuthorityGetRequestSchema,
  AuthDeploymentAuthorityGetResponseSchema,
  AuthDeploymentAuthorityList,
  AuthDeploymentAuthorityListRequestSchema,
  AuthDeploymentAuthorityListResponseSchema,
  AuthDeploymentAuthorityPlan,
  AuthDeploymentAuthorityPlanRequestSchema,
  AuthDeploymentAuthorityPlanResponseSchema,
  AuthDeploymentAuthorityPlansList,
  AuthDeploymentAuthorityPlansListRequestSchema,
  AuthDeploymentAuthorityPlansListResponseSchema,
  AuthDeploymentAuthorityReconcile,
  AuthDeploymentAuthorityReconcileRequestSchema,
  AuthDeploymentAuthorityReconcileResponseSchema,
  AuthDeploymentAuthorityReject,
  AuthDeploymentAuthorityRejectRequestSchema,
  AuthDeploymentAuthorityRejectResponseSchema,
  AuthDeploymentsCreate,
  AuthDeploymentsCreateRequestSchema,
  AuthDeploymentsCreateResponseSchema,
  AuthServiceInstancesProvision,
  AuthServiceInstancesProvisionRequestSchema,
  AuthServiceInstancesProvisionResponseSchema,
  AuthSessionsRevoke,
  AuthSessionsRevokeRequestSchema,
  AuthSessionsRevokeResponseSchema,
} from "@qlever-llc/trellis/sdk/auth";
import { generateSessionSeed } from "./control_plane_config.ts";
import { waitFor } from "./wait.ts";
import type {
  TrellisTestAuthorityPlanClassification,
  TrellisTestContractApproval,
  TrellisTestContractLike,
  TrellisTestServiceKey,
} from "./types.ts";
import { recordTrellisTestDuration } from "./integration/metrics.ts";

function recordTrellisDuration(
  name: Parameters<typeof recordOpenTelemetryDuration>[0],
  durationMs: number,
  attributes?: Parameters<typeof recordOpenTelemetryDuration>[2] & {
    deployment?: string;
    participantId?: string;
  },
): void {
  if (attributes === undefined) {
    recordOpenTelemetryDuration(name, durationMs);
  } else {
    const {
      deployment: _deployment,
      participantId: _participantId,
      ...otelAttributes
    } = attributes;
    recordOpenTelemetryDuration(name, durationMs, otelAttributes);
  }
  void recordTrellisTestDuration(name, durationMs, attributes);
}

const ADMIN_USERNAME = "admin";
const ADMIN_PARTICIPANT = {
  id: "trellis-platform-administration",
  artifactDigest: "c99Tmz1QGCWU8XxvGgTR93M9vmtALE9d7W9M8tATYv4",
  needsDigest: "K1gXzXcB0geFulLLlXrAtXrd3kv8HUatyzuQSTP4Wik",
} as const;

const adminContract = defineAppContract(() => ({
  id: "trellis.test.admin@v1",
  displayName: "Trellis Test Admin",
  description:
    "Automates Trellis test runtime administration through Auth RPCs.",
  uses: [
    AuthDeploymentAuthorityAcceptMigration,
    AuthDeploymentAuthorityAcceptUpdate,
    AuthDeploymentAuthorityGet,
    AuthDeploymentAuthorityList,
    AuthDeploymentAuthorityPlan,
    AuthDeploymentAuthorityReject,
    AuthDeploymentAuthorityPlansList,
    AuthDeploymentAuthorityReconcile,
    AuthDeploymentsCreate,
    AuthServiceInstancesProvision,
    AuthSessionsRevoke,
  ],
}));

type AdminClient = CallerRuntime<typeof adminContract>;

function adminMethod<const I extends TSchema, const O extends TSchema>(
  input: I,
  output: O,
  call: (client: AdminClient, input: Static<I>) => Promise<Static<O>>,
) {
  return {
    input,
    output,
    call: (client: AdminClient, value: unknown) =>
      call(client, value as Static<I>),
  } as const;
}

/** @internal Concrete Auth RPCs available to the shared test host. */
export const adminMethods = {
  authDeploymentsCreate: adminMethod(
    AuthDeploymentsCreateRequestSchema,
    AuthDeploymentsCreateResponseSchema,
    (client, input) => client.authDeploymentsCreate(input).orThrow(),
  ),
  authDeploymentAuthorityPlan: adminMethod(
    AuthDeploymentAuthorityPlanRequestSchema,
    AuthDeploymentAuthorityPlanResponseSchema,
    (client, input) => client.authDeploymentAuthorityPlan(input).orThrow(),
  ),
  authDeploymentAuthorityAcceptUpdate: adminMethod(
    AuthDeploymentAuthorityAcceptUpdateRequestSchema,
    AuthDeploymentAuthorityAcceptUpdateResponseSchema,
    (client, input) =>
      client.authDeploymentAuthorityAcceptUpdate(input).orThrow(),
  ),
  authDeploymentAuthorityAcceptMigration: adminMethod(
    AuthDeploymentAuthorityAcceptMigrationRequestSchema,
    AuthDeploymentAuthorityAcceptMigrationResponseSchema,
    (client, input) =>
      client.authDeploymentAuthorityAcceptMigration(input).orThrow(),
  ),
  authDeploymentAuthorityList: adminMethod(
    AuthDeploymentAuthorityListRequestSchema,
    AuthDeploymentAuthorityListResponseSchema,
    (client, input) => client.authDeploymentAuthorityList(input).orThrow(),
  ),
  authDeploymentAuthorityReconcile: adminMethod(
    AuthDeploymentAuthorityReconcileRequestSchema,
    AuthDeploymentAuthorityReconcileResponseSchema,
    (client, input) => client.authDeploymentAuthorityReconcile(input).orThrow(),
  ),
  authDeploymentAuthorityGet: adminMethod(
    AuthDeploymentAuthorityGetRequestSchema,
    AuthDeploymentAuthorityGetResponseSchema,
    (client, input) => client.authDeploymentAuthorityGet(input).orThrow(),
  ),
  authServiceInstancesProvision: adminMethod(
    AuthServiceInstancesProvisionRequestSchema,
    AuthServiceInstancesProvisionResponseSchema,
    (client, input) => client.authServiceInstancesProvision(input).orThrow(),
  ),
  authDeploymentAuthorityPlansList: adminMethod(
    AuthDeploymentAuthorityPlansListRequestSchema,
    AuthDeploymentAuthorityPlansListResponseSchema,
    (client, input) => client.authDeploymentAuthorityPlansList(input).orThrow(),
  ),
  authDeploymentAuthorityReject: adminMethod(
    AuthDeploymentAuthorityRejectRequestSchema,
    AuthDeploymentAuthorityRejectResponseSchema,
    (client, input) => client.authDeploymentAuthorityReject(input).orThrow(),
  ),
  authSessionsRevoke: adminMethod(
    AuthSessionsRevokeRequestSchema,
    AuthSessionsRevokeResponseSchema,
    (client, input) => client.authSessionsRevoke(input).orThrow(),
  ),
} as const;

type AdminRpc = {
  [M in keyof typeof adminMethods]: {
    input: Static<(typeof adminMethods)[M]["input"]>;
    output: Static<(typeof adminMethods)[M]["output"]>;
  };
};

export type TrellisTestAdminRpcMethod = keyof typeof adminMethods;

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function flowIdFromUrl(url: string): string {
  const flowId = new URL(url).searchParams.get("flowId");
  if (!flowId) throw new Error(`Trellis auth URL is missing flowId: ${url}`);
  return flowId;
}

async function postJson(
  url: string,
  body: Record<string, unknown>,
): Promise<unknown> {
  const response = await fetch(url, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      origin: new URL(url).origin,
    },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "");
    throw new Error(
      `Trellis HTTP request failed (${response.status}) for ${url}${
        text ? `: ${text}` : ""
      }`,
    );
  }
  const payload: unknown = await response.json();
  return payload;
}

async function performLocalLogin(args: {
  trellisUrl: string;
  flowId: string;
  password: string;
}): Promise<void> {
  const startedAt = performance.now();
  try {
    await postJson(`${args.trellisUrl}/auth/login/local`, {
      flowId: args.flowId,
      username: ADMIN_USERNAME,
      password: args.password,
    });
  } finally {
    recordTrellisDuration(
      "trellis.auth.flow.duration",
      performance.now() - startedAt,
      { phase: "local_login", authFlow: "local" },
    );
  }
}

async function approveLocalFlowIfNeeded(args: {
  trellisUrl: string;
  flowId: string;
}): Promise<void> {
  const startedAt = performance.now();
  const initialFetchStartedAt = performance.now();
  const response = await fetch(
    `${args.trellisUrl}/auth/flow/${encodeURIComponent(args.flowId)}`,
  );
  if (!response.ok) {
    throw new Error(`Failed to load portal flow (${response.status})`);
  }
  const state = await response.json() as {
    state: string;
    consentViewDigest?: string;
  };
  recordTrellisDuration(
    "trellis.auth.flow.duration",
    performance.now() - initialFetchStartedAt,
    { phase: "approval_fetch" },
  );
  if (state.state === "bound") {
    recordTrellisDuration(
      "trellis.auth.flow.duration",
      performance.now() - startedAt,
      { phase: "total" },
    );
    return;
  }
  if (state.state === "approval_required") {
    if (typeof state.consentViewDigest !== "string") {
      throw new Error("Trellis auth approval did not include a consent digest");
    }
    const approvalStartedAt = performance.now();
    const approved = await postJson(
      `${args.trellisUrl}/auth/flow/${
        encodeURIComponent(args.flowId)
      }/approval`,
      {
        approved: true,
        consentViewDigest: state.consentViewDigest,
        selectedOptionalBundles: [],
        idempotencyKey: crypto.randomUUID(),
      },
    ) as { state: string };
    recordTrellisDuration(
      "trellis.auth.flow.duration",
      performance.now() - approvalStartedAt,
      { phase: "approval_submit" },
    );
    if (approved.state === "approved") {
      recordTrellisDuration(
        "trellis.auth.flow.duration",
        performance.now() - startedAt,
        { phase: "total" },
      );
      return;
    }
    throw new Error(
      `Trellis auth approval did not complete; portal state is '${approved.state}'`,
    );
  }
  throw new Error(
    `Trellis local login did not reach approval; portal state is '${state.state}'`,
  );
}

async function completeLocalAuthFlow(args: {
  trellisUrl: string;
  loginUrl: string;
  password: string;
}): Promise<ClientAuthContinuation> {
  const startedAt = performance.now();
  const flowId = flowIdFromUrl(args.loginUrl);
  await performLocalLogin({
    trellisUrl: args.trellisUrl,
    flowId,
    password: args.password,
  });
  await approveLocalFlowIfNeeded({
    trellisUrl: args.trellisUrl,
    flowId,
  });
  recordTrellisDuration(
    "trellis.auth.flow.duration",
    performance.now() - startedAt,
    { phase: "total" },
  );
  return { status: "bound", flowId };
}

function deploymentKey(deployment: string): string {
  return `service:${deployment}`;
}

function isAuthorityPlanClassification(
  value: string,
): value is TrellisTestAuthorityPlanClassification {
  return value === "initial" || value === "update" || value === "migration";
}

/** Internal public-surface admin automation used by `TrellisTestRuntime`. */
export class TrellisTestAdminAutomation {
  readonly #trellisUrl: string;
  readonly #adminPassword: string;
  readonly #defaultDeployment: string;
  readonly #defaultMutableDev: boolean;
  readonly #reconciliationMs: number;
  readonly #autoAccept: ReadonlySet<TrellisTestAuthorityPlanClassification>;
  readonly #getBootstrapUrl: () => Promise<string>;
  readonly #createdDeployments = new Map<string, Promise<void>>();
  readonly #deploymentIds = new Map<string, string>();
  readonly #authorityIds = new Map<string, string>();
  readonly #protocolApis = new Map<
    string,
    CompiledProtocolArtifacts["api"]
  >();
  #bootstrapComplete: Promise<void> | undefined;
  #adminClient: Promise<AdminClient> | undefined;
  #connectedAdminClient: AdminClient | undefined;
  readonly #rpcProxy: { url: string; token: string } | undefined;

  /** Creates admin automation backed by the supplied bootstrap URL provider. */
  constructor(args: {
    trellisUrl: string;
    adminPassword: string;
    defaultDeployment: string;
    defaultMutableDev: boolean;
    reconciliationMs: number;
    autoAccept: readonly TrellisTestAuthorityPlanClassification[];
    getBootstrapUrl: () => Promise<string>;
    bootstrapComplete?: boolean;
    rpcProxy?: { url: string; token: string };
  }) {
    this.#trellisUrl = args.trellisUrl.replace(/\/$/, "");
    this.#adminPassword = args.adminPassword;
    this.#defaultDeployment = args.defaultDeployment;
    this.#defaultMutableDev = args.defaultMutableDev;
    this.#reconciliationMs = args.reconciliationMs;
    this.#autoAccept = new Set(args.autoAccept);
    this.#getBootstrapUrl = args.getBootstrapUrl;
    this.#rpcProxy = args.rpcProxy;
    if (args.bootstrapComplete === true) {
      this.#bootstrapComplete = Promise.resolve();
    }
  }

  async #completeBootstrap(): Promise<void> {
    this.#bootstrapComplete ??= (async () => {
      const startedAt = performance.now();
      try {
        const bootstrapUrl = await this.#getBootstrapUrl();
        const flowId = flowIdFromUrl(bootstrapUrl);
        const response = await postJson(
          `${this.#trellisUrl}/auth/account-flow/${
            encodeURIComponent(flowId)
          }/local-password`,
          { username: ADMIN_USERNAME, password: this.#adminPassword },
        );
        if (!isRecord(response) || response.status !== "created") {
          throw new Error(
            "Trellis first-admin bootstrap returned an unexpected response",
          );
        }
      } finally {
        recordTrellisDuration(
          "trellis.admin.workflow.duration",
          performance.now() - startedAt,
          { operation: "complete_bootstrap", phase: "total" },
        );
      }
    })();
    await this.#bootstrapComplete;
  }

  async #client(): Promise<AdminClient> {
    this.#adminClient ??= (async () => {
      const startedAt = performance.now();
      try {
        await this.#completeBootstrap();
        const sessionKeySeed = generateSessionSeed();
        const client = await TrellisClient.connect({
          trellisUrl: this.#trellisUrl,
          name: "trellis-test-admin",
          timeout: 60_000,
          contract: adminContract,
          participant: ADMIN_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed,
            redirectTo: `${this.#trellisUrl}/_trellis/test/admin-auth`,
          },
          onAuthRequired: (ctx: ClientAuthRequiredContext) =>
            completeLocalAuthFlow({
              trellisUrl: this.#trellisUrl,
              loginUrl: ctx.loginUrl,
              password: this.#adminPassword,
            }),
        }).orThrow();
        this.#connectedAdminClient = client;
        return client;
      } finally {
        recordTrellisDuration(
          "trellis.admin.workflow.duration",
          performance.now() - startedAt,
          { operation: "register_service", phase: "connect" },
        );
      }
    })();
    return await this.#adminClient;
  }

  /** Forwards one validated low-level Auth RPC over the local or shared admin transport. */
  async callAdminRpc(
    method: string,
    input: unknown,
  ): Promise<unknown> {
    if (!Object.hasOwn(adminMethods, method)) {
      throw new Error(`unsupported Trellis test admin RPC ${method}`);
    }
    const rpcMethod = method as TrellisTestAdminRpcMethod;
    const descriptor = adminMethods[rpcMethod];
    let decodedInput: unknown;
    try {
      decodedInput = Value.Decode(descriptor.input, input);
    } catch (error) {
      throw new Error(
        `${method} input decode failed for ${JSON.stringify(input)}: ${
          String(error)
        }`,
      );
    }
    const output = this.#rpcProxy === undefined
      ? await descriptor.call(await this.#client(), decodedInput)
      : await postAdminRpc(this.#rpcProxy, rpcMethod, decodedInput);
    try {
      return Value.Decode(descriptor.output, output);
    } catch (error) {
      throw new Error(
        `${method} output decode failed for ${JSON.stringify(output)}: ${
          String(error)
        }`,
      );
    }
  }

  async #rpc<M extends TrellisTestAdminRpcMethod>(
    method: M,
    input: AdminRpc[M]["input"],
  ): Promise<AdminRpc[M]["output"]> {
    return await this.callAdminRpc(method, input) as AdminRpc[M]["output"];
  }

  /** Creates a service deployment through `Auth.Deployments.Create`. */
  async createDeployment(args: {
    deployment?: string;
    mutableDev?: boolean;
  } = {}): Promise<void> {
    const deployment = args.deployment ?? this.#defaultDeployment;
    const key = deploymentKey(deployment);
    const existing = this.#createdDeployments.get(key);
    if (existing !== undefined) return existing;
    const startedAt = performance.now();
    const promise = (async () => {
      const created = await this.#rpc("authDeploymentsCreate", {
        displayName: deployment,
        expiresAt: null,
        idempotencyKey: crypto.randomUUID(),
        kind: "service",
        participantId: null,
        portalId: null,
        requiresDeviceDelegation: false,
      });
      this.#deploymentIds.set(deployment, created.deployment.deploymentId);
      this.#createdDeployments.set(key, Promise.resolve());
      recordTrellisDuration(
        "trellis.admin.workflow.duration",
        performance.now() - startedAt,
        { operation: "register_service", phase: "create_deployment" },
      );
    })();
    this.#createdDeployments.set(
      key,
      promise.catch(() => {
        this.#createdDeployments.delete(key);
      }),
    );
    await promise;
  }

  /** Completes a public app/client authentication flow as the test admin user. */
  async completeClientAuth(
    ctx: ClientAuthRequiredContext,
  ): Promise<ClientAuthContinuation> {
    if (this.#rpcProxy !== undefined) {
      const response = await postAdminRpc(
        this.#rpcProxy,
        "completeClientAuth",
        ctx,
      );
      if (
        !isRecord(response) || response.status !== "bound" ||
        typeof response.flowId !== "string"
      ) {
        throw new Error(
          "Trellis test auth adapter returned an invalid continuation",
        );
      }
      return { status: "bound", flowId: response.flowId };
    }
    const startedAt = performance.now();
    await this.#completeBootstrap();
    const flowId = flowIdFromUrl(ctx.loginUrl);
    await performLocalLogin({
      trellisUrl: this.#trellisUrl,
      flowId,
      password: this.#adminPassword,
    });
    await approveLocalFlowIfNeeded({ trellisUrl: this.#trellisUrl, flowId });
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - startedAt,
      { operation: "register_client", phase: "total" },
    );
    return { status: "bound", flowId };
  }

  /** Plans, accepts, reconciles, and waits for a contract authority change. */
  async approveContract(args: {
    deployment?: string;
    contract: TrellisTestContractLike;
    allowPlanClassifications?:
      readonly TrellisTestAuthorityPlanClassification[];
  }): Promise<TrellisTestContractApproval> {
    const totalStartedAt = performance.now();
    const deployment = args.deployment ?? this.#defaultDeployment;
    await this.createDeployment({ deployment });
    const deploymentId = this.#deploymentIds.get(deployment);
    if (!deploymentId) {
      throw new Error(`Trellis deployment '${deployment}' was not created`);
    }
    const artifacts = await compileProtocolArtifacts(
      args.contract,
      Object.fromEntries(this.#protocolApis),
    );
    const referencedApis = new Map(this.#protocolApis);
    for (const api of artifacts.referencedApis) {
      referencedApis.set(String(api.id), api);
    }
    const planStartedAt = performance.now();
    const planned = await this.#rpc("authDeploymentAuthorityPlan", {
      deploymentId,
      expiresAt: null,
      idempotencyKey: crypto.randomUUID(),
      participantArtifact: artifacts.participant,
      referencedApiArtifacts: [
        artifacts.api,
        ...referencedApis.values(),
      ],
    });
    this.#protocolApis.set(String(artifacts.api.id), artifacts.api);
    const classification = planned.proposal.classification;
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - planStartedAt,
      {
        deployment,
        participantId: String(artifacts.participant.id),
        operation: "approve_contract",
        phase: "plan",
        planClassification: classification,
      },
    );
    if (!isAuthorityPlanClassification(classification)) {
      throw new Error(
        `Trellis test runtime received unsupported authority plan classification '${classification}'`,
      );
    }
    const allowed = args.allowPlanClassifications === undefined
      ? this.#autoAccept
      : new Set(args.allowPlanClassifications);
    if (!allowed.has(classification)) {
      throw new Error(
        `Trellis test runtime cannot auto-accept '${classification}' authority plans; allowed classifications: ${
          [...allowed].join(", ") || "none"
        }`,
      );
    }
    if (
      artifacts.participant.kind !== "service" &&
      artifacts.participant.kind !== "device"
    ) {
      return {
        planId: planned.proposal.proposalId,
        classification,
        participantId: planned.proposal.participantId,
        participantDigest: planned.proposal.participantArtifactDigest,
        participantNeedsDigest: planned.proposal.participantNeedsDigest,
        deploymentId,
      };
    }
    const acceptStartedAt = performance.now();
    if (classification !== "migration") {
      await this.#rpc("authDeploymentAuthorityAcceptUpdate", {
        expectedBaseAuthorityVersion: planned.proposal.baseAuthorityVersion,
        idempotencyKey: crypto.randomUUID(),
        proposalId: planned.proposal.proposalId,
        reason: null,
      });
    } else {
      await this.#rpc("authDeploymentAuthorityAcceptMigration", {
        expectedBaseAuthorityVersion: planned.proposal.baseAuthorityVersion,
        idempotencyKey: crypto.randomUUID(),
        proposalId: planned.proposal.proposalId,
        reason:
          "Approved by TrellisTestRuntime for an isolated integration test.",
      });
    }
    const authority = await this.#rpc("authDeploymentAuthorityList", {
      cursor: undefined,
      deploymentId,
      limit: 1,
      state: "accepted",
    });
    const authorityId = authority.entries[0]?.authorityId;
    if (!authorityId) {
      throw new Error(
        `Trellis deployment '${deployment}' has no accepted authority`,
      );
    }
    this.#authorityIds.set(deployment, authorityId);
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - acceptStartedAt,
      {
        deployment,
        participantId: String(artifacts.participant.id),
        operation: "approve_contract",
        phase: "accept",
        planClassification: classification,
      },
    );
    await this.reconcile(deployment, "approveContract.reconcile");
    await this.waitReady(deployment, "approveContract.waitReady");
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - totalStartedAt,
      {
        deployment,
        participantId: String(artifacts.participant.id),
        operation: "approve_contract",
        phase: "total",
        planClassification: classification,
      },
    );
    return {
      planId: planned.proposal.proposalId,
      classification,
      participantId: planned.proposal.participantId,
      participantDigest: planned.proposal.participantArtifactDigest,
      participantNeedsDigest: planned.proposal.participantNeedsDigest,
      deploymentId,
    };
  }

  /** Triggers deployment-authority reconciliation for a service deployment. */
  async reconcile(deployment: string, label = "reconcile"): Promise<void> {
    const startedAt = performance.now();
    const authorityId = this.#authorityIds.get(deployment);
    if (!authorityId) {
      throw new Error(
        `Trellis deployment '${deployment}' has no accepted authority`,
      );
    }
    await this.#rpc("authDeploymentAuthorityReconcile", {
      authorityId,
      expectedVersion: null,
      idempotencyKey: crypto.randomUUID(),
    });
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - startedAt,
      { operation: label, phase: "reconcile" },
    );
  }

  /** Waits until materialized deployment authority is current. */
  async waitReady(deployment: string, label = "waitReady"): Promise<void> {
    const startedAt = performance.now();
    const authorityId = this.#authorityIds.get(deployment);
    if (!authorityId) {
      throw new Error(
        `Trellis deployment '${deployment}' has no accepted authority`,
      );
    }
    await waitFor(async () => {
      const pollStartedAt = performance.now();
      const result = await this.#rpc("authDeploymentAuthorityGet", {
        authorityId,
      });
      const materialized = result.authority.materialization;
      recordTrellisDuration(
        "trellis.admin.workflow.duration",
        performance.now() - pollStartedAt,
        { operation: `${label}.poll`, phase: "wait_ready" },
      );
      if (materialized?.state === "error") {
        throw new Error(
          `Trellis deployment '${deployment}' reconciliation failed${
            materialized.error ? `: ${materialized.error}` : ""
          }`,
        );
      }
      if (
        materialized?.state === "available" &&
        materialized.authorityVersion === result.authority.version &&
        materialized.reconciledAt !== null
      ) {
        return true;
      }
      return false;
    }, { timeoutMs: this.#reconciliationMs });
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - startedAt,
      { operation: label, phase: "wait_ready" },
    );
  }

  /** Provisions a service instance key through `Auth.ServiceInstances.Provision`. */
  async provisionServiceInstance(args: {
    deployment?: string;
    contract: TrellisTestContractLike;
    sessionKeySeed?: string;
  }): Promise<TrellisTestServiceKey> {
    const startedAt = performance.now();
    const deployment = args.deployment ?? this.#defaultDeployment;
    const approved = await this.approveContract({
      deployment,
      contract: args.contract,
    });
    const identitySeed = args.sessionKeySeed ?? generateSessionSeed();
    const auth = await createAuth({ sessionKeySeed: identitySeed });
    const deploymentId = this.#deploymentIds.get(deployment);
    if (!deploymentId) {
      throw new Error(`Trellis deployment '${deployment}' was not created`);
    }
    const provisioned = await this.#rpc("authServiceInstancesProvision", {
      deploymentId,
      idempotencyKey: crypto.randomUUID(),
      identityPublicKey: auth.sessionKey,
      instanceId: null,
      participantId: approved.participantId,
    });
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - startedAt,
      { operation: "provision_service", phase: "total" },
    );
    return {
      seed: identitySeed,
      sessionSeed: generateSessionSeed(),
      sessionKey: auth.sessionKey,
      deploymentId,
      instanceId: provisioned.instance.instanceId,
      participantId: approved.participantId,
      participantArtifactDigest: approved.participantDigest,
      participantNeedsDigest: approved.participantNeedsDigest,
    };
  }

  /** Runs the full service registration sequence used by test services. */
  async registerService(args: {
    deployment?: string;
    contract: TrellisTestContractLike;
    sessionKeySeed?: string;
  }): Promise<TrellisTestServiceKey> {
    const startedAt = performance.now();
    const deployment = args.deployment ?? this.#defaultDeployment;
    const key = await this.provisionServiceInstance({
      deployment,
      contract: args.contract,
      sessionKeySeed: args.sessionKeySeed,
    });
    await this.reconcile(deployment, "registerService.postProvision.reconcile");
    await this.waitReady(deployment, "registerService.postProvision.waitReady");
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - startedAt,
      { operation: "register_service", phase: "total" },
    );
    return key;
  }

  /** Lists deployment authority plans. */
  async listAuthorityPlans(args: {
    deploymentId?: string;
    state?: "pending" | "accepted" | "rejected" | "superseded" | "expired";
    limit?: number;
    cursor?: string;
  }): Promise<{ entries: unknown[]; nextCursor: string | null }> {
    return await this.#rpc("authDeploymentAuthorityPlansList", {
      deploymentId: args.deploymentId,
      state: args.state,
      limit: args.limit ?? 20,
      cursor: args.cursor,
    });
  }

  /** Rejects a pending deployment authority plan. */
  async rejectAuthorityPlan(args: {
    planId: string;
    reason?: string;
  }): Promise<unknown> {
    return await this.#rpc("authDeploymentAuthorityReject", {
      proposalId: args.planId,
      reason: args.reason ?? null,
      idempotencyKey: crypto.randomUUID(),
    });
  }

  /** Accepts a pending deployment authority update plan. */
  async acceptAuthorityUpdate(args: {
    planId: string;
    expectedDesiredVersion?: number;
  }): Promise<unknown> {
    return await this.#rpc("authDeploymentAuthorityAcceptUpdate", {
      proposalId: args.planId,
      expectedBaseAuthorityVersion: args.expectedDesiredVersion ?? null,
      reason: null,
      idempotencyKey: crypto.randomUUID(),
    });
  }

  /** Accepts a pending deployment authority migration plan. Requires an acknowledgement string. */
  async acceptAuthorityMigration(args: {
    planId: string;
    acknowledgement: string;
    expectedDesiredVersion?: number;
  }): Promise<unknown> {
    return await this.#rpc("authDeploymentAuthorityAcceptMigration", {
      proposalId: args.planId,
      expectedBaseAuthorityVersion: args.expectedDesiredVersion ?? null,
      reason: args.acknowledgement,
      idempotencyKey: crypto.randomUUID(),
    });
  }

  /** Provisions a service instance key without approving the contract or altering authority. */
  async provisionServiceInstanceOnly(args: {
    deployment?: string;
    sessionKeySeed?: string;
  }): Promise<{ seed: string; sessionKey: string }> {
    const deployment = args.deployment ?? this.#defaultDeployment;
    const seed = args.sessionKeySeed ?? generateSessionSeed();
    const auth = await createAuth({ sessionKeySeed: seed });
    const deploymentId = this.#deploymentIds.get(deployment);
    if (!deploymentId) {
      throw new Error(`Trellis deployment '${deployment}' was not created`);
    }
    await this.#rpc("authServiceInstancesProvision", {
      deploymentId,
      idempotencyKey: crypto.randomUUID(),
      identityPublicKey: auth.sessionKey,
      instanceId: null,
      participantId: null,
    });
    return { seed, sessionKey: auth.sessionKey };
  }

  /** Ensures bootstrap is complete and clears the admin connection before a Trellis restart. */
  async prepareForControlPlaneRestart(): Promise<void> {
    await this.#completeBootstrap();
    await this.close();
  }

  /** Closes the lazily connected admin client, when it exists. */
  async close(): Promise<void> {
    const client = this.#connectedAdminClient;
    this.#connectedAdminClient = undefined;
    this.#adminClient = undefined;
    await client?.connection.close();
  }
}

async function postAdminRpc(
  proxy: { url: string; token: string },
  method: TrellisTestAdminRpcMethod | "completeClientAuth",
  input: unknown,
): Promise<unknown> {
  const response = await fetch(proxy.url, {
    method: "POST",
    headers: {
      authorization: `Bearer ${proxy.token}`,
      "content-type": "application/json",
    },
    body: JSON.stringify({ method, input }),
    signal: AbortSignal.timeout(190_000),
  });
  const body: unknown = await response.json();
  if (response.ok && isRecord(body) && body.ok === true) return body.output;
  throw new Error(
    isRecord(body) && typeof body.error === "string"
      ? body.error
      : `Trellis test admin RPC proxy returned ${response.status}`,
  );
}
