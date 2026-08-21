from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


# Check prepares SDKs -> WASM -> portal once and hands all ignored inputs onward.
replace_once(
    ".github/workflows/check.yml",
    '''      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
''',
    '''      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
''',
)
replace_once(
    ".github/workflows/check.yml",
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
)
replace_once(
    ".github/workflows/check.yml",
    '''            generated
            rust/crates/runtime/generated/portal
''',
    '''            generated
            rust/crates/runtime/generated/portal
            ts/packages/trellis/auth/protocol_wasm
''',
)

# Release generates SDK evidence first, then protocol WASM, then portal/package work.
replace_once(
    ".github/workflows/release.yml",
    '''      - name: Set up Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Prepare release version''',
    '''      - name: Set up Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Prepare release version''',
)
replace_once(
    ".github/workflows/release.yml",
    '''      - name: Generate release SDK artifacts
        run: time cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .
''',
    '''      - name: Generate release SDK artifacts
        run: time cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .

      - name: Build protocol WASM
        run: time cargo run --manifest-path rust/xtask/Cargo.toml -- protocol-wasm
''',
)
