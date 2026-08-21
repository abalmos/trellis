from pathlib import Path
import re

path = Path("rust/crates/trellis-test/src/lib.rs")
text = path.read_text()

# These two values are pure propagation of the identity-mutation scope. Remove
# them before the main transform converts the one remaining admin/runtime field
# into a deployment-only namespace.
for label, pattern in (
    (
        "service key scope initializer",
        r'(Ok\(\s*TrellisTestServiceKey \{[\s\S]*?)\n\s*integration_test_scope: self\.integration_test_scope\.clone\(\),',
    ),
    (
        "reconnect scope initializer",
        r'(let reconnect = TrellisTestClientReconnect \{[\s\S]*?)\n\s*integration_test_scope: self\.integration_test_scope\.clone\(\),',
    ),
):
    text, count = re.subn(pattern, r"\1", text, count=1)
    if count != 1:
        raise RuntimeError(f"expected one {label}, found {count}")

needle = "integration_test_scope: self.integration_test_scope.clone(),"
if text.count(needle) != 1:
    raise RuntimeError(
        f"expected only the admin namespace initializer to remain, found {text.count(needle)}"
    )

path.write_text(text)

# The main transform intentionally converts every typed scope field/parameter
# to a simple deployment namespace first. Its later connect_bound_user cleanup
# must therefore remove the transformed namespace parameter, not the old scope
# type. Normalize that scratch-only anchor before executing the main transform.
main_path = Path("scripts/agent_test_boundary_v1.py")
main = main_path.read_text()
old = r'''    r'    integration_test_scope: Option<trellis_rs::integration_test_scoping::IntegrationTestScope>,\n',
    "",
    "connect bound scope parameter",
    minimum=1,
)'''
new = r'''    r'    namespace: Option<String>,\n',
    "",
    "connect bound namespace parameter",
    minimum=1,
)'''
if main.count(old) != 1:
    raise RuntimeError(f"expected one connect_bound_user transform anchor, found {main.count(old)}")
main_path.write_text(main.replace(old, new, 1))
