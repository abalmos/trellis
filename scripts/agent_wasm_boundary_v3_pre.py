from pathlib import Path

# Normalize the core contract import before the main boundary transform narrows it.
path = Path("ts/packages/trellis/contracts/trellis_core.ts")
text = path.read_text()
old = '''import {
  defineServiceContract,
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "@qlever-llc/trellis";'''
new = '''import {
  defineServiceContract,
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "../index.ts";'''
if text.count(old) != 1:
    raise RuntimeError(f"trellis_core import anchor count: {text.count(old)}")
path.write_text(text.replace(old, new, 1))

# The production Pages workflow is Linux-only apart from release packaging and
# should use the requested self-hosted runner while this cleanup is validated.
path = Path(".github/workflows/pages.yml")
text = path.read_text()
if text.count("    runs-on: ubuntu-latest") != 2:
    raise RuntimeError("pages: expected two Linux GitHub-hosted jobs")
path.write_text(
    text.replace(
        "    runs-on: ubuntu-latest",
        "    runs-on: [self-hosted, Linux, X64]",
    )
)
