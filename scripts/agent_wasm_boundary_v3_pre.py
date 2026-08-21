from pathlib import Path

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
