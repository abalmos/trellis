import type { Msg, NatsConnection } from "@nats-io/nats-core";
import { Pointer, Value } from "typebox/value";
import {
  type PermissionAtom,
  routeQueueGroup,
  type RuntimeApi,
} from "../../participant_runtime/api.ts";
import {
  AsyncResult,
  BaseError,
  err,
  isErr,
  ok,
  Result,
} from "@qlever-llc/result";
import { ulid } from "ulid";

import { type JsonValue, parseSchema } from "../../codec.ts";
import {
  canonicalizeJson,
  digestJson,
} from "../../participant_runtime/json.ts";
import {
  AuthError,
  OperationAlreadyTerminalError,
  OperationMismatchError,
  OperationNotFoundError,
  TransferError,
  type TrellisErrorInstance,
  UnexpectedError,
  ValidationError,
} from "../../errors/index.ts";
import type { LoggerLike } from "../../globals.ts";
import { serviceRuntimeLogger } from "./logger.ts";
import {
  recordTrellisError,
  type TrellisErrorMetricAttributes,
} from "../../telemetry/mod.ts";
import {
  type AcceptedOperation,
  annotateHandlerBoundaryError,
  buildRuntimeOperationSnapshot,
  type DurableOperationRecord,
  type HandlerFn,
  isOperationDeferred,
  isResultLike,
  isTerminalRuntimeOperationSnapshot,
  type MethodsOf,
  type OperationHandlerContext,
  type OperationInputOf,
  type OperationOutputOf,
  type OperationProgressOf,
  type OperationRegistration,
  type OperationRuntimeHandle,
  type OperationsOf,
  type OperationTransferContextOf,
  type OperationTransferHandle,
  type OperationUpdateOf,
  type RuntimeOperationAcceptedEnvelope,
  type RuntimeOperationController,
  type RuntimeOperationControlRequest,
  type RuntimeOperationDesc,
  type RuntimeOperationRecord,
  type RuntimeOperationSignal,
  type RuntimeOperationSnapshot,
  type RuntimeOperationState,
  safeJson,
  Trellis,
  type TrellisAuth,
  type TrellisMode,
  type TrellisOpts,
  type VerifiedCaller,
  verifyLocalAuthorization,
} from "../../session.ts";
import {
  type FileInfo,
  FileInfoSchema,
  type SendTransferGrant,
} from "../../transfer.ts";

type TrellisServiceRuntimeOpts<TA extends RuntimeApi> =
  & Omit<TrellisOpts<TA>, "api">
  & {
    api: TA;
    transferSupport?: RuntimeOperationTransferSupport;
    version?: string;
    operationDeploymentId?: string;
    feedOwnerId?: string;
  };

export type TrellisServiceRuntimeFor<TA extends RuntimeApi = RuntimeApi> =
  & Omit<TrellisServiceRuntime, "mount" | "operationHandle">
  & {
    mount<M extends MethodsOf<TA>>(
      method: M,
      fn: HandlerFn<TA, M>,
    ): Promise<void>;
    operationHandle<O extends OperationsOf<TA>>(
      operation: O,
    ): OperationRegistration<
      OperationInputOf<TA, O>,
      OperationProgressOf<TA, O>,
      OperationOutputOf<TA, O>,
      OperationTransferContextOf<TA, O>,
      BaseError,
      OperationUpdateOf<TA, O>
    >;
  };

type RegisteredRuntimeOperationDesc = RuntimeOperationDesc & {
  permissions?: RuntimeApi["operations"][string]["permissions"];
  callerCapabilities?: readonly string[];
  observeCapabilities?: readonly string[];
  cancelCapabilities?: readonly string[];
  controlCapabilities?: readonly string[];
};

type RuntimeOperationTransferSession = {
  grant: SendTransferGrant;
  transfer: OperationTransferHandle;
};

type RuntimeOperationTransferSupport = {
  openOperationTransfer(args: {
    sessionKey: string;
    permission: PermissionAtom | undefined;
    requiredCapabilities: readonly string[];
    store: string;
    key: string;
    expiresInMs: number;
    maxBytes?: number;
    contentType?: string;
    metadata?: Record<string, string>;
    onComplete?: (info: FileInfo) => Promise<void>;
  }): AsyncResult<RuntimeOperationTransferSession, TransferError>;
};

function isJsonValue(value: unknown): value is JsonValue {
  if (
    value === null || typeof value === "string" || typeof value === "number" ||
    typeof value === "boolean"
  ) {
    return true;
  }
  if (Array.isArray(value)) return value.every(isJsonValue);
  if (typeof value !== "object") return false;
  if (Object.getPrototypeOf(value) !== Object.prototype) return false;
  return Object.values(value).every(isJsonValue);
}

function asStringPointerValue(
  operation: string,
  input: unknown,
  pointer: `/${string}`,
  field: string,
): Result<string, TransferError> {
  const value = Pointer.Get(input as Record<string, unknown>, pointer);
  if (typeof value !== "string" || value.length === 0) {
    return err(
      new TransferError({
        operation: "transfer",
        context: { reason: "invalid_input", operation, field, pointer },
      }),
    );
  }
  return ok(value);
}

function asOptionalStringPointerValue(
  input: unknown,
  pointer?: `/${string}`,
): Result<string | undefined, TransferError> {
  if (!pointer) {
    return ok(undefined);
  }
  const value = Pointer.Get(input as Record<string, unknown>, pointer);
  if (value === undefined) {
    return ok(undefined);
  }
  if (typeof value !== "string" || value.length === 0) {
    return err(
      new TransferError({
        operation: "transfer",
        context: { reason: "invalid_input", field: pointer, pointer },
      }),
    );
  }
  return ok(value);
}

function asOptionalStringRecordPointerValue(
  input: unknown,
  pointer?: `/${string}`,
): Result<Record<string, string> | undefined, TransferError> {
  if (!pointer) {
    return ok(undefined);
  }
  const value = Pointer.Get(input as Record<string, unknown>, pointer);
  if (value === undefined) {
    return ok(undefined);
  }
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return err(
      new TransferError({
        operation: "transfer",
        context: { reason: "invalid_input", field: pointer, pointer },
      }),
    );
  }

  const entries = Object.entries(value as Record<string, unknown>);
  if (
    entries.some(([key, item]) => key.length === 0 || typeof item !== "string")
  ) {
    return err(
      new TransferError({
        operation: "transfer",
        context: { reason: "invalid_input", field: pointer, pointer },
      }),
    );
  }

  return ok(Object.fromEntries(entries) as Record<string, string>);
}

function traceIdFromTraceparent(
  traceparent: string | undefined,
): string | undefined {
  const [version, traceId, parentId, flags, extra] = traceparent?.split("-") ??
    [];
  if (
    extra !== undefined ||
    !/^[0-9a-f]{2}$/u.test(version ?? "") ||
    version === "ff" ||
    !/^[0-9a-f]{32}$/u.test(traceId ?? "") ||
    traceId === "00000000000000000000000000000000" ||
    !/^[0-9a-f]{16}$/u.test(parentId ?? "") ||
    parentId === "0000000000000000" ||
    !/^[0-9a-f]{2}$/u.test(flags ?? "")
  ) {
    return undefined;
  }
  return traceId;
}

function recordOperationServiceError(
  error: unknown,
  attributes: TrellisErrorMetricAttributes,
): void {
  recordTrellisError(error, {
    messagingSystem: "nats",
    surface: "operation",
    direction: "server",
    ...attributes,
  });
}

function isOperationRevisionConflict(cause: unknown): boolean {
  const message = cause instanceof Error
    ? `${cause.message} ${
      cause.cause instanceof Error ? cause.cause.message : ""
    }`
      .toLowerCase()
    : "";
  return message.includes("operation revision conflict") ||
    message.includes("wrong last sequence") ||
    message.includes("wrong last revision") ||
    message.includes("revision mismatch") ||
    message.includes("sequence mismatch");
}

