from pathlib import Path

p = Path("ts/packages/trellis/contracts/trellis_core.ts")
text = p.read_text()
old = '''import {
  defineServiceContract,
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "@qlever-llc/trellis";
'''
new = '''import { defineServiceContract } from "../contract.ts";
import {
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "../models/trellis/rpc/TrellisSurfaceStatus.ts";
'''
if text.count(old) != 1:
    raise RuntimeError("trellis_core contract root import changed")
p.write_text(text.replace(old, new, 1))
