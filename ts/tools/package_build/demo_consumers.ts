import { assertEquals } from "@std/assert";
import { ensureDir } from "@std/fs";
import { dirname, fromFileUrl, join } from "@std/path";
import { z } from "zod";

const repository = fromFileUrl(new URL("../../../", import.meta.url));
const isolated = await Deno.makeTempDir({ prefix: "trellis-demo-consumers-" });
const configSchema = z.object({ imports: z.record(z.string(), z.string()) })
  .passthrough();

async function run(command: string, args: string[], cwd = isolated) {
  const result = await new Deno.Command(command, {
    args,
    cwd,
    stdout: "piped",
    stderr: "inherit",
  }).output();
  const output = new TextDecoder().decode(result.stdout);
  assertEquals(result.code, 0, `${command} ${args.join(" ")}\n${output}`);
  return output;
}

try {
  const files = await run("git", [
    "ls-files",
    "-co",
    "--exclude-standard",
    "-z",
    "--",
    "demos/ts",
    "demos/app",
  ], repository);
  for (const file of files.split("\0").filter(Boolean)) {
    if (!(await Deno.lstat(join(repository, file))).isFile) continue;
    const target = join(isolated, file);
    await ensureDir(dirname(target));
    await Deno.copyFile(join(repository, file), target);
  }
  const dependencies = new Map<string, string>();
  for (const file of ["demos/ts/deno.json", "demos/app/deno.json"]) {
    const path = join(isolated, file);
    const config = configSchema.parse(
      JSON.parse(await Deno.readTextFile(path)),
    );
    for (const value of Object.values(config.imports)) {
      const match = /^npm:((?:@[^/]+\/)?[^/@]+)(@[^/]+)?/.exec(value);
      if (
        match && !match[1].startsWith("@qlever-llc/") &&
        (!dependencies.has(match[1]) || match[2])
      ) {
        dependencies.set(match[1], match[1] + (match[2] ?? ""));
      }
    }
    config.nodeModulesDir = "manual";
    await Deno.writeTextFile(path, JSON.stringify(config));
  }
  const lock = z.object({ npm: z.record(z.string(), z.unknown()) }).parse(
    JSON.parse(await Deno.readTextFile(join(isolated, "demos/app/deno.lock"))),
  );
  const lockedVersions = new Map<string, Set<string>>();
  for (const entry of Object.keys(lock.npm)) {
    const specifier = entry.split("_")[0];
    const separator = specifier.lastIndexOf("@");
    const name = specifier.slice(0, separator);
    const versions = lockedVersions.get(name) ?? new Set<string>();
    versions.add(specifier.slice(separator + 1));
    lockedVersions.set(name, versions);
  }
  const overrides: Record<string, string> = {};
  // Preserve unambiguous versions from the demo's checked-in Deno lock instead
  // of accidentally testing a newly resolved frontend dependency graph.
  for (const [name, versions] of lockedVersions) {
    if (versions.size !== 1 || name.startsWith("@qlever-llc/")) continue;
    const [version] = versions;
    if (dependencies.has(name)) dependencies.set(name, `${name}@${version}`);
    else overrides[name] = version;
  }
  await Deno.writeTextFile(
    join(isolated, "package.json"),
    JSON.stringify({ private: true, type: "module", overrides }),
  );
  const tarballs: string[] = [];
  for (const name of ["result", "trellis", "trellis-svelte"]) {
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
    ...dependencies.values(),
  ]);
  for (const project of ["demos/ts/service", "demos/ts/device"]) {
    console.log(
      await run(Deno.execPath(), ["task", "check"], join(isolated, project)),
    );
  }
  const app = join(isolated, "demos/app");
  for (
    const args of [
      ["run", "-A", "@sveltejs/kit", "sync"],
      ["run", "-A", "svelte-check", "--tsconfig", "./tsconfig.check.json"],
      ["run", "-A", "vite", "build"],
    ]
  ) console.log(await run(Deno.execPath(), args, app));
  console.log("Demo consumers pass with packed runtime dependencies.");
} finally {
  await Deno.remove(isolated, { recursive: true });
}
