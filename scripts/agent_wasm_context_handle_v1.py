from pathlib import Path
import re


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


# Rust/WASM: retain the exact verified Rust context behind an opaque wasm-bindgen
# object and delete raw per-message trust-chain reconstruction exports.
path = "rust/crates/protocol-wasm/src/lib.rs"
text = read(path)
text = text.replace(
    "SessionProofPolicyV1, SessionProofV1, VerifiedAuthorizationContextV1,\n",
    "SessionProofPolicyV1, SessionProofV1, VerifiedAuthorizationContextV1,\n"
    "    VerifiedAuthorizationIssuerManifestV1,\n",
    1,
)
text = text.replace(
    '''struct WireAuthorizationRequestV2 {
    root: Value,
    manifest: Value,
    context: Value,
    subject: String,''',
    '''struct WireAuthorizationRequestV2 {
    subject: String,''',
    1,
)
text = text.replace(
    '''struct WireAuthorizationEventV2 {
    root: Value,
    manifest: Value,
    context: Value,
    subject: String,''',
    '''struct WireAuthorizationEventV2 {
    subject: String,''',
    1,
)

old_bundle = '''#[allow(clippy::result_large_err)] // Protocol errors are serialized immediately at the WASM boundary.
fn verify_context_bundle(
    root_value: &Value,
    manifest_value: &Value,
    context_value: &Value,
    policy: &AuthorizationVerificationPolicyV1,
    historical: bool,
) -> Result<VerifiedAuthorizationContextV1, ProtocolError> {
    let root = AuthorizationTrustRootV1::parse(root_value)?;
    let manifest = parse_issuer_manifest_v1(manifest_value)?;
    let context = parse_authorization_context_v1(context_value)?;
    let verification_policy = if historical {
        let mut policy = policy.clone();
        policy.now_unix_seconds = context.unsigned.expires_at;
        policy
    } else {
        policy.clone()
    };
    let verified_manifest = verify_issuer_manifest_v1(&root, &manifest, &verification_policy)?;
    verify_authorization_context_v1(&root, &verified_manifest, &context, &verification_policy)
}
'''
new_bundle = '''#[allow(clippy::result_large_err)] // Protocol errors are serialized immediately at the WASM boundary.
fn verify_context_bundle(
    root_value: &Value,
    manifest_value: &Value,
    context_value: &Value,
    policy: &AuthorizationVerificationPolicyV1,
    historical: bool,
) -> Result<
    (
        AuthorizationTrustRootV1,
        VerifiedAuthorizationIssuerManifestV1,
        VerifiedAuthorizationContextV1,
    ),
    ProtocolError,
> {
    let root = AuthorizationTrustRootV1::parse(root_value)?;
    let manifest = parse_issuer_manifest_v1(manifest_value)?;
    let context = parse_authorization_context_v1(context_value)?;
    let verification_policy = if historical {
        let mut policy = policy.clone();
        policy.now_unix_seconds = context.unsigned.expires_at;
        policy
    } else {
        policy.clone()
    };
    let verified_manifest = verify_issuer_manifest_v1(&root, &manifest, &verification_policy)?;
    let verified_context =
        verify_authorization_context_v1(&root, &verified_manifest, &context, &verification_policy)?;
    Ok((root, verified_manifest, verified_context))
}

#[allow(clippy::result_large_err)] // Protocol errors are serialized immediately at the WASM boundary.
fn verified_context_token_projection(
    root: &AuthorizationTrustRootV1,
    manifest: &VerifiedAuthorizationIssuerManifestV1,
    context: &VerifiedAuthorizationContextV1,
) -> Result<Value, ProtocolError> {
    Ok(json!({
        "authority": root.authority(),
        "rootKeyId": root.key_id(),
        "rootDigest": root.digest()?,
        "manifestDigest": manifest.digest()?,
        "contextDigest": context.context_digest(),
        "context": context.signed_context(),
        "manifestGeneration": manifest.generation(),
    }))
}
'''
if text.count(old_bundle) != 1:
    raise RuntimeError(f"verify_context_bundle anchor changed: {text.count(old_bundle)}")
text = text.replace(old_bundle, new_bundle, 1)

