import { fromFileUrl } from "@std/path";
import { runTrellisIntegrationTests } from "@qlever-llc/trellis-test/integration/runner";
import { trellisRepoRuntimeOptions } from "../_support/runtime.ts";

const idlRoot = fromFileUrl(new URL("./", import.meta.url));

if (import.meta.main) {
  Deno.exit(
    await runTrellisIntegrationTests({
      cwd: idlRoot,
      config: {
        runtime: trellisRepoRuntimeOptions(),
        denoTestArgs: ["--no-check", "-A", "-c", "../deno.json", "--no-lock"],
        cases: [{
          id: "idl_demo::field_ops_out_of_tree",
          fixture: "idl",
          file: "field_ops_demo.integration_test.ts",
          testName:
            "idl_demo::field_ops_out_of_tree generates, builds, and runs copied demos",
          classification: "isolated-process",
          isolationReason: "builds and executes a copied external repository",
        }],
      },
    }),
  );
}
