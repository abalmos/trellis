import { buildDntPackage } from "../../../tools/package_build/build_dnt_package.ts";

const npmPackageJsonPath = new URL("../npm/package.json", import.meta.url);
const npmDirUrl = new URL("../npm/", import.meta.url);

const moduleSpecifierPattern =
  /(?:import|export)\s+(?:[^"']*?\s+from\s+)?["']([^"']+)["']|import\(\s*["']([^"']+)["']\s*\)|require\(\s*["']([^"']+)["']\s*\)/g;

function rewriteCjsPath(path: string): string {
  return path;
}

function normalizeExportValue(value: unknown): unknown {
  if (typeof value === "string") {
    return rewriteCjsPath(value);
  }

  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return value;
  }

  return Object.fromEntries(
    Object.entries(value).map(([key, nestedValue]) => [
      key,
      key === "require" ? normalizeExportValue(nestedValue) : nestedValue,
    ]),
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

async function removeMissingRequireCondition(value: unknown): Promise<unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return value;
  }

  const entries = await Promise.all(
    Object.entries(value).map(async ([key, nestedValue]) => {
      if (key !== "require" || typeof nestedValue !== "string") {
        return [key, nestedValue] as const;
      }

      const fileUrl = new URL(nestedValue, npmPackageJsonPath);
      try {
        await Deno.stat(fileUrl);
        return [key, nestedValue] as const;
      } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
          return undefined;
        }
        throw error;
      }
    }),
  );

  return Object.fromEntries(entries.filter((entry) => entry !== undefined));
}

async function pathExists(path: string): Promise<boolean> {
  try {
    await Deno.stat(new URL(path, npmPackageJsonPath));
    return true;
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return false;
    }
    throw error;
  }
}

