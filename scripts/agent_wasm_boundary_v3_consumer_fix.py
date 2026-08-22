from pathlib import Path

# Contract conformance tests exercise the contextual resolver, so this command
# explicitly requests protocol WASM. `prepare` itself remains generation-only.
p = Path("ts/deno.json")
text = p.read_text()
old = '    "test:contracts": "deno task prepare && deno test -A packages/trellis/contract_support/protocol_test.ts packages/trellis/contract_support/protocol_artifacts_test.ts packages/trellis/contract_support/descriptors_test.ts",'
new = '    "test:contracts": "deno task prepare && deno task protocol:wasm && deno test -A packages/trellis/contract_support/protocol_test.ts packages/trellis/contract_support/protocol_artifacts_test.ts packages/trellis/contract_support/descriptors_test.ts",'
if text.count(old) != 1:
    raise RuntimeError(f"test:contracts task anchor changed: {text.count(old)}")
p.write_text(text.replace(old, new, 1))
