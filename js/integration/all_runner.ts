import { main } from "./runner.ts";

/** Runs the supported TypeScript client/service integration matrix. */
export { main };

if (import.meta.main) {
  Deno.exit(await main(Deno.args));
}
