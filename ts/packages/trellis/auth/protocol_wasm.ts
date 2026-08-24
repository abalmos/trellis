import init, {
  initSync,
  type SyncInitInput,
  type VerifiedAuthorizationContextHandle as WasmAuthorizationContextHandle,
} from "./protocol_wasm/trellis_protocol_wasm.js";
import * as protocolWasmModule from "./protocol_wasm/trellis_protocol_wasm.js";
import { PROTOCOL_WASM_BASE64 } from "./protocol_wasm/trellis_protocol_wasm_bytes.ts";
import { base64urlDecode, type JsonValue } from "./utils.ts";

type JsonObject = { [key: string]: JsonValue };

const protocolWasm = protocolWasmModule;

/** Verification policy accepted by the Rust authorization protocol. */
export type AuthorizationVerificationPolicy = {
  nowUnixSeconds: number;
  allowedClockSkewSeconds: number;
  maximumContextLifetimeSeconds: number;
  maximumContextBytes: number;
  maximumPermissions: number;
  maximumCapabilities: number;
  minimumManifestGeneration: number;
};

/** Context verification policy fields used to schedule refresh. */
export type AuthorizationContextVerificationPolicy =
  & AuthorizationVerificationPolicy
  & {
    refreshLeadSeconds: number;
    refreshJitterSeconds: number;
  };

/** Trust material and signed context supplied to a local verifier. */
export type AuthorizationContextVerificationInput = {
  root: unknown;
  manifest: unknown;
  context: unknown;
};

/** One exact API or participant-resource permission target. */
export type PermissionTarget =
  | {
    kind: "apiSurface";
    api: string;
    surface: "rpc" | "operation" | "event" | "feed" | "state";
    name: string;
  }
  | {
    kind: "participantResource";
    participant: string;
    resource: "kv" | "store" | "jobQueue" | "eventConsumer" | "state";
    name: string;
  }
  | {
    kind: "operationSignal";
    api: string;
    operation: string;
    signal: string;
  };

/** One exact machine-enforceable permission atom. */
export type PermissionAtom = {
  target: PermissionTarget;
  action:
    | "call"
    | "invoke"
    | "observe"
    | "cancel"
    | "control"
    | "publish"
    | "subscribe"
    | "read"
    | "write"
    | "delete"
    | "submit"
    | "process"
    | "consume";
};

/** Grant set projection returned by local authorization verification. */
export type GrantSet = {
  format: "trellis.grant-set.v1";
  permissions: PermissionAtom[];
};

/** Native participant resolution returned by the Rust protocol boundary. */
export type ResolvedParticipant = {
  apiArtifacts: Record<string, JsonObject>;
  apiDigests: Record<string, string>;
  participant: JsonObject;
  participantDigest: string;
  participantNeeds: JsonObject;
  participantNeedsDigest: string;
  requiredGrants: GrantSet;
  optionalGrants: GrantSet;
  authorityProposal: JsonObject;
};

/** Stable principal projection returned by the protocol verifier. */
export type AuthorizationPrincipal = {
  kind: "user" | "service" | "device";
  id: string;
};

/** Exact participant projection returned by the protocol verifier. */
export type AuthorizationParticipant = {
  kind: "service" | "app" | "device" | "agent";
  id: string;
  artifactDigest: string;
  needsDigest: string;
};

/** Durable authority reference bound into a signed context. */
export type AuthorizationAuthorityRef = {
  kind: "identity" | "deployment";
  id: string;
  version: number;
};

/** Complete verified context metadata returned by request/event verification. */
export type VerifiedAuthorizationContextProjection = {
  authority: string;
  authorityRef: AuthorizationAuthorityRef;
  principal: AuthorizationPrincipal;
  participant: AuthorizationParticipant;
  deploymentId: string | null;
  instanceId: string | null;
  issuerKeyId: string;
  sessionId: string;
  sessionKey: string;
  inboxPrefix: string;
  issuedAt: number;
  notBefore: number;
  expiresAt: number;
  grantSet: GrantSet;
  grantDigest: string;
  capabilities: string[];
  extensions: Record<string, unknown>;
  contextDigest: string;
};

/** Verified request caller projection. */
export type VerifiedAuthorizationRequestProjection =
  VerifiedAuthorizationContextProjection;

/** Verified event publisher projection. */
export type VerifiedAuthorizationEventPublisher = {
  kind: "user" | "service" | "device";
  deploymentId: string | null;
  instanceId: string | null;
  participantId: string;
  participantDigest: string;
  sessionId: string;
};

/** Verified event publisher projection. */
export type VerifiedAuthorizationEventProjection =
  & VerifiedAuthorizationContextProjection
  & {
    publisher: VerifiedAuthorizationEventPublisher;
  };

