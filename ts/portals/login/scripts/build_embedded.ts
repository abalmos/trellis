import { dirname, fromFileUrl, join } from "@std/path";

const portalDir = dirname(dirname(fromFileUrl(import.meta.url)));
const destination = join(
  portalDir,
  "../../../rust/crates/runtime/generated/portal",
);

const build = new Deno.Command(Deno.execPath(), {
  args: ["task", "build:static:prebuilt"],
  cwd: portalDir,
  stdout: "inherit",
  stderr: "inherit",
});
const status = await build.spawn().status;
if (!status.success) Deno.exit(status.code);

const fallbackPath = join(portalDir, "build/200.html");
let fallback = await Deno.readTextFile(fallbackPath);
const inlineScripts = [...fallback.matchAll(/<script>([\s\S]*?)<\/script>/g)];
if (inlineScripts.length !== 1) {
  throw new Error(
    `expected one inline bootstrap script, found ${inlineScripts.length}`,
  );
}
await Deno.writeTextFile(
  join(portalDir, "build/_trellis/assets/bootstrap.js"),
  inlineScripts[0][1],
);
fallback = fallback.replace(
  inlineScripts[0][0],
  '<script src="/_trellis/assets/bootstrap.js"></script>',
);
await Deno.writeTextFile(fallbackPath, fallback);

await Deno.remove(destination, { recursive: true }).catch((error) => {
  if (!(error instanceof Deno.errors.NotFound)) throw error;
});

async function copyDirectory(source: string, target: string): Promise<void> {
  await Deno.mkdir(target, { recursive: true });
  for await (const entry of Deno.readDir(source)) {
    const sourcePath = join(source, entry.name);
    const targetPath = join(target, entry.name);
    if (entry.isDirectory) await copyDirectory(sourcePath, targetPath);
    else if (entry.isFile) await Deno.copyFile(sourcePath, targetPath);
  }
}

await copyDirectory(join(portalDir, "build"), destination);
