import { assertEquals } from "@std/assert";
import {
  basename,
  dirname,
  fromFileUrl,
  join,
  resolve,
  toFileUrl,
} from "@std/path";
import { Result } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";

const CASE_ID = "idl_demo::field_ops_out_of_tree" as const;

liveTrellisTest({
  name:
    "idl_demo::field_ops_out_of_tree generates, builds, and runs copied demos",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const repo = resolve(dirname(fromFileUrl(import.meta.url)), "../../..");
    const temp = await Deno.makeTempDir({
      dir: Deno.build.os === "windows" ? undefined : "/tmp",
      prefix: "field-ops-demo-",
    });
    try {
      const demo = join(temp, "field-ops");
      await copySource(join(repo, "demos"), demo);
      console.info(`copied Field Ops demos outside checkout: ${demo}`);

      await injectLocalTypeScriptPackages(join(demo, "ts", "deno.json"), repo);
      await injectLocalTypeScriptPackages(join(demo, "app", "deno.json"), repo);
      for (const root of [join(demo, "ts", "device"), join(demo, "app")]) {
        await installStateArtifact(root, repo);
      }
      for (
        const root of [
          join(demo, "rust", "service"),
          join(demo, "rust", "device"),
        ]
      ) {
        await injectLocalRustPackages(join(root, "Cargo.toml"), repo);
      }

      const trellis = join(repo, "rust", "target", "debug", "trellis");
      for (
        const root of [
          join(demo, "ts", "service"),
          join(demo, "rust", "service"),
          join(demo, "rust", "device"),
        ]
      ) {
        await run(trellis, ["update", "--root", root]);
      }
      for (
        const root of [
          join(demo, "ts", "service"),
          join(demo, "ts", "device"),
          join(demo, "app"),
          join(demo, "rust", "service"),
          join(demo, "rust", "device"),
        ]
      ) {
        await run(trellis, ["generate", "--root", root]);
      }
      console.info("trellis CLI generation succeeded for all copied projects");

      await run(Deno.execPath(), [
        "task",
        "-c",
        join(demo, "ts", "deno.json"),
        "check",
      ]);
      await run(Deno.execPath(), [
        "install",
        "-c",
        join(demo, "app", "deno.json"),
      ]);
      await run(Deno.execPath(), [
        "task",
        "-c",
        join(demo, "app", "deno.json"),
        "check",
      ]);
      console.info("TypeScript service, device, and app checks succeeded");
      await run("cargo", [
        "update",
        "--manifest-path",
        join(demo, "rust", "service", "Cargo.toml"),
        "-p",
        "tinyvec",
        "--precise",
        "1.11.0",
      ]);
      await run("cargo", [
        "update",
        "--manifest-path",
        join(demo, "rust", "device", "Cargo.toml"),
        "-p",
        "tinyvec",
        "--precise",
        "1.11.0",
      ]);
      await run("cargo", [
        "check",
        "--manifest-path",
        join(demo, "rust", "service", "Cargo.toml"),
      ]);
      await run("cargo", [
        "check",
        "--manifest-path",
        join(demo, "rust", "device", "Cargo.toml"),
      ]);
      console.info("Rust service and device checks succeeded");

      const serviceModule = await import(
        toFileUrl(
          join(
            demo,
            "ts",
            "service",
            ".trellis",
            "ts",
            "participants",
            "demo-service",
            "mod.ts",
          ),
        ).href
      );
      const appModule = await import(
        toFileUrl(
          join(
            demo,
            "app",
            ".trellis",
            "ts",
            "participants",
            "demo-app",
            "mod.ts",
          ),
        ).href
      );
      const serviceKey = await runtime.registerService({
        name: "idl-field-ops-service",
        contract: serviceModule.participant,
      });
      const service = await TrellisService.connect({
        authorizationContextEphemeral: true,
        trellisUrl: runtime.trellisUrl,
        participant: serviceModule.participant,
        name: "idl-field-ops-service",
        identity: serviceKey,
        telemetry: false,
        runtime: {},
      }).orThrow();
      try {
        const handleSitesList = Reflect.get(service, "handleSitesList");
        if (typeof handleSitesList !== "function") {
          throw new Error("generated service is missing handleSitesList");
        }
        await Reflect.apply(handleSitesList, service, [() =>
          Result.ok({
            entries: [{
              siteId: "site-idl",
              siteName: "IDL Site",
              openInspections: 1,
              overdueInspections: 0,
              latestStatus: "ready",
              lastReportAt: "2026-09-03T00:00:00Z",
            }],
            count: 1,
            offset: 0,
            limit: 1,
          })]);
        const client = await runtime.connectClient({
          name: "idl-field-ops-app",
          contract: appModule.participant,
        });
        const sitesList = Reflect.get(client, "sitesList");
        if (typeof sitesList !== "function") {
          throw new Error("generated client is missing sitesList");
        }
        const pending = Reflect.apply(sitesList, client, [{ limit: 1 }]);
        const orThrow = Reflect.get(pending, "orThrow");
        if (typeof orThrow !== "function") {
          throw new Error("generated RPC did not return a Result");
        }
        const result = await Reflect.apply(orThrow, pending, []);
        if (
          typeof result !== "object" || result === null ||
          !Array.isArray(Reflect.get(result, "entries"))
        ) {
          throw new Error("generated RPC returned an invalid result");
        }
        assertEquals(Reflect.get(result, "entries")[0]?.siteId, "site-idl");
        console.info("generated participant completed a real Field Ops RPC");
      } finally {
        await service.stop();
      }
    } finally {
      await Deno.remove(temp, { recursive: true });
    }
  },
});

