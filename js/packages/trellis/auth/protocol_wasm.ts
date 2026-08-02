import init, * as protocolWasmModule from "./protocol_wasm/trellis_protocol_wasm.js";
import { base64urlDecode } from "./utils.ts";

type ProtocolWasmModule = typeof protocolWasmModule & {
  verify_authorization_event_v2(inputJson: string): string;
  verify_authorization_request_v2(inputJson: string): string;
};

const protocolWasm = protocolWasmModule as ProtocolWasmModule;

/** Verification policy accepted by the Rust authorization protocol. */
export type AuthorizationVerificationPolicyV1 = {
  nowUnixSeconds: number;
  allowedClockSkewSeconds: number;
  maximumContextLifetimeSeconds: number;
  maximumContextBytes: number;
  maximumPermissions: number;
  maximumCapabilities: number;
  minimumManifestGeneration: number;
};

/** Context verification policy fields used to schedule refresh. */
export type AuthorizationContextVerificationPolicyV1 =
  & AuthorizationVerificationPolicyV1
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
export type PermissionTargetV1 =
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
export type PermissionAtomV1 = {
  target: PermissionTargetV1;
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
export type GrantSetV1 = {
  format: "trellis.grant-set.v1";
  permissions: PermissionAtomV1[];
};

/** Stable principal projection returned by the protocol verifier. */
export type AuthorizationPrincipalV1 = {
  kind: "user" | "service" | "device";
  id: string;
};

/** Exact participant projection returned by the protocol verifier. */
export type AuthorizationParticipantV1 = {
  kind: "service" | "app" | "device" | "agent";
  id: string;
  artifactDigest: string;
  needsDigest: string;
};

/** Durable authority reference bound into a signed context. */
export type AuthorizationAuthorityRefV1 = {
  kind: "identity" | "deployment";
  id: string;
  version: number;
};

/** Complete verified context metadata returned by request/event verification. */
export type VerifiedAuthorizationContextProjection = {
  authority: string;
  authorityRef: AuthorizationAuthorityRefV1;
  principal: AuthorizationPrincipalV1;
  participant: AuthorizationParticipantV1;
  deploymentId: string | null;
  instanceId: string | null;
  issuerKeyId: string;
  sessionId: string;
  sessionKey: string;
  inboxPrefix: string;
  issuedAt: number;
  notBefore: number;
  expiresAt: number;
  grantSet: GrantSetV1;
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
export type VerifyAuthorizationRequestV2Result =
  | ({ ok: true } & VerifiedAuthorizationRequestProjection)
  | { ok: false; error: AuthorizationVerificationError };

/** Result envelope returned by a local event authorization verifier. */
export type VerifyAuthorizationEventV2Result =
  | ({ ok: true } & VerifiedAuthorizationEventProjection)
  | { ok: false; error: AuthorizationVerificationError };

/** Arguments for local context-bound request authorization. */
export type VerifyAuthorizationRequestV2Args =
  & AuthorizationContextVerificationInput
  & {
    subject: string;
    reply: string | null;
    payload: Uint8Array;
    iat: number;
    requestId: string;
    proof: string;
    requiredPermissions: PermissionAtomV1[];
    requiredCapabilities: string[];
    policy: AuthorizationVerificationPolicyV1;
  };

/** Arguments for local context-bound event authorization. */
export type VerifyAuthorizationEventV2Args =
  & AuthorizationContextVerificationInput
  & {
    subject: string;
    payload: Uint8Array;
    eventId: string;
    eventTime: string;
    proof: string;
    requiredPermissions: PermissionAtomV1[];
    requiredCapabilities: string[];
    policy: AuthorizationVerificationPolicyV1;
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

let initialized: Promise<void> | undefined;

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

/** Verify a complete signed authorization context through Rust/WASM. */
export async function verifyAuthorizationContextWasm(args: {
  root: unknown;
  manifest: unknown;
  context: unknown;
  policy: AuthorizationContextVerificationPolicyV1;
}): Promise<VerifiedAuthorizationContextTokenProjection> {
  await initialize();
  const result = JSON.parse(
    protocolWasm.verify_authorization_context(
      JSON.stringify(args.root),
      JSON.stringify(args.manifest),
      JSON.stringify(args.context),
      JSON.stringify(wasmVerificationPolicy(args.policy)),
    ),
  ) as VerifiedAuthorizationContextTokenProjection;
  const jitter = contextJitter(
    result.contextDigest,
    args.policy.refreshJitterSeconds,
  );
  return {
    ...result,
    refreshAt: result.context.expiresAt - args.policy.refreshLeadSeconds -
      jitter,
  };
}

/** Verify a root-signed issuer manifest through Rust/WASM. */
export async function verifyAuthorizationManifestWasm(args: {
  root: unknown;
  manifest: unknown;
  policy: AuthorizationVerificationPolicyV1;
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
export async function verifyAuthorizationRequestV2Wasm(
  args: VerifyAuthorizationRequestV2Args,
): Promise<VerifyAuthorizationRequestV2Result> {
  await initialize();
  const { context, ...input } = args;
  return JSON.parse(
    protocolWasm.verify_authorization_request_v2(
      JSON.stringify({
        ...input,
        policy: wasmVerificationPolicy(args.policy),
        context,
        payload: Array.from(args.payload),
      }),
    ),
  ) as VerifyAuthorizationRequestV2Result;
}

/** Verify one context-bound event proof, including historical time/revocation checks. */
export async function verifyAuthorizationEventV2Wasm(
  args: VerifyAuthorizationEventV2Args,
): Promise<VerifyAuthorizationEventV2Result> {
  await initialize();
  const { context, ...input } = args;
  return JSON.parse(
    protocolWasm.verify_authorization_event_v2(
      JSON.stringify({
        ...input,
        policy: wasmVerificationPolicy(args.policy),
        context,
        payload: Array.from(args.payload),
      }),
    ),
  ) as VerifyAuthorizationEventV2Result;
}

function wasmVerificationPolicy(
  policy: AuthorizationVerificationPolicyV1,
): AuthorizationVerificationPolicyV1 {
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
