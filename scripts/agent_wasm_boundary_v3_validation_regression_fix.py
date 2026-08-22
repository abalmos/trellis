from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(
            f"{label}: expected one occurrence, found {count}: {old[:120]!r}"
        )
    return text.replace(old, new, 1)


# The release workflow already builds one shared live artifact bundle. Preserve
# the cheap inventory-only proof before the full shared live execution instead
# of weakening the existing release invariant.
path = Path(".github/workflows/release.yml")
text = path.read_text()
verify_live = "  verify-live:\n"
insert_at = text.index(verify_live)
inventory = '''  verify-live-inventory:
    name: Verify shared live inventory
    needs:
      - prepare-release
      - verify-live-build
    runs-on: [self-hosted, Linux, X64]

    steps:
      - name: Check out repository
        uses: actions/checkout@v4

      - name: Download prepared release workspace
        uses: actions/download-artifact@v4
        with:
          name: prepared-release
          path: .

      - name: Download shared live artifacts
        uses: actions/download-artifact@v4
        with:
          name: integration-live-artifacts
          path: dist/integration-runtime

      - name: Set up Deno
        uses: denoland/setup-deno@v2
        with:
          deno-version: ${{ env.DENO_VERSION }}

      - name: Verify shared live inventory
        run: deno run -A -c ts/deno.json integration/live_runner.ts --inventory-only --prebuilt-only --artifacts-manifest dist/integration-runtime/manifest.json

'''
text = text[:insert_at] + inventory + text[insert_at:]
text = replace_once(
    text,
    '''  verify-live:
    name: Verify shared live lane
    needs:
      - prepare-release
      - verify-live-build
''',
    '''  verify-live:
    name: Verify shared live lane
    needs:
      - prepare-release
      - verify-live-build
      - verify-live-inventory
''',
    "release live inventory dependency",
)
path.write_text(text)


# The packet removes the package self-import from trellis_core.ts. Tighten the
# publishing guard from one known offender to zero offenders.
path = Path("ts/packages/trellis/tests/publishing_targets_test.ts")
text = path.read_text()
text = replace_once(
    text,
    '  assertEquals(offenders, ["contracts/trellis_core.ts"]);',
    '  assertEquals(offenders, []);',
    "self-import offender expectation",
)
path.write_text(text)
