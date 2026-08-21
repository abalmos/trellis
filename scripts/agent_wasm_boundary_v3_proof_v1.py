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
# explicitly-versioned v1 WASM ABI.
ts_replacements = {
    "VerifyAuthorizationRequestV2Result": "VerifyAuthorizationRequestResult",
    "VerifyAuthorizationEventV2Result": "VerifyAuthorizationEventResult",
    "VerifyAuthorizationRequestV2Args": "VerifyAuthorizationRequestArgs",
    "VerifyAuthorizationEventV2Args": "VerifyAuthorizationEventArgs",
    "verifyAuthorizationRequestV2Wasm": "verifyAuthorizationRequestWasm",
    "verifyAuthorizationEventV2Wasm": "verifyAuthorizationEventWasm",
}
replace_in_tree("ts", (".ts", ".tsx"), ts_replacements)

wrapper = Path("ts/packages/trellis/auth/protocol_wasm.ts")
text = wrapper.read_text()
text = text.replace("verify_authorization_request_v2", "verify_authorization_request_v1")
text = text.replace("verify_authorization_event_v2", "verify_authorization_event_v1")
wrapper.write_text(text)

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
