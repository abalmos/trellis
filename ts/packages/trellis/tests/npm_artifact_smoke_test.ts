import { assertEquals } from "@std/assert";
import { join } from "@std/path";

const forbiddenImportPattern =
  /(?:from|require\()\s*["']@qlever-llc\/trellis-(?!sdk\b)[^"']+["']/;
const staleCliArtifactPattern =
  /defineCliContract|"service" \| "app" \| "device" \| "cli"|defineClientContract\("cli"/;
const privateGeneratedSdkBuildPattern = /\.build\/generated-sdk/;
const dntShimDenoRuntimeDetectionPattern = /"Deno" in dntShim\.dntGlobalThis/;
const rawTransportDeclarationPattern =
  /NatsConnection|natsConnection|nc: NatsConnection|createConnectedService|connectTrellisServiceWithRuntimeDeps|connectDeviceWithDeps/;
const forbiddenBrowserArtifactPattern =
  /_dnt\.shims|@deno\/shim-deno|node:(?:fs|os|module)|\bnew\s+Function\b|\beval\s*\(/;
const moduleSpecifierPattern =
  /(?:import|export)\s+(?:[^"']*?\s+from\s+)?["']([^"']+)["']|import\(\s*["']([^"']+)["']\s*\)|require\(\s*["']([^"']+)["']\s*\)/g;

Deno.test("trellis npm package does not publish a generator CLI", async () => {
  try {
    await Deno.stat(new URL("../npm/package.json", import.meta.url));
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return;
    throw error;
  }
  const packageJson = JSON.parse(
    await Deno.readTextFile(new URL("../npm/package.json", import.meta.url)),
  );
  assertEquals(packageJson.bin, undefined);
});

async function* walkFiles(dir: string): AsyncGenerator<string> {
  for await (const entry of Deno.readDir(dir)) {
    const path = join(dir, entry.name);
    if (entry.isDirectory) {
      yield* walkFiles(path);
    } else {
      yield path;
    }
  }
}

async function collectRelativeJavaScriptGraph(
  entrypoint: URL,
): Promise<Map<string, string>> {
  const pending = [entrypoint];
  const visited = new Map<string, string>();

  while (pending.length) {
    const fileUrl = pending.pop();
    if (!fileUrl || visited.has(fileUrl.href)) continue;

    const source = await Deno.readTextFile(fileUrl);
    visited.set(fileUrl.href, source);

    for (const match of source.matchAll(moduleSpecifierPattern)) {
      const specifier = match[1] ?? match[2] ?? match[3];
      if (!specifier || !specifier.startsWith(".")) continue;
      if (!specifier.endsWith(".js")) continue;
      pending.push(new URL(specifier, fileUrl));
    }
  }

  return visited;
}

Deno.test("trellis npm artifact only depends on allowed published Trellis packages", async () => {
  const npmDir = new URL("../npm", import.meta.url);
  try {
    await Deno.stat(new URL("../npm/package.json", import.meta.url));
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return;
    }
    throw error;
  }
  const packageJson = JSON.parse(
    await Deno.readTextFile(new URL("../npm/package.json", import.meta.url)),
  );

  assertEquals(
    Object.keys(packageJson.dependencies).includes("@qlever-llc/result"),
    true,
  );
  assertEquals(packageJson.dependencies["drizzle-orm"], undefined);
  assertEquals(packageJson.peerDependencies["drizzle-orm"], "^0.44.7");
  assertEquals(packageJson.peerDependenciesMeta["drizzle-orm"], {
    optional: true,
  });
  assertEquals(
    Object.keys(packageJson.dependencies).some((name: string) =>
      name.startsWith("@qlever-llc/trellis-")
    ),
    false,
  );
  assertEquals(
    Object.keys(packageJson.exports).some((name: string) =>
      name.startsWith("./host") || name.startsWith("./internal/") ||
      name.startsWith("./server")
    ),
    false,
  );

  for (const format of ["esm", "script"]) {
    await assertNotExists(new URL(`../npm/${format}/host`, import.meta.url));
    await assertNotExists(new URL(`../npm/${format}/server`, import.meta.url));
    await assertNotExists(
      new URL(`../npm/${format}/server.js`, import.meta.url),
    );
    await assertNotExists(
      new URL(`../npm/${format}/server_logger.js`, import.meta.url),
    );
  }

  for await (const filePath of walkFiles(join(npmDir.pathname, "esm"))) {
    if (!filePath.endsWith(".js") && !filePath.endsWith(".d.ts")) continue;
    const source = await Deno.readTextFile(filePath);
    assertEquals(forbiddenImportPattern.test(source), false, filePath);
    assertEquals(staleCliArtifactPattern.test(source), false, filePath);
    assertEquals(privateGeneratedSdkBuildPattern.test(source), false, filePath);
  }

  for await (const filePath of walkFiles(join(npmDir.pathname, "script"))) {
    if (!filePath.endsWith(".js") && !filePath.endsWith(".d.ts")) continue;
    const source = await Deno.readTextFile(filePath);
    assertEquals(forbiddenImportPattern.test(source), false, filePath);
    assertEquals(staleCliArtifactPattern.test(source), false, filePath);
    assertEquals(privateGeneratedSdkBuildPattern.test(source), false, filePath);
  }
});

Deno.test("trellis npm package does not publish generated SDKs", async () => {
  const packageJsonUrl = new URL("../npm/package.json", import.meta.url);
  try {
    await Deno.stat(packageJsonUrl);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return;
    }
    throw error;
  }

  const packageJson = JSON.parse(await Deno.readTextFile(packageJsonUrl));
  assertEquals(packageJson.exports["."], {
    browser: "./esm/browser.js",
    import: "./esm/index.js",
    require: "./script/index.js",
  });
  assertEquals(
    Object.keys(packageJson.exports).some((name) => name.startsWith("./sdk/")),
    false,
  );
  assertEquals(packageJson.exports["./service/drizzle"], {
    import: "./esm/service/drizzle.js",
    require: "./script/service/drizzle.js",
  });

  await assertNotExists(new URL("../npm/esm/generated-sdk", import.meta.url));
  await assertNotExists(
    new URL("../npm/script/generated-sdk", import.meta.url),
  );
  await assertNotExists(
    new URL("../npm/esm/sdk/_generated", import.meta.url),
  );
  await assertNotExists(
    new URL("../npm/script/sdk/_generated", import.meta.url),
  );
});

