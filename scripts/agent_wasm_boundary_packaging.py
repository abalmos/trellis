from pathlib import Path

p = Path("ts/packages/trellis/tests/npm_artifact_smoke_test.ts")
text = p.read_text().rstrip()
addition = r'''

Deno.test("trellis npm artifact includes generated protocol WASM", async () => {
  const packageJsonUrl = new URL("../npm/package.json", import.meta.url);
  try {
    await Deno.stat(packageJsonUrl);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return;
    throw error;
  }

  for (const format of ["esm", "script"]) {
    const wasm = new URL(
      `../npm/${format}/auth/protocol_wasm/trellis_protocol_wasm_bg.wasm`,
      import.meta.url,
    );
    const info = await Deno.stat(wasm);
    assertEquals(info.isFile, true, wasm.pathname);
    assertEquals((info.size ?? 0) > 0, true, wasm.pathname);
  }
});
'''
p.write_text(text + addition + "\n")