/** Stable error categories returned by local authorization verification. */
export type AuthorizationVerificationErrorCode =
  | "InvalidInput"
  | "SerializationError"
  | "InvalidFormat"
  | "UnsafeJsonInteger"
  | "InvalidEncoding"
  | "InvalidPublicKey"
  | "InvalidKeyId"
  | "InvalidSignature"
  | "WrongAuthority"
  | "UnknownCriticalExtension"
  | "NonCanonicalSet"
  | "InvalidValidityWindow"
  | "ManifestRollback"
  | "ManifestNotYetValid"
  | "ManifestExpired"
  | "IssuerNotListed"
  | "ContextNotYetValid"
  | "ContextExpired"
  | "ContextLifetimeExceeded"
  | "ContextOutlivesManifest"
  | "InvalidSessionKey"
  | "PermissionDenied"
  | "CapabilityDenied"
  | "ContextTooLarge"
  | "ProofIatOutOfRange"
  | "InvalidRequestProof"
  | "ReplySubjectMismatch"
  | "InvalidEventTime"
  | "InvalidEventProof"
  | "EventRevoked";

/** Stable structured local authorization failure. */
export type AuthorizationVerificationError = {
  code: AuthorizationVerificationErrorCode;
  path: string;
};

/** Result envelope returned by a local request authorization verifier. */
export type VerifyAuthorizationRequestResult =
  | ({ ok: true } & VerifiedAuthorizationRequestProjection)
  | { ok: false; error: AuthorizationVerificationError };

/** Result envelope returned by a local event authorization verifier. */
export type VerifyAuthorizationEventResult =
  | ({ ok: true } & VerifiedAuthorizationEventProjection)
  | { ok: false; error: AuthorizationVerificationError };

/** Arguments for local context-bound request authorization. */
export type VerifyAuthorizationRequestArgs = {
  contextHandle: AuthorizationContextHandle;
  subject: string;
  reply: string | null;
  payload: Uint8Array;
  iat: number;
  requestId: string;
  proof: string;
  requiredPermissions: PermissionAtom[];
  requiredCapabilities: string[];
  policy: AuthorizationVerificationPolicy;
};

/** Arguments for local context-bound event authorization. */
export type VerifyAuthorizationEventArgs = {
  contextHandle: AuthorizationContextHandle;
  subject: string;
  payload: Uint8Array;
  eventId: string;
  eventTime: string;
  proof: string;
  requiredPermissions: PermissionAtom[];
  requiredCapabilities: string[];
  policy: AuthorizationVerificationPolicy;
  revokedAt?: number | null;
};

/** Projection returned after verifying a complete context trust chain. */
export type VerifiedAuthorizationContextTokenProjection = {
  authority: string;
  rootKeyId: string;
  rootDigest: string;
  manifestDigest: string;
  contextDigest: string;
  manifestGeneration: number;
  refreshAt: number;
  context: Record<string, unknown> & {
    issuedAt: number;
    notBefore: number;
    expiresAt: number;
  };
};

/** Opaque Rust/WASM verification state for one authorization context. */
export type AuthorizationContextHandle = WasmAuthorizationContextHandle;

let initialized: Promise<void> | undefined;
let initializedSync = false;

async function wasmBytes(): Promise<Uint8Array> {
  const url = new URL(
    "./protocol_wasm/trellis_protocol_wasm_bg.wasm",
    import.meta.url,
  );
  const runtime = globalThis as Record<string, unknown>;
  const deno = runtime["De" + "no"] as
    | { readFile(path: URL): Promise<Uint8Array> }
    | undefined;
  if (deno) return await deno.readFile(url);
  const process = runtime["pro" + "cess"] as
    | {
      versions?: { node?: string };
      getBuiltinModule?: (name: string) => {
        promises: { readFile(path: URL): Promise<Uint8Array> };
      };
    }
    | undefined;
  if (process?.versions?.node && process.getBuiltinModule) {
    return new Uint8Array(
      await process.getBuiltinModule("fs").promises.readFile(url),
    );
  }
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(
      `authorization protocol WASM returned HTTP ${response.status}`,
    );
  }
  return new Uint8Array(await response.arrayBuffer());
}

async function initialize(): Promise<void> {
  initialized ??= (async () => {
    await init({ module_or_path: await wasmBytes() });
  })();
  await initialized;
}

