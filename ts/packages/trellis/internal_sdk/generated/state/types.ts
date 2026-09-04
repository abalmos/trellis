// Generated from ./rust/crates/runtime/.trellis/artifacts/apis/trellis.state@v1.json
import type { SerializableErrorData } from "@qlever-llc/trellis";
import { TrellisError } from "@qlever-llc/trellis";

export type StateAdminDeleteInput = {
  contractDigest: string;
  contractId: string;
  expectedRevision?: string;
  key?: string;
  scope: "userApp";
  store: string;
  userId: string;
} | {
  contractDigest: string;
  contractId: string;
  deviceId: string;
  expectedRevision?: string;
  key?: string;
  scope: "deviceApp";
  store: string;
};
export type StateAdminDeleteOutput = { deleted: boolean };

export type StateAdminGetInput = {
  contractDigest: string;
  contractId: string;
  key?: string;
  scope: "userApp";
  store: string;
  userId: string;
} | {
  contractDigest: string;
  contractId: string;
  deviceId: string;
  key?: string;
  scope: "deviceApp";
  store: string;
};
export type StateAdminGetOutput = { found: false } | {
  entry: {
    expiresAt?: string;
    key?: string;
    revision: string;
    updatedAt: string;
    value: unknown;
  };
  found: true;
} | {
  currentStateVersion: string;
  entry: {
    expiresAt?: string;
    key?: string;
    revision: string;
    updatedAt: string;
    value: unknown;
  };
  migrationRequired: true;
  stateVersion: string;
  writerContractDigest: string;
};

export type StateAdminListInput = {
  contractDigest: string;
  contractId: string;
  limit: number;
  offset?: number;
  prefix?: string;
  scope: "userApp";
  store: string;
  userId: string;
} | {
  contractDigest: string;
  contractId: string;
  deviceId: string;
  limit: number;
  offset?: number;
  prefix?: string;
  scope: "deviceApp";
  store: string;
};
export type StateAdminListOutput = {
  count: number;
  entries: Array<
    ({
      expiresAt?: string;
      key?: string;
      revision: string;
      updatedAt: string;
      value: unknown;
    } | {
      currentStateVersion: string;
      entry: {
        expiresAt?: string;
        key?: string;
        revision: string;
        updatedAt: string;
        value: unknown;
      };
      migrationRequired: true;
      stateVersion: string;
      writerContractDigest: string;
    })
  >;
  limit: number;
  nextOffset?: number;
  offset: number;
};

export type StateDeleteInput = {
  expectedRevision?: string;
  key?: string;
  store: string;
};
export type StateDeleteOutput = { deleted: boolean };

export type StateGetInput = { key?: string; store: string };
export type StateGetOutput = { found: false } | {
  entry: {
    expiresAt?: string;
    key?: string;
    revision: string;
    updatedAt: string;
    value: unknown;
  };
  found: true;
} | {
  currentStateVersion: string;
  entry: {
    expiresAt?: string;
    key?: string;
    revision: string;
    updatedAt: string;
    value: unknown;
  };
  migrationRequired: true;
  stateVersion: string;
  writerContractDigest: string;
};

export type StateListInput = {
  limit: number;
  offset?: number;
  prefix?: string;
  store: string;
};
export type StateListOutput = {
  count: number;
  entries: Array<
    ({
      expiresAt?: string;
      key?: string;
      revision: string;
      updatedAt: string;
      value: unknown;
    } | {
      currentStateVersion: string;
      entry: {
        expiresAt?: string;
        key?: string;
        revision: string;
        updatedAt: string;
        value: unknown;
      };
      migrationRequired: true;
      stateVersion: string;
      writerContractDigest: string;
    })
  >;
  limit: number;
  nextOffset?: number;
  offset: number;
};

export type StatePutInput = {
  expectedRevision?: string | null;
  key?: string;
  store: string;
  ttlMs?: number;
  value: unknown;
};
export type StatePutOutput = {
  applied: true;
  entry: {
    expiresAt?: string;
    key?: string;
    revision: string;
    updatedAt: string;
    value: unknown;
  };
} | {
  applied: false;
  entry?: {
    expiresAt?: string;
    key?: string;
    revision: string;
    updatedAt: string;
    value: unknown;
  } | {
    currentStateVersion: string;
    entry: {
      expiresAt?: string;
      key?: string;
      revision: string;
      updatedAt: string;
      value: unknown;
    };
    migrationRequired: true;
    stateVersion: string;
    writerContractDigest: string;
  };
  found: boolean;
};

export type AuthErrorData = SerializableErrorData;
export class AuthError extends TrellisError<AuthErrorData> {
  override readonly name = "AuthError" as const;
  readonly data: AuthErrorData;
  constructor(data: AuthErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }
  static fromSerializable(data: AuthErrorData): AuthError {
    return new AuthError(data);
  }
  override toSerializable(): AuthErrorData {
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
