import {
  type ClientAuthContinuation,
  type ClientAuthRequiredContext,
  TrellisClient,
} from "@qlever-llc/trellis";
import Value from "typebox/value";

import {
  adminAccountTokenFromUrl,
  approveLocalFlowIfNeeded,
  completeLocalAuthFlow,
  flowIdFromUrl,
  performLocalLogin,
} from "./admin/auth_flow.ts";
import * as adminDeployment from "./admin/deployment.ts";
import {
  ADMIN_PARTICIPANT,
  ADMIN_USERNAME,
  type AdminClient,
  adminMethods,
  adminParticipant,
  type AdminRpc,
  type TrellisTestAdminRpcMethod,
} from "./admin/methods.ts";
import { recordTrellisDuration } from "./admin/metrics.ts";
import { isRecord, postJson } from "./admin/transport.ts";
import { generateSessionSeed } from "./control_plane_config.ts";
import type {
  TrellisTestAuthorityPlanClassification,
  TrellisTestParticipantApproval,
  TrellisTestParticipantLike,
  TrellisTestServiceKey,
} from "./types.ts";

export { adminMethods, type TrellisTestAdminRpcMethod };

/** Internal public-surface admin automation used by `TrellisTestRuntime`. */
export class TrellisTestAdminAutomation {
  readonly #trellisUrl: string;
  readonly #adminPassword: string;
  readonly #getBootstrapUrl: () => Promise<string>;
  #bootstrapComplete: Promise<void> | undefined;
  #adminClient: Promise<AdminClient> | undefined;
  #connectedAdminClient: AdminClient | undefined;
  readonly #deployment: adminDeployment.AdminDeploymentContext;

