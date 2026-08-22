from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(
            f"{label}: expected one occurrence, found {count}: {old[:120]!r}"
        )
    return text.replace(old, new, 1)


path = Path("ts/packages/trellis/tests/publishing_targets_test.ts")
text = path.read_text()

# The standalone live-inventory pass was deliberately removed earlier in the
# cleanup. Make the release guard assert that simplification instead of carrying
# a stale expectation for the deleted lane.
text = replace_once(
    text,
    '''  assertStringIncludes(releaseWorkflow, "\\n  verify-live-inventory:");
  assertStringIncludes(
    releaseWorkflow,
    "integration/live_runner.ts --inventory-only --prebuilt-only --artifacts-manifest dist/integration-runtime/manifest.json",
  );
''',
    '''  assertEquals(releaseWorkflow.includes("\\n  verify-live-inventory:"), false);
  assertEquals(releaseWorkflow.includes("--inventory-only"), false);
''',
    "stale live inventory assertions",
)
text = replace_once(
    text,
    '  assertStringIncludes(verifyLive, "- verify-live-inventory");',
    '  assertEquals(verifyLive.includes("- verify-live-inventory"), false);',
    "stale live inventory dependency assertion",
)

# The packet removes the package self-import from trellis_core.ts. Tighten the
# publishing guard from one known offender to zero offenders.
text = replace_once(
    text,
    '  assertEquals(offenders, ["contracts/trellis_core.ts"]);',
    '  assertEquals(offenders, []);',
    "self-import offender expectation",
)
path.write_text(text)
