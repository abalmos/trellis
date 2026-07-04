import type { ServiceHandle } from "@qlever-llc/trellis/sdk/jobs";
import type { createJobsAdminHandlers } from "./rpc.ts";

type JobsAdminHandlers = ReturnType<typeof createJobsAdminHandlers>;

type JobsRpcRegistrar = {
  handle: {
    rpc: {
      jobs: Pick<
        ServiceHandle["rpc"]["jobs"],
        "health" | "query" | "inspect" | "cancel" | "listServices"
      >;
    };
  };
};

/** Registers the Jobs admin RPC subset implemented by the JS control-plane. */
export async function registerJobsAdmin(deps: {
  trellis: JobsRpcRegistrar;
  handlers: JobsAdminHandlers;
}): Promise<void> {
  await deps.trellis.handle.rpc.jobs.health(deps.handlers.health);
  await deps.trellis.handle.rpc.jobs.query(deps.handlers.query);
  await deps.trellis.handle.rpc.jobs.inspect(deps.handlers.inspect);
  await deps.trellis.handle.rpc.jobs.cancel(deps.handlers.cancel);
  await deps.trellis.handle.rpc.jobs.listServices(deps.handlers.listServices);
}
