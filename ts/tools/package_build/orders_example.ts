import { assert, assertEquals } from "@std/assert";
import { copy, ensureDir } from "@std/fs";
import { fromFileUrl, join, toFileUrl } from "@std/path";
import { z } from "zod";

const repository = fromFileUrl(new URL("../../../", import.meta.url));
const isolated = await Deno.makeTempDir({ prefix: "trellis-orders-consumer-" });
const project = join(isolated, "orders");
const configSchema = z.object({ imports: z.record(z.string(), z.string()) })
  .passthrough();

async function run(
  command: string,
  args: string[],
  env?: Record<string, string>,
) {
  const result = await new Deno.Command(command, {
    args,
    cwd: project,
    env,
    stdout: "piped",
    stderr: "inherit",
  }).output();
  const output = new TextDecoder().decode(result.stdout);
  assertEquals(result.code, 0, `${command} ${args.join(" ")}\n${output}`);
  return output;
}

try {
  await copy(join(repository, "docs/examples/orders"), project);
  const testkit = join(project, "testkit");
  await copy(join(repository, "ts/packages/trellis-test"), testkit);
  const testConfig = configSchema.parse(JSON.parse(
    await Deno.readTextFile(join(testkit, "deno.json")),
  ));
  // Use the source test package as a local dependency, with its imports resolved
  // by this isolated consumer rather than a repository workspace configuration.
  await Deno.remove(join(testkit, "deno.json"));
  const config = configSchema.parse(
    JSON.parse(await Deno.readTextFile(join(project, "deno.json"))),
  );
  const imports = { ...testConfig.imports };
  for (const key of Object.keys(imports)) {
    if (
      key === "@qlever-llc/trellis" || key.startsWith("@qlever-llc/trellis/")
    ) {
      delete imports[key];
    }
  }
  config.imports = {
    ...imports,
    ...config.imports,
    "@qlever-llc/trellis-test": "./testkit/index.ts",
  };
  config.nodeModulesDir = "manual";
  await Deno.writeTextFile(join(project, "deno.json"), JSON.stringify(config));
  await Deno.writeTextFile(
    join(project, "package.json"),
    JSON.stringify({ private: true, type: "module" }),
  );

  const tarballs: string[] = [];
  for (const name of ["result", "trellis"]) {
    const packed: { filename: string }[] = JSON.parse(
      await run("npm", [
        "pack",
        join(repository, "ts/packages", name, "npm"),
        "--json",
        "--pack-destination",
        isolated,
      ]),
    );
    tarballs.push(join(isolated, packed[0].filename));
  }
  await run("npm", [
    "install",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    ...tarballs,
    ...Object.values(imports).filter((value) => value.startsWith("npm:")).map((
      value,
    ) => value.slice(4)),
  ]);

  const bin = join(isolated, "bin");
  await ensureDir(bin);
  await Deno.copyFile(
    Deno.env.get("TRELLIS_TEST_SERVER_BIN") ??
      join(repository, "rust/target/debug/trellis-server"),
    join(bin, "trellis-server"),
  );
  await Deno.symlink(Deno.execPath(), join(bin, "deno"));
  const env = {
    PATH: `${bin}:/usr/bin:/bin`,
    TRELLIS_TEST_CLI_BIN: join(bin, "trellis"),
    TRELLIS_TEST_SERVER_BIN: join(bin, "trellis-server"),
    TRELLIS_CACHE: join(isolated, "empty-api-cache"),
  };
  assertEquals(
    (await new Deno.Command("sh", {
      args: ["-c", "command -v trellis"],
      env,
    }).output()).success,
    false,
    "isolated consumer must not have a Trellis CLI",
  );
  const graph: { modules: { specifier: string }[] } = JSON.parse(
    await run(Deno.execPath(), ["info", "--json", "service_test.ts"], env),
  );
  assert(
    !graph.modules.some(({ specifier }) =>
      specifier.startsWith(toFileUrl(repository).href)
    ),
  );
  console.log(await run(Deno.execPath(), ["task", "check"], env));
  console.log(await run(Deno.execPath(), ["task", "test"], env));
  console.log(
    "Orders builds and tests with no Trellis CLI, API cache, or parent repository imports.",
  );
} finally {
  await Deno.remove(isolated, { recursive: true });
}
