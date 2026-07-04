import type { createJobsAdminHandlers } from "./rpc.ts";

type JobsAdminHandlers = ReturnType<typeof createJobsAdminHandlers>;

type JobsRpcRegistrar = {
  handle: {
    rpc: {
      jobs: {
        query(handler: JobsAdminHandlers["query"]): Promise<void>;
        inspect(handler: JobsAdminHandlers["inspect"]): Promise<void>;
        cancel(handler: JobsAdminHandlers["cancel"]): Promise<void>;
        listServices(
          handler: JobsAdminHandlers["listServices"],
        ): Promise<void>;
      };
    };
  };
};

/** Registers the Jobs admin RPC subset implemented by the JS control-plane. */
export async function registerJobsAdmin(deps: {
  trellis: JobsRpcRegistrar;
  handlers: JobsAdminHandlers;
}): Promise<void> {
  await deps.trellis.handle.rpc.jobs.query(deps.handlers.query);
  await deps.trellis.handle.rpc.jobs.inspect(deps.handlers.inspect);
  await deps.trellis.handle.rpc.jobs.cancel(deps.handlers.cancel);
  await deps.trellis.handle.rpc.jobs.listServices(deps.handlers.listServices);
}
