from pathlib import Path
import re

path = Path("rust/crates/trellis/src/client/transfer.rs")
text = path.read_text()
old = "fn event_proof_v2_verifies_with_context_digest()"
new = "fn event_proof_v1_verifies_with_context_digest()"
if text.count(old) != 1:
    raise RuntimeError(
        f"transfer proof test name anchor changed: expected one occurrence, found {text.count(old)}"
    )
path.write_text(text.replace(old, new, 1))

# The request/event proof layer is first-public v1. Catch stale test/helper names
# that are easy to miss when the implementation symbols themselves are renamed.
pattern = re.compile(r"(?:request|event)_proof_v2_")
for root in ("rust", "ts", "conformance", "docs"):
    for candidate in Path(root).rglob("*"):
        if not candidate.is_file() or candidate.suffix not in {".rs", ".ts", ".tsx", ".md", ".json"}:
            continue
        match = pattern.search(candidate.read_text())
        if match:
            raise RuntimeError(
                f"stale first-public proof v2 name {match.group(0)!r} in {candidate}"
            )