old_context_export = re.compile(
    r'''/// Verify a root, issuer manifest, and signed authorization context JSON value\.\n'''
    r'''#\[wasm_bindgen\]\n'''
    r'''pub fn verify_authorization_context\([\s\S]*?\n\}\n\n'''
    r'''/// Verify a root-signed issuer manifest and return its verified projection\.'''
)
match = old_context_export.search(text)
if not match:
    raise RuntimeError("verify_authorization_context export anchor changed")
replacement = '''/// Authorization context retained inside the WASM instance after its complete trust chain verifies.
#[wasm_bindgen]
pub struct VerifiedAuthorizationContextHandle {
    context: VerifiedAuthorizationContextV1,
    projection_json: String,
}

#[wasm_bindgen]
impl VerifiedAuthorizationContextHandle {
    /// Return the stable verified trust/context projection for this retained context.
    pub fn projection(&self) -> String {
        self.projection_json.clone()
    }

    /// Verify one request proof against this retained verified context.
    pub fn verify_request_v2(&self, request_json: &str) -> String {
        let input: WireAuthorizationRequestV2 = match serde_json::from_str(request_json) {
            Ok(input) => input,
            Err(_) => return input_error_result(""),
        };
        request_result(&self.context, input)
    }

    /// Verify one event proof against this retained verified context.
    pub fn verify_event_v2(&self, event_json: &str) -> String {
        let input: WireAuthorizationEventV2 = match serde_json::from_str(event_json) {
            Ok(input) => input,
            Err(_) => return input_error_result(""),
        };
        event_result(&self.context, input)
    }
}

/// Verify and retain a complete authorization context trust chain inside WASM.
#[wasm_bindgen]
pub fn retain_authorization_context(
    root_json: &str,
    manifest_json: &str,
    context_json: &str,
    policy_json: &str,
    historical: bool,
) -> Result<VerifiedAuthorizationContextHandle, JsError> {
    let policy = authorization_verification_policy(policy_json)?;
    let root_value: Value =
        serde_json::from_str(root_json).map_err(|error| JsError::new(&error.to_string()))?;
    let manifest_value: Value =
        serde_json::from_str(manifest_json).map_err(|error| JsError::new(&error.to_string()))?;
    let context_value: Value =
        serde_json::from_str(context_json).map_err(|error| JsError::new(&error.to_string()))?;
    let (root, manifest, context) = verify_context_bundle(
        &root_value,
        &manifest_value,
        &context_value,
        &policy,
        historical,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    let projection_json = serde_json::to_string(
        &verified_context_token_projection(&root, &manifest, &context)
            .map_err(|error| JsError::new(&error.to_string()))?,
    )
    .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(VerifiedAuthorizationContextHandle {
        context,
        projection_json,
    })
}

/// Verify a root, issuer manifest, and signed authorization context JSON value.
#[wasm_bindgen]
pub fn verify_authorization_context(
    root_json: &str,
    manifest_json: &str,
    context_json: &str,
    policy_json: &str,
) -> Result<String, JsError> {
    Ok(retain_authorization_context(
        root_json,
        manifest_json,
        context_json,
        policy_json,
        false,
    )?
    .projection())
}

/// Verify a root-signed issuer manifest and return its verified projection.'''
text = text[: match.start()] + replacement + text[match.end():]

# Request/event result helpers now consume an already-verified retained context.
text, count = re.subn(
    r'''fn request_result\(input: WireAuthorizationRequestV2\) -> String \{\n'''
    r'''    let policy = match authorization_verification_policy_from_wire\(&input\.policy\) \{[\s\S]*?'''
    r'''    let proof = match AuthorizationRequestProofV2::parse\(input\.proof\) \{''',
    '''fn request_result(
    context: &VerifiedAuthorizationContextV1,
    input: WireAuthorizationRequestV2,
) -> String {
    let policy = match authorization_verification_policy_from_wire(&input.policy) {
        Ok(policy) => policy,
        Err(_) => return input_error_result("/policy"),
    };
    let proof = match AuthorizationRequestProofV2::parse(input.proof) {''',
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"request_result trust-chain block changed: {count}")
text = text.replace(
    '''        &context,
        &input.subject,''',
    '''        context,
        &input.subject,''',
    1,
)

