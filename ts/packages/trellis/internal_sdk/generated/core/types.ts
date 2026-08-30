// Generated from ./ts/packages/trellis/.trellis/generated/protocol/apis/trellis.core@v1.json
import type { SerializableErrorData } from "../../../contracts.ts";
import { TrellisError } from "../../../errors/index.ts";

export type TrellisSurfaceStatusInput = {
  action?: "call" | "publish" | "subscribe" | "observe";
  contractId: string;
  kind: "rpc" | "operation" | "event" | "feed";
  surface: string;
};
export type TrellisSurfaceStatusOutput = {
  status:
    | {
      liveImplementer: boolean;
      runtime: "live" | "no_live_implementer" | "disabled";
      state: "available";
    }
    | { reason: "authority_unavailable"; state: "unavailable" }
    | { missingCapabilities: Array<string>; state: "unauthorized" }
    | { contractId: string; state: "unknown_contract" }
    | {
      contractId: string;
      kind: string;
      state: "unknown_surface";
      surface: string;
    };
};

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