async function copySource(source: string, target: string): Promise<void> {
  await Deno.mkdir(target, { recursive: true });
  for await (const entry of Deno.readDir(source)) {
    if ([".trellis", "node_modules", "target"].includes(entry.name)) continue;
    const from = join(source, entry.name);
    const to = join(target, entry.name);
    if (entry.isDirectory) await copySource(from, to);
    else if (entry.isFile) await Deno.copyFile(from, to);
  }
}

async function injectLocalTypeScriptPackages(
  path: string,
  repo: string,
): Promise<void> {
  const config = JSON.parse(await Deno.readTextFile(path));
  config.imports["@qlever-llc/result"] = join(
    repo,
    "ts",
    "packages",
    "result",
    "mod.ts",
  );
  config.imports["@qlever-llc/trellis"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "index.ts",
  );
  config.imports["@qlever-llc/trellis/auth"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "auth.ts",
  );
  config.imports["@qlever-llc/trellis/browser"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "browser.ts",
  );
  config.imports["@qlever-llc/trellis/device"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "device.ts",
  );
  config.imports["@qlever-llc/trellis/jobs"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "jobs.ts",
  );
  config.imports["@qlever-llc/trellis/telemetry"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "telemetry.ts",
  );
  config.imports["@qlever-llc/trellis/contracts"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "contracts.ts",
  );
  config.imports["@qlever-llc/trellis/device/deno"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "device",
    "deno.ts",
  );
  config.imports["@qlever-llc/trellis/errors"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "errors",
    "index.ts",
  );
  config.imports["@qlever-llc/trellis/service"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "service",
    "mod.ts",
  );
  config.imports["@qlever-llc/trellis/service/deno"] = join(
    repo,
    "ts",
    "packages",
    "trellis",
    "service",
    "deno.ts",
  );
  if (basename(dirname(path)) === "app") {
    config.imports["@qlever-llc/trellis-svelte"] = join(
      repo,
      "ts",
      "packages",
      "trellis-svelte",
      "src",
      "index.ts",
    );
    config.imports["@qlever-llc/trellis/auth/browser"] = join(
      repo,
      "ts",
      "packages",
      "trellis",
      "auth",
      "browser.ts",
    );
    config.links = [
      join(repo, "ts", "packages", "result"),
      join(repo, "ts", "packages", "trellis"),
      join(repo, "ts", "packages", "trellis-svelte"),
    ];
    const checkPath = join(dirname(path), "tsconfig.check.json");
    const checkConfig = JSON.parse(await Deno.readTextFile(checkPath));
    checkConfig.compilerOptions.paths = {
      "$lib": [join(dirname(path), "src", "lib")],
      "$lib/*": [join(dirname(path), "src", "lib", "*")],
      "@qlever-llc/result": [join(repo, "ts", "packages", "result", "mod.ts")],
      "@qlever-llc/trellis": [
        join(repo, "ts", "packages", "trellis", "index.ts"),
      ],
      "@qlever-llc/trellis/*": [join(repo, "ts", "packages", "trellis", "*")],
      "@qlever-llc/trellis-svelte": [
        join(repo, "ts", "packages", "trellis-svelte", "src", "index.ts"),
      ],
    };
    await Deno.writeTextFile(
      checkPath,
      `${JSON.stringify(checkConfig, null, 2)}\n`,
    );
  }
  await Deno.writeTextFile(path, `${JSON.stringify(config, null, 2)}\n`);
}

