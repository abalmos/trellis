from collections import defaultdict
from pathlib import Path
import re

SUBJECT = re.compile(r'"((?:rpc|operations|events|feed|feeds)\.v[1-9][0-9]*\.[A-Za-z0-9_.*>-]+)"')
AUTHORING_ID = re.compile(
    r'ContractBuilder::authoring\(\s*"(trellis\.integration\.[A-Za-z0-9_.-]+@v[1-9][0-9]*)"'
)
JSON_ID = re.compile(
    r'\\?"id\\?"\s*:\s*\\?"(trellis\.integration\.[A-Za-z0-9_.-]+@v[1-9][0-9]*)\\?"'
)
CONST_ID = re.compile(
    r'const\s+[A-Z0-9_]*(?:CONTRACT_)?ID\s*:\s*&str\s*=\s*"(trellis\.integration\.[A-Za-z0-9_.-]+@v[1-9][0-9]*)"'
)
PLATFORM_ROOTS = {"Auth", "EventLog", "Health", "Jobs", "State", "Trellis"}

subject_owners: dict[str, set[str]] = defaultdict(set)
identity_owners: dict[str, set[str]] = defaultdict(set)
root = Path("rust/crates/trellis/tests/integration")
for path in root.glob("*.rs"):
    module = path.stem
    content = path.read_text()
    for subject in SUBJECT.findall(content):
        parts = subject.split(".")
        logical_root = parts[2] if len(parts) >= 3 else ""
        # Platform subjects were intentionally never scoped, so sharing them is
        # not a new consequence of deleting IntegrationTestScope.
        if logical_root in PLATFORM_ROOTS:
            continue
        subject_owners[subject].add(module)
    for participant_id in [
        *AUTHORING_ID.findall(content),
        *JSON_ID.findall(content),
        *CONST_ID.findall(content),
    ]:
        identity_owners[participant_id].add(module)

subject_collisions = {
    subject: sorted(modules)
    for subject, modules in subject_owners.items()
    if len(modules) > 1
}
identity_collisions = {
    participant_id: sorted(modules)
    for participant_id, modules in identity_owners.items()
    if len(modules) > 1
}
if subject_collisions or identity_collisions:
    lines = ["integration fixture identities are not globally unique:"]
    for subject, modules in sorted(subject_collisions.items()):
        lines.append(f"  subject {subject}: {', '.join(modules)}")
    for participant_id, modules in sorted(identity_collisions.items()):
        lines.append(f"  participant {participant_id}: {', '.join(modules)}")
    raise SystemExit("\n".join(lines))

print(
    "validated "
    f"{len(subject_owners)} non-platform subjects and "
    f"{len(identity_owners)} integration participant/API identities: "
    "no cross-fixture collisions"
)
