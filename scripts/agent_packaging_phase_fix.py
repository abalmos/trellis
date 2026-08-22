from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(
            f"{label}: expected one occurrence, found {count}: {old[:120]!r}"
        )
    return text.replace(old, new, 1)


# Package source tests must not depend on ignored build output. Keep only source
# identity assertions here; built-artifact behavior belongs to the existing
# packaging phase.
path = Path("ts/packages/result/tests/package_identity_test.ts")
expected = '''import { assertEquals, assertStringIncludes } from "@std/assert";

const decoder = new TextDecoder();

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

Deno.test("result package is published as @qlever-llc/result", async () => {
  const source = await Deno.readFile(new URL("../deno.json", import.meta.url));
  const config = JSON.parse(decoder.decode(source));

  assertEquals(config.name, "@qlever-llc/result");
});

Deno.test("result package readme uses the standalone result package name", async () => {
  const source = await Deno.readTextFile(
    new URL("../README.md", import.meta.url),
  );

  assertStringIncludes(source, "@qlever-llc/result");
});

Deno.test("result npm build opts out of DNT Deno shims", async () => {
  const source = await Deno.readTextFile(
    new URL("../scripts/build_npm.ts", import.meta.url),
  );

  assertStringIncludes(source, "denoShims: false");
});

Deno.test("result npm declarations do not import DNT polyfills", async () => {
  const modTypes = await Deno.readTextFile(
    new URL("../npm/esm/mod.d.ts", import.meta.url),
  ).catch((error) => {
    if (error instanceof Deno.errors.NotFound) return "";
    throw error;
  });

  assertEquals(modTypes.includes("_dnt.polyfills"), false);
});

Deno.test("result npm build does not emit import-meta ponyfill references", async () => {
  for (
    const dir of [
      new URL("../npm/esm/", import.meta.url),
      new URL("../npm/script/", import.meta.url),
    ]
  ) {
    for await (const path of walkFiles(dir)) {
      const source = await Deno.readTextFile(path);
      assertEquals(source.includes("import-meta-ponyfill"), false);
    }
  }
});

Deno.test("result npm ESM build constructs UnexpectedError", async () => {
  const moduleUrl = new URL("../npm/esm/mod.js", import.meta.url);
  const mod: unknown = await import(moduleUrl.href);
  if (!isBuiltResultModule(mod)) {
    throw new Error("Built result module did not export UnexpectedError");
  }

  const error = new mod.UnexpectedError({ cause: new Error("boom") });
  assertEquals(error instanceof Error, true);
  assertEquals(error.name, "UnexpectedError");
});
'''
actual = path.read_text()
if actual != expected:
    raise RuntimeError("result package identity test changed from audited source")
path.write_text(
    '''import { assertEquals, assertStringIncludes } from "@std/assert";

const decoder = new TextDecoder();

Deno.test("result package is published as @qlever-llc/result", async () => {
  const source = await Deno.readFile(new URL("../deno.json", import.meta.url));
  const config = JSON.parse(decoder.decode(source));

  assertEquals(config.name, "@qlever-llc/result");
});

Deno.test("result package readme uses the standalone result package name", async () => {
  const source = await Deno.readTextFile(
    new URL("../README.md", import.meta.url),
  );

  assertStringIncludes(source, "@qlever-llc/result");
});
'''
)

packaging_test = Path("ts/tools/package_build/result_npm_test.ts")
if packaging_test.exists():
    raise RuntimeError(f"packaging test already exists: {packaging_test}")
packaging_test.write_text(
    '''import { assertEquals, assertStringIncludes } from "@std/assert";

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
'''
)

# The ordinary source-test phase owns portal and non-packaging tools only.
# Package-build assertions run after packages:build:npm in the existing
# packaging phase, so do not discover them recursively from tools here.
path = Path("ts/deno.json")
text = path.read_text()
text = replace_once(
    text,
    '"test:prepared:ui-tools": "deno test -A portals/login tools",',
    '"test:prepared:ui-tools": "deno test -A --ignore=tools/package_build portals/login tools",',
    "exclude package-build tests from source phase",
)
path.write_text(text)

# The clean Check lane must execute the packaging phase explicitly. This phase
# already owns building all npm packages before running built-artifact tests.
path = Path(".github/workflows/check.yml")
text = path.read_text()
text = replace_once(
    text,
    '''      - name: Test TypeScript packages
        run: deno task -c ts/deno.json test:prepared
''',
    '''      - name: Test TypeScript packages
        run: deno task -c ts/deno.json test:prepared

      - name: Test TypeScript package builds
        run: deno task -c ts/deno.json test:prepared:packaging
''',
    "check packaging phase",
)
path.write_text(text)
