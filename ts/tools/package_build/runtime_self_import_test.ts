import { assert } from "@std/assert";
import { fromFileUrl } from "@std/path";
import { build } from "@deno/dnt";

Deno.test("npm runtime self-imports stay inside transformed compiler sources", async () => {
  const root = fromFileUrl(new URL("../../packages/trellis/", import.meta.url));
  const output = await Deno.makeTempDir({ dir: root, prefix: ".npm-compile-" });
  try {
    await build({
      entryPoints: [`${root}index.ts`, `${root}browser.ts`],
      importMap: `${root}import_map.npm.json`,
      outDir: output,
      package: { name: "@qlever-llc/trellis", version: "0.0.0" },
      shims: { deno: true },
      test: false,
      typeCheck: false,
      skipNpmInstall: true,
      scriptModule: false,
    });
    // A self-import escaping to the original runtime changes TypeScript's source
    // root: browser.js then lands under esm/<temporary-output>/src/ instead.
    assert((await Deno.stat(`${output}/esm/browser.js`)).isFile);
    assert(
      (await Deno.readTextFile(`${output}/esm/index.js`)).includes(
        "_dnt.polyfills.js",
      ),
    );
  } finally {
    await Deno.remove(output, { recursive: true });
  }
});
