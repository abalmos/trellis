from pathlib import Path
import re

path = Path("rust/crates/trellis-test/src/lib.rs")
text = path.read_text()

for struct_name in ("TrellisTestClientReconnect", "TrellisTestServiceKey"):
    pattern = rf"(pub struct {struct_name} \{{[\s\S]*?)\n    namespace: Option<String>,([\s\S]*?\n\}})"
    text, count = re.subn(pattern, r"\1\2", text, count=1)
    if count != 1:
        raise RuntimeError(f"expected one transformed namespace field in {struct_name}, found {count}")

path.write_text(text)

# The shared-runtime manifest is private and version-locked. Every active reader
# and writer must move together; do not carry a compatibility branch.
for path in (
    Path("rust/crates/trellis-test/src/lib.rs"),
    Path("ts/packages/trellis-test/src/integration/shared_runtime_protocol.ts"),
    Path("ts/packages/trellis-test/src/integration/shared_runtime_host.ts"),
):
    content = path.read_text()
    if "version: 4" in content or "version != 4" in content or "version !== 4" in content:
        raise RuntimeError(f"stale shared-runtime manifest v4 reference in {path}")

# No identity-mutating harness helper should survive after the implementation is
# deleted. These names are intentionally not retained as no-op compatibility shims.
for root in (Path("rust/crates/trellis-test"), Path("rust/crates/trellis/tests/integration")):
    for candidate in root.rglob("*.rs"):
        content = candidate.read_text()
        for token in (
            "scoped_contract",
            "scope_authoring_source",
            "scope_artifact",
            "scope_source_value",
            "refresh_scoped_api_digests",
        ):
            if token in content:
                raise RuntimeError(f"stale integration identity helper {token!r} in {candidate}")
