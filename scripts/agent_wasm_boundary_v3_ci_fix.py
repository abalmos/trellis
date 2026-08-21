from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}: {old[:120]!r}")
    return text.replace(old, new, 1)


# Check: use the current self-hosted Linux/x64 pool, then edit only preparation.
path = Path(".github/workflows/check.yml")
text = path.read_text()
runner = "    runs-on: ubuntu-latest"
if text.count(runner) != 5:
    raise RuntimeError(f"check runners: expected five ubuntu jobs, found {text.count(runner)}")
text = text.replace(runner, "    runs-on: [self-hosted, Linux, X64]")
start = text.index("  prepare:\n")
end = text.index("\n  generated:\n", start)
prepare = text[start:end]
prepare = replace_once(
    prepare,
    '''      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
''',
    '''      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
''',
    "check prepare Rust setup",
)
prepare = replace_once(
    prepare,
    '''      - name: Cache generator build
        uses: swatinem/rust-cache@v2
        with:
          workspaces: rust/tools/generate -> target

      - name: Generate SDKs
        run: cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .

      - name: Build embedded portal
        run: deno task -c ts/portals/login/deno.json build:embedded
''',
    '''      - name: Cache preparation build
        uses: swatinem/rust-cache@v2
        with:
          workspaces: |
            rust -> target
            rust/tools/generate -> target
            rust/xtask -> target

      - name: Prepare generated artifacts
        run: cargo run --manifest-path rust/xtask/Cargo.toml -- prepare
''',
    "check preparation steps",
)
prepare = replace_once(
    prepare,
    '''            generated
            rust/crates/runtime/generated/portal
''',
    '''            generated
            rust/crates/runtime/generated/portal
            ts/packages/trellis/auth/protocol_wasm
''',
    "check prepared artifact paths",
)
path.write_text(text[:start] + prepare + text[end:])

# Release: SDK evidence first, then protocol WASM; portal/package work remains downstream.
path = Path(".github/workflows/release.yml")
text = path.read_text()
text = replace_once(
    text,
    '''      - name: Set up Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Prepare release version''',
    '''      - name: Set up Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Prepare release version''',
    "release prepare Rust setup",
)
text = replace_once(
    text,
    '''      - name: Generate release SDK artifacts
        run: time cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .
''',
    '''      - name: Generate release SDK artifacts
        run: time cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .

      - name: Build protocol WASM
        run: time cargo run --manifest-path rust/xtask/Cargo.toml -- protocol-wasm
''',
    "release SDK step",
)
path.write_text(text)
