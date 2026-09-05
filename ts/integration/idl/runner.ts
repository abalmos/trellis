import { fromFileUrl } from "@std/path";

const idlRoot = fromFileUrl(new URL("./", import.meta.url));

if (import.meta.main) {
  const status = await new Deno.Command(Deno.execPath(), {
    cwd: idlRoot,
    args: [
      "test",
      "--no-check",
      "-A",
      "-c",
      "../deno.json",
      "--no-lock",
      "field_ops_demo.integration_test.ts",
    ],
  }).spawn().status;
  Deno.exit(status.code);
}
