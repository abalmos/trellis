import type { Msg, NatsConnection } from "@nats-io/nats-core";
import { Pointer } from "typebox/value";
import type {
  PermissionAtomV1,
  RuntimeApi,
} from "../../contract_support/runtime.ts";
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
import type { SendTransferGrant } from "../../transfer.ts";

type TrellisServiceRuntimeOpts<TA extends RuntimeApi> =
  & Omit<TrellisOpts<TA>, "api">
  & {
    api: TA;
    transferSupport?: RuntimeOperationTransferSupport;
    version?: string;
    operationStoreId?: string;
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
    permission: PermissionAtomV1 | undefined;
    requiredCapabilities: readonly string[];
    store: string;
    key: string;
    expiresInMs: number;
    maxBytes?: number;
    contentType?: string;
    metadata?: Record<string, string>;
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

export class TrellisServiceRuntime extends Trellis<RuntimeApi, TrellisMode> {
  #nats: NatsConnection;
  #version?: string;
  #log: LoggerLike;
  #operations = new Map<string, RuntimeOperationRecord>();
  #mountedOperationControls = new Set<string>();
  #stopPromise?: Promise<void>;
  #transferSupport?: RuntimeOperationTransferSupport;
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
    if (opts?.operationStoreId) this.setOperationStoreId(opts.operationStoreId);
    this.#nats = nats;
    this.#version = opts?.version;
    this.#log = (opts?.log ?? serviceRuntimeLogger).child({
      lib: "trellis-service-runtime",
    });
    this.#transferSupport = opts?.transferSupport;
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
      const runtime = await this.#resolveOperation(operationId);
      if (!runtime) {
        return err(this.#operationNotFoundError(operationId));
      }

      const queued = runtime.signals.find((signal) =>
        signal.sequence > afterSequence
      );
      if (queued) return ok(queued);
      if (runtime.terminal) {
        return err(this.#operationAlreadyTerminalError(runtime));
      }

      return await new Promise<Result<RuntimeOperationSignal, BaseError>>(
        (resolve) => {
          const waiter = (
            result: Result<RuntimeOperationSignal, BaseError>,
          ) => {
            const value = result.take();
            if (!isErr(value) && value.sequence <= afterSequence) {
              return;
            }
            runtime.signalWaiters.delete(waiter);
            resolve(result);
          };
          runtime.signalWaiters.add(waiter);
        },
      );
    })());
  }

  async #resolveOperation(
    operationId: string,
  ): Promise<RuntimeOperationRecord | null> {
    const existing = this.#operations.get(operationId);
    if (existing) return existing;

    const durable = await this.loadOperationRecord(operationId);
    if (!durable) return null;

    const runtime: RuntimeOperationRecord = {
      id: durable.snapshot.id,
      service: durable.snapshot.service,
      operation: durable.snapshot.operation,
      ownerSessionKey: durable.ownerSessionKey,
      snapshot: durable.snapshot,
      sequence: durable.sequence,
      terminal: durable.snapshot.state === "completed" ||
        durable.snapshot.state === "failed" ||
        durable.snapshot.state === "cancelled",
      signalSequence: durable.signalSequence ?? 0,
      signals: durable.signals ?? [],
      watchers: new Map(),
      frameQueue: Promise.resolve(),
      waiters: new Set(),
      signalWaiters: new Set(),
    };
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

        runtime.sequence += 1;
        runtime.snapshot = buildRuntimeOperationSnapshot(
          runtime,
          state,
          opts.patch,
        );
        runtime.terminal = state === "completed" || state === "failed" ||
          state === "cancelled";

        await this.saveOperationRecord(runtime);

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
          const terminalFrame = {
            kind: "snapshot",
            snapshot: runtime.snapshot,
          };
          for (const reply of runtime.waiters) {
            await this.#nats.publish(reply, JSON.stringify(terminalFrame));
          }
          runtime.waiters.clear();
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
      if (runtime.terminal) {
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
      cancel: () => {
        if (ctx.cancel !== true) {
          return AsyncResult.err(
            this.#unsupportedCancelError(runtime.operation),
          );
        }
        return this.#applyControlledOperationUpdate(runtime, ctx, "cancelled", {
          event: { type: "cancelled" },
        });
      },
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
      cancel: () => {
        if (ctx.cancel !== true) {
          return AsyncResult.err(
            this.#unsupportedCancelError(runtime.operation),
          );
        }
        return this.operations.cancel(runtime.id);
      },
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
      defer: () => ({ kind: "deferred" as const }),
    };
  }

  #operationAuthRequirements(
    ctx: RegisteredRuntimeOperationDesc,
    control: RuntimeOperationControlRequest,
  ): {
    permission: PermissionAtomV1 | undefined;
    capabilities: readonly string[];
  } {
    switch (control.action) {
      case "signal":
        return {
          permission: ctx.signals?.[control.signal]
            ? ctx.permissions?.control[control.signal]
            : undefined,
          capabilities: ctx.controlCapabilities ?? [],
        };
      case "cancel":
        return {
          permission: ctx.permissions?.cancel,
          capabilities: ctx.cancelCapabilities ?? [],
        };
      case "get":
      case "wait":
      case "watch":
        return {
          permission: ctx.permissions?.observe,
          capabilities: ctx.observeCapabilities ?? [],
        };
      default:
        return { permission: undefined, capabilities: [] };
    }
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

  #unsupportedCancelError(operation: string): ValidationError {
    return new ValidationError({
      errors: [{
        path: "/action",
        message: `Operation '${operation}' does not support cancel`,
      }],
      context: { operation, action: "cancel" },
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

    runtime.signalSequence += 1;
    const acceptedAt = new Date().toISOString();
    const signal: RuntimeOperationSignal = {
      operationId: runtime.id,
      sequence: runtime.signalSequence,
      signal: control.signal,
      ...(control.input !== undefined ? { input: control.input } : {}),
      acceptedAt,
    };
    runtime.signals.push(signal);
    await this.saveOperationRecord(runtime);
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

  async #acceptOperation(
    operation: string,
    ctx: RegisteredRuntimeOperationDesc,
    sessionKey: string,
  ): Promise<
    Result<AcceptedOperation<unknown, unknown, BaseError>, UnexpectedError>
  > {
    const createdAt = new Date().toISOString();
    const operationId = ulid();
    const runtime: RuntimeOperationRecord = {
      id: operationId,
      service: this.name,
      operation,
      ownerSessionKey: sessionKey,
      snapshot: {
        id: operationId,
        service: this.name,
        operation,
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
      waiters: new Set(),
      signalWaiters: new Set(),
    };
    this.#operations.set(operationId, runtime);
    await this.saveOperationRecord(runtime);
    return ok(this.#makeAcceptedOperation(runtime, ctx));
  }

  async #authenticateOperationMessage(
    msg: Msg,
    ctx: RegisteredRuntimeOperationDesc,
    parseInput: boolean,
    permission: PermissionAtomV1 | undefined = ctx.permissions?.invoke,
    requiredCapabilities: readonly string[] = ctx.callerCapabilities ?? [],
  ): Promise<
    Result<{
      input: unknown;
      caller: VerifiedCaller;
      sessionKey: string;
    }, UnexpectedError | AuthError | ValidationError>
  > {
    const jsonData = safeJson(msg).take();
    if (isErr(jsonData)) return jsonData;

    let parsedInput: unknown;
    if (parseInput) {
      const parsedInputResult = parseSchema(
        ctx.input as Parameters<typeof parseSchema>[0],
        jsonData,
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

    const controlSub = this.#nats.subscribe(controlSubject);
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
          typeof (request as RuntimeOperationControlRequest).action !==
            "string" ||
          typeof (request as RuntimeOperationControlRequest).operationId !==
            "string"
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
        const requirements = this.#operationAuthRequirements(ctx, control);
        const validated = await this.#authenticateOperationMessage(
          msg,
          ctx,
          false,
          requirements.permission,
          requirements.capabilities,
        );
        const value = validated.take();
        if (isErr(value)) {
          respondControlError(msg, value.error);
          continue;
        }

        const runtime = await this.#resolveOperation(control.operationId);
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
        const ownerSessionKey = runtime.ownerSessionKey;

        if (ownerSessionKey !== value.sessionKey) {
          respondControlError(
            msg,
            new AuthError({
              reason: "forbidden",
              context: { ownerSessionKey },
            }),
          );
          continue;
        }

        if (control.action === "watch") {
          if (msg.reply) {
            await this.#queueOperationFrame(runtime, async () => {
              if (!runtime.terminal) {
                runtime.watchers.set(msg.reply!, {
                  includeUpdates: control.includeUpdates === true,
                });
              }
              await publishSnapshot(msg.reply!, runtime.snapshot);
            });
          }
          continue;
        }

        if (control.action === "wait") {
          if (
            snapshot.state === "completed" || snapshot.state === "failed" ||
            snapshot.state === "cancelled"
          ) {
            msg.respond(JSON.stringify({ kind: "snapshot", snapshot }));
          } else if (runtime && msg.reply) {
            runtime.waiters.add(msg.reply);
          } else if (msg.reply) {
            respondControlError(
              msg,
              new UnexpectedError({
                cause: new Error("operation is not running in this process"),
              }),
            );
          } else {
            respondControlError(
              msg,
              new UnexpectedError({
                cause: new Error("missing reply subject for wait request"),
              }),
            );
          }
          continue;
        }

        if (control.action === "get") {
          msg.respond(JSON.stringify({ kind: "snapshot", snapshot }));
          continue;
        }

        if (control.action === "cancel") {
          if (ctx.cancel !== true) {
            respondControlError(msg, this.#unsupportedCancelError(operation));
            continue;
          }
          if (runtime.terminal) {
            respondControlError(
              msg,
              this.#operationAlreadyTerminalError(runtime),
            );
            continue;
          }
          const cancelled = await this.#applyControlledOperationUpdate(
            runtime,
            ctx,
            "cancelled",
            { event: { type: "cancelled" } },
          ).take();
          if (isErr(cancelled)) {
            respondControlError(msg, cancelled.error);
          } else {
            msg.respond(
              JSON.stringify({ kind: "snapshot", snapshot: cancelled }),
            );
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

          const accepted = await this.#acceptSignal(runtime, ctx, control);
          const acceptedValue = accepted.take();
          if (isErr(acceptedValue)) {
            respondControlError(msg, acceptedValue.error);
            continue;
          }
          msg.respond(JSON.stringify(acceptedValue));
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
      accept: ({ sessionKey }) => {
        this.#ensureOperationControlLoop(String(operation), ctx);
        if (ctx.transfer) {
          return AsyncResult.err(
            new UnexpectedError({
              cause: new Error(
                `Operation '${
                  String(operation)
                }' uses transfer-capable start semantics and cannot be accepted manually`,
              ),
            }),
          );
        }
        return AsyncResult.from(
          this.#acceptOperation(String(operation), ctx, sessionKey),
        );
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
            defer: () => ({ kind: "deferred" as const }),
          };
        };

        const authenticate = (msg: Msg, parseInput = true) =>
          this.#authenticateOperationMessage(msg, ctx, parseInput);

        this.#log.info(
          { operation: String(operation) },
          `Mounting ${String(operation)} operation handler`,
        );

        this.#ensureOperationControlLoop(String(operation), ctx);
        const startSub = this.#nats.subscribe(startSubject);

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

              const key = asStringPointerValue(
                String(operation),
                value.input,
                ctx.transfer.key,
                "key",
              ).take();
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

            const operationId = ulid();
            const createdAt = now();
            const runtime: RuntimeOperationRecord = {
              id: operationId,
              service: this.name,
              operation: String(operation),
              ownerSessionKey: value.sessionKey,
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
              waiters: new Set(),
              signalWaiters: new Set(),
            };
            this.#operations.set(operationId, runtime);
            await this.saveOperationRecord(runtime);

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

            void (async () => {
              const operationContext = {
                requestId: msg.headers?.get("request-id"),
                traceId: traceIdFromTraceparent(
                  msg.headers?.get("traceparent"),
                ),
              };
              const op = makeOperation(runtime, operationContext);
              try {
                const handlerResult: unknown = await handler(
                  transferSession
                    ? {
                      input: value.input,
                      op,
                      caller: value.caller,
                      transfer: transferSession.transfer,
                    }
                    : {
                      input: value.input,
                      op,
                      caller: value.caller,
                    },
                );
                const handlerOutcome = isResultLike(handlerResult)
                  ? handlerResult.take()
                  : handlerResult;
                if (isErr(handlerOutcome)) {
                  const error = annotateHandlerBoundaryError(
                    handlerOutcome.error,
                    {
                      operation: String(operation),
                      requestId: operationContext.requestId,
                      service: this.name,
                      contractId: this.contractId,
                      contractDigest: this.contractDigest,
                      traceId: operationContext.traceId,
                    },
                  );
                  recordOperationServiceError(error, {
                    operation: String(operation),
                    phase: "handler_result",
                  });
                  await op.fail(error);
                  return;
                }

                if (isOperationDeferred(handlerOutcome)) {
                  return;
                }

                if (isTerminalRuntimeOperationSnapshot(handlerOutcome)) {
                  runtime.sequence = handlerOutcome.revision;
                  runtime.snapshot = handlerOutcome;
                  runtime.terminal = true;
                  await this.saveOperationRecord(runtime);
                  return;
                }

                if (!runtime.terminal) {
                  await op.complete(handlerOutcome);
                }
              } catch (cause) {
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
                await op.fail(error);
              }
            })();
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
