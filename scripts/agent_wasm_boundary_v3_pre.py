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

self_hosted = "    runs-on: [self-hosted, Linux, X64]"

# Ordinary Linux/X64 workflow jobs use the requested self-hosted pool. The
# release CLI matrix keeps its real ARM/macOS lanes; those are not substitutable
# with an X64 Linux runner.
for workflow, expected in [
    (".github/workflows/pages.yml", 2),
    (".github/workflows/release.yml", 15),
]:
    path = Path(workflow)
    text = path.read_text()
    hosted = "    runs-on: ubuntu-latest"
    count = text.count(hosted)
    if count != expected:
        raise RuntimeError(
            f"{workflow}: expected {expected} ordinary Linux jobs, found {count}"
        )
    path.write_text(text.replace(hosted, self_hosted))
