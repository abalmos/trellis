from pathlib import Path
import re
import subprocess


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:80]!r}")
    p.write_text(text.replace(old, new, 1))


def regex_once(path: str, pattern: str, replacement: str, flags: int = re.S) -> None:
    p = Path(path)
    text = p.read_text()
    updated, count = re.subn(pattern, replacement, text, count=1, flags=flags)
    if count != 1:
        raise RuntimeError(f"{path}: expected one regex match, found {count}: {pattern[:100]!r}")
    p.write_text(updated)


# First-public proof cleanup: implementation names are unversioned; wire domains are v1.
replacements = {
    "AUTHORIZATION_REQUEST_PROOF_DOMAIN_V2": "AUTHORIZATION_REQUEST_PROOF_DOMAIN_V1",
    "AUTHORIZATION_EVENT_PROOF_DOMAIN_V2": "AUTHORIZATION_EVENT_PROOF_DOMAIN_V1",
    "AuthorizationRequestProofInputV2": "AuthorizationRequestProofInput",
    "AuthorizationRequestProofV2": "AuthorizationRequestProof",
    "AuthorizationEventProofInputV2": "AuthorizationEventProofInput",
    "AuthorizationEventProofV2": "AuthorizationEventProof",
    "AuthorizationEventPublisherV2": "AuthorizationEventPublisher",
    "VerifiedAuthorizationRequestV2": "VerifiedAuthorizationRequest",
    "VerifiedAuthorizationEventV2": "VerifiedAuthorizationEvent",
    "WireAuthorizationRequestV2": "WireAuthorizationRequest",
    "WireAuthorizationEventV2": "WireAuthorizationEvent",
    "VerifyAuthorizationRequestV2": "VerifyAuthorizationRequest",
    "VerifyAuthorizationEventV2": "VerifyAuthorizationEvent",
    "AuthorizationProviderRequestV2": "AuthorizationProviderRequest",
    "AuthorizationProviderEventV2": "AuthorizationProviderEvent",
    "verifyAuthorizationRequestV2": "verifyAuthorizationRequest",
    "verifyAuthorizationEventV2": "verifyAuthorizationEvent",
    "verifyRequestV2": "verifyRequest",
    "verifyEventV2": "verifyEvent",
    "build_authorization_request_proof_input_v2": "build_authorization_request_proof_input",
    "build_authorization_event_proof_input_v2": "build_authorization_event_proof_input",
    "sign_authorization_request_v2": "sign_authorization_request",
    "sign_authorization_event_v2": "sign_authorization_event",
    "verify_authorization_request_v2": "verify_authorization_request",
    "verify_authorization_event_v2": "verify_authorization_event",
    "create_request_proof_v2": "create_request_proof",
    "create_event_proof_v2": "create_event_proof",
    "verify_event_proof_v2": "verify_event_proof",
    "request_proof_v2": "request_proof",
    "event_proof_v2": "event_proof",
    "verify_v2_signature": "verify_signature",
    "trellis.authorization-request-proof.v2": "trellis.authorization-request-proof.v1",
    "trellis.authorization-event-proof.v2": "trellis.authorization-event-proof.v1",
    "local_authorization_v2_uses_conformance_chain": "local_authorization_uses_conformance_chain",
    "request_proof_v2_matches_language_neutral_conformance_vector": "request_proof_matches_language_neutral_conformance_vector",
    "event_proof_v2_matches_language_neutral_conformance_vector": "event_proof_matches_language_neutral_conformance_vector",
    "context-bound v2 `proof`": "context-bound `proof`",
    "context-bound v2 event proof": "context-bound event proof",
    "v2 context-bound": "context-bound",
    "request-proof v2": "request-proof",
    "event-proof v2": "event-proof",
    "request and event proof v2": "request and event proof",
    "v2 request proof": "request proof",
    "v2 event proof": "event proof",
    "proof v2": "proof",
}

for raw in subprocess.check_output(["git", "ls-files", "-z"]).split(b"\0"):
    if not raw:
        continue
    path = Path(raw.decode())
    if path.as_posix() in {"CLEANUP.md", "scripts/agent_wasm_boundary.py"} or path.parts[:2] == (".github", "workflows"):
        continue
    try:
        text = path.read_text()
    except (UnicodeDecodeError, OSError):
        continue
    original = text
    for old, new in replacements.items():
        text = text.replace(old, new)
    if text != original:
        path.write_text(text)

