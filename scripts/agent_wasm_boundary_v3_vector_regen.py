import sys
from pathlib import Path

path = Path("rust/crates/protocol/src/authorization.rs")
text = path.read_text()
start_marker = "    // AGENT_AUTHORIZATION_PROOF_VECTOR_REGEN_START\n"
end_marker = "    // AGENT_AUTHORIZATION_PROOF_VECTOR_REGEN_END\n"

if len(sys.argv) != 2 or sys.argv[1] not in {"add", "remove"}:
    raise SystemExit("usage: agent_wasm_boundary_v3_vector_regen.py add|remove")

if sys.argv[1] == "remove":
    start = text.find(start_marker)
    if start < 0:
        raise RuntimeError("vector regeneration helper start marker missing")
    end = text.find(end_marker, start)
    if end < 0:
        raise RuntimeError("vector regeneration helper end marker missing")
    end += len(end_marker)
    path.write_text(text[:start] + text[end:])
    raise SystemExit(0)

if start_marker in text:
    raise RuntimeError("vector regeneration helper already present")

helper = r'''    // AGENT_AUTHORIZATION_PROOF_VECTOR_REGEN_START
    #[test]
    #[ignore = "scratch-only deterministic conformance vector regeneration"]
    fn regenerate_authorization_proof_v1_vector() {
        let (_, _, context, session_key) = chain();
        let context_digest_text = context.digest().unwrap();
        let context_digest = decode_base64url::<32>(
            &context_digest_text,
            &["authorization-context"],
            AuthorizationErrorCodeV1::InvalidEncoding,
        )
        .unwrap();
        let request_input = build_authorization_request_proof_input(
            &context_digest,
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            br#"{"id":"doc-1"}"#,
            1_100,
            "req_test",
        )
        .unwrap();
        let request_proof = sign_authorization_request(
            &context_digest_text,
            "rpc.v1.Documents.Get",
            Some("_INBOX.test.reply"),
            br#"{"id":"doc-1"}"#,
            1_100,
            "req_test",
            &session_key,
        )
        .unwrap();
        let event_input = build_authorization_event_proof_input(
            &context_digest,
            "events.v1.Documents.Changed.doc-1",
            br#"{"id":"doc-1"}"#,
            "evt_doc_1",
            "1970-01-01T00:19:10Z",
        )
        .unwrap();
        let event_proof = sign_authorization_event(
            &context_digest_text,
            "events.v1.Documents.Changed.doc-1",
            br#"{"id":"doc-1"}"#,
            "evt_doc_1",
            "1970-01-01T00:19:10Z",
            &session_key,
        )
        .unwrap();
        let hex = |bytes: &[u8]| {
            bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../conformance/authorization-context/vectors.json");
        let original = std::fs::read_to_string(&fixture_path).unwrap();
        let fixture: serde_json::Value = serde_json::from_str(&original).unwrap();
        let chain = &fixture["completeChain"];
        let replacements = [
            ("requestProofInputHex", hex(request_input.as_bytes())),
            ("requestProofDigest", encode_base64url(request_input.digest())),
            ("requestProof", request_proof.as_str().to_owned()),
            ("eventProofInputHex", hex(event_input.as_bytes())),
            ("eventProofDigest", encode_base64url(event_input.digest())),
            ("eventProof", event_proof.as_str().to_owned()),
        ];
        let mut updated = original;
        for (field, value) in replacements {
            let old = chain[field].as_str().unwrap();
            let needle = format!("    \"{field}\": \"{old}\"");
            let replacement = format!("    \"{field}\": \"{value}\"");
            assert_eq!(updated.matches(&needle).count(), 1, "fixture field {field}");
            updated = updated.replacen(&needle, &replacement, 1);
        }
        std::fs::write(fixture_path, updated).unwrap();
    }
    // AGENT_AUTHORIZATION_PROOF_VECTOR_REGEN_END
'''

insert = text.rfind("\n}")
if insert < 0:
    raise RuntimeError("phase_a_tests closing brace not found")
path.write_text(text[:insert] + "\n" + helper + text[insert:])
