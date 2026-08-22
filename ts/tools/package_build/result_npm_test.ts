import { assertEquals, assertStringIncludes } from "@std/assert";

const resultPackage = new URL("../../packages/result/", import.meta.url);

type BuiltResultModule = {
  UnexpectedError: new (options?: { cause?: unknown }) => Error;
};

async function* walkFiles(dir: URL): AsyncGenerator<URL> {
  for await (const entry of Deno.readDir(dir)) {
    if (entry.isDirectory) {
      yield* walkFiles(new URL(`${entry.name}/`, dir));
    } else {
      yield new URL(entry.name, dir);
    }
  }
}

function isBuiltResultModule(value: unknown): value is BuiltResultModule {
  if (!value || typeof value !== "object") return false;

  return typeof Object.getOwnPropertyDescriptor(value, "UnexpectedError")
    ?.value === "function";
}

Deno.test("result npm build opts out of DNT Deno shims", async () => {
  const source = await Deno.readTextFile(
    new URL("scripts/build_npm.ts", resultPackage),
  );

  assertStringIncludes(source, "denoShims: false");
});

Deno.test("result npm declarations do not import DNT polyfills", async () => {
  const modTypes = await Deno.readTextFile(
    new URL("npm/esm/mod.d.ts", resultPackage),
  );

  assertEquals(modTypes.includes("_dnt.polyfills"), false);
});

Deno.test("result npm build does not emit import-meta ponyfill references", async () => {
  for (
    const dir of [
      new URL("npm/esm/", resultPackage),
      new URL("npm/script/", resultPackage),
    ]
  ) {
    for await (const path of walkFiles(dir)) {
      const source = await Deno.readTextFile(path);
      assertEquals(source.includes("import-meta-ponyfill"), false);
    }
  }
});

Deno.test("result npm ESM build constructs UnexpectedError", async () => {
  const moduleUrl = new URL("npm/esm/mod.js", resultPackage);
  const mod: unknown = await import(moduleUrl.href);
  if (!isBuiltResultModule(mod)) {
    throw new Error("Built result module did not export UnexpectedError");
  }

  const error = new mod.UnexpectedError({ cause: new Error("boom") });
  assertEquals(error instanceof Error, true);
  assertEquals(error.name, "UnexpectedError");
});