# Recompute the six deterministic fields affected by the proof-domain bytes.
vectors = Path("conformance/authorization-context/vectors.json")
text = vectors.read_text()
vector_updates = {
    "requestProofInputHex": "000000267472656c6c69732e617574686f72697a6174696f6e2d726571756573742d70726f6f662e7631000000204011b452d9757ab84bb35698561369ca87b42462d37c16eb462c0efc898ad34c000000147270632e76312e446f63756d656e74732e476574000000115f494e424f582e746573742e7265706c79000000209eb803b2c601706a797e2e58a58cb40b01ff0a0b024b1056ae044cafce2da7860000000431313030000000087265715f74657374",
    "requestProofDigest": "fHyDchawXxUdjCSg6htzOV04ET9FLKg8E4hBufPfz64",
    "requestProof": "3dsRlkhh_cDiRyrUlKYeVKoYrRnxsxl_9CrvhZOMTrqPnWDcchMAIubEPqW78IL5ABxNqkT1KFlvWjoQSK74Cw",
    "eventProofInputHex": "000000247472656c6c69732e617574686f72697a6174696f6e2d6576656e742d70726f6f662e7631000000204011b452d9757ab84bb35698561369ca87b42462d37c16eb462c0efc898ad34c000000216576656e74732e76312e446f63756d656e74732e4368616e6765642e646f632d31000000209eb803b2c601706a797e2e58a58cb40b01ff0a0b024b1056ae044cafce2da786000000096576745f646f635f3100000014313937302d30312d30315430303a31393a31305a",
    "eventProofDigest": "Gnrw9PeHuWExhLrBAxQOHcXFEd-9eq9U_UTFWn79ris",
    "eventProof": "v_zQhj4LNAggTfoWqqTvXOgyNeB0UqtRRGxlWKZRBgMJOGJk94uW119bVW3KgkUKLZhBN1dkU53v-OtxacDTBg",
}
for name, value in vector_updates.items():
    pattern = rf'("{re.escape(name)}":\s*)"[^"]*"'
    text, count = re.subn(pattern, lambda m, value=value: f'{m.group(1)}"{value}"', text, count=1)
    if count != 1:
        raise RuntimeError(f"expected one vector field {name}, found {count}")
vectors.write_text(text)

# Generic prepare no longer owns protocol WASM. Expose the existing builder as a focused xtask.
replace_once(
    "rust/xtask/src/main.rs",
    "    #[command(name = \"prepare-watch\")]\n    PrepareWatch,",
    "    #[command(name = \"prepare-watch\")]\n    PrepareWatch,\n    #[command(name = \"protocol-wasm\")]\n    ProtocolWasm,",
)
replace_once(
    "rust/xtask/src/main.rs",
    "        XtaskCommand::PrepareWatch => run_prepare_watch(),\n        XtaskCommand::Build { args } => run_build(&args),",
    "        XtaskCommand::PrepareWatch => run_prepare_watch(),\n        XtaskCommand::ProtocolWasm => generate_protocol_wasm(),\n        XtaskCommand::Build { args } => run_build(&args),",
)
replace_once(
    "rust/xtask/src/main.rs",
    "fn run_prepare() -> Result<()> {\n    generate_protocol_wasm()?;\n    run_generate_prepare(&[])?;",
    "fn run_prepare() -> Result<()> {\n    run_generate_prepare(&[])?;",
)
replace_once(
    "rust/xtask/src/main.rs",
    "    fn parse_prepare_watch_command() {",
    "    fn parse_protocol_wasm_command() {\n        let command = parse_command([\"protocol-wasm\".to_string()].into_iter())\n            .expect(\"parse protocol-wasm\")\n            .expect(\"protocol-wasm command\");\n        assert_eq!(command, XtaskCommand::ProtocolWasm);\n    }\n\n    #[test]\n    fn parse_prepare_watch_command() {",
)

# WASM output is generated TS/package state, not repository source state.
replace_once(
    ".gitignore",
    "# Generated build artifacts\ngenerated/",
    "# Generated build artifacts\ngenerated/\nts/packages/trellis/auth/protocol_wasm/",
)
for generated in [
    "ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js",
    "ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm_bg.wasm",
    "ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm_bytes.ts",
]:
    Path(generated).unlink(missing_ok=True)

# Local TS/package entry points build WASM only when the TS verifier/package needs it.
replace_once(
    "ts/deno.json",
    '    "prepare:watch": "deno run -A @qlever-llc/trellis/generate prepare --watch ..",',
    '    "prepare:watch": "deno run -A @qlever-llc/trellis/generate prepare --watch ..",\n    "protocol:wasm": "cargo run --manifest-path ../rust/xtask/Cargo.toml -- protocol-wasm",',
)
replace_once(
    "ts/deno.json",
    '    "check": "deno task prepare && deno check packages/trellis/index.ts packages/trellis-svelte/src/index.ts packages/trellis-svelte/src/context.svelte.ts packages/trellis-test/index.ts",',
    '    "check": "deno task prepare && deno task protocol:wasm && deno check packages/trellis/index.ts packages/trellis-svelte/src/index.ts packages/trellis-svelte/src/context.svelte.ts packages/trellis-test/index.ts",',
)
replace_once(
    "ts/deno.json",
    '    "test:auth": "deno task prepare && deno test -A packages/trellis/auth/conformance_test.ts packages/trellis/auth/session_auth_test.ts",',
    '    "test:auth": "deno task prepare && deno task protocol:wasm && deno test -A packages/trellis/auth/conformance_test.ts packages/trellis/auth/session_auth_test.ts",',
)
replace_once(
    "ts/packages/trellis/deno.json",
    '    "build:npm": "deno run -A ./scripts/build_npm.ts",',
    '    "build:npm": "deno task -c ../../deno.json protocol:wasm && deno run -A ./scripts/build_npm.ts",',
)

