import re
import subprocess
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(
            f"{path}: expected one occurrence, found {count}: {old[:120]!r}"
        )
    p.write_text(text.replace(old, new, 1))


# The login portal only supplies intrinsic client identity. TrellisClient owns
# contextual participant resolution at connection time.
replace_once(
    "ts/portals/login/src/lib/device_activation.ts",
    "  needsDigest: contract.PARTICIPANT_NEEDS_DIGEST,\n",
    "",
)

# Device activation bootstrap evidence genuinely contains the contextual needs
# digest. Resolve it once when the fixture starts the bootstrap request, then
# return that exact binding to callers that later connect the same device.
p = Path("ts/integration/device-activation/_fixture.ts")
text = p.read_text()
anchor = 'import { nativeProtocolPresentation } from "../../packages/trellis/contract_support/protocol_artifacts.ts";\n'
if text.count(anchor) != 1:
    raise RuntimeError("device activation intrinsic presentation import anchor changed")
text = text.replace(
    anchor,
    anchor
    + 'import { resolveNativeProtocolPresentation } from "../../packages/trellis/contract_support/protocol_resolution.ts";\n',
    1,
)
anchor = '''  ) {
    const nonce = crypto.randomUUID();'''
if text.count(anchor) != 1:
    raise RuntimeError("device activation setup request anchor changed")
text = text.replace(
    anchor,
    '''  ) {
    const presentation = resolveNativeProtocolPresentation(deviceContract);
    const nonce = crypto.randomUUID();''',
    1,
)
replace = "      participantNeedsDigest: deviceContract.PARTICIPANT_NEEDS_DIGEST,\n"
if text.count(replace) != 1:
    raise RuntimeError("device activation bootstrap digest anchor changed")
text = text.replace(
    replace,
    "      participantNeedsDigest: presentation.participantNeedsDigest,\n",
    1,
)
old = "    return { confirmationCode, flowId: review.reviewId };\n"
if text.count(old) != 1:
    raise RuntimeError("device activation setup return anchor changed")
text = text.replace(
    old,
    '''    return {
      confirmationCode,
      flowId: review.reviewId,
      participantNeedsDigest: presentation.participantNeedsDigest,
    };
''',
    1,
)
p.write_text(text)

# Browser login tests construct public clients in many scenarios. All of those
# stale caller-owned contextual fields disappear with the client API change.
p = Path("ts/browser/login_portal_smoke.browser_test.ts")
text = p.read_text()
pattern = re.compile(
    r"\n(?P<indent>[ \t]*)needsDigest:\s*(?:\n[ \t]*)?"
    r"(?:liveLocalLoginFixture|fixture)\.clientContract\.PARTICIPANT_NEEDS_DIGEST,"
)
text, removed = pattern.subn("", text)
if removed != 14:
    raise RuntimeError(f"expected 14 browser client eager digests, removed {removed}")

# The successful device flow reuses the exact contextual binding returned by
# setupActivationRequest for both the wait and the eventual device connection.
old = '''      const { confirmationCode, flowId } = await fixture.setupActivationRequest(
        runtime,
        admin,
        deploymentId,
        identity,
        provisioned.device.instanceId,
      );'''
new = '''      const { confirmationCode, flowId, participantNeedsDigest } = await fixture
        .setupActivationRequest(
          runtime,
          admin,
          deploymentId,
          identity,
          provisioned.device.instanceId,
        );'''
if text.count(old) != 3:
    raise RuntimeError("expected three browser device activation setup anchors")
# Replace the first separately because it feeds both waitForDeviceActivation and
# TrellisDevice.connect in the successful flow.
text = text.replace(old, new, 1)
old_digest = "        participantNeedsDigest: fixture.deviceContract.PARTICIPANT_NEEDS_DIGEST,\n"
if text.count(old_digest) != 1:
    raise RuntimeError("successful browser activation wait digest anchor changed")
text = text.replace(
    old_digest,
    "        participantNeedsDigest,\n",
    1,
)
old_digest = '''          participantNeedsDigest:
            fixture.deviceContract.PARTICIPANT_NEEDS_DIGEST,
'''
if text.count(old_digest) != 1:
    raise RuntimeError("successful browser device connect digest anchor changed")
text = text.replace(old_digest, "          participantNeedsDigest,\n", 1)

# Pending/rejected flows pass the same resolved binding to the negative-connect
# assertion instead of deriving contextual state from the authored contract.
if text.count(old) != 2:
    raise RuntimeError("expected two remaining browser activation setup anchors")
text = text.replace(old, new, 2)
old_call = '''        deploymentId,
        instanceId: provisioned.device.instanceId,
      });'''
new_call = '''        deploymentId,
        instanceId: provisioned.device.instanceId,
        participantNeedsDigest,
      });'''
if text.count(old_call) != 2:
    raise RuntimeError("browser rejected device connect call anchors changed")
text = text.replace(old_call, new_call, 2)
old_type = '''  deploymentId: string;
  instanceId: string;
}): Promise<void> {'''
new_type = '''  deploymentId: string;
  instanceId: string;
  participantNeedsDigest: string;
}): Promise<void> {'''
if text.count(old_type) != 1:
    raise RuntimeError("rejected device helper args anchor changed")
text = text.replace(old_type, new_type, 1)
old_digest = '''      participantNeedsDigest:
        args.fixture.deviceContract.PARTICIPANT_NEEDS_DIGEST,
'''
if text.count(old_digest) != 1:
    raise RuntimeError("rejected device helper eager digest anchor changed")
text = text.replace(
    old_digest,
    "      participantNeedsDigest: args.participantNeedsDigest,\n",
    1,
)
if "PARTICIPANT_NEEDS_DIGEST" in text:
    raise RuntimeError("browser login smoke still contains eager participant needs")
p.write_text(text)

# Browser contract authoring proves intrinsic identity only. Contextual needs do
# not exist until a runtime resolution boundary is crossed.
replace_once(
    "ts/browser/define_contract_fixture/main.ts",
    "    needsDigest: contract.PARTICIPANT_NEEDS_DIGEST,\n",
    "",
)
replace_once(
    "ts/browser/define_contract.browser_test.ts",
    "    assertEquals(identity.needsDigest.length > 0, true);\n",
    "",
)

subprocess.run(
    [
        "deno",
        "fmt",
        "-c",
        "ts/deno.json",
        "ts/portals/login/src/lib/device_activation.ts",
        "ts/integration/device-activation/_fixture.ts",
        "ts/browser/login_portal_smoke.browser_test.ts",
        "ts/browser/define_contract_fixture/main.ts",
        "ts/browser/define_contract.browser_test.ts",
    ],
    check=True,
)
