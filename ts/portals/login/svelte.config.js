import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

const rootDir = dirname(fileURLToPath(import.meta.url));
const env = (name) =>
  globalThis.Deno?.env.get(name) ??
    globalThis.process?.env?.[name];
const buildDir = env("TRELLIS_LOGIN_PORTAL_BUILD_DIR") ?? "build";
const svelteKitDir = env("TRELLIS_LOGIN_PORTAL_SVELTE_KIT_DIR") ??
  ".svelte-kit";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    appDir: "_trellis/assets",
    version: { name: env("TRELLIS_LOGIN_PORTAL_VERSION") ?? "embedded" },
    outDir: svelteKitDir,
    adapter: adapter({
      pages: buildDir,
      assets: buildDir,
      fallback: "200.html",
    }),
    alias: {
      "@qlever-llc/result": resolve(rootDir, "../../packages/result/mod.ts"),
      "@qlever-llc/trellis-svelte": resolve(
        rootDir,
        "../../packages/trellis-svelte/src/index.ts",
      ),
      "@qlever-llc/trellis/auth/browser": resolve(
        rootDir,
        "../../packages/trellis/auth/browser.ts",
      ),
      "@qlever-llc/trellis/auth": resolve(
        rootDir,
        "../../packages/trellis/auth.ts",
      ),
      "@qlever-llc/trellis/browser": resolve(
        rootDir,
        "../../packages/trellis/browser.ts",
      ),
      "@qlever-llc/trellis/contracts": resolve(
        rootDir,
        "../../packages/trellis/contracts.ts",
      ),
      "@qlever-llc/trellis/errors": resolve(
        rootDir,
        "../../packages/trellis/errors/index.ts",
      ),
      "@qlever-llc/trellis/device/deno": resolve(
        rootDir,
        "../../packages/trellis/device/deno.ts",
      ),
      "@qlever-llc/trellis/device": resolve(
        rootDir,
        "../../packages/trellis/device.ts",
      ),
      "@qlever-llc/trellis/health": resolve(
        rootDir,
        "../../packages/trellis/health.ts",
      ),
      "@trellis/apis/trellis.auth": resolve(
        rootDir,
        ".trellis/generated/ts/trellis-apis/trellis.auth.ts",
      ),
      "@qlever-llc/trellis/service/deno": resolve(
        rootDir,
        "../../packages/trellis/service/deno.ts",
      ),
      "@qlever-llc/trellis/service/node": resolve(
        rootDir,
        "../../packages/trellis/service/node.ts",
      ),
      "@qlever-llc/trellis/service": resolve(
        rootDir,
        "../../packages/trellis/service/mod.ts",
      ),
      "@qlever-llc/trellis/jobs": resolve(
        rootDir,
        "../../packages/trellis/jobs.ts",
      ),
      "@qlever-llc/trellis/telemetry": resolve(
        rootDir,
        "../../packages/trellis/telemetry.ts",
      ),
      "@qlever-llc/trellis": resolve(
        rootDir,
        "../../packages/trellis/index.ts",
      ),
    },
  },
};

export default config;