text, count = re.subn(
    r'''fn event_result\(input: WireAuthorizationEventV2\) -> String \{\n'''
    r'''    let policy = match authorization_verification_policy_from_wire\(&input\.policy\) \{[\s\S]*?'''
    r'''    let proof = match AuthorizationEventProofV2::parse\(input\.proof\) \{''',
    '''fn event_result(
    context: &VerifiedAuthorizationContextV1,
    input: WireAuthorizationEventV2,
) -> String {
    let policy = match authorization_verification_policy_from_wire(&input.policy) {
        Ok(policy) => policy,
        Err(_) => return input_error_result("/policy"),
    };
    let proof = match AuthorizationEventProofV2::parse(input.proof) {''',
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"event_result trust-chain block changed: {count}")
# The second remaining call site begins with &context.
idx = text.index("fn event_result(")
post = text[idx:]
if post.count("        &context,\n        &input.subject,") != 1:
    raise RuntimeError("event_result verified context call changed")
post = post.replace(
    "        &context,\n        &input.subject,",
    "        context,\n        &input.subject,",
    1,
)
text = text[:idx] + post

# Delete the parallel raw request/event WASM exports. The retained handle is the only
# per-message authorization boundary.
text, count = re.subn(
    r'''\n/// Verify one context-bound authorization request proof from a JSON argument\.[\s\S]*?'''
    r'''pub fn verify_authorization_event_v2\(event_json: &str\) -> String \{\n'''
    r'''    let input: WireAuthorizationEventV2 = match serde_json::from_str\(event_json\) \{\n'''
    r'''        Ok\(input\) => input,\n'''
    r'''        Err\(_\) => return input_error_result\(""\),\n'''
    r'''    \};\n'''
    r'''    event_result\(input\)\n'''
    r'''\}\n''',
    "\n",
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"raw request/event WASM exports changed: {count}")

for token in (
    "pub fn verify_authorization_request_v2(request_json",
    "pub fn verify_authorization_event_v2(event_json",
):
    if token in text:
        raise RuntimeError(f"stale raw per-message export survived: {token}")
write(path, text)

# TypeScript WASM wrapper: expose one opaque retained object and no raw
# request/event function that accepts root/manifest/context material.
path = "ts/packages/trellis/auth/protocol_wasm.ts"
text = read(path)
old_module = '''type ProtocolWasmModule = typeof protocolWasmModule & {
  resolve_participant_v1(participantJson: string, apisJson: string): string;
  verify_authorization_event_v2(inputJson: string): string;
  verify_authorization_request_v2(inputJson: string): string;
};
'''
new_module = '''type RetainedAuthorizationContextWasm = {
  projection(): string;
  verify_request_v2(inputJson: string): string;
  verify_event_v2(inputJson: string): string;
  free(): void;
};

type ProtocolWasmModule = typeof protocolWasmModule & {
  resolve_participant_v1(participantJson: string, apisJson: string): string;
  retain_authorization_context(
    rootJson: string,
    manifestJson: string,
    contextJson: string,
    policyJson: string,
    historical: boolean,
  ): RetainedAuthorizationContextWasm;
};
'''
if text.count(old_module) != 1:
    raise RuntimeError("protocol WASM module type anchor changed")
text = text.replace(old_module, new_module, 1)
text = text.replace(
    '''export type VerifyAuthorizationRequestV2Args =
  & AuthorizationContextVerificationInput
  & {''',
    '''export type VerifyAuthorizationRequestV2Args = {''',
    1,
)
text = text.replace(
    '''    policy: AuthorizationVerificationPolicyV1;
  };''',
    '''    policy: AuthorizationVerificationPolicyV1;
  };''',
    1,
)
# Remove the leading intersection from the event args independently.
text = text.replace(
    '''export type VerifyAuthorizationEventV2Args =
  & AuthorizationContextVerificationInput
  & {''',
    '''export type VerifyAuthorizationEventV2Args = {''',
    1,
)

handle_type_marker = '''export type VerifiedAuthorizationContextTokenProjection = {
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
'''
handle_type = handle_type_marker + '''
/** Opaque verified context retained inside the Rust/WASM verifier. */
export type VerifiedAuthorizationContextHandle = {
  readonly projection: VerifiedAuthorizationContextTokenProjection;
  verifyRequest(args: VerifyAuthorizationRequestV2Args): VerifyAuthorizationRequestV2Result;
  verifyEvent(args: VerifyAuthorizationEventV2Args): VerifyAuthorizationEventV2Result;
  dispose(): void;
};
'''
if text.count(handle_type_marker) != 1:
    raise RuntimeError("verified context projection type anchor changed")