# Check builds protocol WASM once for the TS/live lanes; the Rust lane consumes only SDK/portal artifacts.
check = Path(".github/workflows/check.yml")
text = check.read_text()
anchor = "  rust:\n    name: Rust\n    needs: prepare\n"
wasm_job = '''  wasm:\n    name: Protocol WASM\n    needs: prepare\n    runs-on: ubuntu-latest\n    steps:\n      - name: Check out repository\n        uses: actions/checkout@v4\n\n      - name: Download prepared artifacts\n        uses: actions/download-artifact@v4\n        with:\n          name: check-generated\n          path: .\n\n      - name: Set up Rust\n        uses: dtolnay/rust-toolchain@stable\n        with:\n          targets: wasm32-unknown-unknown\n\n      - name: Cache protocol WASM build\n        uses: swatinem/rust-cache@v2\n        with:\n          workspaces: |\n            rust -> target\n            rust/xtask -> target\n\n      - name: Build protocol WASM\n        run: cargo run --manifest-path rust/xtask/Cargo.toml -- protocol-wasm\n\n      - name: Upload protocol WASM\n        uses: actions/upload-artifact@v4\n        with:\n          name: check-protocol-wasm\n          path: ts/packages/trellis/auth/protocol_wasm\n          retention-days: 1\n\n'''
if text.count(anchor) != 1:
    raise RuntimeError("check.yml rust anchor changed")
text = text.replace(anchor, wasm_job + anchor, 1)
text = text.replace("    needs: prepare\n    runs-on: ubuntu-latest\n    steps:\n      - name: Check out repository\n        uses: actions/checkout@v4\n\n      - name: Download prepared artifacts\n        uses: actions/download-artifact@v4\n        with:\n          name: check-generated\n          path: .\n\n      - name: Set up Deno",
'''    needs: [prepare, wasm]\n    runs-on: ubuntu-latest\n    steps:\n      - name: Check out repository\n        uses: actions/checkout@v4\n\n      - name: Download prepared artifacts\n        uses: actions/download-artifact@v4\n        with:\n          name: check-generated\n          path: .\n\n      - name: Download protocol WASM\n        uses: actions/download-artifact@v4\n        with:\n          name: check-protocol-wasm\n          path: ts/packages/trellis/auth/protocol_wasm\n\n      - name: Set up Deno''', 1)
# Live has Rust setup between artifact download and Deno setup, so patch its needs and add the artifact separately.
text = text.replace("  live:\n    name: Live Trellis\n    needs: prepare\n", "  live:\n    name: Live Trellis\n    needs: [prepare, wasm]\n", 1)
live_download = '''      - name: Download prepared artifacts\n        uses: actions/download-artifact@v4\n        with:\n          name: check-generated\n          path: .\n\n      - name: Set up Rust\n'''
replacement = '''      - name: Download prepared artifacts\n        uses: actions/download-artifact@v4\n        with:\n          name: check-generated\n          path: .\n\n      - name: Download protocol WASM\n        uses: actions/download-artifact@v4\n        with:\n          name: check-protocol-wasm\n          path: ts/packages/trellis/auth/protocol_wasm\n\n      - name: Set up Rust\n'''
# There are Rust and live occurrences; replace the last one only.
pos = text.rfind(live_download)
if pos < 0:
    raise RuntimeError("check.yml live download anchor changed")
text = text[:pos] + text[pos:].replace(live_download, replacement, 1)
check.write_text(text)

# The WASM verifier owns an already-verified context. Message calls no longer accept the trust chain.
wasm = Path("rust/crates/protocol-wasm/src/lib.rs")
text = wasm.read_text()
text = re.sub(
    r'(?s)#\[derive\(Deserialize\)\]\n#\[serde\(deny_unknown_fields, rename_all = "camelCase"\)\]\nstruct WireAuthorizationRequest \{.*?\n\}\n\n#\[derive\(Deserialize\)\]\n#\[serde\(deny_unknown_fields, rename_all = "camelCase"\)\]\nstruct WireAuthorizationEvent \{.*?\n\}',
    '''#[derive(Deserialize)]\n#[serde(deny_unknown_fields, rename_all = "camelCase")]\nstruct WireAuthorizationRequest {\n    subject: String,\n    reply: RequiredNullable<String>,\n    payload: Vec<u8>,\n    iat: i64,\n    request_id: String,\n    proof: String,\n    required_permissions: Vec<PermissionAtomV1>,\n    required_capabilities: Vec<String>,\n    policy: WireAuthorizationVerificationPolicyV1,\n}\n\n#[derive(Deserialize)]\n#[serde(deny_unknown_fields, rename_all = "camelCase")]\nstruct WireAuthorizationEvent {\n    subject: String,\n    payload: Vec<u8>,\n    event_id: String,\n    event_time: String,\n    proof: String,\n    required_permissions: Vec<PermissionAtomV1>,\n    required_capabilities: Vec<String>,\n    #[serde(default)]\n    revoked_at: Option<i64>,\n    policy: WireAuthorizationVerificationPolicyV1,\n}''',
    text,
    count=1,
)

