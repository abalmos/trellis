from pathlib import Path

# Device bootstrap is a runtime boundary: resolve contextual participant needs
# once here instead of carrying an eager digest on authored contracts.
p = Path("ts/packages/trellis/device.ts")
text = p.read_text()
old = 'import { nativeProtocolPresentation } from "./contract_support/protocol_artifacts.ts";\n'
new = 'import { resolveNativeProtocolPresentation } from "./contract_support/protocol_resolution.ts";\n'
if text.count(old) != 1: raise RuntimeError("device protocol import anchor changed")
text = text.replace(old, new, 1)
old = "  readonly PARTICIPANT: Readonly<Record<string, unknown>>;\n  readonly PARTICIPANT_NEEDS_DIGEST: string;\n"
if text.count(old) != 1: raise RuntimeError("device contract eager digest anchor changed")
text = text.replace(old, "  readonly PARTICIPANT: Readonly<Record<string, unknown>>;\n", 1)
old = "}): Promise<DeviceBootstrapResponse> {\n  for (let attempt = 0; attempt < 2; attempt += 1) {"
if text.count(old) != 1: raise RuntimeError("device bootstrap loop anchor changed")
text = text.replace(old, "}): Promise<DeviceBootstrapResponse> {\n  const presentation = resolveNativeProtocolPresentation(args.contract);\n  for (let attempt = 0; attempt < 2; attempt += 1) {", 1)
old = "      args.provisioned.participantNeedsDigest !==\n        args.contract.PARTICIPANT_NEEDS_DIGEST"
if text.count(old) != 1: raise RuntimeError("device identity digest check anchor changed")
text = text.replace(old, "      args.provisioned.participantNeedsDigest !==\n        presentation.participantNeedsDigest", 1)
old = "    const presentation = nativeProtocolPresentation(args.contract);\n"
if text.count(old) != 1: raise RuntimeError("device presentation anchor changed")
text = text.replace(old, "", 1)
old = "      participantNeedsDigest: args.contract.PARTICIPANT_NEEDS_DIGEST,\n"
if text.count(old) != 1: raise RuntimeError("device bootstrap needs anchor changed")
text = text.replace(old, "      participantNeedsDigest: presentation.participantNeedsDigest,\n", 1)
p.write_text(text)

# Browser auth request creation is likewise a runtime bootstrap boundary.
p = Path("ts/packages/trellis/auth/browser/login.ts")
text = p.read_text()
old = 'import {\n  type NativeProtocolContract,\n  nativeProtocolPresentation,\n} from "../../contract_support/protocol_artifacts.ts";'
new = 'import type { NativeProtocolContract } from "../../contract_support/protocol_artifacts.ts";\nimport { resolveNativeProtocolPresentation } from "../../contract_support/protocol_resolution.ts";'
if text.count(old) != 1: raise RuntimeError("browser login protocol import anchor changed")
text = text.replace(old, new, 1)
old = "  const presentation = nativeProtocolPresentation(args.contract);\n"
if text.count(old) != 1: raise RuntimeError("browser login presentation anchor changed")
text = text.replace(old, "  const presentation = resolveNativeProtocolPresentation(args.contract);\n", 1)
old = "    participantNeedsDigest: args.contract.PARTICIPANT_NEEDS_DIGEST,\n"
if text.count(old) != 1: raise RuntimeError("browser login eager digest anchor changed")
text = text.replace(old, "    participantNeedsDigest: presentation.participantNeedsDigest,\n", 1)
p.write_text(text)

# Tests that need contextual evidence request it explicitly, just like runtime.
p = Path("ts/packages/trellis/device/deno.test.ts")
text = p.read_text()
anchor = 'import { defineDeviceContract } from "../contract.ts";\n'
if text.count(anchor) != 1: raise RuntimeError("device test import anchor changed")
text = text.replace(anchor, anchor + 'import { resolveNativeProtocolPresentation } from "../contract_support/protocol_resolution.ts";\n', 1)
old = "const rootSecret = new Uint8Array(32).fill(1);\n"
if text.count(old) != 1: raise RuntimeError("device test root secret anchor changed")
text = text.replace(old, old + "const devicePresentation = resolveNativeProtocolPresentation(deviceContract);\n", 1)
old = "  participantNeedsDigest: deviceContract.PARTICIPANT_NEEDS_DIGEST,\n"
if text.count(old) != 1: raise RuntimeError("device test eager digest anchor changed")
text = text.replace(old, "  participantNeedsDigest: devicePresentation.participantNeedsDigest,\n", 1)
p.write_text(text)

p = Path("ts/packages/trellis/auth/browser/login.test.ts")
text = p.read_text()
anchor = '} from "../utils.ts";\n'
if text.count(anchor) != 1: raise RuntimeError("browser login test utils import anchor changed")
text = text.replace(anchor, anchor + 'import { resolveNativeProtocolPresentation } from "../../contract_support/protocol_resolution.ts";\n', 1)
anchor = 'Deno.test("startAuthRequest signs provider, contract, and canonical context", async () => {\n'
if text.count(anchor) != 1: raise RuntimeError("signed login test anchor changed")
text = text.replace(anchor, anchor + "  const signedPresentation = resolveNativeProtocolPresentation(SIGNED_CONTRACT);\n", 1)
old = "              participantNeedsDigest: SIGNED_CONTRACT.PARTICIPANT_NEEDS_DIGEST,\n"
if text.count(old) != 1: raise RuntimeError("signed login eager digest anchor changed")
text = text.replace(old, "              participantNeedsDigest: signedPresentation.participantNeedsDigest,\n", 1)
p.write_text(text)
