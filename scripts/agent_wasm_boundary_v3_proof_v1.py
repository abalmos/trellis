from pathlib import Path


def replace_in_tree(root: str, suffixes: tuple[str, ...], replacements: dict[str, str]) -> None:
    for path in Path(root).rglob("*"):
        if not path.is_file() or path.suffix not in suffixes:
            continue
        text = path.read_text()
        updated = text
        for old, new in replacements.items():
            updated = updated.replace(old, new)
        if updated != text:
            path.write_text(updated)


# The request/event proof layer is unreleased. Publish one coherent first-public
# API rather than carrying the abandoned v2 names as compatibility baggage.
core_replacements = {
    "AUTHORIZATION_REQUEST_PROOF_DOMAIN_V2": "AUTHORIZATION_REQUEST_PROOF_DOMAIN_V1",
    "AUTHORIZATION_EVENT_PROOF_DOMAIN_V2": "AUTHORIZATION_EVENT_PROOF_DOMAIN_V1",
    "AuthorizationRequestProofInputV2": "AuthorizationRequestProofInput",
    "AuthorizationRequestProofV2": "AuthorizationRequestProof",
    "AuthorizationEventProofInputV2": "AuthorizationEventProofInput",
    "AuthorizationEventProofV2": "AuthorizationEventProof",
    "AuthorizationEventPublisherV2": "AuthorizationEventPublisher",
    "VerifiedAuthorizationRequestV2": "VerifiedAuthorizationRequest",
    "VerifiedAuthorizationEventV2": "VerifiedAuthorizationEvent",
    "build_authorization_request_proof_input_v2": "build_authorization_request_proof_input",
    "build_authorization_event_proof_input_v2": "build_authorization_event_proof_input",
    "sign_authorization_request_v2": "sign_authorization_request",
    "sign_authorization_event_v2": "sign_authorization_event",
    "verify_authorization_request_v2": "verify_authorization_request",
    "verify_authorization_event_v2": "verify_authorization_event",
}
replace_in_tree("rust", (".rs",), core_replacements)

# The WASM wire ABI remains explicitly versioned even though the Rust API is the
# first-public unversioned proof API.
wasm = Path("rust/crates/protocol-wasm/src/lib.rs")
text = wasm.read_text()
text = text.replace("WireAuthorizationRequestV2", "WireAuthorizationRequestV1")
text = text.replace("WireAuthorizationEventV2", "WireAuthorizationEventV1")
text = text.replace("verify_authorization_request(", "verify_authorization_request_v1(")
text = text.replace("verify_authorization_event(", "verify_authorization_event_v1(")
text = text.replace("local_authorization_v2_", "local_authorization_v1_")
wasm.write_text(text)

# TypeScript exposes the same first-public unversioned API while calling the
# explicitly-versioned v1 WASM ABI. Provider request/event values are ordinary
# in-memory verifier inputs, not independently-versioned serialized formats.
ts_replacements = {
    "VerifyAuthorizationRequestV2Result": "VerifyAuthorizationRequestResult",
    "VerifyAuthorizationEventV2Result": "VerifyAuthorizationEventResult",
    "VerifyAuthorizationRequestV2Args": "VerifyAuthorizationRequestArgs",
    "VerifyAuthorizationEventV2Args": "VerifyAuthorizationEventArgs",
    "verifyAuthorizationRequestV2Wasm": "verifyAuthorizationRequestWasm",
    "verifyAuthorizationEventV2Wasm": "verifyAuthorizationEventWasm",
    "AuthorizationProviderRequestV2": "AuthorizationProviderRequest",
    "AuthorizationProviderEventV2": "AuthorizationProviderEvent",
    "verifyRequestV2": "verifyRequest",
    "verifyEventV2": "verifyEvent",
}
replace_in_tree("ts", (".ts", ".tsx"), ts_replacements)

wrapper = Path("ts/packages/trellis/auth/protocol_wasm.ts")
text = wrapper.read_text()
text = text.replace("verify_authorization_request_v2", "verify_authorization_request_v1")
text = text.replace("verify_authorization_event_v2", "verify_authorization_event_v1")
wrapper.write_text(text)

