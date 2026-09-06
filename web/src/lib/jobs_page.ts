import { AsyncResult, BaseError, isErr } from "@qlever-llc/result";
import { type apis } from "trellis-web-generated";

export type JobInspection = apis.jobs.JobsInspectOutput;

export type JobsPageData = {
  available: boolean;
  message?: string;
  services: apis.jobs.JobsListServicesOutput["entries"];
  jobs: apis.jobs.JobsQueryOutput["entries"];
  groups: apis.jobs.JobsQueryOutput["groups"];
  stats: apis.jobs.JobsQueryOutput["stats"];
  count: apis.jobs.JobsQueryOutput["count"];
  offset: apis.jobs.JobsQueryOutput["offset"];
  limit: apis.jobs.JobsQueryOutput["limit"];
  nextOffset?: apis.jobs.JobsQueryOutput["nextOffset"];
};

export type JobsDetailData = {
  available: boolean;
  message?: string;
  inspection?: JobInspection;
};

type JobsPageRpc = {
  listServices(
    input: apis.jobs.JobsListServicesInput,
  ): AsyncResult<apis.jobs.JobsListServicesOutput, BaseError>;
  queryJobs(
    filter: apis.jobs.JobsQueryInput,
  ): AsyncResult<apis.jobs.JobsQueryOutput, BaseError>;
};

type JobsDetailRpc = {
  inspect(
    input: apis.jobs.JobsInspectInput,
  ): AsyncResult<apis.jobs.JobsInspectOutput, BaseError>;
};

type JobsActionRpc<TOutput> = {
  action(
    input: { id: string; reason?: string },
  ): AsyncResult<TOutput, BaseError>;
};

function unavailableResult<T extends { available: false; message: string }>(
  result: T,
): T {
  return result;
}

function normalizedJobsUnavailable(error: unknown): string | null {
  let message: string;
  if (error instanceof BaseError) {
    message = String(error.getContext().causeMessage ?? error.message);
  } else if (error instanceof Error) {
    message = error.message;
  } else {
    message = String(error);
  }

  if (
    message.includes("Permissions Violation") &&
    message.includes("rpc.v1.Jobs.")
  ) {
    return "Your current session is not approved for Jobs RPCs. Sign out and sign back in to refresh permissions.";
  }

  const normalizedMessage = message.toLowerCase();
  if (
    normalizedMessage.includes("no responders") ||
    message.includes("No responders available for request") ||
    message.includes("references inactive contract") ||
    message.includes("not currently reachable")
  ) {
    return "Jobs admin runtime is not currently reachable.";
  }

  return null;
}

function isJobsNotFound(error: unknown): boolean {
  return error instanceof BaseError && error.name === "NotFoundError";
}

async function takeOrThrow<T>(result: AsyncResult<T, BaseError>): Promise<T> {
  const value = await result.take();
  if (isErr(value)) {
    throw value.error;
  }
  return value;
}

/** Queries Jobs workbench data through the typed Jobs.Query RPC boundary. */
export function queryJobs(
  rpc: Pick<JobsPageRpc, "queryJobs">,
  filter: apis.jobs.JobsQueryInput,
): AsyncResult<apis.jobs.JobsQueryOutput, BaseError> {
  return rpc.queryJobs(filter);
}

/** Loads the Jobs list page data and normalizes unavailable Jobs runtime errors. */
export async function loadJobsPageData(
  rpc: JobsPageRpc,
  filter: apis.jobs.JobsQueryInput = { limit: 50 },
): Promise<JobsPageData> {
  try {
    const servicesResponse = rpc.listServices({ limit: 500 });
    const jobsResponse = queryJobs(rpc, filter);
    const [servicesValue, jobsValue] = await Promise.all([
      takeOrThrow(servicesResponse),
      takeOrThrow(jobsResponse),
    ]);

    return {
      available: true,
      services: servicesValue.entries,
      jobs: jobsValue.entries,
      groups: jobsValue.groups,
      stats: jobsValue.stats,
      count: jobsValue.count,
      offset: jobsValue.offset,
      limit: jobsValue.limit,
      nextOffset: jobsValue.nextOffset,
    };
  } catch (error) {
    const message = normalizedJobsUnavailable(error);
    if (message) {
      return {
        available: false,
        message,
        services: [],
        jobs: [],
        groups: [],
        stats: { byState: {}, total: 0 },
        count: 0,
        offset: 0,
        limit: filter.limit,
      };
    }
    throw error;
  }
}

/** Loads a single job by globally addressable job id. */
export async function loadJobDetailData(
  rpc: JobsDetailRpc,
  id: string,
): Promise<JobsDetailData> {
  try {
    const value = await takeOrThrow(rpc.inspect({ id }));
    return { available: true, inspection: value };
  } catch (error) {
    const message = normalizedJobsUnavailable(error);
    if (message) {
      return unavailableResult({ available: false, message });
    }
    if (isJobsNotFound(error)) {
      return { available: true };
    }
    throw error;
  }
}

/** Cancels a cancellable job by id. */
export async function cancelJob(
  rpc: JobsActionRpc<apis.jobs.JobsCancelOutput>,
  id: string,
): Promise<apis.jobs.JobsCancelOutput> {
  return takeOrThrow(rpc.action({ id }));
}

/** Retries a failed job by id. */
export async function retryJob(
  rpc: JobsActionRpc<apis.jobs.JobsRetryOutput>,
  id: string,
): Promise<apis.jobs.JobsRetryOutput> {
  return takeOrThrow(rpc.action({ id }));
}

/** Replays a dead-lettered job by id. */
export async function replayDlqJob(
  rpc: JobsActionRpc<apis.jobs.JobsReplayDLQOutput>,
  id: string,
): Promise<apis.jobs.JobsReplayDLQOutput> {
  return takeOrThrow(rpc.action({ id }));
}

/** Dismisses a dead-lettered job by id. */
export async function dismissDlqJob(
  rpc: JobsActionRpc<apis.jobs.JobsDismissDLQOutput>,
  id: string,
): Promise<apis.jobs.JobsDismissDLQOutput> {
  return takeOrThrow(rpc.action({ id }));
}
