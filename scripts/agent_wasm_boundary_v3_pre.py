from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    target.write_text(text.replace(old, new, 1))


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

# Console bundles the runtime resolver, so local dev/build/check must explicitly
# materialize protocol WASM once it is no longer checked into source control.
for task, command in {
    '"dev"': '"dev": "deno task prepare && deno run -A vite dev"',
    '"build"': '"build": "deno task prepare && deno run -A vite build"',
    '"check"': '"check": "deno task prepare && deno run -A @sveltejs/kit sync && deno run -A svelte-check --tsconfig ./tsconfig.check.json"',
}.items():
    pass
replace_once(
    "ts/apps/console/deno.json",
    '    "dev": "deno task prepare && deno run -A vite dev",',
    '    "dev": "deno task prepare && deno task -c ../../deno.json protocol:wasm && deno run -A vite dev",',
)
replace_once(
    "ts/apps/console/deno.json",
    '    "build": "deno task prepare && deno run -A vite build",',
    '    "build": "deno task prepare && deno task -c ../../deno.json protocol:wasm && deno run -A vite build",',
)
replace_once(
    "ts/apps/console/deno.json",
    '    "check": "deno task prepare && deno run -A @sveltejs/kit sync && deno run -A svelte-check --tsconfig ./tsconfig.check.json"',
    '    "check": "deno task prepare && deno task -c ../../deno.json protocol:wasm && deno run -A @sveltejs/kit sync && deno run -A svelte-check --tsconfig ./tsconfig.check.json"',
)

# API-doc generation imports the Trellis package surface and therefore is also
# a direct protocol-WASM consumer.
replace_once(
    "docs/deno.json",
    '    "docs:api": "deno task -c ../ts/deno.json prepare && deno run -A ./scripts/generate_ts_api_docs.ts",',
    '    "docs:api": "deno task -c ../ts/deno.json prepare && deno task -c ../ts/deno.json protocol:wasm && deno run -A ./scripts/generate_ts_api_docs.ts",',
)

# Pages bypasses those Deno tasks and invokes Vite/docs generation directly.
# Build WASM only for a source tree that does not already contain the historical
# checked-in artifact; this keeps the latest-release worktree buildable while
# current/future source uses the explicit xtask boundary.
path = Path(".github/workflows/pages.yml")
text = path.read_text()
if text.count("    runs-on: ubuntu-latest") != 2:
    raise RuntimeError("pages: expected two Linux GitHub-hosted jobs")
text = text.replace(
    "    runs-on: ubuntu-latest",
    "    runs-on: [self-hosted, Linux, X64]",
)
text = text.replace(
    '''      - uses: dtolnay/rust-toolchain@stable
''',
    '''      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
''',
    1,
)
anchor = '''          build_docs() {
'''
helper = '''          prepare_protocol_wasm() {
            local repo_root="$1"

            if [ -f "${repo_root}/ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js" ]; then
              return 0
            fi

            echo "Building protocol WASM for ${repo_root}"
            cargo run --manifest-path "${repo_root}/rust/xtask/Cargo.toml" -- protocol-wasm
          }

'''
if text.count(anchor) != 1:
    raise RuntimeError("pages: build_docs anchor changed")
text = text.replace(anchor, helper + anchor, 1)
anchor = '''          prepare_once "${release_worktree}" "${latest_tag}"
          prepare_once "${current_root}" "${current_tag}"

          build_docs'''
replacement = '''          prepare_once "${release_worktree}" "${latest_tag}"
          prepare_once "${current_root}" "${current_tag}"

          prepare_protocol_wasm "${release_docs_root}"
          prepare_protocol_wasm "${current_root}"
          prepare_protocol_wasm "${release_console_root}"

          build_docs'''
if text.count(anchor) != 1:
    raise RuntimeError("pages: prepared roots anchor changed")
path.write_text(text.replace(anchor, replacement, 1))
