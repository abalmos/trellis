import {
  AsyncResult,
  isErr,
  Result,
  type Result as ResultType,
} from "@qlever-llc/result";
import {
  headers as natsHeaders,
  type Msg,
  type NatsConnection,
  type Subscription,
} from "@nats-io/nats-core";
import { ulid } from "ulid";
import { sha256 } from "@noble/hashes/sha256";
import { base64urlEncode } from "../../auth/utils.ts";

import type { PermissionAtom } from "../../contract_support/runtime.ts";
import {
  type OperationTransferHandle,
  type RuntimeOperationTransferProgress,
  type TrellisAuth,
  verifyLocalAuthorization,
} from "../../session.ts";
import type { StoreError } from "../../errors/StoreError.ts";
import { TransferError } from "../../errors/TransferError.ts";
import { type StoreInfo, TypedStore, TypedStoreEntry } from "../../store.ts";
import type {
  FileInfo,
  ReceiveTransferGrant,
  SendTransferGrant,
} from "../../transfer.ts";
import { transferFrameProofPayload } from "../../transfer_protocol.ts";
import { MAX_TRANSFER_CHUNK_BYTES } from "../../transfer.ts";

const UPLOAD_SUBJECT_PREFIX = "transfer.v2.upload";
const DOWNLOAD_SUBJECT_PREFIX = "transfer.v2.download";
const TRANSFER_SEQUENCE_HEADER = "trellis-transfer-seq";
const TRANSFER_EOF_HEADER = "trellis-transfer-eof";
const TRANSFER_CONTROL_HEADER = "trellis-transfer-control";
const DEFAULT_TRANSFER_CHUNK_BYTES = 256 * 1024;

export type TransferStoreHandle = {
  open(): AsyncResult<TypedStore, StoreError>;
};

export type InitiateUploadArgs = {
  sessionKey: string;
  permission: PermissionAtom | undefined;
  requiredCapabilities: readonly string[];
  store: string;
  key: string;
  expiresInMs: number;
  maxBytes?: number;
  contentType?: string;
  metadata?: Record<string, string>;
  onProgress?: (
    progress: RuntimeOperationTransferProgress,
  ) => Promise<void> | void;
  onComplete?: (info: FileInfo) => Promise<void> | void;
  onError?: (error: TransferError) => Promise<void> | void;
  onStored?: (stored: StoredTransfer) => Promise<void> | void;
};

export type OperationUploadTransfer = {
  grant: SendTransferGrant;
  transfer: OperationTransferHandle;
};

export type InitiateDownloadArgs = {
  sessionKey: string;
  permission: PermissionAtom;
  requiredCapabilities?: readonly string[];
  inboxPrefix: string;
  store: string;
  key: string;
  expiresInMs: number;
};

type ServiceTransferOpts = {
  name: string;
  nc: NatsConnection;
  auth: TrellisAuth;
  stores: Record<string, TransferStoreHandle>;
  chunkBytes?: number;
};

export type StoredTransfer = {
  transferId: string;
  sessionKey: string;
  store: TypedStore;
  entry: TypedStoreEntry;
  info: FileInfo;
};

type UploadSession = {
  kind: "upload";
  subject: string;
  transferId: string;
  sessionKey: string;
  permission: PermissionAtom | undefined;
  requiredCapabilities: readonly string[];
  expiresAtMs: number;
  store: TypedStore;
  key: string;
  maxBytes?: number;
  contentType?: string;
  metadata?: Record<string, string>;
  onProgress?: (
    progress: RuntimeOperationTransferProgress,
  ) => Promise<void> | void;
  onComplete?: (info: FileInfo) => Promise<void> | void;
  onError?: (error: TransferError) => Promise<void> | void;
  onStored?: (stored: StoredTransfer) => Promise<void> | void;
  subscription: Subscription;
  timeoutId: ReturnType<typeof setTimeout>;
  queue: AsyncChunkQueue;
  putPromise: AsyncResult<void, StoreError>;
  cancellation: AbortController;
  committing: boolean;
  nextSeq: number;
  receivedBytes: number;
  hasher: ReturnType<typeof sha256.create>;
};

function raceCancellation<T>(
  promise: Promise<T>,
  signal: AbortSignal,
): Promise<T> {
  if (signal.aborted) return Promise.reject(signal.reason);
  return new Promise<T>((resolve, reject) => {
    const cancel = () => {
      finish();
      reject(signal.reason);
    };
    const finish = () => signal.removeEventListener("abort", cancel);
    signal.addEventListener("abort", cancel, { once: true });
    promise.then(
      (value) => {
        finish();
        resolve(value);
      },
      (error) => {
        finish();
        reject(error);
      },
    );
  });
}