Deno.test("trellis npm browser graph excludes DNT and Node shims", async () => {
  const browserEntrypoint = new URL("../npm/esm/browser.js", import.meta.url);
  try {
    await Deno.stat(browserEntrypoint);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return;
    throw error;
  }

  const graph = await collectRelativeJavaScriptGraph(browserEntrypoint);
  for (const [fileHref, source] of graph) {
    assertEquals(
      forbiddenBrowserArtifactPattern.test(source),
      false,
      fileHref,
    );
  }
});

Deno.test("trellis npm public export declarations hide raw NATS handles", async () => {
  const packageJsonUrl = new URL("../npm/package.json", import.meta.url);
  try {
    await Deno.stat(packageJsonUrl);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return;
    throw error;
  }

  const packageJson = JSON.parse(await Deno.readTextFile(packageJsonUrl));
  for (
    const exportTarget of Object.values(packageJson.exports) as Array<
      Record<"import" | "require", string>
    >
  ) {
    for (const target of Object.values(exportTarget)) {
      const declarationTarget = target.replace(/\.js$/, ".d.ts");
      const source = await Deno.readTextFile(
        new URL(`../npm/${declarationTarget}`, import.meta.url),
      );
      assertEquals(
        rawTransportDeclarationPattern.test(source),
        false,
        declarationTarget,
      );
    }
  }
});

Deno.test("trellis npm runtime transport falls back to npm native transport in Deno", async () => {
  const packageJsonUrl = new URL("../npm/package.json", import.meta.url);
  const esmRuntimeTransport = new URL(
    "../npm/esm/runtime_transport.js",
    import.meta.url,
  );
  const scriptRuntimeTransport = new URL(
    "../npm/script/runtime_transport.js",
    import.meta.url,
  );

  try {
    await Deno.stat(packageJsonUrl);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return;
    throw error;
  }

  await Deno.stat(esmRuntimeTransport);
  await Deno.stat(scriptRuntimeTransport);
  for (const path of [esmRuntimeTransport, scriptRuntimeTransport]) {
    const source = await Deno.readTextFile(path);
    assertEquals(
      source.includes('["@nats-io", "transport-deno"].join("/")'),
      true,
      path.pathname,
    );
    assertEquals(
      source.includes('"transport-node"'),
      true,
      path.pathname,
    );
    assertEquals(
      dntShimDenoRuntimeDetectionPattern.test(source),
      false,
      path.pathname,
    );
  }
});

async function assertNotExists(url: URL): Promise<void> {
  try {
    await Deno.stat(url);
    throw new Error(`Expected ${url.pathname} not to exist`);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return;
    throw error;
  }
}
