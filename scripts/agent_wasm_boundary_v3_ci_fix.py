from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}: {old[:120]!r}")
    return text.replace(old, new, 1)


# `prepare` is source/protocol generation only. WASM and the embedded portal are
# consumer build state. Keep `xtask build` as the full application build path.
path = Path("rust/xtask/src/main.rs")
text = path.read_text()
text = replace_once(
    text,
    '''fn run_prepare() -> Result<()> {
    run_generate_prepare(&["--no-npm"])?;
    generate_protocol_wasm()?;
    build_embedded_login_portal()?;
    Ok(())
}
''',
    '''fn run_prepare() -> Result<()> {
    run_generate_prepare(&["--no-npm"])
}
''',
    "xtask prepare boundary",
)
text = replace_once(
    text,
    '''fn run_build(args: &[String]) -> Result<()> {
    run_prepare()?;
    let workspace_root = repo_root()?.join("rust");
''',
    '''fn run_build(args: &[String]) -> Result<()> {
    run_prepare()?;
    generate_protocol_wasm()?;
    build_embedded_login_portal()?;
    let workspace_root = repo_root()?.join("rust");
''',
    "xtask full build preparation",
)
path.write_text(text)


# Check: SDK generation is independent. WASM is built once in parallel, the
# portal consumes it, and each downstream lane downloads only what it needs.
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
            rust/tools/generate -> target
            rust/xtask -> target

      - name: Prepare generated artifacts
        run: cargo run --manifest-path rust/xtask/Cargo.toml -- prepare
''',
    "check SDK preparation",
)
prepare = replace_once(
    prepare,
    '''          path: |
            generated
            rust/crates/runtime/generated/portal
''',
    '''          path: generated
''',
    "check SDK artifact",
)
text = text[:start] + prepare + text[end:]

insert_at = text.index("\n  rust:\n")
consumer_jobs = r'''
  wasm:
    name: Protocol WASM
    runs-on: [self-hosted, Linux, X64]
    steps:
      - name: Check out repository
        uses: actions/checkout@v4

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Cache protocol WASM build
        uses: swatinem/rust-cache@v2
        with:
          workspaces: |
            rust -> target
            rust/xtask -> target

      - name: Build protocol WASM
        run: cargo run --manifest-path rust/xtask/Cargo.toml -- protocol-wasm

      - name: Upload protocol WASM
        uses: actions/upload-artifact@v4
        with:
          name: check-protocol-wasm
          path: ts/packages/trellis/auth/protocol_wasm
          retention-days: 1

  portal:
    name: Embedded login portal
    needs: [prepare, wasm]
    runs-on: [self-hosted, Linux, X64]
    steps:
      - name: Check out repository
        uses: actions/checkout@v4

      - name: Download generated artifacts
        uses: actions/download-artifact@v4
        with:
          name: check-generated
          path: .

      - name: Download protocol WASM
        uses: actions/download-artifact@v4
        with:
          name: check-protocol-wasm
          path: ts/packages/trellis/auth/protocol_wasm

      - name: Set up Deno
        uses: denoland/setup-deno@v2
        with:
          deno-version: ${{ env.DENO_VERSION }}

      - name: Build embedded portal
        run: deno task -c ts/portals/login/deno.json build:embedded

      - name: Upload embedded portal
        uses: actions/upload-artifact@v4
        with:
          name: check-portal
          path: rust/crates/runtime/generated/portal
          retention-days: 1
'''
text = text[:insert_at] + "\n" + consumer_jobs + text[insert_at:]

text = replace_once(
    text,
    '''  rust:
    name: Rust
    needs: prepare
''',
    '''  rust:
    name: Rust
    needs: [prepare, portal]
''',
    "rust dependencies",
)
text = replace_once(
    text,
    '''      - name: Download prepared artifacts
        uses: actions/download-artifact@v4
        with:
          name: check-generated
          path: .

      - name: Set up Rust
''',
    '''      - name: Download generated artifacts
        uses: actions/download-artifact@v4
        with:
          name: check-generated
          path: .

      - name: Download embedded portal
        uses: actions/download-artifact@v4
        with:
          name: check-portal
          path: rust/crates/runtime/generated/portal

      - name: Set up Rust
''',
    "rust artifacts",
)

text = replace_once(
    text,
    '''  typescript:
    name: TypeScript
    needs: prepare
''',
    '''  typescript:
    name: TypeScript
    needs: [prepare, wasm]
''',
    "typescript dependencies",
)
# The second original prepared-artifact block belongs to TypeScript now that the
# Rust block above was rewritten.
text = replace_once(
    text,
    '''      - name: Download prepared artifacts
        uses: actions/download-artifact@v4
        with:
          name: check-generated
          path: .

      - name: Set up Deno
''',
    '''      - name: Download generated artifacts
        uses: actions/download-artifact@v4
        with:
          name: check-generated
          path: .

      - name: Download protocol WASM
        uses: actions/download-artifact@v4
        with:
          name: check-protocol-wasm
          path: ts/packages/trellis/auth/protocol_wasm

      - name: Set up Deno
''',
    "typescript artifacts",
)

text = replace_once(
    text,
    '''  live:
    name: Live Trellis
    needs: prepare
''',
    '''  live:
    name: Live Trellis
    needs: [prepare, wasm, portal]
''',
    "live dependencies",
)
text = replace_once(
    text,
    '''      - name: Download prepared artifacts
        uses: actions/download-artifact@v4
        with:
          name: check-generated
          path: .

      - name: Set up Rust
''',
    '''      - name: Download generated artifacts
        uses: actions/download-artifact@v4
        with:
          name: check-generated
          path: .

      - name: Download protocol WASM
        uses: actions/download-artifact@v4
        with:
          name: check-protocol-wasm
          path: ts/packages/trellis/auth/protocol_wasm

      - name: Download embedded portal
        uses: actions/download-artifact@v4
        with:
          name: check-portal
          path: rust/crates/runtime/generated/portal

      - name: Set up Rust
''',
    "live artifacts",
)
path.write_text(text)


# Release is a consumer pipeline, so it may build WASM before portal/package
# assembly. Keep the cross-platform release matrix unchanged.
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