text = text.replace(handle_type_marker, handle_type, 1)

verify_context_start = text.index("/** Verify a complete signed authorization context through Rust/WASM. */")
manifest_start = text.index("/** Verify a root-signed issuer manifest through Rust/WASM. */", verify_context_start)
new_context_wrapper = r'''/** Verify and retain a signed authorization context inside Rust/WASM. */
export async function retainVerifiedAuthorizationContextWasm(args: {
  root: unknown;
  manifest: unknown;
  context: unknown;
  policy: AuthorizationContextVerificationPolicyV1;
  historical?: boolean;
}): Promise<VerifiedAuthorizationContextHandle> {
  await initialize();
  const retained = protocolWasm.retain_authorization_context(
    JSON.stringify(args.root),
    JSON.stringify(args.manifest),
    JSON.stringify(args.context),
    JSON.stringify(wasmVerificationPolicy(args.policy)),
    args.historical ?? false,
  );
  const projection = JSON.parse(
    retained.projection(),
  ) as Omit<VerifiedAuthorizationContextTokenProjection, "refreshAt">;
  const verifiedProjection: VerifiedAuthorizationContextTokenProjection = {
    ...projection,
    refreshAt: projection.context.expiresAt - args.policy.refreshLeadSeconds -
      contextJitter(
        projection.contextDigest,
        args.policy.refreshJitterSeconds,
      ),
  };
  let disposed = false;
  const requireLive = () => {
    if (disposed) throw new Error("verified authorization context handle is disposed");
  };
  return {
    projection: verifiedProjection,
    verifyRequest(request) {
      requireLive();
      return JSON.parse(
        retained.verify_request_v2(JSON.stringify({
          ...request,
          policy: wasmVerificationPolicy(request.policy),
          payload: Array.from(request.payload),
        })),
      ) as VerifyAuthorizationRequestV2Result;
    },
    verifyEvent(event) {
      requireLive();
      return JSON.parse(
        retained.verify_event_v2(JSON.stringify({
          ...event,
          policy: wasmVerificationPolicy(event.policy),
          payload: Array.from(event.payload),
        })),
      ) as VerifyAuthorizationEventV2Result;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      retained.free();
    },
  };
}

/** Verify a complete signed authorization context through Rust/WASM. */
export async function verifyAuthorizationContextWasm(args: {
  root: unknown;
  manifest: unknown;
  context: unknown;
  policy: AuthorizationContextVerificationPolicyV1;
}): Promise<VerifiedAuthorizationContextTokenProjection> {
  const retained = await retainVerifiedAuthorizationContextWasm(args);
  try {
    return structuredClone(retained.projection);
  } finally {
    retained.dispose();
  }
}

'''
text = text[:verify_context_start] + new_context_wrapper + text[manifest_start:]

# Delete raw wrapper functions that accepted full context verification input per message.
start = text.index("/** Verify one context-bound request proof using actual received request bytes. */")
end = text.index("function wasmVerificationPolicy(", start)
text = text[:start] + text[end:]
for token in (
    "verifyAuthorizationRequestV2Wasm",
    "verifyAuthorizationEventV2Wasm",
    "protocolWasm.verify_authorization_request_v2",
    "protocolWasm.verify_authorization_event_v2",
):
    if token in text:
        raise RuntimeError(f"stale raw TS per-message verifier survived: {token}")
write(path, text)