type DownloadSession = {
  kind: "download";
  subject: string;
  transferId: string;
  sessionKey: string;
  permission: PermissionAtom;
  requiredCapabilities: readonly string[];
  inboxPrefix: string;
  expiresAtMs: number;
  store: TypedStore;
  key: string;
  info: FileInfo;
  subscription: Subscription;
  timeoutId: ReturnType<typeof setTimeout>;
  reader?: ReadableStreamDefaultReader<Uint8Array>;
  pending?: Uint8Array;
  pendingOffset: number;
  nextSeq: number;
  sentBytes: number;
  hasher: ReturnType<typeof sha256.create>;
  cancellation: AbortController;
};

function effectiveUploadMaxBytes(
  argsMaxBytes?: number,
  storeMaxObjectBytes?: number,
): number | undefined {
  if (argsMaxBytes === undefined) {
    return storeMaxObjectBytes;
  }
  if (storeMaxObjectBytes === undefined) {
    return argsMaxBytes;
  }
  return Math.min(argsMaxBytes, storeMaxObjectBytes);
}

function deferred<T>(): { promise: Promise<T>; resolve(value: T): void } {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

class AsyncValueQueue<T> implements AsyncIterable<T> {
  #values: T[] = [];
  #resolvers: Array<(result: IteratorResult<T>) => void> = [];
  #closed = false;

  push(value: T): void {
    if (this.#closed) {
      return;
    }

    const resolver = this.#resolvers.shift();
    if (resolver) {
      resolver({ value, done: false });
      return;
    }

    this.#values.push(value);
  }

  close(): void {
    if (this.#closed) {
      return;
    }
    this.#closed = true;
    while (this.#resolvers.length > 0) {
      const resolver = this.#resolvers.shift();
      resolver?.({ value: undefined as T, done: true });
    }
  }

  [Symbol.asyncIterator](): AsyncIterator<T> {
    return {
      next: async (): Promise<IteratorResult<T>> => {
        const value = this.#values.shift();
        if (value !== undefined) {
          return { value, done: false };
        }
        if (this.#closed) {
          return { value: undefined as T, done: true };
        }
        return await new Promise<IteratorResult<T>>((resolve) => {
          this.#resolvers.push(resolve);
        });
      },
    };
  }
}

class AsyncValueBroadcaster<T> {
  #subscribers = new Set<AsyncValueQueue<T>>();
  #closed = false;

  push(value: T): void {
    if (this.#closed) {
      return;
    }

    for (const subscriber of this.#subscribers) {
      subscriber.push(value);
    }
  }

  close(): void {
    if (this.#closed) {
      return;
    }

    this.#closed = true;
    for (const subscriber of this.#subscribers) {
      subscriber.close();
    }
    this.#subscribers.clear();
  }

  subscribe(): AsyncIterable<T> {
    const subscriber = new AsyncValueQueue<T>();
    if (this.#closed) {
      subscriber.close();
    } else {
      this.#subscribers.add(subscriber);
    }

    const subscribers = this.#subscribers;

    return {
      async *[Symbol.asyncIterator]() {
        try {
          for await (const value of subscriber) {
            yield value;
          }
        } finally {
          subscribers.delete(subscriber);
        }
      },
    };
  }
}

class AsyncChunkQueue implements AsyncIterable<Uint8Array> {
  #values: Array<{ chunk: Uint8Array; consumed: () => void }> = [];
  #resolvers: Array<(result: IteratorResult<Uint8Array>) => void> = [];
  #closed = false;
  #error: unknown;

  async push(chunk: Uint8Array): Promise<void> {
    if (this.#closed) {
      return;
    }

    const resolver = this.#resolvers.shift();
    if (resolver) {
      resolver({ value: chunk, done: false });
      return;
    }

    await new Promise<void>((consumed) => {
      this.#values.push({ chunk, consumed });
    });
  }

  close(): void {
    if (this.#closed) {
      return;
    }
    this.#closed = true;
    while (this.#resolvers.length > 0) {
      this.#resolvers.shift()?.({ value: undefined, done: true });
    }
  }

  fail(error: unknown): void {
    if (this.#closed) {
      return;
    }
    this.#error = error;
    this.#closed = true;
    for (const value of this.#values.splice(0)) {
      value.consumed();
    }
    while (this.#resolvers.length > 0) {
      this.#resolvers.shift()?.({ value: undefined, done: true });
    }
  }

  async next(): Promise<IteratorResult<Uint8Array>> {
    if (this.#values.length > 0) {
      const value = this.#values.shift()!;
      value.consumed();
      return { value: value.chunk, done: false };
    }
    if (this.#error) {
      throw this.#error;
    }
    if (this.#closed) {
      return { value: undefined, done: true };
    }

    return await new Promise<IteratorResult<Uint8Array>>((resolve) => {
      this.#resolvers.push(resolve);
    });
  }

  [Symbol.asyncIterator](): AsyncIterator<Uint8Array> {
    return {
      next: () => this.next(),
    };
  }
}

function fileInfoFromStoreInfo(info: StoreInfo): FileInfo {
  return {
    key: info.key,
    size: info.size,
    updatedAt: info.updatedAt,
    ...(info.digest ? { digest: info.digest } : {}),
    ...(info.contentType ? { contentType: info.contentType } : {}),
    metadata: info.metadata,
  };
}

function replyError(msg: Msg, error: TransferError): void {
  const headers = natsHeaders();
  headers.set("status", "error");
  msg.respond(JSON.stringify(error.toSerializable()), { headers });
}

function publishError(
  nc: NatsConnection,
  subject: string,
  error: TransferError,
): void {
  const headers = natsHeaders();
  headers.set("status", "error");
  nc.publish(subject, JSON.stringify(error.toSerializable()), { headers });
}

function parseSeq(msg: Msg): ResultType<number, TransferError> {
  const raw = msg.headers?.get(TRANSFER_SEQUENCE_HEADER);
  if (!raw) {
    return Result.err(
      new TransferError({
        operation: "transfer",
        context: { reason: "missing_sequence" },
      }),
    );
  }
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 0) {
    return Result.err(
      new TransferError({
        operation: "transfer",
        context: { reason: "invalid_sequence", raw },
      }),
    );
  }
  return Result.ok(value);
}

