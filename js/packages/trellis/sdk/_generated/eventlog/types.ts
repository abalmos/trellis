// Generated from ./generated/contracts/manifests/trellis.eventlog@v1.json
import type {
  AsyncResult,
  BaseError,
  HandlerTrellis,
  Result,
  RpcHandlerContext,
  SessionCaller,
  TrellisErrorInstance,
  UnexpectedError,
  ValidationError,
} from "../../../index.ts";

import type { Api } from "./api.ts";

import type { SerializableErrorData } from "../../../contracts.ts";
import { TrellisError } from "../../../errors/index.ts";

import { NotFoundErrorDataSchema } from "./schemas.ts";

export type HandlerClient = HandlerTrellis<Api>;

export const CONTRACT_ID = "trellis.eventlog@v1" as const;
export const CONTRACT_DIGEST =
  "A-5LNMwhnF1jpE7COy2chzjy8BNPs4TvO-oFutFMWpE" as const;

export type EventLogConsumersInspectInput = {
  consumerName: string;
  stream?: string;
};
export type EventLogConsumersInspectOutput = { [k: string]: unknown };

export type EventLogConsumersQueryInput = {
  contractId?: string;
  deploymentId?: string;
  limit: number;
  offset?: number;
  ownerContractId?: string;
  status?: Array<
    (
      | "current"
      | "processing"
      | "behind"
      | "saturated"
      | "inactive"
      | "failing"
      | "missing"
      | "orphaned"
    )
  >;
  subject?: string;
};
export type EventLogConsumersQueryOutput = {
  consumers: Array<unknown>;
  limit: number;
  offset: number;
  total: number;
};

export type EventLogInspectInput = {
  eventId?: string;
  streamSequence?: number;
};
export type EventLogInspectOutput = { [k: string]: unknown };

export type EventLogMetricsInput = {
  window?: "15m" | "1h" | "6h" | "24h" | "7d";
};
export type EventLogMetricsOutput = {
  buckets: Array<{ [k: string]: unknown }>;
  summary: {
    byResolution: {
      malformed?: number;
      resolved?: number;
      unresolved?: number;
    };
    byVerificationStatus: {
      "auth-unavailable"?: number;
      "invalid-signature"?: number;
      "missing-proof"?: number;
      "missing-session"?: number;
      "outside-session-window"?: number;
      "subject-denied"?: number;
      verified?: number;
    };
    eventTypes: Array<
      { count: number; ownerContractId: string; ownerEventName: string }
    >;
    payloadSizeBytes: number;
    total: number;
    uniqueSubjects: number;
  };
};

export type EventLogQueryInput = {
  consumerDeploymentId?: string;
  consumerName?: string;
  excludeEventTypes?: Array<
    { ownerContractId: string; ownerEventName: string }
  >;
  includeEventTypes?: Array<
    { ownerContractId: string; ownerEventName: string }
  >;
  limit: number;
  offset?: number;
  ownerContractId?: string;
  ownerEventName?: string;
  publisherContractId?: string;
  publisherDeploymentId?: string;
  resolution?: Array<("resolved" | "unresolved" | "malformed")>;
  search?: string;
  sort?: { [k: string]: unknown };
  subject?: string;
  verificationStatus?: Array<
    (
      | "verified"
      | "missing-proof"
      | "invalid-signature"
      | "missing-session"
      | "subject-denied"
      | "outside-session-window"
      | "auth-unavailable"
    )
  >;
  window?: "15m" | "1h" | "6h" | "24h" | "7d";
};
export type EventLogQueryOutput = {
  events: Array<unknown>;
  limit: number;
  offset: number;
  total: number;
};

export type EventLogWatchInput = { [k: string]: unknown };
export type EventLogWatchEvent = { [k: string]: unknown };
export type EventLogWatchFeedHandler = (
  context: {
    input: EventLogWatchInput;
    caller: SessionCaller;
    signal: AbortSignal;
    emit(
      event: EventLogWatchEvent,
    ): AsyncResult<void, ValidationError | UnexpectedError>;
    client: HandlerClient;
  },
) => unknown | Promise<unknown>;

export type NotFoundErrorData = {
  context?: { [k: string]: unknown };
  id: string;
  message: string;
  type: "NotFoundError";
  [k: string]: unknown;
};
export class NotFoundError extends TrellisError<NotFoundErrorData> {
  static readonly schema = NotFoundErrorDataSchema;
  override readonly name = "NotFoundError" as const;
  readonly data: NotFoundErrorData;

  constructor(data: NotFoundErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }

  static fromSerializable(data: NotFoundErrorData): NotFoundError {
    return new NotFoundError(data);
  }

  override toSerializable(): NotFoundErrorData {
    return this.data;
  }
}

export interface RpcMap {
  "EventLog.Consumers.Inspect": {
    input: EventLogConsumersInspectInput;
    output: EventLogConsumersInspectOutput;
  };
  "EventLog.Consumers.Query": {
    input: EventLogConsumersQueryInput;
    output: EventLogConsumersQueryOutput;
  };
  "EventLog.Inspect": {
    input: EventLogInspectInput;
    output: EventLogInspectOutput;
  };
  "EventLog.Metrics": {
    input: EventLogMetricsInput;
    output: EventLogMetricsOutput;
  };
  "EventLog.Query": { input: EventLogQueryInput; output: EventLogQueryOutput };
}

export type EventLogConsumersInspectHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type EventLogConsumersInspectHandlerResult = Result<
  EventLogConsumersInspectOutput,
  EventLogConsumersInspectHandlerError
>;
export type EventLogConsumersInspectHandler = (
  args: {
    input: EventLogConsumersInspectInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) =>
  | EventLogConsumersInspectHandlerResult
  | Promise<EventLogConsumersInspectHandlerResult>;
export type EventLogConsumersQueryHandlerError = TrellisErrorInstance;
export type EventLogConsumersQueryHandlerResult = Result<
  EventLogConsumersQueryOutput,
  EventLogConsumersQueryHandlerError
>;
export type EventLogConsumersQueryHandler = (
  args: {
    input: EventLogConsumersQueryInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) =>
  | EventLogConsumersQueryHandlerResult
  | Promise<EventLogConsumersQueryHandlerResult>;
export type EventLogInspectHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type EventLogInspectHandlerResult = Result<
  EventLogInspectOutput,
  EventLogInspectHandlerError
>;
export type EventLogInspectHandler = (
  args: {
    input: EventLogInspectInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => EventLogInspectHandlerResult | Promise<EventLogInspectHandlerResult>;
export type EventLogMetricsHandlerError = TrellisErrorInstance;
export type EventLogMetricsHandlerResult = Result<
  EventLogMetricsOutput,
  EventLogMetricsHandlerError
>;
export type EventLogMetricsHandler = (
  args: {
    input: EventLogMetricsInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => EventLogMetricsHandlerResult | Promise<EventLogMetricsHandlerResult>;
export type EventLogQueryHandlerError = TrellisErrorInstance;
export type EventLogQueryHandlerResult = Result<
  EventLogQueryOutput,
  EventLogQueryHandlerError
>;
export type EventLogQueryHandler = (
  args: {
    input: EventLogQueryInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => EventLogQueryHandlerResult | Promise<EventLogQueryHandlerResult>;

export interface EventMap {
}

export interface FeedMap {
  "EventLog.Watch": { input: EventLogWatchInput; event: EventLogWatchEvent };
}

export interface SubjectMap {
}