async function injectLocalRustPackages(
  path: string,
  repo: string,
): Promise<void> {
  const cargo = await Deno.readTextFile(path);
  await Deno.writeTextFile(
    path,
    `${cargo}\n[patch.crates-io]\ntrellis-rs = { path = ${
      JSON.stringify(join(repo, "rust", "crates", "trellis"))
    } }\n`,
  );
}

async function installStateArtifact(root: string, repo: string): Promise<void> {
  const manifest = root.endsWith(`${join("ts", "device")}`)
    ? {
      format: 1,
      registries: { trellis: { prefix: "ghcr.io/qlever-llc/trellis-apis" } },
      apis: {
        "demo.service@v1": { version: "1.0.0", path: "../service" },
        "trellis.state@v1": { version: "^1.0.0", registry: "trellis" },
      },
    }
    : {
      format: 1,
      registries: { trellis: { prefix: "ghcr.io/qlever-llc/trellis-apis" } },
      apis: {
        "demo.service@v1": { version: "1.0.0", path: "../ts/service" },
        "trellis.state@v1": { version: "^1.0.0", registry: "trellis" },
      },
    };
  const digest = await digestJson(manifest);
  await Deno.writeTextFile(
    join(root, "trellis.lock"),
    `format = 1\nmanifest-digest = ${
      JSON.stringify(digest)
    }\n\n[[api]]\nid = "trellis.state@v1"\nversion = "1.0.0"\napi-digest = "qs1u5DVKbglPE25fTx77AadXTT-MDwSkmHVFJlCwE2A"\nregistry = "trellis"\noci-digest = "sha256:${
      "0".repeat(64)
    }"\n`,
  );
  const artifactDir = join(
    root,
    ".trellis",
    "apis",
    "trellis.state@v1",
    "1.0.0",
  );
  await Deno.mkdir(artifactDir, { recursive: true });
  await Deno.copyFile(
    join(repo, "conformance", "baselines", "trellis-state-3ef0aa94.api.json"),
    join(artifactDir, "trellis.api.json"),
  );
}

async function digestJson(value: unknown): Promise<string> {
  const canonical = JSON.stringify(sortJson(value));
  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(canonical),
  );
  return new Uint8Array(digest).toBase64({
    alphabet: "base64url",
    omitPadding: true,
  });
}

function sortJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortJson);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).sort(([left], [right]) => left.localeCompare(right))
        .map(([key, item]) => [key, sortJson(item)]),
    );
  }
  return value;
}

async function run(command: string, args: string[]): Promise<void> {
  const status = await new Deno.Command(command, {
    args,
    stdout: "inherit",
    stderr: "inherit",
  }).spawn().status;
  if (!status.success) {
    throw new Error(`${command} ${args.join(" ")} failed with ${status.code}`);
  }
}