  /** Creates admin automation backed by the supplied bootstrap URL provider. */
  constructor(args: {
    trellisUrl: string;
    adminPassword: string;
    defaultDeployment: string;
    reconciliationMs: number;
    autoAccept: readonly TrellisTestAuthorityPlanClassification[];
    getBootstrapUrl: () => Promise<string>;
    bootstrapComplete?: boolean;
  }) {
    this.#trellisUrl = args.trellisUrl.replace(/\/$/, "");
    this.#adminPassword = args.adminPassword;
    this.#getBootstrapUrl = args.getBootstrapUrl;
    this.#deployment = {
      defaultDeployment: args.defaultDeployment,
      reconciliationMs: args.reconciliationMs,
      autoAccept: new Set(args.autoAccept),
      createdDeployments: new Map(),
      deploymentIds: new Map(),
      authorityIds: new Map(),
      protocolApis: new Map(),
      rpc: <M extends TrellisTestAdminRpcMethod>(
        method: M,
        input: AdminRpc[M]["input"],
      ) => this.#rpc(method, input),
    };
    if (args.bootstrapComplete === true) {
      this.#bootstrapComplete = Promise.resolve();
    }
  }

  async #completeBootstrap(): Promise<void> {
    this.#bootstrapComplete ??= (async () => {
      const startedAt = performance.now();
      try {
        const bootstrapUrl = await this.#getBootstrapUrl();
        const flowId = adminAccountTokenFromUrl(bootstrapUrl);
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
          participant: adminParticipant,
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
    const output = await descriptor.call(await this.#client(), decodedInput);
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
    kind?: "service" | "device";
  } = {}): Promise<void> {
    return await adminDeployment.createDeployment(this.#deployment, args);
  }

  async provisionDevice(
    input: import("../.trellis/ts/apis/auth/mod.ts").AuthDevicesProvisionInput,
  ): Promise<
    import("../.trellis/ts/apis/auth/mod.ts").AuthDevicesProvisionOutput
  > {
    return await this.#rpc("authDevicesProvision", input);
  }

  async stateAdminGet(
    input: import("../.trellis/ts/apis/state/mod.ts").StateAdminGetInput,
  ): Promise<import("../.trellis/ts/apis/state/mod.ts").StateAdminGetOutput> {
    return await this.#rpc("stateAdminGet", input);
  }

  async stateAdminList(
    input: import("../.trellis/ts/apis/state/mod.ts").StateAdminListInput,
  ): Promise<import("../.trellis/ts/apis/state/mod.ts").StateAdminListOutput> {
    return await this.#rpc("stateAdminList", input);
  }

  async stateAdminDelete(
    input: import("../.trellis/ts/apis/state/mod.ts").StateAdminDeleteInput,
  ): Promise<
    import("../.trellis/ts/apis/state/mod.ts").StateAdminDeleteOutput
  > {
    return await this.#rpc("stateAdminDelete", input);
  }

  /** Completes a public app/client authentication flow as the test admin user. */
  async completeClientAuth(
    ctx: ClientAuthRequiredContext,
  ): Promise<ClientAuthContinuation> {
    const startedAt = performance.now();
    await this.#completeBootstrap();
    const flowId = flowIdFromUrl(ctx.loginUrl);
    const binding = await performLocalLogin({
      trellisUrl: this.#trellisUrl,
      flowId,
      password: this.#adminPassword,
    });
    await approveLocalFlowIfNeeded({
      trellisUrl: this.#trellisUrl,
      flowId,
      binding,
    });
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
    contract: TrellisTestParticipantLike;
    allowPlanClassifications?:
      readonly TrellisTestAuthorityPlanClassification[];
  }): Promise<TrellisTestParticipantApproval> {
    return await adminDeployment.approveContract(this.#deployment, args);
  }

  /** Triggers deployment-authority reconciliation for a service deployment. */
  async reconcile(deployment: string, label = "reconcile"): Promise<void> {
    return await adminDeployment.reconcile(this.#deployment, deployment, label);
  }

  /** Waits until materialized deployment authority is current. */
  async waitReady(deployment: string, label = "waitReady"): Promise<void> {
    return await adminDeployment.waitReady(this.#deployment, deployment, label);
  }

  /** Provisions a service instance key through `Auth.ServiceInstances.Provision`. */
  async provisionServiceInstance(args: {
    deployment?: string;
    contract: TrellisTestParticipantLike;
    sessionKeySeed?: string;
  }): Promise<TrellisTestServiceKey> {
    return await adminDeployment.provisionServiceInstance(
      this.#deployment,
      args,
    );
  }

  /** Runs the full service registration sequence used by test services. */
  async registerService(args: {
    deployment?: string;
    contract: TrellisTestParticipantLike;
    sessionKeySeed?: string;
  }): Promise<TrellisTestServiceKey> {
    return await adminDeployment.registerService(this.#deployment, args);
  }

  /** Lists deployment authority plans. */
  async listAuthorityPlans(args: {
    deploymentId?: string;
    state?: "pending" | "accepted" | "rejected" | "superseded" | "expired";
    limit?: number;
    cursor?: string;
  }): Promise<{ entries: unknown[]; nextCursor: string | null }> {
    return await adminDeployment.listAuthorityPlans(this.#deployment, args);
  }

  /** Rejects a pending deployment authority plan. */
  async rejectAuthorityPlan(args: {
    planId: string;
    reason?: string;
  }): Promise<unknown> {
    return await adminDeployment.rejectAuthorityPlan(this.#deployment, args);
  }

  /** Accepts a pending deployment authority update plan. */
  async acceptAuthorityUpdate(args: {
    planId: string;
    expectedDesiredVersion?: number;
  }): Promise<unknown> {
    return await adminDeployment.acceptAuthorityUpdate(this.#deployment, args);
  }

  /** Accepts a pending deployment authority migration plan. Requires an acknowledgement string. */
  async acceptAuthorityMigration(args: {
    planId: string;
    acknowledgement: string;
    expectedDesiredVersion?: number;
  }): Promise<unknown> {
    return await adminDeployment.acceptAuthorityMigration(
      this.#deployment,
      args,
    );
  }

  /** Provisions a service instance key without approving the contract or altering authority. */
  async provisionServiceInstanceOnly(args: {
    deployment?: string;
    sessionKeySeed?: string;
  }): Promise<{ seed: string; sessionKey: string }> {
    return await adminDeployment.provisionServiceInstanceOnly(
      this.#deployment,
      args,
    );
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
