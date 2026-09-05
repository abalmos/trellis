import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

const rootDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(rootDir, "../..");

const config = {
  plugins: [tailwindcss(), sveltekit()],
  resolve: {
    alias: {
      "@qlever-llc/result": resolve(repoRoot, "ts/packages/result/mod.ts"),
      "@qlever-llc/trellis-svelte": resolve(
        repoRoot,
        "ts/packages/trellis-svelte/src/index.ts",
      ),
      "@qlever-llc/trellis/auth/browser": resolve(
        repoRoot,
        "ts/packages/trellis/auth/browser.ts",
      ),
      "@qlever-llc/trellis/auth": resolve(
        repoRoot,
        "ts/packages/trellis/auth.ts",
      ),
      "@qlever-llc/trellis/browser": resolve(
        repoRoot,
        "ts/packages/trellis/browser.ts",
      ),
      "@qlever-llc/trellis": resolve(repoRoot, "ts/packages/trellis/index.ts"),
    },
    dedupe: ["svelte"],
  },
  server: {
    fs: {
      allow: [repoRoot],
    },
  },
};

export default config;
