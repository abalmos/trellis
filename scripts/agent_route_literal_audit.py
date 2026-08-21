from pathlib import Path
import re

ROOTS = [Path("rust"), Path("ts"), Path("integration"), Path("conformance"), Path("docs")]
SUFFIXES = {".rs", ".ts", ".tsx", ".json", ".md", ".toml", ".yml", ".yaml"}
ROUTE = re.compile(
    r'(?P<subject>(?:rpc|operations|events|feed)\.v[1-9][0-9]*\.[A-Za-z0-9_@*><.-]+)'
)

errors: list[str] = []
for root in ROOTS:
    if not root.exists():
        continue
    for path in root.rglob("*"):
        if not path.is_file() or path.suffix not in SUFFIXES:
            continue
        # Generated package/build caches are not source-of-truth inputs.
        if any(part in {"node_modules", "target", "dist"} for part in path.parts):
            continue
        text = path.read_text(errors="ignore")
        for match in ROUTE.finditer(text):
            subject = match.group("subject")
            remainder = subject.split(".", 2)[2]
            # Broad family-version wildcard patterns are intentionally API-agnostic.
            if remainder in {">", "*"} or remainder.startswith((">.", "*.")):
                continue
            # Exact canonical routes contain a versioned API identity before the
            # logical surface. API IDs may themselves contain dot tokens.
            if re.search(r'@v[1-9][0-9]*(?:\.|$)', remainder):
                continue
            line = text.count("\n", 0, match.start()) + 1
            errors.append(f"{path}:{line}: {subject}")

if errors:
    raise SystemExit(
        "unqualified canonical Trellis route literals remain:\n  " + "\n  ".join(errors)
    )

print("all exact canonical Trellis route literals are API-qualified")