export class ServiceTransfer {
  readonly #name: string;
  readonly #nc: NatsConnection;
  readonly #auth: TrellisAuth;
  readonly #stores: Record<string, TransferStoreHandle>;
  readonly #chunkBytes: number;
  readonly #uploadSessions = new Map<string, UploadSession>();
  readonly #downloadSessions = new Map<string, DownloadSession>();

  constructor(opts: ServiceTransferOpts) {
    this.#name = opts.name;
    this.#nc = opts.nc;
    this.#auth = opts.auth;
    this.#stores = opts.stores;
    this.#chunkBytes = opts.chunkBytes ?? DEFAULT_TRANSFER_CHUNK_BYTES;
    if (
      !Number.isSafeInteger(this.#chunkBytes) || this.#chunkBytes < 1 ||
      this.#chunkBytes > MAX_TRANSFER_CHUNK_BYTES
    ) {
      throw new RangeError(
        `transfer chunk size must be between 1 and ${MAX_TRANSFER_CHUNK_BYTES} bytes`,
      );
    }
  }

  async initiateUpload(
    args: InitiateUploadArgs,
  ): Promise<ResultType<SendTransferGrant, TransferError>> {
    const store = await this.#openStore(args.store, "initiateUpload");
    const storeValue = store.take();
    if (isErr(storeValue)) {
      return Result.err(storeValue.error);
    }

    const storeStatus = await storeValue.status();
    const storeStatusValue = storeStatus.take();
    if (isErr(storeStatusValue)) {
      return Result.err(
        new TransferError({
          operation: "initiateUpload",
          cause: storeStatusValue.error,
        }),
      );
    }

    const maxBytes = effectiveUploadMaxBytes(
      args.maxBytes,
      storeStatusValue.maxObjectBytes,
    );

    const transferId = ulid();
    const subject = `${UPLOAD_SUBJECT_PREFIX}.${
      this.#auth.sessionKey.slice(0, 16)
    }.${transferId}`;
    const expiresAtMs = Date.now() + args.expiresInMs;
    const queue = new AsyncChunkQueue();
    const subscription = this.#nc.subscribe(subject);
    const putPromise = storeValue.put(args.key, queue, {
      ...(args.contentType ? { contentType: args.contentType } : {}),
      ...(args.metadata ? { metadata: args.metadata } : {}),
    });

    const session: UploadSession = {
      kind: "upload",
      subject,
      transferId,
      sessionKey: args.sessionKey,
      permission: args.permission,
      requiredCapabilities: args.requiredCapabilities,
      expiresAtMs,
      store: storeValue,
      key: args.key,
      ...(maxBytes !== undefined ? { maxBytes } : {}),
      ...(args.contentType ? { contentType: args.contentType } : {}),
      ...(args.metadata ? { metadata: args.metadata } : {}),
      ...(args.onProgress ? { onProgress: args.onProgress } : {}),
      ...(args.onComplete ? { onComplete: args.onComplete } : {}),
      ...(args.onError ? { onError: args.onError } : {}),
      ...(args.onStored ? { onStored: args.onStored } : {}),
      subscription,
      timeoutId: setTimeout(
        () => this.#expireUploadSession(subject),
        args.expiresInMs,
      ),
      queue,
      putPromise,
      cancellation: new AbortController(),
      committing: false,
      nextSeq: 0,
      receivedBytes: 0,
      hasher: sha256.create(),
    };

    this.#uploadSessions.set(subject, session);
    this.#runUploadSession(session);

    if (!this.#uploadSessions.has(subject)) {
      return Result.err(
        new TransferError({
          operation: "initiateUpload",
          context: { reason: "session_closed" },
        }),
      );
    }
    if (Date.now() >= expiresAtMs) {
      const error = new TransferError({
        operation: "initiateUpload",
        context: { reason: "expired" },
      });
      this.#expireUploadSession(subject, error);
      return Result.err(error);
    }

    return Result.ok({
      type: "TransferGrant",
      direction: "send",
      service: this.#name,
      sessionKey: args.sessionKey,
      transferId,
      subject,
      expiresAt: new Date(expiresAtMs).toISOString(),
      chunkBytes: this.#chunkBytes,
      ...(maxBytes !== undefined ? { maxBytes } : {}),
      ...(args.contentType ? { contentType: args.contentType } : {}),
      ...(args.metadata ? { metadata: args.metadata } : {}),
    });
  }

  createOperationUpload(
    args: InitiateUploadArgs,
  ): AsyncResult<OperationUploadTransfer, TransferError> {
    return AsyncResult.from(
      (async (): Promise<
        ResultType<OperationUploadTransfer, TransferError>
      > => {
        const updates = new AsyncValueBroadcaster<
          RuntimeOperationTransferProgress
        >();
        const completed = deferred<ResultType<FileInfo, TransferError>>();
        let settled = false;
        const settle = (value: ResultType<FileInfo, TransferError>) => {
          if (settled) {
            return;
          }
          settled = true;
          updates.close();
          completed.resolve(value);
        };

        const grant = await this.initiateUpload({
          ...args,
          onProgress: async (progress) => {
            updates.push(progress);
            await args.onProgress?.(progress);
          },
          onComplete: async (info) => {
            await args.onComplete?.(info);
            settle(Result.ok(info));
          },
          onError: async (error) => {
            await args.onError?.(error);
            settle(Result.err(error));
          },
          onStored: async (stored) => {
            await args.onStored?.(stored);
          },
        });
        const grantValue = grant.take();
        if (isErr(grantValue)) {
          return Result.err(grantValue.error);
        }

        return Result.ok({
          grant: grantValue,
          transfer: {
            updates: () => updates.subscribe(),
            completed: () => AsyncResult.from(completed.promise),
          },
        });
      })(),
    );
  }

  async initiateDownload(
    args: InitiateDownloadArgs,
  ): Promise<ResultType<ReceiveTransferGrant, TransferError>> {
    const store = await this.#openStore(args.store, "initiateDownload");
    const storeValue = store.take();
    if (isErr(storeValue)) {
      return Result.err(storeValue.error);
    }

    const entry = await storeValue.get(args.key);
    const entryValue = entry.take();
    if (isErr(entryValue)) {
      return Result.err(
        new TransferError({
          operation: "initiateDownload",
          cause: entryValue.error,
        }),
      );
    }
    const info = fileInfoFromStoreInfo(entryValue.info);
    if (info.digest === undefined) {
      return Result.err(
        new TransferError({
          operation: "initiateDownload",
          context: { reason: "missing_digest", key: args.key },
        }),
      );
    }
    const downloadInfo = { ...info, digest: info.digest };

    const transferId = ulid();
    const subject = `${DOWNLOAD_SUBJECT_PREFIX}.${
      this.#auth.sessionKey.slice(0, 16)
    }.${transferId}`;
    const expiresAtMs = Date.now() + args.expiresInMs;
    const subscription = this.#nc.subscribe(subject);
    const session: DownloadSession = {
      kind: "download",
      subject,
      transferId,
      sessionKey: args.sessionKey,
      permission: args.permission,
      requiredCapabilities: [...(args.requiredCapabilities ?? [])],
      inboxPrefix: args.inboxPrefix,
      expiresAtMs,
      store: storeValue,
      key: args.key,
      info: downloadInfo,
      subscription,
      pendingOffset: 0,
      nextSeq: 0,
      sentBytes: 0,
      hasher: sha256.create(),
      cancellation: new AbortController(),
      timeoutId: setTimeout(
        () => this.#cleanupDownloadSession(subject),
        args.expiresInMs,
      ),
    };

    this.#downloadSessions.set(subject, session);
    this.#runDownloadSession(session);

    if (!this.#downloadSessions.has(subject)) {
      return Result.err(
        new TransferError({
          operation: "initiateDownload",
          context: { reason: "session_closed" },
        }),
      );
    }
    if (Date.now() >= expiresAtMs) {
      this.#cleanupDownloadSession(subject);
      return Result.err(
        new TransferError({
          operation: "initiateDownload",
          context: { reason: "expired" },
        }),
      );
    }

    return Result.ok({
      type: "TransferGrant",
      direction: "receive",
      service: this.#name,
      sessionKey: args.sessionKey,
      transferId,
      subject,
      expiresAt: new Date(expiresAtMs).toISOString(),
      chunkBytes: this.#chunkBytes,
      info: downloadInfo,
    });
  }

  async stop(): Promise<void> {
    for (const subject of [...this.#uploadSessions.keys()]) {
      this.#expireUploadSession(subject);
    }
    for (const subject of [...this.#downloadSessions.keys()]) {
      this.#cleanupDownloadSession(subject);
    }
  }

  async #openStore(
    alias: string,
    operation: string,
  ): Promise<ResultType<TypedStore, TransferError>> {
    const handle = this.#stores[alias];
    if (!handle) {
      return Result.err(
        new TransferError({
          operation,
          context: { reason: "unknown_store", store: alias },
        }),
      );
    }

    const store = await handle.open();
    const value = store.take();
    if (isErr(value)) {
      return Result.err(
        new TransferError({
          operation,
          cause: value.error,
          context: { store: alias },
        }),
      );
    }
    return Result.ok(value);
  }

  async #runUploadSession(session: UploadSession): Promise<void> {
    let pending: Promise<void> | undefined;
    try {
      for await (const msg of session.subscription) {
        const cancellation = msg.headers?.get(TRANSFER_CONTROL_HEADER) ===
          "cancel";
        if (pending && !cancellation) {
          replyError(
            msg,
            new TransferError({
              operation: "put",
              context: { reason: "pending_frame" },
            }),
          );
          continue;
        }
        const handling = this.#handleUploadMessage(session, msg);
        if (pending) {
          await handling;
          await pending.catch(() => undefined);
          break;
        }
        let tracked: Promise<void>;
        tracked = handling.catch((cause) => {
          const error = cause instanceof TransferError
            ? cause
            : new TransferError({ operation: "put", cause });
          replyError(msg, error);
          this.#expireUploadSession(session.subject, error);
        }).finally(() => {
          if (pending === tracked) pending = undefined;
        });
        pending = tracked;
      }
    } finally {
      await pending?.catch(() => undefined);
      this.#cleanupUploadSession(session.subject);
    }
  }

  async #handleUploadMessage(session: UploadSession, msg: Msg): Promise<void> {
    if (Date.now() >= session.expiresAtMs) {
      const error = new TransferError({
        operation: "put",
        context: { reason: "expired" },
      });
      replyError(msg, error);
      this.#expireUploadSession(session.subject, error);
      return;
    }

    const seqResult = parseSeq(msg).take();
    if (isErr(seqResult)) {
      replyError(msg, seqResult.error);
      this.#expireUploadSession(session.subject, seqResult.error);
      return;
    }
    const control = msg.headers?.get(TRANSFER_CONTROL_HEADER) || undefined;
    const authenticated = await verifyLocalAuthorization({
      kind: "request",
      cache: this.#auth.authorizationProviderCache,
      message: msg,
      permission: session.permission,
      requiredCapabilities: session.requiredCapabilities,
      proofPayload: transferFrameProofPayload(seqResult, control, msg.data),
    });
    const authenticatedValue = authenticated.take();
    if (
      isErr(authenticatedValue) ||
      authenticatedValue.sessionKey !== session.sessionKey
    ) {
      const error = new TransferError({
        operation: "put",
        context: { reason: "invalid_proof" },
      });
      replyError(msg, error);
      this.#expireUploadSession(session.subject, error);
      return;
    }

    if (msg.headers?.get(TRANSFER_EOF_HEADER) === "true") {
      const error = new TransferError({
        operation: "put",
        context: { reason: "legacy_eof_header" },
      });
      replyError(msg, error);
      this.#expireUploadSession(session.subject, error);
      return;
    }
    if (control === "cancel") {
      try {
        if (
          (JSON.parse(new TextDecoder().decode(msg.data)) as {
            action?: string;
          })
            .action !== "cancel"
        ) {
          throw new Error(
            "cancellation payload does not match its control header",
          );
        }
        if (session.committing) {
          replyError(
            msg,
            new TransferError({
              operation: "put",
              context: { reason: "cancel_after_validated_eof" },
            }),
          );
          return;
        }
        this.#expireUploadSession(
          session.subject,
          new TransferError({
            operation: "put",
            context: { reason: "cancelled" },
          }),
        );
        msg.respond(JSON.stringify({ status: "cancelled" }));
        return;
      } catch (cause) {
        const error = new TransferError({
          operation: "put",
          cause,
          context: { reason: "invalid_control" },
        });
        replyError(msg, error);
        this.#expireUploadSession(session.subject, error);
        return;
      }
    }
    if (control !== undefined && control !== "complete") {
      const error = new TransferError({
        operation: "put",
        context: { reason: "invalid_control", control },
      });
      replyError(msg, error);
      this.#expireUploadSession(session.subject, error);
      return;
    }