# Provider cache: retain one verified WASM context per current entry and release it
# on every cache invalidation boundary. Historical old-generation handles are one-shot.
path = "ts/packages/trellis/auth/authorization/provider_cache.ts"
text = read(path)
old_import = '''import {
  type AuthorizationContextVerificationPolicyV1,
  type AuthorizationVerificationErrorCode,
  type VerifiedAuthorizationContextTokenProjection,
  verifyAuthorizationContextWasm,
  type VerifyAuthorizationEventV2Args,
  type VerifyAuthorizationEventV2Result,
  verifyAuthorizationEventV2Wasm,
  verifyAuthorizationManifestWasm,
  type VerifyAuthorizationRequestV2Args,
  type VerifyAuthorizationRequestV2Result,
  verifyAuthorizationRequestV2Wasm,
} from "../protocol_wasm.ts";'''
new_import = '''import {
  type AuthorizationContextVerificationPolicyV1,
  type AuthorizationVerificationErrorCode,
  retainVerifiedAuthorizationContextWasm,
  type VerifiedAuthorizationContextHandle,
  type VerifiedAuthorizationContextTokenProjection,
  type VerifyAuthorizationEventV2Result,
  verifyAuthorizationManifestWasm,
  type VerifyAuthorizationRequestV2Result,
} from "../protocol_wasm.ts";'''
if text.count(old_import) != 1:
    raise RuntimeError("provider cache WASM import anchor changed")
text = text.replace(old_import, new_import, 1)
text = text.replace(
    "  verified?: VerifiedAuthorizationContextTokenProjection;",
    "  verified?: VerifiedAuthorizationContextHandle;",
    1,
)
text = text.replace(
    "        verified: structuredClone(material.verified),\n",
    "",
    1,
)
text = text.replace(
    '''    this.#readyWaiters.clear();
  }
''',
    '''    this.#readyWaiters.clear();
    this.#clearContexts();
  }
''',
    1,
)
text = text.replace(
    '''    return structuredClone(
      await this.#ensureVerified(entry, false, this.#now()),
    );''',
    '''    return structuredClone(
      (await this.#ensureVerified(entry, false, this.#now())).projection,
    );''',
    1,
)

request_start = text.index("  /** Verify a presented request-v2 proof with exact route permissions. */")
event_comment = text.index("  /** Verify a presented event-v2 proof with exact publish permissions. */", request_start)
request_block = r'''  /** Verify a presented request-v2 proof with exact route permissions. */
  async verifyRequestV2(
    request: AuthorizationProviderRequestV2,
  ): Promise<VerifyAuthorizationRequestV2Result> {
    let retained: VerifiedAuthorizationContextHandle | undefined;
    let entry: ProviderContextEntry | undefined;
    try {
      this.#requireHealthy();
      entry = await this.#resolveEntry(request.contextDigest);
      retained = await this.#ensureVerified(entry, false, this.#now());
      const result = retained.verifyRequest({
        subject: request.subject,
        reply: request.reply,
        payload: new Uint8Array(request.payload),
        iat: request.iat,
        requestId: request.requestId,
        proof: request.proof,
        requiredPermissions: structuredClone(request.requiredPermissions),
        requiredCapabilities: [...request.requiredCapabilities],
        policy: this.#policyFor(entry, this.#now()),
      });
      if (!result.ok) return structuredClone(result);
      if (this.#revocationEvidence(entry.contextDigest) !== undefined) {
        return providerRequestFailure(
          "PermissionDenied",
          "/authorization-context",
        );
      }
      return structuredClone(result);
    } catch {
      return providerRequestFailure("InvalidInput", "/authorization-context");
    } finally {
      if (retained && entry?.verified !== retained) retained.dispose();
    }
  }

'''
text = text[:request_start] + request_block + text[event_comment:]

event_start = text.index("  /** Verify a presented event-v2 proof with exact publish permissions. */")
run_start = text.index("  async #run(): Promise<void> {", event_start)
event_block = r'''  /** Verify a presented event-v2 proof with exact publish permissions. */
  async verifyEventV2(
    event: AuthorizationProviderEventV2,
  ): Promise<VerifyAuthorizationEventV2Result> {
    let retained: VerifiedAuthorizationContextHandle | undefined;
    let entry: ProviderContextEntry | undefined;
    try {
      this.#requireHealthy();
      parseEventTime(event.eventTime);
      entry = await this.#resolveEntry(event.contextDigest);
      const revokedAt = this.#revocationEvidence(entry.contextDigest);
      retained = await this.#ensureVerified(entry, true, this.#now());
      const result = retained.verifyEvent({
        subject: event.subject,
        payload: new Uint8Array(event.payload),
        eventId: event.eventId,
        eventTime: event.eventTime,
        proof: event.proof,
        requiredPermissions: structuredClone(event.requiredPermissions),
        requiredCapabilities: [...event.requiredCapabilities],
        policy: this.#policyFor(entry, this.#now(), true),
        revokedAt: revokedAt ?? null,
      });
      return structuredClone(result);
    } catch (error) {
      if (error instanceof AuthorizationProviderUnavailableError) throw error;
      return providerEventFailure("InvalidInput", "/authorization-context");
    } finally {
      if (retained && entry?.verified !== retained) retained.dispose();
    }
  }

'''
text = text[:event_start] + event_block + text[run_start:]

