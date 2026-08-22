import { assertEquals, assertStringIncludes } from "@std/assert";
import { parse } from "jsonc-parser";

const decoder = new TextDecoder();
const trellisSelfImportPattern =
  /(?:from\s+|import\()\s*["']@qlever-llc\/trellis(?:\/[^"']*)?["']/;

async function* walkPublishableSources(
  dir: URL,
): AsyncGenerator<URL> {
  for await (const entry of Deno.readDir(dir)) {
    const url = new URL(`${entry.name}${entry.isDirectory ? "/" : ""}`, dir);
    if (entry.isDirectory) {
      if ([".build", "npm", "scripts", "tests"].includes(entry.name)) {
        continue;
      }
      yield* walkPublishableSources(url);
      continue;
    }

    if (
      !entry.name.endsWith(".ts") ||
      entry.name.endsWith("_test.ts") ||
      entry.name.endsWith(".test.ts") ||
      entry.name.endsWith(".api_check.ts")
    ) {
      continue;
    }

    yield url;
  }
}

Deno.test("workspace npm build task only builds the supported published packages", async () => {
  const source = await Deno.readFile(
    new URL("../../../deno.json", import.meta.url),
  );
  const config = parse(decoder.decode(source)) as {
    tasks: Record<string, string>;
  };

  assertEquals(
    config.tasks["packages:build:npm"],
    "deno task -c packages/result/deno.json build:npm && deno task -c packages/trellis/deno.json build:npm && deno task -c packages/trellis-svelte/deno.json build:npm",
  );
  assertEquals(
    config.tasks["packages:build:npm:prepared"],
    "deno task -c packages/result/deno.json build:npm && deno task -c packages/trellis/deno.json build:npm:prepared && deno task -c packages/trellis-svelte/deno.json build:npm",
  );
  assertEquals(
    config.tasks["test:prepared:packaging"],
    "deno task packages:build:npm:prepared && deno task test:prepared:packaging:built",
  );
  assertEquals(
    config.tasks["build:npm"],
    "deno task prepare && deno task packages:build:npm",
  );
});

Deno.test("release workflows use generated package-manager targets", async () => {
  let releaseWorkflow = "";
  const dryRunScript = await Deno.readTextFile(
    new URL("../../../../scripts/release-ts-dry-run.sh", import.meta.url),
  );
  for (
    const workflow of [
      "release.yml",
      "pages.yml",
    ]
  ) {
    const source = await Deno.readTextFile(
      new URL(`../../../../.github/workflows/${workflow}`, import.meta.url),
    );

    assertEquals(source.includes("generated/rust/sdks"), false, workflow);
    assertEquals(source.includes("generate rust"), false, workflow);
    if (workflow === "release.yml") releaseWorkflow = source;
  }

  assertStringIncludes(
    releaseWorkflow,
    "cargo run --manifest-path rust/xtask/Cargo.toml -- release lane typescript",
  );
  assertStringIncludes(
    releaseWorkflow,
    "cargo run --manifest-path rust/xtask/Cargo.toml -- release lane live",
  );
  assertStringIncludes(
    releaseWorkflow,
    "cargo run --manifest-path rust/tools/generate/Cargo.toml -- -f prepare --no-npm .",
  );
  const prepareRelease = releaseWorkflow.split("\n  prepare-release:")[1].split(
    "\n  verify-static:",
  )[0];
  const generateReleaseSdk = prepareRelease.indexOf(
    "- name: Generate release SDK artifacts",
  );
  const buildEmbeddedPortal = prepareRelease.indexOf(
    "- name: Build embedded login portal",
  );
  const uploadPreparedRelease = prepareRelease.indexOf(
    "- name: Upload prepared release workspace",
  );
  assertStringIncludes(
    prepareRelease,
    "deno task -c ts/portals/login/deno.json build:embedded",
  );
  assertEquals(generateReleaseSdk < buildEmbeddedPortal, true);
  assertEquals(buildEmbeddedPortal < uploadPreparedRelease, true);
  assertStringIncludes(
    releaseWorkflow,
    `cp "rust/target/\${target}/release/trellis-generate"`,
  );
  assertStringIncludes(
    releaseWorkflow,
    "denoland/setup-deno@v2",
  );
  assertStringIncludes(
    releaseWorkflow,
    "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true",
  );
  assertEquals(
    releaseWorkflow.includes(
      "false && needs.prepare-release.outputs.should-publish",
    ),
    false,
  );
  assertStringIncludes(releaseWorkflow, "publish_or_skip ts/packages/result");
  assertStringIncludes(releaseWorkflow, "publish_or_skip ts/packages/trellis");
  assertStringIncludes(
    releaseWorkflow,
    "file: rust/crates/runtime/Containerfile",
  );
  assertEquals(
    releaseWorkflow.includes("file: ts/services/trellis/Containerfile"),
    false,
  );
  assertEquals(
    releaseWorkflow.includes("io.trellis.contract.path"),
    false,
  );
  assertEquals(
    releaseWorkflow.includes("publish_or_skip ts/services/trellis"),
    false,
  );
  assertStringIncludes(
    releaseWorkflow,
    "publish_or_skip ts/packages/trellis-test",
  );
  assertStringIncludes(
    releaseWorkflow,
    "publish_or_skip ts/packages/trellis-svelte/jsr",
  );
  assertStringIncludes(
    dryRunScript,
    "deno publish --dry-run --allow-slow-types --allow-dirty",
  );
  assertStringIncludes(dryRunScript, "ts/packages/trellis-test");
  assertStringIncludes(
    releaseWorkflow,
    `TRELLIS_SVELTE_JSR_RUNTIME_DEPENDENCY_VERSION="\${RELEASE_VERSION}" \\
            deno task -c ts/packages/trellis-svelte/deno.json build:npm`,
  );
  assertStringIncludes(dryRunScript, "ts/packages/trellis");
  assertEquals(dryRunScript.includes("ts/services/trellis"), false);
  assertStringIncludes(dryRunScript, "ts/packages/trellis-test");
  assertEquals(releaseWorkflow.includes("trellis-svelte-jsr-package"), false);
  assertEquals(
    releaseWorkflow.includes(["services/trellis", "jsr"].join("/")),
    false,
  );
  assertEquals(releaseWorkflow.includes(["prepare", "jsr"].join(":")), false);
  assertEquals(
    releaseWorkflow.includes(["trellis-service", "trellis"].join("-")),
    false,
  );
  assertStringIncludes(
    dryRunScript,
    "deno publish --dry-run --allow-slow-types --allow-dirty",
  );
  assertStringIncludes(
    releaseWorkflow,
    "deno publish --allow-slow-types --allow-dirty",
  );
  assertEquals(releaseWorkflow.includes("\n  verify-format:"), false);
  assertStringIncludes(releaseWorkflow, "\n  verify-static:");
  assertStringIncludes(releaseWorkflow, "\n  verify-rust:");
  assertStringIncludes(releaseWorkflow, "\n  verify-js:");
  assertStringIncludes(releaseWorkflow, "\n  verify-live:");
  assertEquals(releaseWorkflow.includes("verify-js-integration"), false);
  assertEquals(releaseWorkflow.includes("verify-rust-integration"), false);
  assertStringIncludes(
    releaseWorkflow,
    "outputs: type=oci,dest=/tmp/$" + "{{ matrix.image }}.tar",
  );
  assertStringIncludes(
    releaseWorkflow,
    "verified-image-$" + "{{ matrix.image }}",
  );
  assertStringIncludes(releaseWorkflow, "release lane live-build");
  assertStringIncludes(
    releaseWorkflow,
    "integration/live_runner.ts --prebuilt-only --artifacts-manifest dist/integration-runtime/manifest.json",
  );
  assertStringIncludes(releaseWorkflow, "name: integration-live-artifacts");
  assertStringIncludes(releaseWorkflow, "path: dist/integration-runtime");
  assertEquals(releaseWorkflow.includes("\n  verify-live-inventory:"), false);
  assertEquals(releaseWorkflow.includes("--inventory-only"), false);
  const verifyLive = releaseWorkflow.split("\n  verify-live:")[1].split(
    "\n  release-gate:",
  )[0];
  assertStringIncludes(verifyLive, "- prepare-release");
  assertStringIncludes(verifyLive, "- verify-live-build");
  assertEquals(verifyLive.includes("- verify-live-inventory"), false);
  assertEquals(verifyLive.includes("- verify-static"), false);
  assertEquals(verifyLive.includes("- verify-rust"), false);
  assertEquals(verifyLive.includes("- verify-js"), false);
  assertStringIncludes(releaseWorkflow, "skopeo copy --all");
  assertEquals(releaseWorkflow.includes("Build and push image"), false);
  assertEquals(releaseWorkflow.includes("rust-msrv"), false);
  assertStringIncludes(
    releaseWorkflow,
    "needs.verify-static.result == 'success'",
  );
  assertEquals(releaseWorkflow.includes("deno eval --allow-read"), false);
});

Deno.test("runtime image executes only the Rust server", async () => {
  const containerfile = await Deno.readTextFile(
    new URL("../../../../rust/crates/runtime/Containerfile", import.meta.url),
  );
  const runtimeStage = containerfile.split(
    "FROM docker.io/library/debian:bookworm-slim AS runtime",
  )[1];

  assertStringIncludes(containerfile, 'ENTRYPOINT ["trellis-server"]');
  assertStringIncludes(containerfile, "USER trellis");
  assertEquals(runtimeStage.includes("deno"), false);
  assertEquals(containerfile.includes("ts/services/trellis"), false);
});

Deno.test("pages workflow cleans generator fallback temp dirs explicitly", async () => {
  const source = await Deno.readTextFile(
    new URL("../../../../.github/workflows/pages.yml", import.meta.url),
  );

  assertEquals(source.includes("trap cleanup_temp RETURN"), false);
  assertStringIncludes(source, "cleanup_temp");
  assertStringIncludes(source, "release_worktree_path");
  assertStringIncludes(source, "release_worktree_created");
  assertStringIncludes(
    source,
    "Published trellis-generate archive is not available",
  );
  assertStringIncludes(
    source,
    "Published trellis-generate checksum is not available",
  );
  assertStringIncludes(source, "Latest release tag worktree is missing docs");
  assertStringIncludes(
    source,
    "Latest release tag worktree is missing console sources",
  );
  assertStringIncludes(source, "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true");
});

Deno.test("release workflow publishes only public Rust crates", async () => {
  const source = await Deno.readTextFile(
    new URL("../../../../.github/workflows/release.yml", import.meta.url),
  );

  for (const crate of ["trellis-contracts", "trellis-rs"]) {
    assertStringIncludes(source, `publish_workspace_crate ${crate}`);
  }
  assertStringIncludes(source, '[ "$crate" = "trellis-rs" ]');
  assertStringIncludes(source, "trellis-test");
  for (
    const crate of [
      "trellis-auth",
      "trellis-cli",
      "trellis-client",
      "trellis-codegen-rust",
      "trellis-codegen-ts",
      "trellis-generate-runner",
      "trellis-local-bootstrap",
      "trellis-sdk-auth",
      "trellis-sdk-core",
      "trellis-service",
    ]
  ) {
    assertEquals(source.includes(`publish_workspace_crate ${crate}`), false);
    assertEquals(source.includes("publish_generated_crate"), false);
  }
});

Deno.test("trellis package exports the first-party SDK subpaths", async () => {
  const source = await Deno.readTextFile(
    new URL("../deno.json", import.meta.url),
  );

  assertStringIncludes(source, '"./sdk/auth": "./sdk/auth.ts"');
  assertStringIncludes(source, '"./sdk/core": "./sdk/core.ts"');
  assertStringIncludes(source, '"./sdk/health": "./sdk/health.ts"');
  assertStringIncludes(source, '"./sdk/jobs": "./sdk/jobs.ts"');
  assertStringIncludes(source, '"./sdk/state": "./sdk/state.ts"');
});

Deno.test("trellis generate wrapper reads package metadata from remote modules", async () => {
  const packageRoot = new URL("../", import.meta.url);
  const generateSource = await Deno.readTextFile(
    new URL("generate.ts", packageRoot),
  );
  const packageManifest = await Deno.readTextFile(
    new URL("deno.json", packageRoot),
  );
  const packageVersion = JSON.parse(packageManifest).version as string;
  const tempDir = await Deno.makeTempDir();
  const fakeGenerator = `${tempDir}/trellis-generate`;
  await Deno.writeTextFile(
    fakeGenerator,
    `#!/bin/sh
printf 'trellis-generate ${packageVersion}\n'
`,
  );
  await Deno.chmod(fakeGenerator, 0o755);

  const server = Deno.serve({
    hostname: "127.0.0.1",
    port: 0,
    onListen: () => {},
  }, (request) => {
    const url = new URL(request.url);
    if (url.pathname === "/generate.ts") {
      return new Response(generateSource, {
        headers: { "content-type": "application/typescript" },
      });
    }
    if (url.pathname === "/deno.json") {
      return new Response(packageManifest, {
        headers: { "content-type": "application/json" },
      });
    }
    return new Response("not found", { status: 404 });
  });

  try {
    const output = await new Deno.Command(Deno.execPath(), {
      args: [
        "run",
        "-A",
        `http://127.0.0.1:${server.addr.port}/generate.ts`,
        "--version",
      ],
      env: { TRELLIS_GENERATE_BIN: fakeGenerator },
      stdout: "piped",
      stderr: "piped",
    }).output();

    assertEquals(output.success, true, decoder.decode(output.stderr));
  } finally {
    await server.shutdown();
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("published trellis sources do not self-import package subpaths", async () => {
  const offenders: string[] = [];
  const packageRoot = new URL("../", import.meta.url);

  for await (const sourceUrl of walkPublishableSources(packageRoot)) {
    const source = await Deno.readTextFile(sourceUrl);
    if (trellisSelfImportPattern.test(source)) {
      offenders.push(sourceUrl.pathname.replace(packageRoot.pathname, ""));
    }
  }

  assertEquals(offenders, []);
});

Deno.test("workspace config does not shadow publishable package members", async () => {
  const source = await Deno.readTextFile(
    new URL("../../../deno.json", import.meta.url),
  );

  assertEquals(source.includes('"@qlever-llc/result":'), false);
  assertEquals(source.includes('"@qlever-llc/trellis":'), false);
  assertEquals(source.includes('"@qlever-llc/trellis-test":'), false);
  assertEquals(source.includes('"@qlever-llc/trellis/sdk/jobs":'), false);
  assertEquals(source.includes('"@qlever-llc/trellis-svelte":'), false);
});

Deno.test("trellis npm build depends on the standalone result package name", async () => {
  const source = await Deno.readTextFile(
    new URL("../scripts/build_npm.ts", import.meta.url),
  );

  assertStringIncludes(source, '"@qlever-llc/result"');
  assertStringIncludes(source, '"@qlever-llc/result": "^0.12.0"');
});

Deno.test("trellis service export keeps SQL outbox generic and Drizzle isolated", async () => {
  const serviceSource = await Deno.readTextFile(
    new URL("../service/mod.ts", import.meta.url),
  );
  const buildSource = await Deno.readTextFile(
    new URL("../scripts/build_npm.ts", import.meta.url),
  );

  for (
    const publicName of [
      "SqlOutbox",
      "SqlOutboxEventEnqueueFacade",
      "SqlOutboxTransactionContext",
      "SqlOutboxTransactionRunner",
      "TrellisServiceSqlOutboxCommonOptions",
      "TrellisServiceSqlOutboxExecutorOptions",
      "TrellisServiceSqlOutboxOptions",
      "getSqlOutboxMigrations",
      "SqlOutboxMigration",
      "SqlOutboxMigrationOptions",
    ]
  ) {
    assertStringIncludes(serviceSource, publicName);
  }

  for (
    const drizzleName of [
      "bindDrizzleSqlStatement",
      "createDrizzleSqlExecutor",
      "DrizzleSqlDatabase",
      "DrizzleSqlOutboxOptions",
      "DrizzleSqlTransactionRunner",
      "runDrizzleSqlTransaction",
      "drizzle-orm",
    ]
  ) {
    assertEquals(serviceSource.includes(drizzleName), false, drizzleName);
  }

  assertStringIncludes(
    buildSource,
    '"./ts/packages/trellis/service/mod.ts"',
  );
  assertStringIncludes(
    buildSource,
    '"./ts/packages/trellis/service/drizzle.ts"',
  );
});

Deno.test("trellis-svelte npm build uses current Trellis package bases", async () => {
  const source = await Deno.readTextFile(
    new URL("../../trellis-svelte/scripts/build_npm.ts", import.meta.url),
  );

  assertStringIncludes(source, '"@qlever-llc/result": "^0.12.0"');
  assertStringIncludes(source, '"@qlever-llc/trellis": "^0.12.0"');
});

Deno.test("trellis package exports public runtime subpaths", async () => {
  const source = await Deno.readTextFile(
    new URL("../deno.json", import.meta.url),
  );

  assertStringIncludes(source, '"./errors": "./errors/index.ts"');
  assertEquals(source.includes('"./health":'), false);
  assertEquals(source.includes('"./host'), false);
  assertEquals(source.includes('"./internal/'), false);
  assertStringIncludes(source, '"./jobs": "./jobs.ts"');
  assertStringIncludes(source, '"./service": "./service/mod.ts"');
  assertStringIncludes(source, '"./service/drizzle": "./service/drizzle.ts"');
  assertStringIncludes(source, '"./telemetry": "./telemetry.ts"');
  assertEquals(source.includes('"./tracing":'), false);
});
