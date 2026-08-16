import { resolve } from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  resolve: {
    alias: {
      "@qlever-llc/result": resolve(
        import.meta.dirname,
        "../../packages/result/mod.ts",
      ),
      "@qlever-llc/trellis": resolve(
        import.meta.dirname,
        "../../packages/trellis/index.ts",
      ),
    },
  },
});