context_block = '''/// Opaque verified authorization context retained inside WASM for hot-path proof checks.\n#[wasm_bindgen]\npub struct AuthorizationContextVerifier {\n    context: VerifiedAuthorizationContextV1,\n    token_projection: String,\n    context_projection: String,\n}\n\n#[wasm_bindgen]\nimpl AuthorizationContextVerifier {\n    /// Return the verified trust/context token projection as JSON.\n    pub fn token_projection(&self) -> String {\n        self.token_projection.clone()\n    }\n\n    /// Return the verified caller-context projection as JSON.\n    pub fn context_projection(&self) -> String {\n        self.context_projection.clone()\n    }\n\n    /// Verify one request against this already-verified context.\n    pub fn verify_request(&self, request_json: &str) -> String {\n        let input: WireAuthorizationRequest = match serde_json::from_str(request_json) {\n            Ok(input) => input,\n            Err(_) => return input_error_result(""),\n        };\n        request_result(&self.context, input)\n    }\n\n    /// Verify one event against this already-verified context.\n    pub fn verify_event(&self, event_json: &str) -> String {\n        let input: WireAuthorizationEvent = match serde_json::from_str(event_json) {\n            Ok(input) => input,\n            Err(_) => return input_error_result(""),\n        };\n        event_result(&self.context, input)\n    }\n}\n\nfn authorization_context_verifier(\n    root_json: &str,\n    manifest_json: &str,\n    context_json: &str,\n    policy_json: &str,\n    historical: bool,\n) -> Result<AuthorizationContextVerifier, JsError> {\n    let policy = authorization_verification_policy(policy_json)?;\n    let root_value: Value =\n        serde_json::from_str(root_json).map_err(|error| JsError::new(&error.to_string()))?;\n    let root = AuthorizationTrustRootV1::parse(&root_value)\n        .map_err(|error| JsError::new(&error.to_string()))?;\n    let manifest_value: Value =\n        serde_json::from_str(manifest_json).map_err(|error| JsError::new(&error.to_string()))?;\n    let manifest = parse_issuer_manifest_v1(&manifest_value)\n        .map_err(|error| JsError::new(&error.to_string()))?;\n    let context_value: Value =\n        serde_json::from_str(context_json).map_err(|error| JsError::new(&error.to_string()))?;\n    let context = parse_authorization_context_v1(&context_value)\n        .map_err(|error| JsError::new(&error.to_string()))?;\n    let verification_policy = if historical {\n        let mut policy = policy;\n        policy.now_unix_seconds = context.unsigned.expires_at;\n        policy\n    } else {\n        policy\n    };\n    let verified_manifest = verify_issuer_manifest_v1(&root, &manifest, &verification_policy)\n        .map_err(|error| JsError::new(&error.to_string()))?;\n    let verified = verify_authorization_context_v1(\n        &root,\n        &verified_manifest,\n        &context,\n        &verification_policy,\n    )\n    .map_err(|error| JsError::new(&error.to_string()))?;\n    let context_projection = verified_context_projection(&verified)\n        .map_err(|error| JsError::new(&error.to_string()))?;\n    let token_projection = serde_json::to_string(&json!({\n        "authority": root.authority(),\n        "rootKeyId": root.key_id(),\n        "rootDigest": root.digest().map_err(|error| JsError::new(&error.to_string()))?,\n        "manifestDigest": verified_manifest.digest().map_err(|error| JsError::new(&error.to_string()))?,\n        "contextDigest": verified.context_digest(),\n        "context": verified.signed_context(),\n        "manifestGeneration": verified_manifest.generation(),\n    }))\n    .map_err(|error| JsError::new(&error.to_string()))?;\n    Ok(AuthorizationContextVerifier {\n        context: verified,\n        token_projection,\n        context_projection: serde_json::to_string(&context_projection)\n            .map_err(|error| JsError::new(&error.to_string()))?,\n    })\n}\n\n/// Verify and retain a complete authorization context chain inside WASM.\n#[wasm_bindgen]\npub fn create_authorization_context_verifier(\n    root_json: &str,\n    manifest_json: &str,\n    context_json: &str,\n    policy_json: &str,\n    historical: bool,\n) -> Result<AuthorizationContextVerifier, JsError> {\n    authorization_context_verifier(\n        root_json,\n        manifest_json,\n        context_json,\n        policy_json,\n        historical,\n    )\n}\n\n/// Verify a root, issuer manifest, and signed authorization context JSON value.\n#[wasm_bindgen]\npub fn verify_authorization_context(\n    root_json: &str,\n    manifest_json: &str,\n    context_json: &str,\n    policy_json: &str,\n) -> Result<String, JsError> {\n    Ok(authorization_context_verifier(\n        root_json,\n        manifest_json,\n        context_json,\n        policy_json,\n        false,\n    )?\n    .token_projection)\n}\n\n'''
text, count = re.subn(
    r'(?s)/// Verify a root, issuer manifest, and signed authorization context JSON value\.\n#\[wasm_bindgen\]\npub fn verify_authorization_context\(.*?\n\}\n\n(?=/// Verify a root-signed issuer manifest)',
    context_block,
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"protocol-wasm context block match count {count}")
# The old helper is no longer needed because context verification happens only at handle creation.
text, count = re.subn(
    r'(?s)#\[allow\(clippy::result_large_err\)\] // Protocol errors are serialized immediately at the WASM boundary\.\nfn verify_context_bundle\(.*?\n\}\n\n(?=#\[allow\(clippy::result_large_err\).*?fn verified_context_projection)',
    '',
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"protocol-wasm verify_context_bundle match count {count}")

