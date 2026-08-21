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
        let mut fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&fixture_path).unwrap(),
        )
        .unwrap();
        let chain = &mut fixture["completeChain"];
        chain["requestProofInputHex"] = serde_json::json!(hex(request_input.as_bytes()));
        chain["requestProofDigest"] = serde_json::json!(encode_base64url(request_input.digest()));
        chain["requestProof"] = serde_json::json!(request_proof.as_str());
        chain["eventProofInputHex"] = serde_json::json!(hex(event_input.as_bytes()));
        chain["eventProofDigest"] = serde_json::json!(encode_base64url(event_input.digest()));
        chain["eventProof"] = serde_json::json!(event_proof.as_str());
        std::fs::write(
            fixture_path,
            format!("{}\n", serde_json::to_string_pretty(&fixture).unwrap()),
        )
        .unwrap();
    }
    // AGENT_AUTHORIZATION_PROOF_VECTOR_REGEN_END
'''

insert = text.rfind("\n}")
if insert < 0:
    raise RuntimeError("phase_a_tests closing brace not found")
path.write_text(text[:insert] + "\n" + helper + text[insert:])
