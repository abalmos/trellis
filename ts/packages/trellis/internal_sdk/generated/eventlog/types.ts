// Generated from ./rust/crates/eventlog-runtime/.trellis/artifacts/apis/trellis.eventlog@v1.json
import type { SerializableErrorData } from "@qlever-llc/trellis";
import { TrellisError } from "@qlever-llc/trellis";
import { NotFoundErrorDataSchema } from "./schemas.ts";

export type EventLogConsumersInspectInput = {
  consumerName: string;
  stream?: string;
};
export type EventLogConsumersInspectOutput = {};

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
  consumers: Array<
    {
      ackPending: number;
      ackWaitMs?: number;
      consumerName: string;
      contractId?: string;
      deploymentId?: string;
      filterSubjects: Array<string>;
      group?: string;
      managedBy?: "authority" | "platform" | "external";
      maxDeliver?: number;
      oldestPendingAt?: string;
      oldestPendingEventId?: string;
      pending: number;
      redelivered?: number;
      status:
        | "current"
        | "processing"
        | "behind"
        | "saturated"
        | "inactive"
        | "failing"
        | "missing"
        | "orphaned";
      stream: string;
      waitingPulls: number;
    }
  >;
  limit: number;
  offset: number;
  total: number;
};

export type EventLogInspectInput = {
  eventId?: string;
  streamSequence?: number;
};
export type EventLogInspectOutput = {};

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
  sort?: {};
  subject?: string;
  verificationStatus?: Array<"verified">;
  window?: "15m" | "1h" | "6h" | "24h" | "7d";
};
export type EventLogQueryOutput = {
  events: Array<
    {
      eventId: string;
      eventTime: string;
      headerCount: number;
      ownerContractId?: string;
      ownerEventName?: string;
      payloadSizeBytes: number;
      publisherDeploymentId?: string;
      publisherInstanceId?: string;
      publisherKind?: "service" | "device" | "user";
      publisherParticipantDigest?: string;
      publisherParticipantId?: string;
      resolution: "resolved" | "unresolved" | "malformed";
      streamSequence: number;
      subject: string;
      traceId?: string;
      verificationStatus: "verified";
    }
  >;
  limit: number;
  offset: number;
  total: number;
};

export type EventLogWatchInput = {};
export type EventLogWatchEvent = {};

export type NotFoundErrorData =
  & SerializableErrorData
  & ({ context?: {}; id: string; message: string; type: "NotFoundError" });
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