request_event = '''fn request_result(\n    context: &VerifiedAuthorizationContextV1,\n    input: WireAuthorizationRequest,\n) -> String {\n    let policy = match authorization_verification_policy_from_wire(&input.policy) {\n        Ok(policy) => policy,\n        Err(_) => return input_error_result("/policy"),\n    };\n    let proof = match AuthorizationRequestProof::parse(input.proof) {\n        Ok(proof) => proof,\n        Err(error) => return protocol_error_result(&error),\n    };\n    match verify_authorization_request(\n        context,\n        &input.subject,\n        input.reply.0.as_deref(),\n        &input.payload,\n        input.iat,\n        &input.request_id,\n        &proof,\n        &policy,\n        &input.required_permissions,\n        &input.required_capabilities,\n    ) {\n        Ok(_) => json_result(json!({ "ok": true })),\n        Err(error) => protocol_error_result(&error),\n    }\n}\n\nfn event_result(\n    context: &VerifiedAuthorizationContextV1,\n    input: WireAuthorizationEvent,\n) -> String {\n    let policy = match authorization_verification_policy_from_wire(&input.policy) {\n        Ok(policy) => policy,\n        Err(_) => return input_error_result("/policy"),\n    };\n    let proof = match AuthorizationEventProof::parse(input.proof) {\n        Ok(proof) => proof,\n        Err(error) => return protocol_error_result(&error),\n    };\n    match verify_authorization_event(\n        context,\n        &input.subject,\n        &input.payload,\n        &input.event_id,\n        &input.event_time,\n        &proof,\n        &policy,\n        &input.required_permissions,\n        &input.required_capabilities,\n        input.revoked_at,\n    ) {\n        Ok(_) => json_result(json!({ "ok": true })),\n        Err(error) => protocol_error_result(&error),\n    }\n}\n\n'''
text, count = re.subn(
    r'(?s)fn request_result\(input: WireAuthorizationRequest\) -> String \{.*?\n\}\n\nfn event_publisher_projection\(.*?\n\}\n\nfn event_result\(input: WireAuthorizationEvent\) -> String \{.*?\n\}\n\n(?=/// Verify one context-bound authorization request proof)',
    request_event,
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"protocol-wasm request/event block match count {count}")
text, count = re.subn(
    r'(?s)/// Verify one context-bound authorization request proof from a JSON argument\..*?pub fn verify_authorization_event\(event_json: &str\) -> String \{.*?\n\}\n\n(?=#\[cfg\(any\(\)\)\])',
    '',
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"protocol-wasm free verifier export block match count {count}")
wasm.write_text(text)

# Trellis client authorization core now collides with the unversioned protocol result names; alias them explicitly.
core = Path("rust/crates/trellis/src/client/authorization/core.rs")
text = core.read_text()
text = text.replace(
    "PermissionAtomV1, ProtocolError, VerifiedAuthorizationContextV1, VerifiedAuthorizationEvent,\n    VerifiedAuthorizationRequest,",
    "PermissionAtomV1, ProtocolError, VerifiedAuthorizationContextV1,\n    VerifiedAuthorizationEvent as ProtocolVerifiedAuthorizationEvent,\n    VerifiedAuthorizationRequest as ProtocolVerifiedAuthorizationRequest,",
)
text = text.replace("    request: VerifiedAuthorizationRequest,", "    request: ProtocolVerifiedAuthorizationRequest,")
text = text.replace("    event: VerifiedAuthorizationEvent,", "    event: ProtocolVerifiedAuthorizationEvent,")
core.write_text(text)