function initializeSync(): void {
  if (initializedSync) return;
  const url = new URL(
    "./protocol_wasm/trellis_protocol_wasm_bg.wasm",
    import.meta.url,
  );
  const runtime = globalThis as Record<string, unknown>;
  const deno = runtime["De" + "no"] as
    | { readFileSync(path: URL): Uint8Array }
    | undefined;
  if (deno) {
    initSync({ module: deno.readFileSync(url) as SyncInitInput });
    initializedSync = true;
    return;
  }
  const process = runtime["pro" + "cess"] as
    | {
      versions?: { node?: string };
      getBuiltinModule?: (name: string) => {
        readFileSync(path: URL): Uint8Array;
      };
    }
    | undefined;
  if (process?.versions?.node && process.getBuiltinModule) {
    initSync({
      module: process.getBuiltinModule("fs").readFileSync(url) as SyncInitInput,
    });
    initializedSync = true;
    return;
  }
  const binary = atob(PROTOCOL_WASM_BASE64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  initSync({ module: bytes as SyncInitInput });
  initializedSync = true;
}

/** Resolve a native participant through the authoritative Rust protocol resolver. */
export function resolveParticipantV1WasmSync(args: {
  participant: unknown;
  apis: Record<string, unknown>;
}): ResolvedParticipant {
  initializeSync();
  return JSON.parse(
    protocolWasm.resolve_participant(
      JSON.stringify(args.participant),
      JSON.stringify(args.apis),
    ),
  ) as ResolvedParticipant;
}

/** Verify a complete signed authorization context through Rust/WASM. */
export async function verifyAuthorizationContextWasm(args: {
  root: unknown;
  manifest: unknown;
  context: unknown;
  policy: AuthorizationContextVerificationPolicy;
}): Promise<VerifiedAuthorizationContextTokenProjection> {
  const { handle, verified } = await createAuthorizationContextHandleWasm(args);
  handle.free();
  return verified;
}

/** Verify and retain one authorization context for repeated proof checks. */
export async function createAuthorizationContextHandleWasm(args: {
  root: unknown;
  manifest: unknown;
  context: unknown;
  policy: AuthorizationContextVerificationPolicy;
  historical?: boolean;
}): Promise<{
  handle: AuthorizationContextHandle;
  verified: VerifiedAuthorizationContextTokenProjection;
}> {
  await initialize();
  const handle = protocolWasm.create_authorization_context_handle(
    JSON.stringify(args.root),
    JSON.stringify(args.manifest),
    JSON.stringify(args.context),
    JSON.stringify(wasmVerificationPolicy(args.policy)),
    args.historical ?? false,
  );
  try {
    const result = JSON.parse(
      handle.projection(),
    ) as VerifiedAuthorizationContextTokenProjection;
    const jitter = contextJitter(
      result.contextDigest,
      args.policy.refreshJitterSeconds,
    );
    return {
      handle,
      verified: {
        ...result,
        refreshAt: result.context.expiresAt - args.policy.refreshLeadSeconds -
          jitter,
      },
    };
  } catch (error) {
    handle.free();
    throw error;
  }
}

/** Require an opaque verified context to be currently eligible. */
export function assertAuthorizationContextHandleCurrentWasm(
  handle: AuthorizationContextHandle,
  policy: AuthorizationContextVerificationPolicy,
): void {
  handle.assert_current(JSON.stringify(wasmVerificationPolicy(policy)));
}

/** Verify a root-signed issuer manifest through Rust/WASM. */
export async function verifyAuthorizationManifestWasm(args: {
  root: unknown;
  manifest: unknown;
  policy: AuthorizationVerificationPolicy;
}): Promise<{
  authority: string;
  rootKeyId: string;
  generation: number;
  digest: string;
  issuerKeyIds: string[];
}> {
  await initialize();
  return JSON.parse(
    protocolWasm.verify_authorization_manifest(
      JSON.stringify(args.root),
      JSON.stringify(args.manifest),
      JSON.stringify(wasmVerificationPolicy(args.policy)),
    ),
  );
}

/** Verify one context-bound request proof using actual received request bytes. */
export async function verifyAuthorizationRequestWasm(
  args: VerifyAuthorizationRequestArgs,
): Promise<VerifyAuthorizationRequestResult> {
  await initialize();
  const { contextHandle, payload, ...input } = args;
  return JSON.parse(
    protocolWasm.verify_authorization_request(
      contextHandle,
      JSON.stringify({
        ...input,
        policy: wasmVerificationPolicy(args.policy),
      }),
      payload,
    ),
  ) as VerifyAuthorizationRequestResult;
}

/** Verify one context-bound event proof, including historical time/revocation checks. */
export async function verifyAuthorizationEventWasm(
  args: VerifyAuthorizationEventArgs,
): Promise<VerifyAuthorizationEventResult> {
  await initialize();
  const { contextHandle, payload, ...input } = args;
  return JSON.parse(
    protocolWasm.verify_authorization_event(
      contextHandle,
      JSON.stringify({
        ...input,
        policy: wasmVerificationPolicy(args.policy),
      }),
      payload,
    ),
  ) as VerifyAuthorizationEventResult;
}

function wasmVerificationPolicy(
  policy: AuthorizationVerificationPolicy,
): AuthorizationVerificationPolicy {
  return {
    nowUnixSeconds: policy.nowUnixSeconds,
    allowedClockSkewSeconds: policy.allowedClockSkewSeconds,
    maximumContextLifetimeSeconds: policy.maximumContextLifetimeSeconds,
    maximumContextBytes: policy.maximumContextBytes,
    maximumPermissions: policy.maximumPermissions,
    maximumCapabilities: policy.maximumCapabilities,
    minimumManifestGeneration: policy.minimumManifestGeneration,
  };
}

function contextJitter(contextDigest: string, maximum: number): number {
  const bytes = base64urlDecode(contextDigest);
  let value = 0n;
  for (const byte of bytes.slice(0, 8)) value = (value << 8n) | BigInt(byte);
  return Number(value % BigInt(maximum + 1));
}
