import type { ClientOpts } from "@qlever-llc/trellis";
import type { TrellisTestRuntime } from "../runtime.ts";
import type { NatsMessageObserver } from "../nats_container.ts";
import type {
  TrellisTestClientAuth,
  TrellisTestClientKey,
  TrellisTestClientParticipant,
  TrellisTestConnectedClient,
  TrellisTestParticipantLike,
  TrellisTestRawAuthConnectionPresence,
  TrellisTestRawStateEntry,
  TrellisTestRuntimeStartOptions,
  TrellisTestServiceKey,
  WaitForOptions,
} from "../types.ts";

/** Describes how a Trellis integration test obtains runtime access. */
export type TrellisIntegrationScope =
  | { readonly kind: "isolated" }
  | { readonly kind: "shared-case"; readonly caseId: string };

/** Options used when starting direct or shared Trellis integration runtimes. */
export type TrellisIntegrationRuntimeOptions =
  & Partial<
    TrellisTestRuntimeStartOptions
  >
  & {
    /** Caller-supplied Trellis control-plane process command and options. */
    readonly trellis: TrellisTestRuntimeStartOptions["trellis"];
  };

/** Runtime surface available inside `trellisIntegrationTest` bodies. */
export type TrellisIntegrationRuntime = {
  /** Base URL for the test Trellis control plane. */
  readonly trellisUrl: string;
  /** NATS URL used by services and clients in this runtime. */
  readonly natsUrl: string;
  /** Runtime working directory containing generated config and manifests. */
  readonly workdir: string;
  /** Deployment admin helpers scoped to the runtime default deployment. */
  readonly deployments: TrellisTestRuntime["deployments"];
  /** Contract approval helpers scoped to the runtime default deployment. */
  readonly contracts: TrellisTestRuntime["contracts"];
  /** Service provisioning helpers scoped to the runtime default deployment. */
  readonly services: TrellisTestRuntime["services"];
  /** Device provisioning helpers backed by the existing Auth admin surface. */
  readonly devices: TrellisTestRuntime["devices"];
  /** State administration helpers backed by the existing State admin surface. */
  readonly state: TrellisTestRuntime["state"];
  /** Authority-plan helpers for tests that assert approval workflows. */
  readonly authority: TrellisTestRuntime["authority"];
  /** Test-only handles available when this runtime owns the control-plane process. */
  readonly controlPlane?: TrellisTestRuntime["controlPlane"];
  /** Registers a service contract and returns service session-key material. */
  registerService(args: {
    readonly name: string;
    readonly contract: TrellisTestParticipantLike;
    readonly deployment?: string;
    readonly sessionKeySeed?: string;
  }): Promise<TrellisTestServiceKey>;
  /** Creates app/client session-key material for public Trellis clients. */
  registerClient(args: {
    readonly name: string;
    readonly contract: TrellisTestClientParticipant;
    readonly sessionKeySeed?: string;
  }): Promise<TrellisTestClientKey>;
  /** Returns auth options and continuation handling for a registered client key. */
  clientAuth(key: TrellisTestClientKey): TrellisTestClientAuth;
  /** Connects an app/client participant through the generated client surface. */
  connectClient<TContract extends TrellisTestClientParticipant>(
    args: ClientOpts & {
      readonly name: string;
      readonly contract: TContract;
      readonly sessionKeySeed?: string;
    },
  ): Promise<TrellisTestConnectedClient<TContract>>;
  /** Polls until `fn` returns a truthy value. */
  waitFor<T>(
    fn: () =>
      | T
      | null
      | undefined
      | false
      | Promise<T | null | undefined | false>,
    opts?: WaitForOptions,
  ): Promise<T>;
  /** Flushes runtime transport work that should be visible before assertions. */
  flush(): Promise<void>;
  /** Seeds one raw auth connection-presence KV entry for malformed-entry tests. */
  seedRawAuthConnectionPresence?(
    args: TrellisTestRawAuthConnectionPresence,
  ): Promise<void>;
  /** Seeds one raw state KV entry in runtimes that support backing-store injection. */
  seedRawStateEntry?(args: TrellisTestRawStateEntry): Promise<void>;
  /** Publishes one synthetic immutable revocation in owned live runtimes. */
  publishAuthorizationRevocation?(
    contextDigest: string,
    value: Record<string, unknown>,
  ): Promise<void>;
  /** Observes raw NATS messages on a runtime-owned scratch NATS server. */
  startNatsMessageObserver?(
    subject: string,
    headerNames?: readonly string[],
  ): Promise<NatsMessageObserver>;
  /** Restarts only the Trellis control-plane process when the runtime supports it. */
  restartControlPlane?(): Promise<void>;
  /** Rotates the browser WebSocket endpoint when the runtime supports it. */
  rotateWebsocketProxy?(): Promise<[string, string]>;
  /** Stops runtime-owned resources when this runtime owns cleanup. */
  stop?(): Promise<void>;
};

/** Options accepted by `trellisIntegrationTest`. */
export type TrellisIntegrationTestOptions = {
  /** Stable matrix case id reported to the integration runner. */
  readonly caseId: string;
  /** Deno test name. */
  readonly name: string;
  /** Runtime isolation strategy for the test body. */
  readonly scope: TrellisIntegrationScope;
  /** Direct runtime startup options. Required unless shared-runtime mode is active. */
  readonly runtime?: TrellisIntegrationRuntimeOptions;
  /** Deno test resource sanitization override. */
  readonly sanitizeResources?: boolean;
  /** Deno test operation sanitization override. */
  readonly sanitizeOps?: boolean;
  /** Test body invoked with an isolated or attached Trellis runtime. */
  readonly fn: (runtime: TrellisIntegrationRuntime) => Promise<void>;
};

/** Case descriptor consumed by generic Trellis integration runners. */
export type TrellisIntegrationCase = {
  /** Stable case id used for test filters and deterministic scoping. */
  readonly id: string;
  /** Logical fixture or feature area this case belongs to. */
  readonly fixture: string;
  /** Test module path containing this case. */
  readonly file: string;
  /** Deno test name registered for this case. */
  readonly testName: string;
  /** Optional coverage tags used by runner filters. */
  readonly coverage?: readonly string[];
  /** Shared-host behavior or explicit process isolation. */
  readonly classification?: "shared" | "isolated-process";
  /** Required rationale for isolated-process cases. */
  readonly isolationReason?: string;
};