# TS wrapper exposes an opaque verified-context handle. Full projections are materialized once at handle creation.
protocol = Path("ts/packages/trellis/auth/protocol_wasm.ts")
text = protocol.read_text()
text = re.sub(
    r'(?s)type ProtocolWasmModule = typeof protocolWasmModule & \{.*?\};\n\nconst protocolWasm',
    '''type ProtocolAuthorizationContextVerifier = {\n  token_projection(): string;\n  context_projection(): string;\n  verify_request(inputJson: string): string;\n  verify_event(inputJson: string): string;\n  free(): void;\n};\n\ntype ProtocolWasmModule = typeof protocolWasmModule & {\n  resolve_participant_v1(participantJson: string, apisJson: string): string;\n  create_authorization_context_verifier(\n    rootJson: string,\n    manifestJson: string,\n    contextJson: string,\n    policyJson: string,\n    historical: boolean,\n  ): ProtocolAuthorizationContextVerifier;\n};\n\nconst protocolWasm''',
    text,
    count=1,
)
text = re.sub(
    r'(?s)export type VerifyAuthorizationRequestArgs =\n  & AuthorizationContextVerificationInput\n  & \{(.*?)\n  \};',
    r'export type VerifyAuthorizationRequestArgs = {\1\n};',
    text,
    count=1,
)
text = re.sub(
    r'(?s)export type VerifyAuthorizationEventArgs =\n  & AuthorizationContextVerificationInput\n  & \{(.*?)\n  \};',
    r'export type VerifyAuthorizationEventArgs = {\1\n};',
    text,
    count=1,
)
# Keep the existing plain context verification API for client-cache installation, but implement it through the same handle factory.
text, count = re.subn(
    r'(?s)export async function verifyAuthorizationContextWasm\(args: \{\n  root: unknown;\n  manifest: unknown;\n  context: unknown;\n  policy: AuthorizationContextVerificationPolicyV1;\n\}\): Promise<VerifiedAuthorizationContextTokenProjection> \{.*?\n\}',
    '''export async function verifyAuthorizationContextWasm(args: {\n  root: unknown;\n  manifest: unknown;\n  context: unknown;\n  policy: AuthorizationContextVerificationPolicyV1;\n}): Promise<VerifiedAuthorizationContextTokenProjection> {\n  const verifier = await createAuthorizationContextVerifierWasm(args);\n  try {\n    return structuredClone(verifier.token);\n  } finally {\n    verifier.free();\n  }\n}''',
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"protocol_wasm.ts context wrapper match count {count}")
class_block = '''type ProofVerificationResult =\n  | { ok: true }\n  | { ok: false; error: AuthorizationVerificationError };\n\n/** Opaque Rust/WASM verifier bound to one already-verified authorization context. */\nexport class AuthorizationContextVerifierWasm {\n  constructor(\n    readonly token: VerifiedAuthorizationContextTokenProjection,\n    readonly context: VerifiedAuthorizationContextProjection,\n    #handle: ProtocolAuthorizationContextVerifier,\n  ) {}\n\n  verifyRequest(args: VerifyAuthorizationRequestArgs): VerifyAuthorizationRequestResult {\n    const result = JSON.parse(\n      this.#handle.verify_request(JSON.stringify({\n        ...args,\n        policy: wasmVerificationPolicy(args.policy),\n        payload: Array.from(args.payload),\n      })),\n    ) as ProofVerificationResult;\n    return result.ok ? { ok: true, ...structuredClone(this.context) } : result;\n  }\n\n  verifyEvent(args: VerifyAuthorizationEventArgs): VerifyAuthorizationEventResult {\n    const result = JSON.parse(\n      this.#handle.verify_event(JSON.stringify({\n        ...args,\n        policy: wasmVerificationPolicy(args.policy),\n        payload: Array.from(args.payload),\n      })),\n    ) as ProofVerificationResult;\n    if (!result.ok) return result;\n    const context = structuredClone(this.context);\n    return {\n      ok: true,\n      ...context,\n      publisher: {\n        kind: context.principal.kind,\n        deploymentId: context.deploymentId,\n        instanceId: context.instanceId,\n        participantId: context.participant.id,\n        participantDigest: context.participant.artifactDigest,\n        sessionId: context.sessionId,\n      },\n    };\n  }\n\n  free(): void {\n    this.#handle.free();\n  }\n}\n\n/** Verify a context chain once and retain the verified context inside WASM. */\nexport async function createAuthorizationContextVerifierWasm(\n  args: AuthorizationContextVerificationInput & {\n    policy: AuthorizationContextVerificationPolicyV1;\n    historical?: boolean;\n  },\n): Promise<AuthorizationContextVerifierWasm> {\n  await initialize();\n  const handle = protocolWasm.create_authorization_context_verifier(\n    JSON.stringify(args.root),\n    JSON.stringify(args.manifest),\n    JSON.stringify(args.context),\n    JSON.stringify(wasmVerificationPolicy(args.policy)),\n    args.historical ?? false,\n  );\n  try {\n    const token = JSON.parse(\n      handle.token_projection(),\n    ) as VerifiedAuthorizationContextTokenProjection;\n    token.refreshAt = token.context.expiresAt - args.policy.refreshLeadSeconds -\n      contextJitter(token.contextDigest, args.policy.refreshJitterSeconds);\n    const context = JSON.parse(\n      handle.context_projection(),\n    ) as VerifiedAuthorizationContextProjection;\n    return new AuthorizationContextVerifierWasm(token, context, handle);\n  } catch (error) {\n    handle.free();\n    throw error;\n  }\n}\n\n'''
text, count = re.subn(
    r'(?s)/\*\* Verify one context-bound request proof using actual received request bytes\. \*/\nexport async function verifyAuthorizationRequestWasm\(.*?\n\}\n\n/\*\* Verify one context-bound event proof, including historical time/revocation checks\. \*/\nexport async function verifyAuthorizationEventWasm\(.*?\n\}\n\n(?=function wasmVerificationPolicy)',
    class_block,
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"protocol_wasm.ts free verifier wrapper block match count {count}")
protocol.write_text(text)

