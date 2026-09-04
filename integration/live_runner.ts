import { fromFileUrl } from "@std/path";

const repoRoot = fromFileUrl(new URL("../", import.meta.url));

async function run(command: string, args: string[]): Promise<boolean> {
  return (await new Deno.Command(command, {
    args,
    cwd: repoRoot,
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  }).output()).success;
}

if (import.meta.main) {
  const rust = await run(Deno.execPath(), [
    "run",
    "-A",
    "-c",
    "ts/deno.json",
    "rust/crates/trellis-test/integration_runner.ts",
  ]);
  Deno.exit(rust ? 0 : 1);
}
