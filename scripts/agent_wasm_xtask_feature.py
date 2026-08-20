from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if text.count(old) != 1:
        raise RuntimeError(f"{path}: expected one occurrence: {old[:80]!r}")
    p.write_text(text.replace(old, new, 1))


replace_once(
    "rust/xtask/Cargo.toml",
    "[workspace]\n\n[dependencies]",
    '[workspace]\n\n[features]\nprotocol-wasm = ["dep:wasm-bindgen-cli-support"]\n\n[dependencies]',
)
replace_once(
    "rust/xtask/Cargo.toml",
    'wasm-bindgen-cli-support = "=0.2.125"',
    'wasm-bindgen-cli-support = { version = "=0.2.125", optional = true }',
)

p = Path("rust/xtask/src/main.rs")
text = p.read_text()
text = text.replace(
    '    #[command(name = "protocol-wasm")]\n    ProtocolWasm,',
    '    #[cfg(feature = "protocol-wasm")]\n    #[command(name = "protocol-wasm")]\n    ProtocolWasm,',
    1,
)
text = text.replace(
    '        XtaskCommand::ProtocolWasm => generate_protocol_wasm(),',
    '        #[cfg(feature = "protocol-wasm")]\n        XtaskCommand::ProtocolWasm => generate_protocol_wasm(),',
    1,
)
text = text.replace(
    'fn generate_protocol_wasm() -> Result<()> {',
    '#[cfg(feature = "protocol-wasm")]\nfn generate_protocol_wasm() -> Result<()> {',
    1,
)
text = text.replace(
    'fn base64(bytes: &[u8]) -> String {',
    '#[cfg(feature = "protocol-wasm")]\nfn base64(bytes: &[u8]) -> String {',
    1,
)
text = text.replace(
    '    #[test]\n    fn parse_protocol_wasm_command() {',
    '    #[cfg(feature = "protocol-wasm")]\n    #[test]\n    fn parse_protocol_wasm_command() {',
    1,
)
p.write_text(text)

# Only focused TS/WASM builds enable the bindgen dependency.
for path in ["ts/deno.json", ".github/workflows/check.yml"]:
    p = Path(path)
    text = p.read_text().replace(
        "cargo run --manifest-path ../rust/xtask/Cargo.toml -- protocol-wasm",
        "cargo run --manifest-path ../rust/xtask/Cargo.toml --features protocol-wasm -- protocol-wasm",
    ).replace(
        "cargo run --manifest-path rust/xtask/Cargo.toml -- protocol-wasm",
        "cargo run --manifest-path rust/xtask/Cargo.toml --features protocol-wasm -- protocol-wasm",
    )
    p.write_text(text)