# Provider cache owns and invalidates opaque WASM verifiers alongside each resolved context.
provider = Path("ts/packages/trellis/auth/authorization/provider_cache.ts")
text = provider.read_text()
text = text.replace(
    "  type AuthorizationContextVerificationPolicyV1,\n  type AuthorizationVerificationErrorCode,\n  type VerifiedAuthorizationContextTokenProjection,\n  verifyAuthorizationContextWasm,\n  type VerifyAuthorizationEventArgs,\n  type VerifyAuthorizationEventResult,\n  verifyAuthorizationEventWasm,\n  verifyAuthorizationManifestWasm,\n  type VerifyAuthorizationRequestArgs,\n  type VerifyAuthorizationRequestResult,\n  verifyAuthorizationRequestWasm,",
    "  AuthorizationContextVerifierWasm,\n  type AuthorizationContextVerificationPolicyV1,\n  type AuthorizationVerificationErrorCode,\n  createAuthorizationContextVerifierWasm,\n  type VerifiedAuthorizationContextTokenProjection,\n  type VerifyAuthorizationEventArgs,\n  type VerifyAuthorizationEventResult,\n  verifyAuthorizationManifestWasm,\n  type VerifyAuthorizationRequestArgs,\n  type VerifyAuthorizationRequestResult,",
)
text = text.replace(
    "  verified?: VerifiedAuthorizationContextTokenProjection;",
    "  requestVerifier?: AuthorizationContextVerifierWasm;\n  eventVerifier?: AuthorizationContextVerifierWasm;",
)
text = text.replace("        verified: structuredClone(material.verified),\n", "")
text = text.replace(
    "    const entry = await this.#resolveEntry(contextDigest);\n    return structuredClone(\n      await this.#ensureVerified(entry, false, this.#now()),\n    );",
    "    const entry = await this.#resolveEntry(contextDigest);\n    const verifier = await this.#ensureVerifier(entry, false, this.#now());\n    return structuredClone(verifier.token);",
)
text = text.replace(
    "      const entry = await this.#resolveEntry(request.contextDigest);\n      const result = await verifyAuthorizationRequestWasm(\n        await this.#requestInput(entry, request),\n      );",
    "      const entry = await this.#resolveEntry(request.contextDigest);\n      const verifier = await this.#ensureVerifier(entry, false, this.#now());\n      const result = verifier.verifyRequest(this.#requestInput(request));",
)
text = text.replace(
    "      const entry = await this.#resolveEntry(event.contextDigest);\n      const revokedAt = this.#revocationEvidence(entry.contextDigest);\n      const result = await verifyAuthorizationEventWasm(\n        await this.#eventInput(\n          entry,\n          event,\n          revokedAt ?? null,\n        ),\n      );",
    "      const entry = await this.#resolveEntry(event.contextDigest);\n      const revokedAt = this.#revocationEvidence(entry.contextDigest);\n      const verifier = await this.#ensureVerifier(entry, true, this.#now());\n      const result = verifier.verifyEvent(\n        this.#eventInput(event, revokedAt ?? null),\n      );",
)
# Replace verifier cache implementation.
text, count = re.subn(
    r'(?s)  async #ensureVerified\(.*?\n  \}\n\n  async #resolveChain',
    '''  async #ensureVerifier(\n    entry: ProviderContextEntry,\n    historical: boolean,\n    verificationTime: number,\n  ): Promise<AuthorizationContextVerifierWasm> {\n    if (!historical && entry.epoch !== this.#manifestEpoch) {\n      throw new AuthorizationProviderUnavailableError(\n        "authorization manifest advanced during context verification",\n      );\n    }\n    if (\n      !historical &&\n      entry.manifestGeneration !== this.#currentManifest?.pointer.generation\n    ) {\n      throw new Error("authorization context manifest is not current");\n    }\n    const existing = historical ? entry.eventVerifier : entry.requestVerifier;\n    if (existing) return existing;\n    const chain = await this.#resolveChain(entry);\n    const policy = this.#policyFor(entry, verificationTime, historical);\n    const verifier = await createAuthorizationContextVerifierWasm({\n      root: entry.root,\n      manifest: chain.value,\n      context: entry.context,\n      policy,\n      historical,\n    });\n    const verified = verifier.token;\n    if ((!historical && entry.epoch !== this.#manifestEpoch) || this.#stopped) {\n      verifier.free();\n      throw new AuthorizationProviderUnavailableError(\n        "authorization registry changed during verification",\n      );\n    }\n    if (\n      verified.contextDigest !== entry.contextDigest ||\n      (chain.pointer.digest !== "" &&\n        verified.manifestDigest !== chain.pointer.digest) ||\n      verified.manifestGeneration !== chain.pointer.generation\n    ) {\n      verifier.free();\n      throw new Error("authorization registry trust identity mismatch");\n    }\n    if (historical) entry.eventVerifier = verifier;\n    else entry.requestVerifier = verifier;\n    return verifier;\n  }\n\n  async #resolveChain''',
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"provider ensureVerifier block match count {count}")
# Request/event input helpers no longer carry root/manifest/context.
text, count = re.subn(
    r'(?s)  async #requestInput\(.*?\n  \}\n\n  async #eventInput\(.*?\n  \}\n\n(?=  #policyFor)',
    '''  #requestInput(\n    request: AuthorizationProviderRequest,\n  ): VerifyAuthorizationRequestArgs {\n    return {\n      subject: request.subject,\n      reply: request.reply,\n      payload: new Uint8Array(request.payload),\n      iat: request.iat,\n      requestId: request.requestId,\n      proof: request.proof,\n      requiredPermissions: structuredClone(request.requiredPermissions),\n      requiredCapabilities: [...request.requiredCapabilities],\n      policy: this.#policyForDigest(request.contextDigest, this.#now()),\n    };\n  }\n\n  #eventInput(\n    event: AuthorizationProviderEvent,\n    revokedAt: number | null,\n  ): VerifyAuthorizationEventArgs {\n    return {\n      subject: event.subject,\n      payload: new Uint8Array(event.payload),\n      eventId: event.eventId,\n      eventTime: event.eventTime,\n      proof: event.proof,\n      requiredPermissions: structuredClone(event.requiredPermissions),\n      requiredCapabilities: [...event.requiredCapabilities],\n      policy: this.#policyForDigest(event.contextDigest, this.#now(), true),\n      revokedAt,\n    };\n  }\n\n''',
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"provider request/event input block match count {count}")
# Add digest policy lookup beside the existing entry policy helper.
needle = "  #policyFor(\n    entry: ProviderContextEntry,\n"
if text.count(needle) != 1:
    raise RuntimeError("provider policy helper anchor changed")