    const seq = seqResult;
    if (seq !== session.nextSeq) {
      const error = new TransferError({
        operation: "put",
        context: {
          reason: "out_of_order",
          expected: session.nextSeq,
          actual: seq,
        },
      });
      replyError(msg, error);
      this.#expireUploadSession(session.subject, error);
      return;
    }
    const eof = control === "complete";
    if (!eof && msg.data.length > 0) {
      if (msg.data.length > this.#chunkBytes) {
        const error = new TransferError({
          operation: "put",
          context: {
            reason: "chunk_too_large",
            maxChunkBytes: this.#chunkBytes,
          },
        });
        replyError(msg, error);
        this.#expireUploadSession(session.subject, error);
        return;
      }
      session.receivedBytes += msg.data.length;
      if (
        session.maxBytes !== undefined &&
        session.receivedBytes > session.maxBytes
      ) {
        const error = new TransferError({
          operation: "put",
          context: {
            reason: "max_bytes_exceeded",
            maxBytes: session.maxBytes,
            attemptedBytes: session.receivedBytes,
          },
        });
        replyError(msg, error);
        this.#expireUploadSession(session.subject, error);
        return;
      }
      await raceCancellation(
        session.queue.push(msg.data),
        session.cancellation.signal,
      );
      session.hasher.update(msg.data);
      await session.onProgress?.({
        chunkIndex: session.nextSeq,
        chunkBytes: msg.data.length,
        transferredBytes: session.receivedBytes,
      });
    }
    session.nextSeq += 1;

