import { resolve } from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  base: "/__default-client/",
  resolve: {
    alias: [
      {
        find: "@qlever-llc/result",
        replacement: resolve(
          import.meta.dirname,
          "../../packages/result/mod.ts",
        ),
      },
      {
        find: "@qlever-llc/trellis/contracts",
        replacement: resolve(
          import.meta.dirname,
          "../../packages/trellis/contracts.ts",
        ),
      },
      {
        find: "@qlever-llc/trellis/errors",
        replacement: resolve(
          import.meta.dirname,
          "../../packages/trellis/errors/index.ts",
        ),
      },
      {
        find: /^@qlever-llc\/trellis$/,
        replacement: resolve(
          import.meta.dirname,
          "../../packages/trellis/index.ts",
        ),
      },
      {
        find: "@trellis/apis/trellis.auth",
        replacement: resolve(
          import.meta.dirname,
          "../.trellis/generated/ts/trellis-apis/trellis.auth.ts",
        ),
      },
    ],
  },
});
