// Generated from ./generated/protocol/apis/trellis.eventlog@v1.json
import type { SerializableErrorData } from "../../../contracts.ts";
import { TrellisError } from "../../../errors/index.ts";
import { NotFoundErrorDataSchema } from "./schemas.ts";

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
  buckets: Array<
    {
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
      integrityExceptions: number;
      payloadSizeBytes: number;
      start: string;
      total: number;
    }
  >;
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
    integrityExceptions: number;
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
  integrityExceptionOnly?: boolean;
  limit: number;
  offset?: number;
  ownerContractId?: string;
  ownerEventName?: string;
  publisherDeploymentId?: string;
  publisherParticipantId?: string;
  resolution?: Array<("resolved" | "unresolved" | "malformed")>;
  search?: string;
  sort?: { [k: string]: unknown };
  subject?: string;
  verificationStatus?: Array<("verified")>;
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

export type NotFoundErrorData =
  & SerializableErrorData
  & ({
    context?: { [k: string]: unknown };
    id: string;
    message: string;
    type: "NotFoundError";
    [k: string]: unknown;
  });
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

export type UnexpectedErrorData = SerializableErrorData;
export class UnexpectedError extends TrellisError<UnexpectedErrorData> {
  override readonly name = "UnexpectedError" as const;
  readonly data: UnexpectedErrorData;
  constructor(data: UnexpectedErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }
  static fromSerializable(data: UnexpectedErrorData): UnexpectedError {
    return new UnexpectedError(data);
  }
  override toSerializable(): UnexpectedErrorData {
    return this.data;
  }
}

export type ValidationErrorData = SerializableErrorData;
export class ValidationError extends TrellisError<ValidationErrorData> {
  override readonly name = "ValidationError" as const;
  readonly data: ValidationErrorData;
  constructor(data: ValidationErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }
  static fromSerializable(data: ValidationErrorData): ValidationError {
    return new ValidationError(data);
  }
  override toSerializable(): ValidationErrorData {
    return this.data;
  }
}
