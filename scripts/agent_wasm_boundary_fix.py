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
p.write_text(text)

# Fix generated TypeScript class syntax: private #fields cannot be constructor parameter properties.
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
p.write_text(text.replace(old, new, 1))

# The free WASM verifier exports are gone, so the Rust protocol functions no longer need local aliases.
p = Path("rust/crates/protocol-wasm/src/lib.rs")
text = p.read_text()
text = text.replace(
    "    verify_authorization_event as verify_authorization_event_protocol,\n    verify_authorization_request as verify_authorization_request_protocol,",
    "    verify_authorization_event, verify_authorization_request,",
)
text = text.replace("    AuthorizationEventPublisher, ", "")
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
