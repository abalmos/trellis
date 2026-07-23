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