async function removeFileDntPolyfills(fileUrl: URL) {
  const original = await Deno.readTextFile(fileUrl);
  const updated = original
    .replaceAll("dntShim.dntGlobalThis", "globalThis")
    .replace(
      /^import \* as dntShim from ["'](?:\.\/|\.\.\/)_dnt\.shims\.js["'];\r?\n/gm,
      "",
    )
    .replace(/^import ["'](?:\.\/|\.\.\/)_dnt\.polyfills\.js["'];\r?\n/gm, "")
    .replace(
      /^require\(["'](?:\.\/|\.\.\/)_dnt\.polyfills\.js["']\);\r?\n/gm,
      "",
    );

  if (updated !== original) {
    await Deno.writeTextFile(fileUrl, updated);
  }
}

async function collectRelativeJavaScriptGraph(
  entrypoint: URL,
): Promise<URL[]> {
  const pending = [entrypoint];
  const visited = new Map<string, URL>();

  while (pending.length) {
    const fileUrl = pending.pop();
    if (!fileUrl || visited.has(fileUrl.href)) continue;

    const source = await Deno.readTextFile(fileUrl);
    visited.set(fileUrl.href, fileUrl);

    for (const match of source.matchAll(moduleSpecifierPattern)) {
      const specifier = match[1] ?? match[2] ?? match[3];
      if (!specifier?.startsWith(".")) continue;
      if (!specifier.endsWith(".js")) continue;
      pending.push(new URL(specifier, fileUrl));
    }
  }

  return [...visited.values()];
}

async function removeBrowserGraphDntPolyfills() {
  for (const format of ["esm", "script"]) {
    const entrypoint = new URL(
      `../npm/${format}/browser.js`,
      import.meta.url,
    );
    for (const fileUrl of await collectRelativeJavaScriptGraph(entrypoint)) {
      await removeFileDntPolyfills(fileUrl);
    }
  }
}

async function normalizeExportTargets(value: unknown): Promise<unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return value;
  }

  const entries = await Promise.all(
    Object.entries(value).map(async ([condition, nestedValue]) => {
      if (
        (condition !== "import" && condition !== "require") ||
        typeof nestedValue !== "string"
      ) {
        return [condition, nestedValue] as const;
      }

      if (await pathExists(nestedValue)) {
        return [condition, nestedValue] as const;
      }

      const fallback = nestedValue.replace(
        /^\.\/(esm|script)\//,
        "./$1/npm/src/",
      );
      if (fallback !== nestedValue && await pathExists(fallback)) {
        return [condition, fallback] as const;
      }

      return undefined;
    }),
  );

  return Object.fromEntries(entries.filter((entry) => entry !== undefined));
}

async function* walkFiles(dir: URL): AsyncGenerator<URL> {
  for await (const entry of Deno.readDir(dir)) {
    const entryUrl = new URL(entry.name, dir);
    if (entry.isDirectory) {
      yield* walkFiles(new URL(`${entry.name}/`, dir));
      continue;
    }

    yield entryUrl;
  }
}

async function normalizeModuleSpecifiers() {
  const relativeTsSpecifierPattern = /(["'])(\.{1,2}\/[^"']+)\.ts\1/g;

  for await (const fileUrl of walkFiles(npmDirUrl)) {
    if (!fileUrl.pathname.endsWith(".js")) {
      continue;
    }

    const original = await Deno.readTextFile(fileUrl);
    const updated = original.replace(
      relativeTsSpecifierPattern,
      (_match, quote, specifier) => `${quote}${specifier}.js${quote}`,
    );

    if (updated !== original) {
      await Deno.writeTextFile(fileUrl, updated);
    }
  }
}

async function normalizePackageJsonExports() {
  const packageJson = JSON.parse(await Deno.readTextFile(npmPackageJsonPath));
  const exports = packageJson.exports ?? {};
  const normalizedEntries = await Promise.all(
    Object.entries(exports).map(async ([key, value]) => {
      if (key === ".") {
        const rootExportValue = await removeMissingRequireCondition(
          normalizeExportValue(value),
        );
        return [
          key,
          {
            browser: "./esm/browser.js",
            ...(isRecord(rootExportValue) ? rootExportValue : {}),
          },
        ];
      }

      const normalizedKey = key
        .replace("./ts/packages/trellis", ".")
        .replace(/\/mod$/, "")
        .replace(/\/index$/, "");

      const normalizedValue = normalizeExportValue(value);
      return [
        normalizedKey,
        await normalizeExportTargets(normalizedValue),
      ];
    }),
  );

  packageJson.exports = Object.fromEntries(normalizedEntries);
  delete packageJson.bin;
  delete packageJson.dependencies?.["drizzle-orm"];
  packageJson.peerDependenciesMeta = {
    ...(packageJson.peerDependenciesMeta ?? {}),
    "drizzle-orm": { optional: true },
  };
  if (typeof packageJson.main === "string") {
    packageJson.main = rewriteCjsPath(packageJson.main);
  }
  await Deno.writeTextFile(
    npmPackageJsonPath,
    JSON.stringify(packageJson, null, 2) + "\n",
  );
}

await buildDntPackage({
  buildRoot: "../../..",
  denoConfigPath: "./deno.npm.json",
  importMap: "./import_map.npm.json",
  skipNpmInstall: true,
  compilerOptions: {
    stripInternal: true,
  },
  entryPoints: [
    "./ts/packages/trellis/index.ts",
    "./ts/packages/trellis/auth.ts",
    "./ts/packages/trellis/auth/browser.ts",
    "./ts/packages/trellis/auth/file.ts",
    "./ts/packages/trellis/browser.ts",
    "./ts/packages/trellis/contracts.ts",
    "./ts/packages/trellis/device.ts",
    "./ts/packages/trellis/device/deno.ts",
    "./ts/packages/trellis/errors/index.ts",
    "./ts/packages/trellis/service/mod.ts",
    "./ts/packages/trellis/service/drizzle.ts",
    "./ts/packages/trellis/service/deno.ts",
    "./ts/packages/trellis/service/node.ts",
    "./ts/packages/trellis/telemetry.ts",
  ],
  description:
    "Client-side Trellis runtime, models, and contract helpers for TypeScript applications.",
  dependencies: {
    "@opentelemetry/api": "^1.9.0",
    "@opentelemetry/exporter-metrics-otlp-proto": "^0.56.0",
    "@opentelemetry/exporter-trace-otlp-proto": "^0.56.0",
    "@opentelemetry/resources": "^1.30.1",
    "@opentelemetry/sdk-metrics": "^1.30.1",
    "@opentelemetry/sdk-trace-base": "^1.30.1",
    "@opentelemetry/sdk-trace-node": "^1.30.1",
    "@opentelemetry/semantic-conventions": "^1.28.0",
    "@nats-io/jetstream": "^3.3.0",
    "@nats-io/kv": "^3.2.0",
    "@nats-io/obj": "^3.3.1",
    "@nats-io/nats-core": "^3.3.1",
    "@nats-io/transport-node": "^3.3.1",
    "@noble/curves": "^2.0.1",
    "@noble/hashes": "1.8.0",
    "@qlever-llc/result": "^0.12.0",
    "js-sha256": "^0.11.1",
    pino: "^9.11.0",
    tweetnacl: "^1.0.3",
    "ts-deepmerge": "^7.0.3",
    typebox: "^1.0.15",
    ulid: "^3.0.1",
  },
  npmInstallDeps: {
    "@opentelemetry/api": "^1.9.0",
    "@opentelemetry/exporter-metrics-otlp-proto": "^0.56.0",
    "@opentelemetry/exporter-trace-otlp-proto": "^0.56.0",
    "@opentelemetry/resources": "^1.30.1",
    "@opentelemetry/sdk-metrics": "^1.30.1",
    "@opentelemetry/sdk-trace-base": "^1.30.1",
    "@opentelemetry/sdk-trace-node": "^1.30.1",
    "@opentelemetry/semantic-conventions": "^1.28.0",
    "@nats-io/jetstream": "^3.3.0",
    "@nats-io/kv": "^3.2.0",
    "@nats-io/obj": "^3.3.1",
    "@nats-io/nats-core": "^3.3.1",
    "@nats-io/transport-node": "^3.3.1",
    "drizzle-orm": "^0.44.7",
    "js-sha256": "^0.11.1",
    pino: "^9.11.0",
    tweetnacl: "^1.0.3",
    "ts-deepmerge": "^7.0.3",
    typebox: "^1.0.15",
    ulid: "^3.0.1",
  },
  peerDependencies: {
    "drizzle-orm": "^0.44.7",
  },
  externalizePackageDirs: {
    result: "@qlever-llc/result",
  },
});

for (const format of ["esm", "script"]) {
  const target = new URL(
    `${format}/auth/protocol_wasm/trellis_protocol_wasm_bg.wasm`,
    npmDirUrl,
  );
  await Deno.mkdir(new URL("./", target), { recursive: true });
  await Deno.copyFile(
    new URL(
      "../auth/protocol_wasm/trellis_protocol_wasm_bg.wasm",
      import.meta.url,
    ),
    target,
  );
}

await normalizeModuleSpecifiers();
await removeBrowserGraphDntPolyfills();
await normalizePackageJsonExports();