text = text.replace(
    needle,
    '''  #policyForDigest(\n    contextDigest: string,\n    nowUnixSeconds: number,\n    historical = false,\n  ): AuthorizationContextVerificationPolicyV1 {\n    const entry = this.#contexts.get(contextDigest);\n    if (!entry) throw new Error("authorization context is not resolved");\n    return this.#policyFor(entry, nowUnixSeconds, historical);\n  }\n\n  #policyFor(\n    entry: ProviderContextEntry,\n''',
    1,
)
# Release WASM heap state on cache invalidation/eviction.
text = text.replace(
    "      if (entry.retainedUntil <= now) this.#contexts.delete(digest);",
    "      if (entry.retainedUntil <= now) {\n        this.#dropVerifiers(entry);\n        this.#contexts.delete(digest);\n      }",
)
text = text.replace("    this.#contexts.clear();\n    this.#health.manifestRevision", "    this.#clearContexts();\n    this.#health.manifestRevision", 1)
text = text.replace(
    "      for (const entry of this.#contexts.values()) {\n        entry.verified = undefined;\n      }",
    "      for (const entry of this.#contexts.values()) this.#dropVerifiers(entry);",
)
# stop() is a real lifecycle boundary; discard retained WASM handles.
text = text.replace(
    "    this.#health.healthy = false;\n    for (const waiter of this.#readyWaiters) {",
    "    this.#health.healthy = false;\n    this.#clearContexts();\n    for (const waiter of this.#readyWaiters) {",
    1,
)
# Insert helper methods before evict.
anchor = "  #evictExpiredContexts(): void {\n"
if text.count(anchor) != 1:
    raise RuntimeError("provider evict anchor changed")
text = text.replace(
    anchor,
    '''  #dropVerifiers(entry: ProviderContextEntry): void {\n    entry.requestVerifier?.free();\n    entry.eventVerifier?.free();\n    entry.requestVerifier = undefined;\n    entry.eventVerifier = undefined;\n  }\n\n  #clearContexts(): void {\n    for (const entry of this.#contexts.values()) this.#dropVerifiers(entry);\n    this.#contexts.clear();\n  }\n\n  #evictExpiredContexts(): void {\n''',
    1,
)
provider.write_text(text)

# Repeated-message conformance test proves a single handle serves many hot-path verifications.
conformance = Path("ts/packages/trellis/auth/conformance_test.ts")
text = conformance.read_text()
text = text.replace(
    'import vectors from "../../../../conformance/authorization-context/vectors.json" with {',
    'import { createAuthorizationContextVerifierWasm } from "./protocol_wasm.ts";\nimport vectors from "../../../../conformance/authorization-context/vectors.json" with {',
    1,
)
append = r'''

Deno.test("verified context handle serves repeated request and event proofs", async () => {
  const chain = vectors.completeChain as unknown as Chain & {
    rootCanonicalJson: string;
    manifestCanonicalJson: string;
    contextCanonicalJson: string;
  };
  const defaults = vectors.defaults as unknown as VectorDefaults & {
    permission: unknown;
    policy: {
      nowUnixSeconds: number;
      allowedClockSkewSeconds: number;
      maximumContextLifetimeSeconds: number;
      maximumContextBytes: number;
      maximumPermissions: number;
      maximumCapabilities: number;
      minimumManifestGeneration: number;
    };
  };
  const policy = {
    ...defaults.policy,
    refreshLeadSeconds: 60,
    refreshJitterSeconds: 0,
  };
  const verifier = await createAuthorizationContextVerifierWasm({
    root: JSON.parse(chain.rootCanonicalJson),
    manifest: JSON.parse(chain.manifestCanonicalJson),
    context: JSON.parse(chain.contextCanonicalJson),
    policy,
  });
  try {
    for (let index = 0; index < 32; index += 1) {
      const request = verifier.verifyRequest({
        subject: defaults.request.subject,
        reply: defaults.request.reply,
        payload: utf8(defaults.request.payload),
        iat: defaults.request.iat,
        requestId: defaults.request.requestId,
        proof: chain.requestProof,
        requiredPermissions: [defaults.permission as never],
        requiredCapabilities: ["platform.read"],
        policy,
      });
      assert(request.ok);
      assertEquals(request.sessionKey, chain.sessionPublicKey);

      const event = verifier.verifyEvent({
        subject: defaults.event.subject,
        payload: utf8(defaults.event.payload),
        eventId: defaults.event.eventId,
        eventTime: defaults.event.eventTime,
        proof: chain.eventProof,
        requiredPermissions: [defaults.permission as never],
        requiredCapabilities: [],
        policy,
        revokedAt: null,
      });
      assert(event.ok);
      assertEquals(event.publisher.sessionId, "ses_test");
    }
  } finally {
    verifier.free();
  }
});
'''
conformance.write_text(text.rstrip() + append + "\n")

print("WASM boundary transform complete")