    if (eof) {
      let completion: { action: "complete"; size: number; digest: string };
      try {
        completion = JSON.parse(new TextDecoder().decode(msg.data));
      } catch (cause) {
        const error = new TransferError({
          operation: "put",
          cause,
          context: { reason: "invalid_completion" },
        });
        replyError(msg, error);
        this.#expireUploadSession(session.subject, error);
        return;
      }
      const digest = `SHA-256=${base64urlEncode(session.hasher.digest())}`;
      if (
        completion.action !== "complete" ||
        completion.size !== session.receivedBytes ||
        completion.digest !== digest
      ) {
        const error = new TransferError({
          operation: "put",
          context: {
            reason: "completion_mismatch",
            expectedSize: session.receivedBytes,
            actualSize: completion.size,
            expectedDigest: digest,
            actualDigest: completion.digest,
          },
        });
        replyError(msg, error);
        this.#expireUploadSession(session.subject, error);
        return;
      }
      session.queue.close();
      session.committing = true;
      clearTimeout(session.timeoutId);
      const putResult = await session.putPromise;
      const putValue = putResult.take();
      if (isErr(putValue)) {
        const error = new TransferError({
          operation: "put",
          cause: putValue.error,
        });
        replyError(msg, error);
        this.#expireUploadSession(session.subject, error);
        return;
      }

      const stored = await session.store.get(session.key);
      const storedValue = stored.take();
      if (isErr(storedValue)) {
        const error = new TransferError({
          operation: "put",
          cause: storedValue.error,
        });
        replyError(msg, error);
        this.#expireUploadSession(session.subject, error);
        return;
      }

      const info = fileInfoFromStoreInfo(storedValue.info);
      if (
        info.key !== session.key || info.size !== session.receivedBytes ||
        info.digest === undefined ||
        info.digest.replace(/=+$/, "") !== digest.replace(/=+$/, "")
      ) {
        const error = new TransferError({
          operation: "put",
          context: {
            reason: "stored_metadata_mismatch",
            expectedKey: session.key,
            actualKey: info.key,
            expectedSize: session.receivedBytes,
            actualSize: info.size,
            expectedDigest: digest,
            actualDigest: info.digest,
          },
        });
        replyError(msg, error);
        this.#expireUploadSession(session.subject, error);
        return;
      }
      msg.respond(JSON.stringify({ status: "complete", info }));
      await session.onComplete?.(info);
      if (session.onStored) {
        void Promise.resolve(session.onStored({
          transferId: session.transferId,
          sessionKey: session.sessionKey,
          store: session.store,
          entry: storedValue,
          info,
        })).catch((error) => {
          console.error("transfer onStored callback failed", error);
        });
      }
      this.#cleanupUploadSession(session.subject);
      return;
    }