text = text.replace("    this.#contexts.clear();", "    this.#clearContexts();", 1)

old_evict = '''  #evictExpiredContexts(): void {
    const now = this.#now();
    for (const [digest, entry] of this.#contexts) {
      if (entry.retainedUntil <= now) this.#contexts.delete(digest);
    }
  }
'''
new_evict = '''  #evictExpiredContexts(): void {
    const now = this.#now();
    for (const [digest, entry] of this.#contexts) {
      if (entry.retainedUntil > now) continue;
      this.#releaseEntry(entry);
      this.#contexts.delete(digest);
    }
  }
'''
if text.count(old_evict) != 1:
    raise RuntimeError("provider expiry eviction anchor changed")
text = text.replace(old_evict, new_evict, 1)

ensure_start = text.index("  async #ensureVerified(")
chain_start = text.index("  async #resolveChain(", ensure_start)
new_ensure = r'''  async #ensureVerified(
    entry: ProviderContextEntry,
    historical: boolean,
    verificationTime: number,
  ): Promise<VerifiedAuthorizationContextHandle> {
    if (!historical && entry.epoch !== this.#manifestEpoch) {
      throw new AuthorizationProviderUnavailableError(
        "authorization manifest advanced during context verification",
      );
    }
    if (
      !historical &&
      entry.manifestGeneration !== this.#currentManifest?.pointer.generation
    ) {
      throw new Error("authorization context manifest is not current");
    }
    const current = entry.manifestGeneration ===
        this.#currentManifest?.pointer.generation &&
      entry.epoch === this.#manifestEpoch;
    const existing = current ? entry.verified : undefined;
    if (existing) return existing;
    const chain = await this.#resolveChain(entry);
    const policy = this.#policyFor(entry, verificationTime, historical);
    const retained = await retainVerifiedAuthorizationContextWasm({
      root: entry.root,
      manifest: chain.value,
      context: entry.context,
      policy,
      historical,
    });
    const verified = retained.projection;
    if ((!historical && entry.epoch !== this.#manifestEpoch) || this.#stopped) {
      retained.dispose();
      throw new AuthorizationProviderUnavailableError(
        "authorization registry changed during verification",
      );
    }
    if (
      verified.contextDigest !== entry.contextDigest ||
      (chain.pointer.digest !== "" &&
        verified.manifestDigest !== chain.pointer.digest) ||
      verified.manifestGeneration !== chain.pointer.generation
    ) {
      retained.dispose();
      throw new Error("authorization registry trust identity mismatch");
    }
    if (current) entry.verified = retained;
    return retained;
  }

'''
text = text[:ensure_start] + new_ensure + text[chain_start:]

# Delete raw trust-chain request/event input assembly and replace it with explicit
# retained-handle release helpers.
request_input_start = text.index("  async #requestInput(")
policy_start = text.index("  #policyFor(", request_input_start)
release_helpers = r'''  #releaseEntry(entry: ProviderContextEntry): void {
    entry.verified?.dispose();
    entry.verified = undefined;
  }

  #clearContexts(): void {
    for (const entry of this.#contexts.values()) this.#releaseEntry(entry);
    this.#contexts.clear();
  }

'''
text = text[:request_input_start] + release_helpers + text[policy_start:]

text = text.replace(
    '''      for (const entry of this.#contexts.values()) {
        entry.verified = undefined;
      }''',
    '''      for (const entry of this.#contexts.values()) {
        this.#releaseEntry(entry);
      }''',
    1,
)

for token in (
    "verifyAuthorizationContextWasm",
    "verifyAuthorizationRequestV2Wasm",
    "verifyAuthorizationEventV2Wasm",
    "#requestInput(",
    "#eventInput(",
    "verified?: VerifiedAuthorizationContextTokenProjection",
):
    if token in text:
        raise RuntimeError(f"stale provider raw trust-chain path survived: {token}")
write(path, text)
