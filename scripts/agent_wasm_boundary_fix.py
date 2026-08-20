from pathlib import Path
import subprocess

p = Path("ts/packages/trellis/auth/authorization/provider_cache.ts")
text = p.read_text()
text = text.replace(
    "const result = verifier.verifyRequest(this.#requestInput(request));",
    "const result = verifier.verifyRequest(this.#requestInput(entry, request));",
)
text = text.replace(
    "this.#eventInput(event, revokedAt ?? null),",
    "this.#eventInput(entry, event, revokedAt ?? null),",
)
text = text.replace(
    "  #requestInput(\n    request: AuthorizationProviderRequest,",
    "  #requestInput(\n    entry: ProviderContextEntry,\n    request: AuthorizationProviderRequest,",
)
text = text.replace(
    "      policy: this.#policyForDigest(request.contextDigest, this.#now()),",
    "      policy: this.#policyFor(entry, this.#now()),",
)
text = text.replace(
    "  #eventInput(\n    event: AuthorizationProviderEvent,",
    "  #eventInput(\n    entry: ProviderContextEntry,\n    event: AuthorizationProviderEvent,",
)
text = text.replace(
    "      policy: this.#policyForDigest(event.contextDigest, this.#now(), true),",
    "      policy: this.#policyFor(entry, this.#now(), true),",
)
start = text.find("  #policyForDigest(\n")
end = text.find("  #policyFor(\n", start)
if start < 0 or end < 0:
    raise RuntimeError("generated policyForDigest block not found")
text = text[:start] + text[end:]

# Opaque WASM handles follow the same cache lifecycle as their verified contexts.
old = '''    for (const [digest, entry] of this.#contexts) {
      if (entry.retainedUntil <= now) this.#contexts.delete(digest);
    }'''
new = '''    for (const [digest, entry] of this.#contexts) {
      if (entry.retainedUntil <= now) {
        entry.verifier?.free();
        this.#contexts.delete(digest);
      }
    }'''
if text.count(old) != 1:
    raise RuntimeError("expired-context eviction block changed")
text = text.replace(old, new, 1)
old = '''      for (const entry of this.#contexts.values()) {
        entry.verified = undefined;
      }'''
new = '''      for (const entry of this.#contexts.values()) {
        entry.verifier?.free();
        entry.verifier = undefined;
        entry.verified = undefined;
      }'''
if text.count(old) != 1:
    raise RuntimeError("trust-floor invalidation block changed")
text = text.replace(old, new, 1)
p.write_text(text)

# Fix generated TypeScript class syntax and pass large payloads through wasm-bindgen bytes, not JSON arrays.
p = Path("ts/packages/trellis/auth/protocol_wasm.ts")
text = p.read_text()
old = '''export class AuthorizationContextVerifierWasm {
  constructor(
    readonly token: VerifiedAuthorizationContextTokenProjection,
    readonly context: VerifiedAuthorizationContextProjection,
    #handle: ProtocolAuthorizationContextVerifier,
  ) {}
'''
new = '''export class AuthorizationContextVerifierWasm {
  #handle: ProtocolAuthorizationContextVerifier;

  constructor(
    readonly token: VerifiedAuthorizationContextTokenProjection,
    readonly context: VerifiedAuthorizationContextProjection,
    handle: ProtocolAuthorizationContextVerifier,
  ) {
    this.#handle = handle;
  }
'''
if text.count(old) != 1:
    raise RuntimeError("generated AuthorizationContextVerifierWasm constructor changed")
text = text.replace(old, new, 1)
text = text.replace(
    "  verify_request(inputJson: string): string;\n  verify_event(inputJson: string): string;",
    "  verify_request(inputJson: string, payload: Uint8Array): string;\n  verify_event(inputJson: string, payload: Uint8Array): string;",
)
constructor_end = '''  ) {
    this.#handle = handle;
  }

  verifyRequest'''
replacement = '''  ) {
    this.#handle = handle;
  }

  free(): void {
    this.#handle.free();
  }

  verifyRequest'''
if text.count(constructor_end) != 1:
    raise RuntimeError("AuthorizationContextVerifierWasm method anchor changed")
text = text.replace(constructor_end, replacement, 1)
text = text.replace(
    '''    const result = JSON.parse(
      this.#handle.verify_request(JSON.stringify({
        ...args,
        policy: wasmVerificationPolicy(args.policy),
        payload: Array.from(args.payload),
      })),
    ) as ProofVerificationResult;''',
    '''    const { payload, ...input } = args;
    const result = JSON.parse(
      this.#handle.verify_request(
        JSON.stringify({ ...input, policy: wasmVerificationPolicy(args.policy) }),
        payload,
      ),
    ) as ProofVerificationResult;''',
)
text = text.replace(
    '''    const result = JSON.parse(
      this.#handle.verify_event(JSON.stringify({
        ...args,
        policy: wasmVerificationPolicy(args.policy),
        payload: Array.from(args.payload),
      })),
    ) as ProofVerificationResult;''',
    '''    const { payload, ...input } = args;
    const result = JSON.parse(
      this.#handle.verify_event(
        JSON.stringify({ ...input, policy: wasmVerificationPolicy(args.policy) }),
        payload,
      ),
    ) as ProofVerificationResult;''',
)
p.write_text(text)

# The free WASM verifier exports are gone, so the Rust protocol functions no longer need local aliases.
# Message payloads cross wasm-bindgen directly as &[u8] instead of JSON Vec<u8> fields.
p = Path("rust/crates/protocol-wasm/src/lib.rs")
text = p.read_text()
text = text.replace(
    "    verify_authorization_event as verify_authorization_event_protocol,\n    verify_authorization_request as verify_authorization_request_protocol,",
    "    verify_authorization_event, verify_authorization_request,",
)
text = text.replace("    AuthorizationEventPublisher, ", "")
text = text.replace("    payload: Vec<u8>,\n", "", 2)
text = text.replace(
    "    pub fn verify_request(&self, request_json: &str) -> String {",
    "    pub fn verify_request(&self, request_json: &str, payload: &[u8]) -> String {",
)
text = text.replace(
    "        request_result(&self.context, input)",
    "        request_result(&self.context, input, payload)",
)
text = text.replace(
    "    pub fn verify_event(&self, event_json: &str) -> String {",
    "    pub fn verify_event(&self, event_json: &str, payload: &[u8]) -> String {",
)
text = text.replace(
    "        event_result(&self.context, input)",
    "        event_result(&self.context, input, payload)",
)
text = text.replace(
    "    input: WireAuthorizationRequest,\n) -> String {",
    "    input: WireAuthorizationRequest,\n    payload: &[u8],\n) -> String {",
)
text = text.replace("        &input.payload,\n", "        payload,\n", 1)
text = text.replace(
    "    input: WireAuthorizationEvent,\n) -> String {",
    "    input: WireAuthorizationEvent,\n    payload: &[u8],\n) -> String {",
)
text = text.replace("        &input.payload,\n", "        payload,\n", 1)
p.write_text(text)

subprocess.run(
    [
        "git",
        "rm",
        "--cached",
        "--ignore-unmatch",
        "ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js",
        "ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm_bg.wasm",
        "ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm_bytes.ts",
    ],
    check=True,
)