    msg.respond(JSON.stringify({ status: "continue" }));
  }

  async #runDownloadSession(session: DownloadSession): Promise<void> {
    let pending: Promise<boolean> | undefined;
    try {
      for await (const msg of session.subscription) {
        const cancellation = msg.headers?.get(TRANSFER_CONTROL_HEADER) ===
          "cancel";
        if (pending && !cancellation) {
          replyError(
            msg,
            new TransferError({
              operation: "get",
              context: { reason: "pending_frame" },
            }),
          );
          continue;
        }
        const handling = this.#handleDownloadRequest(session, msg);
        if (pending) {
          if (await handling) break;
          continue;
        }
        let tracked: Promise<boolean>;
        tracked = handling.then((done) => {
          if (done) this.#cleanupDownloadSession(session.subject);
          return done;
        }).catch(() => {
          this.#cleanupDownloadSession(session.subject);
          return true;
        }).finally(() => {
          if (pending === tracked) pending = undefined;
        });
        pending = tracked;
      }
    } finally {
      await pending?.catch(() => undefined);
      this.#cleanupDownloadSession(session.subject);
    }
  }

  async #handleDownloadRequest(
    session: DownloadSession,
    msg: Msg,
  ): Promise<boolean> {
    const reply = msg.reply;
    if (
      !reply || !reply.startsWith(`${session.inboxPrefix}.`)
    ) {
      replyError(
        msg,
        new TransferError({
          operation: "get",
          context: {
            reason: "reply_subject_mismatch",
            expected: session.inboxPrefix,
            actual: reply,
          },
        }),
      );
      return false;
    }
    if (Date.now() >= session.expiresAtMs) {
      publishError(
        this.#nc,
        reply,
        new TransferError({ operation: "get", context: { reason: "expired" } }),
      );
      return true;
    }

    const parsedSeq = parseSeq(msg).take();
    if (isErr(parsedSeq)) {
      replyError(msg, parsedSeq.error);
      return false;
    }
    const control = msg.headers?.get(TRANSFER_CONTROL_HEADER) || undefined;
    const authenticated = await verifyLocalAuthorization({
      kind: "request",
      cache: this.#auth.authorizationProviderCache,
      message: msg,
      permission: session.permission,
      requiredCapabilities: session.requiredCapabilities,
      proofPayload: transferFrameProofPayload(parsedSeq, control, msg.data),
    });
    const authenticatedValue = authenticated.take();
    if (
      isErr(authenticatedValue) ||
      authenticatedValue.sessionKey !== session.sessionKey
    ) {
      publishError(
        this.#nc,
        reply,
        new TransferError({
          operation: "get",
          context: { reason: "invalid_proof" },
        }),
      );
      return false;
    }

    if (control === "cancel") {
      try {
        if (
          (JSON.parse(new TextDecoder().decode(msg.data)) as {
            action?: string;
          })
            .action !== "cancel"
        ) {
          throw new Error(
            "cancellation payload does not match its control header",
          );
        }
        session.cancellation.abort(
          new TransferError({
            operation: "get",
            context: { reason: "cancelled" },
          }),
        );
        await session.reader?.cancel().catch(() => undefined);
        msg.respond(JSON.stringify({ status: "cancelled" }));
        return true;
      } catch (cause) {
        replyError(
          msg,
          new TransferError({
            operation: "get",
            cause,
            context: { reason: "invalid_control" },
          }),
        );
        return false;
      }
    }
    if (
      control !== undefined ||
      msg.headers?.get(TRANSFER_EOF_HEADER) === "true"
    ) {
      replyError(
        msg,
        new TransferError({
          operation: "get",
          context: { reason: "invalid_control", control },
        }),
      );
      return false;
    }

    if (parsedSeq !== session.nextSeq) {
      replyError(
        msg,
        new TransferError({
          operation: "get",
          context: {
            reason: "sequence",
            expected: session.nextSeq,
            actual: parsedSeq,
          },
        }),
      );
      return false;
    }
    if (msg.data.length !== 0) {
      replyError(
        msg,
        new TransferError({
          operation: "get",
          context: { reason: "invalid_control" },
        }),
      );
      return false;
    }

    try {
      if (!session.reader) {
        const entryValue = (await session.store.get(session.key)).take();
        if (isErr(entryValue)) throw entryValue.error;
        const streamValue = (await entryValue.stream()).take();
        if (isErr(streamValue)) throw streamValue.error;
        session.reader = streamValue.getReader();
      }
      while (
        !session.pending || session.pendingOffset >= session.pending.length
      ) {
        const next = await raceCancellation(
          session.reader.read(),
          session.cancellation.signal,
        );
        if (next.done) {
          if (session.sentBytes !== session.info.size) {
            throw new TransferError({
              operation: "get",
              context: {
                reason: "size_mismatch",
                expected: session.info.size,
                actual: session.sentBytes,
              },
            });
          }
          const digest = `SHA-256=${base64urlEncode(session.hasher.digest())}`;
          if (
            session.info.digest &&
            session.info.digest.replace(/=+$/, "") !== digest.replace(/=+$/, "")
          ) {
            throw new TransferError({
              operation: "get",
              context: {
                reason: "digest_mismatch",
                expected: session.info.digest,
                actual: digest,
              },
            });
          }
          const headers = natsHeaders();
          headers.set(TRANSFER_SEQUENCE_HEADER, String(session.nextSeq));
          headers.set(TRANSFER_EOF_HEADER, "true");
          msg.respond(new Uint8Array(), { headers });
          return true;
        }
        session.pending = next.value;
        session.pendingOffset = 0;
      }

      const chunk = session.pending.slice(
        session.pendingOffset,
        session.pendingOffset + this.#chunkBytes,
      );
      session.pendingOffset += chunk.length;
      session.sentBytes += chunk.length;
      if (session.sentBytes > session.info.size) {
        throw new TransferError({
          operation: "get",
          context: {
            reason: "size_mismatch",
            expected: session.info.size,
            actual: session.sentBytes,
          },
        });
      }
      session.hasher.update(chunk);
      const headers = natsHeaders();
      headers.set(TRANSFER_SEQUENCE_HEADER, String(session.nextSeq));
      msg.respond(chunk, { headers });
      session.nextSeq += 1;
      return false;
    } catch (cause) {
      replyError(
        msg,
        cause instanceof TransferError
          ? cause
          : new TransferError({ operation: "get", cause }),
      );
      return true;
    }
  }

  #expireUploadSession(
    subject: string,
    error = new TransferError({
      operation: "put",
      context: { reason: "expired" },
    }),
  ): void {
    const session = this.#uploadSessions.get(subject);
    if (!session) {
      return;
    }
    session.cancellation.abort(error);
    session.queue.fail(error);
    void Promise.resolve(session.onError?.(error));
    this.#cleanupUploadSession(subject);
  }

  #cleanupUploadSession(subject: string): void {
    const session = this.#uploadSessions.get(subject);
    if (!session) {
      return;
    }
    clearTimeout(session.timeoutId);
    session.cancellation.abort();
    session.subscription.unsubscribe();
    this.#uploadSessions.delete(subject);
  }

  #cleanupDownloadSession(subject: string): void {
    const session = this.#downloadSessions.get(subject);
    if (!session) {
      return;
    }
    clearTimeout(session.timeoutId);
    session.cancellation.abort();
    session.subscription.unsubscribe();
    void session.reader?.cancel().catch(() => undefined);
    this.#downloadSessions.delete(subject);
  }
}
