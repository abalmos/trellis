from collections import defaultdict
from pathlib import Path
import re

SUBJECT = re.compile(r'"((?:rpc|operations|events|feed)\.v[1-9][0-9]*\.[A-Za-z0-9_.*>-]+)"')
PLATFORM_ROOTS = {"Auth", "EventLog", "Health", "Jobs", "State", "Trellis"}

owners: dict[str, set[str]] = defaultdict(set)
root = Path("rust/crates/trellis/tests/integration")
for path in root.glob("*.rs"):
    module = path.stem
    for subject in SUBJECT.findall(path.read_text()):
        parts = subject.split(".")
        logical_root = parts[2] if len(parts) >= 3 else ""
        if logical_root in PLATFORM_ROOTS:
            continue
        owners[subject].add(module)

collisions = {
    subject: sorted(modules)
    for subject, modules in owners.items()
    if len(modules) > 1
}
if collisions:
    lines = ["cross-fixture integration subjects are not globally unique:"]
    for subject, modules in sorted(collisions.items()):
        lines.append(f"  {subject}: {', '.join(modules)}")
    raise SystemExit("\n".join(lines))

print(f"validated {len(owners)} non-platform integration subjects: no cross-fixture collisions")