export class TrellisServiceRuntime extends Trellis<RuntimeApi, TrellisMode> {
  #nats: NatsConnection;
  #version?: string;
  #log: LoggerLike;
  #operations = new Map<string, RuntimeOperationRecord>();
  #mountedOperationControls = new Set<string>();
  #stopPromise?: Promise<void>;
  #transferSupport?: RuntimeOperationTransferSupport;
  #operationOwnerId: string;
  readonly operations: RuntimeOperationController;

  private constructor(
    name: string,
    nats: NatsConnection,
    auth: TrellisAuth,
    opts?: TrellisServiceRuntimeOpts<RuntimeApi>,
  ) {
    super(name, nats, auth, {
      ...opts,
      log: opts?.log ?? serviceRuntimeLogger,
    });
    if (opts?.operationDeploymentId) {
      this.setOperationDeploymentId(opts.operationDeploymentId);
    }
    if (opts?.feedOwnerId) this.setFeedOwnerId(opts.feedOwnerId);
    this.#nats = nats;
    this.#version = opts?.version;
    this.#log = (opts?.log ?? serviceRuntimeLogger).child({
      lib: "trellis-service-runtime",
    });
    this.#transferSupport = opts?.transferSupport;
    this.#operationOwnerId = opts?.feedOwnerId ?? ulid();
    this.operations = {
      get: (operationId) =>
        AsyncResult.from((async () => {
          const runtime = await this.#resolveOperation(operationId);
          if (!runtime) {
            return err(this.#operationNotFoundError(operationId));
          }
          return ok(runtime.snapshot);
        })()),
      started: (operationId) =>
        this.#applyOperationUpdate(operationId, "running", {
          event: { type: "started" },
        }),
      progress: (operationId, progress) =>
        this.#applyOperationUpdate(operationId, "running", {
          patch: { progress },
          event: { type: "progress", progress },
        }),
      complete: (operationId, output) =>
        this.#applyOperationUpdate(operationId, "completed", {
          patch: { output },
          event: { type: "completed" },
        }),
      fail: (operationId, error) =>
        this.#applyOperationUpdate(operationId, "failed", {
          patch: { error: error.toSerializable() },
          event: { type: "failed" },
        }),
      cancel: (operationId) =>
        this.#applyOperationUpdate(operationId, "cancelled", {
          event: { type: "cancelled" },
        }),
      signals: (operationId) => this.#signals(operationId),
      nextSignal: (operationId, name) => this.#nextSignal(operationId, name),
    };
  }

  async *#signals(operationId: string): AsyncIterable<RuntimeOperationSignal> {
    let cursor = 0;
    while (true) {
      const next = await this.#nextSignalAfter(operationId, cursor).take();
      if (isErr(next)) {
        throw next.error;
      }
      cursor = next.sequence;
      yield next;
    }
  }

  #nextSignal(
    operationId: string,
    name?: string,
  ): AsyncResult<RuntimeOperationSignal, BaseError> {
    return AsyncResult.from((async () => {
      let cursor = 0;
      while (true) {
        const next = await this.#nextSignalAfter(operationId, cursor).take();
        if (isErr(next)) return next;
        cursor = next.sequence;
        if (!name || next.signal === name) return ok(next);
      }
    })());
  }

  #nextSignalAfter(
    operationId: string,
    afterSequence: number,
  ): AsyncResult<RuntimeOperationSignal, BaseError> {
    return AsyncResult.from((async () => {
      while (true) {
        const runtime = await this.#resolveOperation(operationId);
        if (!runtime) {
          return err(this.#operationNotFoundError(operationId));
        }

        const durable = await this.loadOperationRecord(operationId);
        if (durable && durable.revision > runtime.revision) {
          runtime.revision = durable.revision;
          runtime.snapshot = durable.snapshot;
          runtime.sequence = durable.sequence;
          runtime.signalSequence = durable.signalSequence;
          runtime.signals = durable.signals;
          runtime.terminal = durable.snapshot.state === "completed" ||
            durable.snapshot.state === "failed" ||
            durable.snapshot.state === "cancelled";
          if (durable.cancelRequestedAt) {
            runtime.cancelRequestedAt = durable.cancelRequestedAt;
          }
        }

        const queued = runtime.signals.find((signal) =>
          !signal.acknowledged && signal.sequence > afterSequence
        );
        if (queued) {
          return ok(queued);
        }
        if (runtime.terminal) {
          return err(this.#operationAlreadyTerminalError(runtime));
        }
        await new Promise((resolve) => setTimeout(resolve, 50));
      }
    })());
  }

  async #resolveOperation(
    operationId: string,
  ): Promise<RuntimeOperationRecord | null> {
    const existing = this.#operations.get(operationId);
    const durable = await this.loadOperationRecord(operationId);
    if (!durable) return null;

    if (
      existing?.ownerInstanceId === this.#operationOwnerId &&
      durable.ownerInstanceId === this.#operationOwnerId &&
      durable.ownerEpoch === existing.ownerEpoch
    ) {
      if (durable.revision > existing.revision) {
        existing.revision = durable.revision;
        existing.leaseExpiresAt = durable.leaseExpiresAt;
        existing.cancelRequestedAt = durable.cancelRequestedAt;
        existing.snapshot = durable.snapshot;
        existing.sequence = durable.sequence;
        existing.signalSequence = durable.signalSequence;
        existing.signals = durable.signals;
        existing.terminal = durable.snapshot.state === "completed" ||
          durable.snapshot.state === "failed" ||
          durable.snapshot.state === "cancelled";
      }
      if (existing.cancelRequestedAt || existing.terminal) {
        existing.cancellation.abort("operation cancelled");
      }
      return existing;
    }
    if (existing) this.#operations.delete(operationId);

    const cancellation = new AbortController();
    const runtime: RuntimeOperationRecord = {
      id: durable.invocationId,
      service: durable.snapshot.service,
      operation: durable.snapshot.operation,
      callerSessionKey: durable.callerSessionKey,
      invocationDigest: durable.invocationDigest,
      caller: durable.caller,
      creatorPrincipalId: durable.creatorPrincipalId,
      creatorParticipantId: durable.creatorParticipantId,
      apiId: durable.apiId,
      input: durable.input,
      revision: durable.revision,
      ownerInstanceId: durable.ownerInstanceId,
      ownerEpoch: durable.ownerEpoch,
      leaseExpiresAt: durable.leaseExpiresAt,
      ...(durable.cancelRequestedAt
        ? { cancelRequestedAt: durable.cancelRequestedAt }
        : {}),
      ...(durable.transferGrant
        ? { transferGrant: durable.transferGrant }
        : {}),
      snapshot: durable.snapshot,
      sequence: durable.sequence,
      terminal: durable.snapshot.state === "completed" ||
        durable.snapshot.state === "failed" ||
        durable.snapshot.state === "cancelled",
      signalSequence: durable.signalSequence,
      signals: durable.signals,
      watchers: new Map(),
      frameQueue: Promise.resolve(),
      signalWaiters: new Set(),
      cancellation,
    };
    if (runtime.cancelRequestedAt || runtime.terminal) {
      cancellation.abort("operation cancelled");
    }
    if (
      !runtime.terminal && runtime.ownerInstanceId !== this.#operationOwnerId &&
      Date.parse(runtime.leaseExpiresAt) <= Date.now()
    ) {
      runtime.ownerInstanceId = this.#operationOwnerId;
      runtime.ownerEpoch += 1;
      runtime.leaseExpiresAt = new Date(Date.now() + 30_000).toISOString();
      try {
        await this.saveOperationRecord(runtime);
      } catch {
        return await this.#resolveOperation(operationId);
      }
      runtime.reclaimed = true;
    }
    this.#operations.set(operationId, runtime);
    return runtime;
  }

  #applyOperationUpdate(
    operationId: string,
    state: RuntimeOperationState,
    opts: {
      patch?: Partial<RuntimeOperationSnapshot>;
      event: Record<string, unknown> & { type: string };
    },
    retry = 0,
  ): AsyncResult<RuntimeOperationSnapshot, BaseError> {
    return AsyncResult.from((async () => {
      const runtime = await this.#resolveOperation(operationId);
      if (!runtime) {
        return err(this.#operationNotFoundError(operationId));
      }

      return await this.#queueOperationFrame(runtime, async () => {
        if (runtime.terminal) {
          return err(this.#operationAlreadyTerminalError(runtime));
        }
        if (runtime.ownerInstanceId !== this.#operationOwnerId) {
          return err(
            new UnexpectedError({
              cause: new Error("operation update rejected by owner fence"),
            }),
          );
        }
        if (runtime.cancelRequestedAt || runtime.cancellation.signal.aborted) {
          return err(this.#operationAlreadyTerminalError(runtime));
        }
        runtime.leaseExpiresAt = new Date(Date.now() + 30_000).toISOString();

        runtime.sequence += 1;
        runtime.snapshot = buildRuntimeOperationSnapshot(
          runtime,
          state,
          opts.patch,
        );
        runtime.terminal = state === "completed" || state === "failed" ||
          state === "cancelled";

        try {
          await this.saveOperationRecord(runtime);
        } catch (cause) {
          this.#operations.delete(runtime.id);
          if (retry < 3 && isOperationRevisionConflict(cause)) {
            return await this.#applyOperationUpdate(
              operationId,
              state,
              opts,
              retry + 1,
            );
          }
          runtime.cancellation.abort("operation ownership lost");
          throw cause;
        }

        const frame = {
          kind: "event",
          sequence: runtime.sequence,
          event: {
            snapshot: runtime.snapshot,
            ...opts.event,
          },
        };
        for (const reply of runtime.watchers.keys()) {
          await this.#nats.publish(reply, JSON.stringify(frame));
        }

        if (runtime.terminal) {
          this.#rejectSignalWaiters(runtime);
        }

        return ok(runtime.snapshot);
      });
    })());
  }

  async #queueOperationFrame<T>(
    runtime: RuntimeOperationRecord,
    task: () => Promise<T>,
  ): Promise<T> {
    const previous = runtime.frameQueue;
    let release: (() => void) | undefined;
    runtime.frameQueue = new Promise<void>((resolve) => {
      release = resolve;
    });
    await previous;
    try {
      return await task();
    } finally {
      release?.();
    }
  }

  #emitOperationUpdate(
    runtime: RuntimeOperationRecord,
    ctx: RegisteredRuntimeOperationDesc,
    update: unknown,
  ): Promise<Result<void, BaseError>> {
    return this.#queueOperationFrame(runtime, async () => {
      const durable = await this.loadOperationRecord(runtime.id);
      if (
        runtime.terminal || runtime.cancellation.signal.aborted || !durable ||
        durable.cancelRequestedAt || durable.snapshot.state === "cancelled" ||
        durable.ownerInstanceId !== this.#operationOwnerId ||
        durable.ownerEpoch !== runtime.ownerEpoch
      ) {
        runtime.cancellation.abort("operation cancelled or ownership lost");
        return err(this.#operationAlreadyTerminalError(runtime));
      }
      if (ctx.update === undefined) {
        return err(
          new ValidationError({
            errors: [{
              path: "/",
              message:
                `Operation '${runtime.operation}' does not declare updates`,
            }],
            context: { operation: runtime.operation },
          }),
        );
      }
      const parsed = this.#validateOperationValue(ctx, "update", update).take();
      if (isErr(parsed)) return parsed;

      runtime.sequence += 1;
      const frame = {
        kind: "event",
        sequence: runtime.sequence,
        event: { type: "update", update: parsed, snapshot: runtime.snapshot },
      };
      for (const [reply, watcher] of runtime.watchers) {
        if (watcher.includeUpdates) {
          await this.#nats.publish(reply, JSON.stringify(frame));
        }
      }
      return ok(undefined);
    });
  }

  #validateOperationValue(
    ctx: RegisteredRuntimeOperationDesc,
    kind: "progress" | "update" | "output",
    value: unknown,
  ): Result<unknown, BaseError> {
    const schema = kind === "progress"
      ? ctx.progress
      : kind === "update"
      ? ctx.update
      : ctx.output;
    if (schema === undefined) return ok(value);
    if (!isJsonValue(value)) {
      return err(
        new ValidationError({
          errors: [{
            path: "/",
            message: `Operation ${kind} must be JSON-serializable`,
          }],
          context: { kind },
        }),
      );
    }
    const parsed = parseSchema(
      schema as Parameters<typeof parseSchema>[0],
      value,
    ).take();
    if (isErr(parsed)) return err(parsed.error);
    return ok(parsed);
  }

  #applyControlledOperationUpdate(
    runtime: RuntimeOperationRecord,
    ctx: RegisteredRuntimeOperationDesc,
    state: RuntimeOperationState,
    opts: {
      patch?: Partial<RuntimeOperationSnapshot>;
      event: Record<string, unknown> & { type: string };
    },
  ): AsyncResult<RuntimeOperationSnapshot, BaseError> {
    return AsyncResult.from((async () => {
      if (runtime.ownerInstanceId !== this.#operationOwnerId) {
        return err(
          new UnexpectedError({
            cause: new Error("operation update rejected by owner fence"),
          }),
        );
      }
      runtime.leaseExpiresAt = new Date(Date.now() + 30_000).toISOString();
      if (opts.patch?.progress !== undefined) {
        const parsed = this.#validateOperationValue(
          ctx,
          "progress",
          opts.patch.progress,
        ).take();
        if (isErr(parsed)) return parsed;
        opts.patch.progress = parsed;
        if ("progress" in opts.event) opts.event.progress = parsed;
      }
      if (opts.patch?.output !== undefined) {
        const parsed = this.#validateOperationValue(
          ctx,
          "output",
          opts.patch.output,
        ).take();
        if (isErr(parsed)) return parsed;
        opts.patch.output = parsed;
      }
      return await this.#applyOperationUpdate(
        runtime.id,
        state,
        opts,
      );
    })());
  }

  #controlOperation(
    operation: string,
    ctx: RegisteredRuntimeOperationDesc,
    operationId: string,
  ): AsyncResult<
    OperationRuntimeHandle<unknown, unknown, BaseError>,
    BaseError
  > {
    return AsyncResult.from((async () => {
      const runtime = await this.#resolveOperation(operationId);
      if (!runtime) {
        return err(this.#operationNotFoundError(operationId));
      }
      if (runtime.service !== this.name) {
        return err(this.#operationMismatchError(runtime, operation));
      }
      if (runtime.operation !== operation) {
        return err(this.#operationMismatchError(runtime, operation));
      }
      return ok(this.#makeControlledOperation(runtime, ctx));
    })());
  }

  #makeControlledOperation(
    runtime: RuntimeOperationRecord,
    ctx: RegisteredRuntimeOperationDesc,
  ): OperationRuntimeHandle<unknown, unknown, BaseError> {
    return {
      id: runtime.id,
      started: () =>
        this.#applyControlledOperationUpdate(runtime, ctx, "running", {
          event: { type: "started" },
        }),
      progress: (value: unknown) =>
        this.#applyControlledOperationUpdate(runtime, ctx, "running", {
          patch: { progress: value },
          event: { type: "progress", progress: value },
        }),
      emitUpdate: (value: unknown) =>
        AsyncResult.from(this.#emitOperationUpdate(runtime, ctx, value)),
      complete: (value: unknown) =>
        this.#applyControlledOperationUpdate(runtime, ctx, "completed", {
          patch: { output: value },
          event: { type: "completed" },
        }),
      fail: (error: BaseError) =>
        this.#applyControlledOperationUpdate(runtime, ctx, "failed", {
          patch: { error: error.toSerializable() },
          event: { type: "failed" },
        }),
      cancel: () =>
        this.#applyControlledOperationUpdate(runtime, ctx, "cancelled", {
          event: { type: "cancelled" },
        }),
      attach: (job: { wait(): AsyncResult<unknown, BaseError> }) =>
        AsyncResult.from((async () => {
          const waited = await job.wait();
          const waitedValue = waited.take();
          if (isErr(waitedValue)) {
            return err(new UnexpectedError({ cause: waitedValue.error }));
          }

          const finalRuntime = await this.#resolveOperation(runtime.id);
          if (!finalRuntime || !finalRuntime.terminal) {
            return err(
              new UnexpectedError({
                cause: new Error(
                  "attached job completed without terminal operation state",
                ),
              }),
            );
          }

          return ok(finalRuntime.snapshot);
        })()),
      signals: () => this.#signals(runtime.id),
      nextSignal: (name?: string) => this.#nextSignal(runtime.id, name),
      acknowledgeSignal: (sequence: number) =>
        this.#acknowledgeSignal(runtime.id, sequence),
      defer: () => ({ kind: "deferred" as const }),
    };
  }

  #makeAcceptedOperation(
    runtime: RuntimeOperationRecord,
    ctx: RegisteredRuntimeOperationDesc,
  ): AcceptedOperation<unknown, unknown, BaseError> {
    return {
      id: runtime.id,
      ref: {
        id: runtime.id,
        service: runtime.service,
        operation: runtime.operation,
      },
      snapshot: runtime.snapshot,
      started: () => this.operations.started(runtime.id),
      progress: (value: unknown) => this.operations.progress(runtime.id, value),
      emitUpdate: (value: unknown) =>
        AsyncResult.from(this.#emitOperationUpdate(runtime, ctx, value)),
      complete: (value: unknown) => this.operations.complete(runtime.id, value),
      fail: (error: BaseError) => this.operations.fail(runtime.id, error),
      cancel: () => this.operations.cancel(runtime.id),
      attach: (job: { wait(): AsyncResult<unknown, BaseError> }) =>
        AsyncResult.from((async () => {
          const waited = await job.wait();
          const waitedValue = waited.take();
          if (isErr(waitedValue)) {
            return err(new UnexpectedError({ cause: waitedValue.error }));
          }

          const finalRuntime = await this.#resolveOperation(runtime.id);
          if (!finalRuntime || !finalRuntime.terminal) {
            return err(
              new UnexpectedError({
                cause: new Error(
                  "attached job completed without terminal operation state",
                ),
              }),
            );
          }

          return ok(finalRuntime.snapshot);
        })()),
      signals: () => this.#signals(runtime.id),
      nextSignal: (name?: string) => this.#nextSignal(runtime.id, name),
      acknowledgeSignal: (sequence: number) =>
        this.#acknowledgeSignal(runtime.id, sequence),
      defer: () => ({ kind: "deferred" as const }),
    };
  }

  #operationNotFoundError(operationId: string): OperationNotFoundError {
    return new OperationNotFoundError({ operationId });
  }

  #operationAlreadyTerminalError(
    runtime: RuntimeOperationRecord,
  ): OperationAlreadyTerminalError {
    return new OperationAlreadyTerminalError({
      operationId: runtime.id,
      state: runtime.snapshot.state,
      operation: runtime.operation,
      service: runtime.service,
    });
  }

  #operationMismatchError(
    runtime: RuntimeOperationRecord,
    expectedOperation: string,
  ): OperationMismatchError {
    return new OperationMismatchError({
      operationId: runtime.id,
      expectedService: this.name,
      expectedOperation,
      actualService: runtime.service,
      actualOperation: runtime.operation,
    });
  }

  #rejectSignalWaiters(runtime: RuntimeOperationRecord): void {
    const result = err(this.#operationAlreadyTerminalError(runtime));
    for (const waiter of runtime.signalWaiters) {
      waiter(result);
    }
    runtime.signalWaiters.clear();
  }

  async #acceptSignal(
    runtime: RuntimeOperationRecord,
    ctx: RegisteredRuntimeOperationDesc,
    control: Extract<RuntimeOperationControlRequest, { action: "signal" }>,
    requestId: string,
    retry = 0,
  ): Promise<
    Result<{
      kind: "signal-accepted";
      operationId: string;
      signal: string;
      signalSequence: number;
      acceptedAt: string;
      snapshot: RuntimeOperationSnapshot;
    }, BaseError>
  > {
    if (runtime.terminal) {
      return err(this.#operationAlreadyTerminalError(runtime));
    }
    const replay = runtime.signals.find((signal) =>
      signal.requestId === requestId
    );
    if (replay) {
      if (
        replay.signal !== control.signal ||
        canonicalizeJson((replay.input ?? null) as JsonValue) !==
          canonicalizeJson((control.input ?? null) as JsonValue)
      ) {
        return err(
          new ValidationError({
            errors: [{
              path: "/requestId",
              message:
                "Signal request id was already accepted with different input",
            }],
          }),
        );
      }
      return ok({
        kind: "signal-accepted",
        operationId: runtime.id,
        signal: replay.signal,
        signalSequence: replay.sequence,
        acceptedAt: replay.acceptedAt,
        snapshot: runtime.snapshot,
      });
    }
    if (runtime.signals.length >= 100) {
      return err(
        new ValidationError({
          errors: [{
            path: "/signal",
            message: "Operation signal limit exceeded",
          }],
        }),
      );
    }

    const descriptor = ctx.signals?.[control.signal];
    if (!descriptor) {
      return err(
        new ValidationError({
          errors: [{
            path: "/signal",
            message: `Unknown operation signal '${control.signal}'`,
          }],
          context: { operation: runtime.operation, signal: control.signal },
        }),
      );
    }

    const input = control.input as JsonValue;
    const parsed = parseSchema(
      descriptor.input as Parameters<typeof parseSchema>[0],
      input,
    ).take();
    if (isErr(parsed)) {
      return err(parsed.error as ValidationError | UnexpectedError);
    }

    const encodedInput = new TextEncoder().encode(
      JSON.stringify(control.input ?? null),
    ).byteLength;
    if (encodedInput > 64 * 1024) {
      return err(
        new ValidationError({
          errors: [{
            path: "/input",
            message: "Operation signal payload exceeds 64 KiB",
          }],
        }),
      );
    }

    runtime.signalSequence += 1;
    const acceptedAt = new Date().toISOString();
    const signal: RuntimeOperationSignal = {
      operationId: runtime.id,
      sequence: runtime.signalSequence,
      requestId,
      signal: control.signal,
      ...(control.input !== undefined ? { input: control.input } : {}),
      acceptedAt,
      acknowledged: false,
    };
    runtime.signals.push(signal);
    try {
      await this.saveOperationRecord(runtime);
    } catch (cause) {
      this.#operations.delete(runtime.id);
      if (retry < 3 && isOperationRevisionConflict(cause)) {
        const current = await this.#resolveOperation(runtime.id);
        if (current) {
          return await this.#acceptSignal(
            current,
            ctx,
            control,
            requestId,
            retry + 1,
          );
        }
      }
      throw cause;
    }
    const result = ok(signal);
    for (const waiter of runtime.signalWaiters) {
      waiter(result);
    }

    return ok({
      kind: "signal-accepted",
      operationId: runtime.id,
      signal: signal.signal,
      signalSequence: signal.sequence,
      acceptedAt,
      snapshot: runtime.snapshot,
    });
  }

  #acknowledgeSignal(
    operationId: string,
    sequence: number,
    retry = 0,
  ): AsyncResult<void, BaseError> {
    return AsyncResult.from((async () => {
      const runtime = await this.#resolveOperation(operationId);
      if (!runtime) return err(this.#operationNotFoundError(operationId));
      return await this.#queueOperationFrame(runtime, async () => {
        if (
          runtime.terminal || runtime.cancelRequestedAt ||
          runtime.ownerInstanceId !== this.#operationOwnerId ||
          Date.parse(runtime.leaseExpiresAt) <= Date.now()
        ) {
          return err(this.#operationAlreadyTerminalError(runtime));
        }
        const signal = runtime.signals.find((item) =>
          item.sequence === sequence
        );
        if (!signal) {
          return err(
            new ValidationError({
              errors: [{
                path: "/sequence",
                message: `Operation signal ${sequence} was not found`,
              }],
            }),
          );
        }
        if (signal.acknowledged) return ok(undefined);
        signal.acknowledged = true;
        try {
          await this.saveOperationRecord(runtime);
        } catch (cause) {
          this.#operations.delete(runtime.id);
          if (retry < 3 && isOperationRevisionConflict(cause)) {
            return await this.#acknowledgeSignal(
              operationId,
              sequence,
              retry + 1,
            );
          }
          runtime.cancellation.abort("operation ownership lost");
          throw cause;
        }
        return ok(undefined);
      });
    })());
  }

  async #authenticateOperationMessage(
    msg: Msg,
    ctx: RegisteredRuntimeOperationDesc,
    parseInput: boolean,
    permission: PermissionAtom | undefined = ctx.permissions?.invoke,
    requiredCapabilities: readonly string[] = ctx.callerCapabilities ?? [],
  ): Promise<
    Result<{
      input: unknown;
      invocationId?: string;
      caller: VerifiedCaller;
      sessionKey: string;
    }, UnexpectedError | AuthError | ValidationError>
  > {
    const jsonData = safeJson(msg).take();
    if (isErr(jsonData)) return jsonData;

    let parsedInput: unknown;
    let invocationId: string | undefined;
    if (parseInput) {
      if (
        !jsonData || typeof jsonData !== "object" || Array.isArray(jsonData) ||
        typeof (jsonData as Record<string, unknown>).invocationId !==
          "string" ||
        !/^[0-9A-HJKMNP-TV-Z]{26}$/u.test(
          (jsonData as Record<string, string>).invocationId,
        ) || !("input" in jsonData)
      ) {
        return err(
          new ValidationError({
            errors: [{
              path: "",
              message: "Operation start requires a ULID invocationId and input",
            }],
          }),
        );
      }
      invocationId = (jsonData as Record<string, string>).invocationId;
      const parsedInputResult = parseSchema(
        ctx.input as Parameters<typeof parseSchema>[0],
        (jsonData as Record<string, JsonValue>).input,
      ).take();
      if (isErr(parsedInputResult)) {
        return err(
          parsedInputResult.error as ValidationError | UnexpectedError,
        );
      }
      parsedInput = parsedInputResult;
    } else {
      parsedInput = jsonData;
    }

    const auth = await verifyLocalAuthorization({
      kind: "request",
      cache: this.auth.authorizationProviderCache,
      message: msg,
      permission,
      requiredCapabilities,
    });
    const authValue = auth.take();
    if (isErr(authValue)) return err(authValue.error);

    return ok({
      input: parsedInput,
      ...(invocationId ? { invocationId } : {}),
      caller: authValue,
      sessionKey: authValue.sessionKey,
    });
  }

  #ensureOperationControlLoop(
    operation: string,
    ctx: RegisteredRuntimeOperationDesc,
  ): void {
    const controlSubject = `${ctx.subject}.control`;
    if (this.#mountedOperationControls.has(controlSubject)) {
      return;
    }
    this.#mountedOperationControls.add(controlSubject);

    const publishFrame = async (reply: string, frame: unknown) => {
      await this.#nats.publish(reply, JSON.stringify(frame));
    };

    const publishSnapshot = async (
      reply: string,
      snapshot: RuntimeOperationSnapshot,
    ) => {
      await publishFrame(reply, { kind: "snapshot", snapshot });
    };

    const respondControlError = (msg: Msg, error: Error | BaseError) => {
      const trellisError = error instanceof BaseError
        ? error
        : new UnexpectedError({ cause: error });
      recordOperationServiceError(trellisError, {
        operation,
        phase: "control",
      });
      msg.respond(JSON.stringify({
        kind: "error",
        error: trellisError.toSerializable(),
      }));
    };

    const controlSub = this.#nats.subscribe(controlSubject, {
      queue: routeQueueGroup(controlSubject),
    });
    void (async () => {
      for await (const msg of controlSub) {
        const request = safeJson(msg).take();
        if (isErr(request)) {
          respondControlError(msg, request.error);
          continue;
        }

        if (
          !request ||
          typeof request !== "object" ||
          !["get", "watch", "signal", "cancel"].includes(
            String((request as RuntimeOperationControlRequest).action),
          ) ||
          typeof (request as RuntimeOperationControlRequest).operationId !==
            "string" ||
          ((request as RuntimeOperationControlRequest).action === "signal" &&
            typeof (request as { signal?: unknown }).signal !== "string")
        ) {
          respondControlError(
            msg,
            new UnexpectedError({
              cause: new Error("Invalid operation control request"),
            }),
          );
          continue;
        }

        const control = request as RuntimeOperationControlRequest;
        if (control.action === "signal" && !ctx.signals?.[control.signal]) {
          respondControlError(
            msg,
            new ValidationError({
              errors: [{
                path: "/signal",
                message: `Unknown operation signal '${control.signal}'`,
              }],
            }),
          );
          continue;
        }
        let permission = ctx.permissions?.observe;
        let capabilities = ctx.observeCapabilities ?? [];
        if (control.action === "cancel") {
          permission = ctx.permissions?.cancel;
          capabilities = ctx.cancelCapabilities ?? [];
        } else if (control.action === "signal") {
          permission = ctx.permissions?.control[control.signal];
          capabilities = ctx.controlCapabilities ?? [];
        }
        const validated = await this.#authenticateOperationMessage(
          msg,
          ctx,
          false,
          permission,
          capabilities,
        );
        const value = validated.take();
        if (isErr(value)) {
          respondControlError(msg, value.error);
          continue;
        }

        let runtime: RuntimeOperationRecord | null;
        try {
          runtime = await this.#resolveOperation(control.operationId);
        } catch (cause) {
          respondControlError(
            msg,
            cause instanceof Error ? cause : new Error(String(cause)),
          );
          continue;
        }
        if (!runtime) {
          respondControlError(
            msg,
            this.#operationNotFoundError(control.operationId),
          );
          continue;
        }

        if (runtime.service !== this.name || runtime.operation !== operation) {
          respondControlError(
            msg,
            this.#operationMismatchError(runtime, operation),
          );
          continue;
        }

        const snapshot = runtime.snapshot;
        if (
          runtime.creatorPrincipalId !== value.caller.principalId ||
          runtime.creatorParticipantId !== value.caller.participantId
        ) {
          respondControlError(
            msg,
            new AuthError({
              reason: "forbidden",
            }),
          );
          continue;
        }
        if (control.action === "watch") {
          if (msg.reply) {
            const reply = msg.reply;
            await publishSnapshot(reply, runtime.snapshot);
            void (async () => {
              let sequence = runtime.sequence;
              while (!runtime.terminal) {
                await new Promise((resolve) => setTimeout(resolve, 50));
                const retainedAuthorization = await this
                  .#authenticateOperationMessage(
                    msg,
                    ctx,
                    false,
                    ctx.permissions?.observe,
                    ctx.observeCapabilities ?? [],
                  );
                const retainedAuthorizationValue = retainedAuthorization.take();
                if (isErr(retainedAuthorizationValue)) {
                  respondControlError(msg, retainedAuthorizationValue.error);
                  break;
                }
                const durable = await this.loadOperationRecord(
                  control.operationId,
                );
                if (!durable || durable.sequence === sequence) continue;
                sequence = durable.sequence;
                await publishSnapshot(reply, durable.snapshot);
                if (
                  durable.snapshot.state === "completed" ||
                  durable.snapshot.state === "failed" ||
                  durable.snapshot.state === "cancelled"
                ) break;
              }
            })().catch((error) => {
              if (!this.#nats.isClosed()) {
                this.#log.warn(
                  { error, operation: String(operation) },
                  "Operation watch stopped",
                );
              }
            });
          }
          continue;
        }

        if (control.action === "get") {
          msg.respond(JSON.stringify({ kind: "snapshot", snapshot }));
          continue;
        }

        if (control.action === "cancel") {
          if (runtime.terminal) {
            respondControlError(
              msg,
              this.#operationAlreadyTerminalError(runtime),
            );
            continue;
          }
          try {
            let current = runtime;
            for (let retry = 0;; retry++) {
              current.cancelRequestedAt = new Date().toISOString();
              current.sequence += 1;
              current.snapshot = buildRuntimeOperationSnapshot(
                current,
                "cancelled",
                { completedAt: current.cancelRequestedAt },
              );
              current.terminal = true;
              try {
                await this.saveOperationRecord(current);
                break;
              } catch (cause) {
                this.#operations.delete(current.id);
                if (retry >= 3 || !isOperationRevisionConflict(cause)) {
                  throw cause;
                }
                const reloaded = await this.#resolveOperation(current.id);
                if (!reloaded || reloaded.terminal) throw cause;
                current = reloaded;
              }
            }
            current.cancellation.abort("operation cancelled");
            msg.respond(
              JSON.stringify({ kind: "snapshot", snapshot: current.snapshot }),
            );
          } catch (cause) {
            respondControlError(msg, new UnexpectedError({ cause }));
          }
          continue;
        }

        if (control.action === "signal") {
          if (!runtime) {
            respondControlError(
              msg,
              new UnexpectedError({
                cause: new Error("operation is not running in this process"),
              }),
            );
            continue;
          }

          try {
            const accepted = await this.#acceptSignal(
              runtime,
              ctx,
              control,
              msg.headers?.get("request-id") ?? ulid(),
            );
            const acceptedValue = accepted.take();
            if (isErr(acceptedValue)) {
              respondControlError(msg, acceptedValue.error);
              continue;
            }
            msg.respond(JSON.stringify(acceptedValue));
          } catch (cause) {
            respondControlError(
              msg,
              cause instanceof Error ? cause : new Error(String(cause)),
            );
            continue;
          }
          continue;
        }

        respondControlError(
          msg,
          new UnexpectedError({
            cause: new Error(
              `Unknown operation control action '${control.action}' for '${operation}'`,
            ),
          }),
        );
      }
    })();
  }

  mountRuntime(
    method: string,
    fn: Parameters<Trellis<RuntimeApi, TrellisMode>["mount"]>[1],
  ): Promise<void> {
    return super.mount(method, fn);
  }

  static create<TA extends RuntimeApi>(
    name: string,
    nats: NatsConnection,
    auth: TrellisAuth,
    opts: TrellisServiceRuntimeOpts<TA>,
  ): TrellisServiceRuntimeFor<TA> {
    const runtime = new TrellisServiceRuntime(
      name,
      nats,
      auth,
      opts as TrellisServiceRuntimeOpts<RuntimeApi>,
    );
    return runtime as TrellisServiceRuntime & TrellisServiceRuntimeFor<TA>;
  }

  override operationHandle(
    operation: string,
  ): OperationRegistration<unknown, unknown, unknown, undefined, BaseError> {
    const ctx = this.api["operations"]
      ?.[operation as keyof typeof this.api.operations] as
        | RegisteredRuntimeOperationDesc
        | undefined;
    if (!ctx) {
      throw new Error(
        `Unknown operation '${operation.toString()}'. Did you forget to include its API module?`,
      );
    }

    return {
      control: (operationId) => {
        this.#ensureOperationControlLoop(String(operation), ctx);
        return this.#controlOperation(String(operation), ctx, operationId);
      },
      handle: async (
        handler: (
          context: OperationHandlerContext<
            unknown,
            unknown,
            unknown,
            OperationTransferHandle | undefined,
            BaseError
          >,
        ) => unknown | Promise<unknown>,
      ) => {
        const startSubject = ctx.subject;
        const now = () => new Date().toISOString();

        const publishFrame = async (reply: string, frame: unknown) => {
          await this.#nats.publish(reply, JSON.stringify(frame));
        };

        const makeOperation = (
          runtime: RuntimeOperationRecord,
          context: { requestId?: string; traceId?: string },
        ) => {
          return {
            id: runtime.id,
            started: () =>
              this.#applyControlledOperationUpdate(runtime, ctx, "running", {
                event: { type: "started" },
              }),
            progress: (value: unknown) =>
              this.#applyControlledOperationUpdate(runtime, ctx, "running", {
                patch: { progress: value },
                event: { type: "progress", progress: value },
              }),
            emitUpdate: (value: unknown) =>
              AsyncResult.from(this.#emitOperationUpdate(runtime, ctx, value)),
            complete: (value: unknown) =>
              this.#applyControlledOperationUpdate(runtime, ctx, "completed", {
                patch: { output: value, completedAt: now() },
                event: { type: "completed" },
              }),
            fail: (error: BaseError) =>
              AsyncResult.from((async () => {
                const annotatedError = annotateHandlerBoundaryError(error, {
                  operation: String(operation),
                  requestId: context.requestId,
                  service: this.name,
                  contractId: this.contractId,
                  contractDigest: this.contractDigest,
                  traceId: context.traceId,
                });
                return await this.#applyControlledOperationUpdate(
                  runtime,
                  ctx,
                  "failed",
                  {
                    patch: {
                      error: annotatedError.toSerializable(),
                      completedAt: now(),
                    },
                    event: { type: "failed" },
                  },
                );
              })()),
            cancel: () =>
              this.#applyControlledOperationUpdate(runtime, ctx, "cancelled", {
                patch: { completedAt: now() },
                event: { type: "cancelled" },
              }),
            attach: (job: { wait: () => AsyncResult<unknown, BaseError> }) =>
              AsyncResult.from((async () => {
                const waited = await job.wait();
                const waitedValue = waited.take();
                if (isErr(waitedValue)) {
                  return err(new UnexpectedError({ cause: waitedValue.error }));
                }

                const finalRuntime = await this.#resolveOperation(runtime.id);
                if (!finalRuntime || !finalRuntime.terminal) {
                  return err(
                    new UnexpectedError({
                      cause: new Error(
                        "attached job completed without terminal operation state",
                      ),
                    }),
                  );
                }

                return ok(finalRuntime.snapshot);
              })()),
            signals: () => this.#signals(runtime.id),
            nextSignal: (name?: string) => this.#nextSignal(runtime.id, name),
            acknowledgeSignal: (sequence: number) =>
              this.#acknowledgeSignal(runtime.id, sequence),
            defer: () => ({ kind: "deferred" as const }),
          };
        };

        const executeHandler = async (
          runtime: RuntimeOperationRecord,
          caller: VerifiedCaller,
          transferSession?: RuntimeOperationTransferSession,
          operationContext: { requestId?: string; traceId?: string } = {},
          resuming = false,
        ) => {
          const op = makeOperation(runtime, operationContext);
          try {
            const handlerResult: unknown = await handler(
              transferSession
                ? {
                  input: runtime.input,
                  op,
                  caller,
                  signal: runtime.cancellation.signal,
                  resuming,
                  ...(runtime.snapshot.progress !== undefined
                    ? { progress: runtime.snapshot.progress }
                    : {}),
                  transfer: transferSession.transfer,
                }
                : {
                  input: runtime.input,
                  op,
                  caller,
                  signal: runtime.cancellation.signal,
                  resuming,
                  ...(runtime.snapshot.progress !== undefined
                    ? { progress: runtime.snapshot.progress }
                    : {}),
                },
            );
            const handlerOutcome = isResultLike(handlerResult)
              ? handlerResult.take()
              : handlerResult;
            if (isErr(handlerOutcome)) {
              const error = annotateHandlerBoundaryError(handlerOutcome.error, {
                operation: String(operation),
                requestId: operationContext.requestId,
                service: this.name,
                contractId: this.contractId,
                contractDigest: this.contractDigest,
                traceId: operationContext.traceId,
              });
              recordOperationServiceError(error, {
                operation: String(operation),
                phase: "handler_result",
              });
              await op.fail(error);
              return;
            }
            if (isOperationDeferred(handlerOutcome)) return;
            if (isTerminalRuntimeOperationSnapshot(handlerOutcome)) {
              return;
            }
            if (!runtime.terminal) await op.complete(handlerOutcome);
          } catch (cause) {
            if (runtime.cancellation.signal.aborted) return;
            const error = annotateHandlerBoundaryError(cause, {
              operation: String(operation),
              requestId: operationContext.requestId,
              service: this.name,
              contractId: this.contractId,
              contractDigest: this.contractDigest,
              traceId: operationContext.traceId,
            });
            recordOperationServiceError(error, {
              operation: String(operation),
              phase: "handler_throw",
            });
            try {
              await op.fail(error).take();
            } catch (failure) {
              recordOperationServiceError(failure, {
                operation: String(operation),
                phase: "failure_persist",
              });
            }
          }
        };

        const startLeaseHeartbeat = (runtime: RuntimeOperationRecord) => {
          const leaseHeartbeat = setInterval(() => {
            if (runtime.terminal) {
              clearInterval(leaseHeartbeat);
              return;
            }
            void this.#queueOperationFrame(runtime, async () => {
              for (let retry = 0;; retry++) {
                const durable = await this.loadOperationRecord(runtime.id);
                if (
                  !durable || durable.cancelRequestedAt ||
                  durable.snapshot.state === "cancelled" ||
                  durable.ownerInstanceId !== this.#operationOwnerId ||
                  durable.ownerEpoch !== runtime.ownerEpoch
                ) throw new Error("operation ownership lost");
                runtime.revision = durable.revision;
                runtime.snapshot = durable.snapshot;
                runtime.sequence = durable.sequence;
                runtime.signalSequence = durable.signalSequence;
                runtime.signals = durable.signals;
                runtime.leaseExpiresAt = new Date(Date.now() + 30_000)
                  .toISOString();
                try {
                  await this.saveOperationRecord(runtime);
                  return;
                } catch (cause) {
                  if (retry >= 3 || !isOperationRevisionConflict(cause)) {
                    throw cause;
                  }
                }
              }
            }).catch(() => {
              runtime.cancellation.abort("operation ownership lost");
              this.#operations.delete(runtime.id);
              clearInterval(leaseHeartbeat);
            });
          }, 10_000);
        };

        const watchCancellation = (runtime: RuntimeOperationRecord) => {
          let reading = false;
          const cancellationWatch = setInterval(() => {
            if (
              reading || runtime.terminal || runtime.cancellation.signal.aborted
            ) {
              if (runtime.terminal || runtime.cancellation.signal.aborted) {
                clearInterval(cancellationWatch);
              }
              return;
            }
            reading = true;
            void this.loadOperationRecord(runtime.id).then((durable) => {
              if (
                !durable || durable.cancelRequestedAt ||
                durable.ownerInstanceId !== runtime.ownerInstanceId ||
                durable.ownerEpoch !== runtime.ownerEpoch
              ) {
                runtime.cancellation.abort(
                  "operation cancelled or ownership lost",
                );
                clearInterval(cancellationWatch);
              }
            }).catch(() => {
              runtime.cancellation.abort("operation state unavailable");
              clearInterval(cancellationWatch);
            }).finally(() => {
              reading = false;
            });
          }, 100);
        };

        const authenticate = (msg: Msg, parseInput = true) =>
          this.#authenticateOperationMessage(msg, ctx, parseInput);

        this.#log.info(
          { operation: String(operation) },
          `Mounting ${String(operation)} operation handler`,
        );

        this.#ensureOperationControlLoop(String(operation), ctx);
        const recover = async (durable: DurableOperationRecord) => {
          if (durable.operation !== String(operation)) return;
          const runtime = await this.#resolveOperation(durable.invocationId);
          if (!runtime?.reclaimed) return;
          runtime.reclaimed = false;
          if (runtime.transferGrant) {
            const committed = Reflect.get(runtime.transferGrant, "committed");
            if (Value.Check(FileInfoSchema, committed)) {
              startLeaseHeartbeat(runtime);
              watchCancellation(runtime);
              void executeHandler(
                runtime,
                runtime.caller,
                {
                  grant: runtime.transferGrant,
                  transfer: {
                    updates: async function* () {},
                    completed: () =>
                      AsyncResult.from(Promise.resolve(ok(committed))),
                  },
                },
                {},
                true,
              );
              return;
            }
            if (!ctx.transfer || !this.#transferSupport) return;
            const key = asStringPointerValue(
              String(operation),
              runtime.input,
              ctx.transfer.key,
              "key",
            ).take();
            const contentType = asOptionalStringPointerValue(
              runtime.input,
              ctx.transfer.contentType,
            ).take();
            const metadata = asOptionalStringRecordPointerValue(
              runtime.input,
              ctx.transfer.metadata,
            ).take();
            if (isErr(key) || isErr(contentType) || isErr(metadata)) return;
            const reopened = await this.#transferSupport
              .openOperationTransfer({
                sessionKey: runtime.callerSessionKey,
                permission: ctx.permissions?.invoke,
                requiredCapabilities: ctx.callerCapabilities ?? [],
                store: ctx.transfer.store,
                key,
                expiresInMs: ctx.transfer.expiresInMs ?? 60_000,
                ...(ctx.transfer.maxBytes !== undefined
                  ? { maxBytes: ctx.transfer.maxBytes }
                  : {}),
                ...(contentType !== undefined ? { contentType } : {}),
                ...(metadata !== undefined ? { metadata } : {}),
                onComplete: async (info) => {
                  await this.#queueOperationFrame(runtime, async () => {
                    if (!runtime.transferGrant) return;
                    Reflect.set(runtime.transferGrant, "committed", info);
                    await this.saveOperationRecord(runtime);
                  });
                },
              }).take();
            if (isErr(reopened)) return;
            runtime.transferGrant = reopened.grant;
            await this.saveOperationRecord(runtime);
            startLeaseHeartbeat(runtime);
            void (async () => {
              for await (const progress of reopened.transfer.updates()) {
                await this.#applyOperationUpdate(runtime.id, "running", {
                  patch: { transfer: progress },
                  event: { type: "transfer", transfer: progress },
                });
              }
            })();
            watchCancellation(runtime);
            void executeHandler(runtime, runtime.caller, reopened, {}, true);
            return;
          }
          startLeaseHeartbeat(runtime);
          watchCancellation(runtime);
          void executeHandler(runtime, runtime.caller, undefined, {}, true);
        };
        const recoverExpired = async () => {
          for (const durable of await this.listNonterminalOperationRecords()) {
            if (Date.parse(durable.leaseExpiresAt) <= Date.now()) {
              await recover(durable);
            }
          }
        };
        await recoverExpired();
        const recoveryScan = setInterval(() => {
          if (this.#nats.isClosed()) {
            clearInterval(recoveryScan);
            return;
          }
          void recoverExpired().catch((error) => {
            if (!this.#nats.isClosed()) {
              this.#log.warn(
                { error, operation: String(operation) },
                "Operation recovery scan failed",
              );
            }
          });
        }, 1_000);
        const startSub = this.#nats.subscribe(startSubject, {
          queue: routeQueueGroup(startSubject),
        });

        void (async () => {
          for await (const msg of startSub) {
            const validated = await authenticate(msg, true);
            const value = validated.take();
            if (isErr(value)) {
              recordOperationServiceError(value.error, {
                operation: String(operation),
                phase: "start",
              });
              this.respondWithError(msg, value.error);
              continue;
            }

            let transferSession: RuntimeOperationTransferSession | undefined;
            const operationId = value.invocationId!;
            const apiId = `${ctx.permissions?.invoke.apiId ?? ""}@${
              ctx.permissions?.invoke.apiVersion ?? ""
            }`;
            const invocationDigest = (await digestJson({
              apiId,
              operation: String(operation),
              creatorPrincipalId: value.caller.principalId,
              creatorParticipantId: value.caller.participantId,
              input: value.input as JsonValue,
            })).digest;
            let reclaimed: RuntimeOperationRecord | undefined;
            const existing = await this.#resolveOperation(operationId);
            if (existing) {
              if (
                existing.operation !== String(operation) ||
                existing.invocationDigest !== invocationDigest
              ) {
                this.respondWithError(
                  msg,
                  new ValidationError({
                    errors: [{
                      path: "/invocationId",
                      message:
                        "Invocation id was already accepted with different input",
                    }],
                  }),
                );
                continue;
              }
              if (existing.reclaimed && !ctx.transfer) {
                reclaimed = existing;
              } else {
                msg.respond(JSON.stringify(
                  {
                    kind: "accepted",
                    ref: {
                      id: existing.id,
                      service: this.name,
                      operation: String(operation),
                    },
                    snapshot: existing.snapshot,
                    ...(existing.transferGrant
                      ? { transfer: existing.transferGrant }
                      : {}),
                  } satisfies RuntimeOperationAcceptedEnvelope,
                ));
                continue;
              }
            }
            if (ctx.transfer) {
              if (!this.#transferSupport) {
                const error = new UnexpectedError({
                  cause: new Error(
                    `Operation '${
                      String(operation)
                    }' declared transfer support but no runtime transfer support is configured`,
                  ),
                });
                recordOperationServiceError(error, {
                  operation: String(operation),
                  phase: "start",
                });
                this.respondWithError(
                  msg,
                  error,
                );
                continue;
              }

              const key = ctx.transfer.key
                ? asStringPointerValue(
                  String(operation),
                  value.input,
                  ctx.transfer.key,
                  "key",
                ).take()
                : operationId;
              if (isErr(key)) {
                recordOperationServiceError(key.error, {
                  operation: String(operation),
                  phase: "start",
                });
                this.respondWithError(msg, key.error);
                continue;
              }

              const contentType = asOptionalStringPointerValue(
                value.input,
                ctx.transfer.contentType,
              ).take();
              if (isErr(contentType)) {
                recordOperationServiceError(contentType.error, {
                  operation: String(operation),
                  phase: "start",
                });
                this.respondWithError(msg, contentType.error);
                continue;
              }

              const metadata = asOptionalStringRecordPointerValue(
                value.input,
                ctx.transfer.metadata,
              ).take();
              if (isErr(metadata)) {
                recordOperationServiceError(metadata.error, {
                  operation: String(operation),
                  phase: "start",
                });
                this.respondWithError(msg, metadata.error);
                continue;
              }

              const openedTransferValue = await this.#transferSupport
                .openOperationTransfer({
                  sessionKey: value.sessionKey,
                  permission: ctx.permissions?.invoke,
                  requiredCapabilities: ctx.callerCapabilities ?? [],
                  store: ctx.transfer.store,
                  key,
                  expiresInMs: ctx.transfer.expiresInMs ?? 60_000,
                  ...(ctx.transfer.maxBytes !== undefined
                    ? { maxBytes: ctx.transfer.maxBytes }
                    : {}),
                  ...(contentType !== undefined ? { contentType } : {}),
                  ...(metadata !== undefined ? { metadata } : {}),
                  onComplete: async (info) => {
                    await this.#queueOperationFrame(runtime, async () => {
                      if (!runtime.transferGrant) return;
                      Reflect.set(runtime.transferGrant, "committed", info);
                      await this.saveOperationRecord(runtime);
                    });
                  },
                }).take();
              if (isErr(openedTransferValue)) {
                recordOperationServiceError(openedTransferValue.error, {
                  operation: String(operation),
                  phase: "start",
                });
                this.respondWithError(msg, openedTransferValue.error);
                continue;
              }
              transferSession = openedTransferValue;
            }

            const createdAt = now();
            const runtime: RuntimeOperationRecord = reclaimed ?? {
              id: operationId,
              service: this.name,
              operation: String(operation),
              callerSessionKey: value.sessionKey,
              invocationDigest,
              caller: value.caller,
              creatorPrincipalId: value.caller.principalId,
              creatorParticipantId: value.caller.participantId,
              apiId,
              input: value.input,
              revision: 1,
              ownerInstanceId: this.#operationOwnerId,
              ownerEpoch: 1,
              leaseExpiresAt: new Date(Date.now() + 30_000).toISOString(),
              ...(transferSession
                ? { transferGrant: transferSession.grant }
                : {}),
              snapshot: {
                id: operationId,
                service: this.name,
                operation: String(operation),
                revision: 1,
                state: "pending",
                createdAt,
                updatedAt: createdAt,
              },
              sequence: 0,
              signalSequence: 0,
              signals: [],
              terminal: false,
              watchers: new Map(),
              frameQueue: Promise.resolve(),
              signalWaiters: new Set(),
              cancellation: new AbortController(),
            };
            if (!reclaimed) {
              this.#operations.set(operationId, runtime);
              try {
                await this.saveOperationRecord(runtime);
              } catch (cause) {
                this.#operations.delete(operationId);
                const accepted = await this.#resolveOperation(operationId);
                if (!accepted) {
                  this.respondWithError(
                    msg,
                    cause instanceof Error
                      ? new ValidationError({
                        errors: [{ path: "/", message: cause.message }],
                      })
                      : new UnexpectedError({ cause }),
                  );
                  continue;
                }
                if (
                  accepted.operation !== String(operation) ||
                  accepted.invocationDigest !== invocationDigest
                ) {
                  this.respondWithError(
                    msg,
                    new ValidationError({
                      errors: [{
                        path: "/invocationId",
                        message:
                          "Invocation id was already accepted with different input",
                      }],
                    }),
                  );
                  continue;
                }
                msg.respond(JSON.stringify(
                  {
                    kind: "accepted",
                    ref: {
                      id: accepted.id,
                      service: this.name,
                      operation: String(operation),
                    },
                    snapshot: accepted.snapshot,
                    ...(accepted.transferGrant
                      ? { transfer: accepted.transferGrant }
                      : {}),
                  } satisfies RuntimeOperationAcceptedEnvelope,
                ));
                continue;
              }
            }

            if (transferSession) {
              void (async () => {
                for await (
                  const progress of transferSession.transfer.updates()
                ) {
                  await this.#applyOperationUpdate(runtime.id, "running", {
                    patch: { transfer: progress },
                    event: { type: "transfer", transfer: progress },
                  });
                }
              })();
            }

            startLeaseHeartbeat(runtime);
            watchCancellation(runtime);

            const accepted: RuntimeOperationAcceptedEnvelope = {
              kind: "accepted",
              ref: {
                id: operationId,
                service: this.name,
                operation: String(operation),
              },
              snapshot: runtime.snapshot,
              ...(transferSession ? { transfer: transferSession.grant } : {}),
            };
            msg.respond(JSON.stringify(accepted));

            void executeHandler(runtime, value.caller, transferSession, {
              requestId: msg.headers?.get("request-id"),
              traceId: traceIdFromTraceparent(msg.headers?.get("traceparent")),
            });
          }
        })();

        return Promise.resolve();
      },
    };
  }

  async stop(): Promise<void> {
    this.#stopPromise ??= (async () => {
      if (this.#nats.isClosed()) {
        return;
      }

      try {
        await this.#nats.drain();
      } catch (cause) {
        if (
          !(cause instanceof Error) ||
          cause.name !== "DrainingConnectionError"
        ) {
          throw cause;
        }

        await this.#nats.closed().catch(() => undefined);
      }
    })();

    await this.#stopPromise;
  }
}
