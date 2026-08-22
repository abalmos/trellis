from pathlib import Path
import re

path = Path("rust/tools/generate/tests/auto_mode_test.rs")
text = path.read_text()
placeholder = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"

old = '''fn write_ts_contract(path: &Path, id: &str, display_name: &str, kind: &str) {
    fs::write(
        path,
        format!(
            "const API = {{\\n  format: \\"trellis.api.v1\\",\\n  id: \\"{id}\\",\\n  displayName: \\"{display_name}\\",\\n  description: \\"Fixture API\\",\\n}};\\nconst PARTICIPANT = {{\\n  format: \\"trellis.participant.v1\\",\\n  id: \\"{id}\\",\\n  displayName: \\"{display_name}\\",\\n  description: \\"Fixture participant\\",\\n  kind: \\"{kind}\\",\\n  implements: {{ self: {{ api: \\"{id}\\", apiDigest: \\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\\" }} }},\\n}};\\n\\nexport default {{ API, PARTICIPANT }};\\n"
        ),
    )
    .unwrap();
}
'''
new = '''fn write_ts_contract(path: &Path, id: &str, display_name: &str, kind: &str) {
    let api_digest = trellis_contracts::ApiBuilder::new(serde_json::json!({
        "format": "trellis.api.v1",
        "id": id,
        "displayName": display_name,
        "description": "Fixture API",
    }))
    .digest()
    .unwrap();
    fs::write(
        path,
        format!(
            "const API = {{\\n  format: \\"trellis.api.v1\\",\\n  id: \\"{id}\\",\\n  displayName: \\"{display_name}\\",\\n  description: \\"Fixture API\\",\\n}};\\nconst PARTICIPANT = {{\\n  format: \\"trellis.participant.v1\\",\\n  id: \\"{id}\\",\\n  displayName: \\"{display_name}\\",\\n  description: \\"Fixture participant\\",\\n  kind: \\"{kind}\\",\\n  implements: {{ self: {{ api: \\"{id}\\", apiDigest: \\"{api_digest}\\" }} }},\\n}};\\n\\nexport default {{ API, PARTICIPANT }};\\n"
        ),
    )
    .unwrap();
}
'''
if text.count(old) != 1:
    raise SystemExit("write_ts_contract fixture shape changed")
text = text.replace(old, new, 1)


def patch_test(name: str, digests: list[str]) -> None:
    global text
    marker = f"#[test]\nfn {name}() {{"
    start = text.find(marker)
    if start < 0:
        raise SystemExit(f"missing test {name}")
    next_start = text.find("#[test]", start + len(marker))
    if next_start < 0:
        next_start = len(text)
    segment = text[start:next_start]
    count = segment.count(placeholder)
    if count != len(digests):
        raise SystemExit(
            f"{name}: expected {len(digests)} digest placeholders, found {count}"
        )
    for digest in digests:
        segment = segment.replace(placeholder, digest, 1)
    text = text[:start] + segment + text[next_start:]


# These hand-written native-artifact fixtures bypass normal TypeScript authoring,
# so their participant self binding must carry the exact semantic digest of the
# API literal they present. Keep the intentionally-invalid closed-schema fixture
# untouched because it is expected to fail API authoring lint before resolution.
patch_test(
    "prepare_bootstraps_repo_without_discover_summary",
    [
        "zUfuTGEjjiCbjt57ucSJo1FS7MO5BQUFug0bRxCMwyM",
        "IWSLPqsRVWP2jwWfRuC-arNDiq3ML32fNaH7Xit2WYs",
    ],
)
patch_test(
    "prepare_in_local_runtime_repo_keeps_typescript_package_specifiers",
    ["va2NvbdTudYLfxVBLgiwZXyEXCUvKB-ilGDWl_4yklQ"],
)
patch_test(
    "local_mode_generates_service_artifacts_from_node_project_contracts",
    [
        "7aZlI7NfeGJOqx4ypFKNCcsAf2CEL4PqJpY6IQfQLTQ",
        "7aZlI7NfeGJOqx4ypFKNCcsAf2CEL4PqJpY6IQfQLTQ",
    ],
)

remaining = [match.start() for match in re.finditer(re.escape(placeholder), text)]
if len(remaining) != 1:
    raise SystemExit(
        f"expected only the intentionally-invalid fixture placeholder to remain, found {len(remaining)}"
    )
invalid_start = text.find("#[test]\nfn prepare_warns_for_public_closed_intersect_schemas() {")
invalid_end = text.find("#[test]", invalid_start + 1)
if not (invalid_start <= remaining[0] < invalid_end):
    raise SystemExit("remaining digest placeholder is not in the intentional lint-failure fixture")

path.write_text(text)
