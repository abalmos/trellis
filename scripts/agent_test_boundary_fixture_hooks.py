from pathlib import Path


path = Path("rust/crates/trellis/tests/integration/rpc.rs")
text = path.read_text()
old = "    assert_ne!(client_subject, EntityGetRpc::SUBJECT);\n"
if text.count(old) != 1:
    raise RuntimeError(
        f"rpc.rs: expected one old descriptor-rewrite assertion, found {text.count(old)}"
    )
text = text.replace(
    old,
    "    assert_eq!(client_subject, EntityGetRpc::SUBJECT);\n",
    1,
)
path.write_text(text)

# The main deletion pass converts every simple descriptor test hook to the
# authored string directly. Auth's two multiline platform-subject calls are
# handled by the auth fixture pass before this residual check.
for path in Path("rust/crates/trellis/tests/integration").glob("*.rs"):
    content = path.read_text()
    if "integration_test_descriptor_subject" in content or "integration_test_descriptor_capability" in content:
        raise RuntimeError(f"stale descriptor-scoping fixture hook in {path}")