# Lightweight proof transcript construction/signing intentionally stays native
# in TypeScript, but it must use the same first-public domains as Rust.
proof = Path("ts/packages/trellis/auth/proof.ts")
text = proof.read_text()
text = text.replace(
    'utf8("trellis.authorization-request-proof.v2")',
    'utf8("trellis.authorization-request-proof.v1")',
)
text = text.replace(
    'utf8("trellis.authorization-event-proof.v2")',
    'utf8("trellis.authorization-event-proof.v1")',
)
text = text.replace("canonical v2 context-bound request proof", "canonical v1 context-bound request proof")
text = text.replace("canonical v2 context-bound event proof", "canonical v1 context-bound event proof")
proof.write_text(text)

mod = Path("ts/packages/trellis/auth/mod.ts")
text = mod.read_text().replace(
    "// Context-bound v2 proof helpers for local signing and signature verification.",
    "// Context-bound proof helpers for local signing and signature verification.",
)
mod.write_text(text)

types = Path("ts/packages/trellis/auth/authorization/types.ts")
text = types.read_text()
text = text.replace(
    "/** Presented request-v2 data supplied by a provider transport adapter. */",
    "/** Presented request proof data supplied by a provider transport adapter. */",
)
text = text.replace(
    "/** Presented event-v2 data supplied by a provider event adapter. */",
    "/** Presented event proof data supplied by a provider event adapter. */",
)
types.write_text(text)

provider = Path("ts/packages/trellis/auth/authorization/provider_cache.ts")
text = provider.read_text()
text = text.replace(
    "/** Verify a presented request-v2 proof with exact route permissions. */",
    "/** Verify a presented request proof with exact route permissions. */",
)
text = text.replace(
    "/** Verify a presented event-v2 proof with exact publish permissions. */",
    "/** Verify a presented event proof with exact publish permissions. */",
)
provider.write_text(text)

conformance = Path("ts/packages/trellis/auth/conformance_test.ts")
text = conformance.read_text().replace(
    'Deno.test("request and event proof v2 match language-neutral vectors", async () => {',
    'Deno.test("request and event proof v1 match language-neutral vectors", async () => {',
)
conformance.write_text(text)

# Only the request/event proof domains change. Authorization context, manifest,
# grant-set, and session-proof v1 formats are independent serialized contracts.
auth = Path("rust/crates/protocol/src/authorization.rs")
text = auth.read_text()
text = text.replace(
    '"trellis.authorization-request-proof.v2"',
    '"trellis.authorization-request-proof.v1"',
)
text = text.replace(
    '"trellis.authorization-event-proof.v2"',
    '"trellis.authorization-event-proof.v1"',
)
text = text.replace("request-proof v2", "request-proof v1")
text = text.replace("event-proof v2", "event-proof v1")
text = text.replace("request_proof_v2_", "request_proof_v1_")
text = text.replace("authorization_event_proof_v2_", "authorization_event_proof_v1_")
auth.write_text(text)

readme = Path("conformance/README.md")
text = readme.read_text().replace("request-proof v2 vectors", "request/event-proof v1 vectors")
readme.write_text(text)

# Guard the clean break inside the transform too, so any newly-added stale
# prerelease name retriggers validation and fails before the test/build phases.
stale = (
    "AuthorizationRequestProofV2",
    "AuthorizationEventProofV2",
    "AuthorizationRequestProofInputV2",
    "AuthorizationEventProofInputV2",
    "AuthorizationEventPublisherV2",
    "VerifiedAuthorizationRequestV2",
    "VerifiedAuthorizationEventV2",
    "AuthorizationProviderRequestV2",
    "AuthorizationProviderEventV2",
    "verifyRequestV2",
    "verifyEventV2",
    "VerifyAuthorizationRequestV2",
    "VerifyAuthorizationEventV2",
    "verifyAuthorizationRequestV2Wasm",
    "verifyAuthorizationEventV2Wasm",
    "trellis.authorization-request-proof.v2",
    "trellis.authorization-event-proof.v2",
)
for root in ("rust", "ts", "conformance", "docs"):
    for candidate in Path(root).rglob("*"):
        if not candidate.is_file() or candidate.suffix not in {".rs", ".ts", ".tsx", ".md", ".json"}:
            continue
        content = candidate.read_text()
        for token in stale:
            if token in content:
                raise RuntimeError(f"stale first-public proof token {token!r} in {candidate}")
