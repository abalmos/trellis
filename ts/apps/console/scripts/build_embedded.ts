import { dirname, fromFileUrl, join } from "@std/path";

const consoleDir = dirname(dirname(fromFileUrl(import.meta.url)));
const buildDir = join(consoleDir, "build-embedded");
const destination = join(
  consoleDir,
  "../../../rust/crates/runtime/generated/console",
);

const build = new Deno.Command(Deno.execPath(), {
  args: ["task", "build:static:prebuilt"],
  cwd: consoleDir,
  env: {
    SITE_BASE_PATH: "/console",
    TRELLIS_CONSOLE_BUILD_DIR: buildDir,
    TRELLIS_CONSOLE_VERSION: "embedded",
  },
  stdout: "inherit",
  stderr: "inherit",
});
const status = await build.spawn().status;
if (!status.success) Deno.exit(status.code);

const fallbackPath = join(buildDir, "index.html");
let fallback = await Deno.readTextFile(fallbackPath);
const head = fallback.slice(0, fallback.indexOf("</head>"));
const inlineScripts = [...head.matchAll(/<script>([\s\S]*?)<\/script>/g)];
if (inlineScripts.length !== 2) {
  throw new Error(
    `expected route restoration and theme scripts, found ${inlineScripts.length}`,
  );
}
for (const [index, script] of inlineScripts.entries()) {
  const name = index === 0 ? "restore-route.js" : "theme.js";
  await Deno.writeTextFile(join(buildDir, `assets/${name}`), script[1]);
  fallback = fallback.replace(
    script[0],
    `<script src="/console/assets/${name}"></script>`,
  );
}
const bootstrapScripts = [
  ...fallback.matchAll(/<script>([\s\S]*?)<\/script>/g),
];
if (bootstrapScripts.length !== 1) {
  throw new Error(
    `expected SvelteKit bootstrap script, found ${bootstrapScripts.length}`,
  );
}
await Deno.writeTextFile(
  join(buildDir, "assets/bootstrap.js"),
  bootstrapScripts[0][1],
);
fallback = fallback.replace(
  bootstrapScripts[0][0],
  '<script src="/console/assets/bootstrap.js"></script>',
);
await Deno.writeTextFile(fallbackPath, fallback);
await Deno.writeTextFile(
  join(buildDir, "runtime-config.js"),
  "globalThis.__TRELLIS_RUNTIME_CONFIG__ = { authUrl: globalThis.location.origin };\n",
);

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

await copyDirectory(buildDir, destination);
