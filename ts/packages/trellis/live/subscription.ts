// Consumer-side live subscription core and public owned handle.

import { AsyncResult, UnexpectedError } from "@qlever-llc/result";
import { liveConstants } from "../auth/protocol_wasm.ts";
import {
  LiveCancellation,
  type LiveCloseReceipt,
  LiveEnd,
  LiveStreamError,
} from "./types.ts";

const C = liveConstants();

/** One verified and admitted application item. */
export type Admitted<T> = { value: T; encodedLen: number };

/** Consumer-visible session phase. */
export type ConsumerPhase =
  | "prepared"
  | "activating"
  | "active"
  | "draining"
  | "closing"
  | "closed";

/** Local usage error for an unsupported iterator call. */
export class LiveUsageError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "LiveUsageError";
  }
}

/** Bounded receipt describing the outcome of an explicit close. */
export type { LiveCloseReceipt };

/**
 * Queue and cursors for one consumer session.
 *
 * The queue is the single bounded ingress; count and encoded bytes stay within
 * the negotiated window. Cursors are monotonic `bigint` values, never JS
 * numbers, so they cannot wrap at 65,535.
 */
export class ConsumerCore<T> {
  readonly sessionId: string;
  readonly start = Promise.withResolvers<void>();
  readonly #started = Promise.withResolvers<void>();
  #queue: Admitted<T>[] = [];
  #queuedBytes = 0;
  #end: LiveEnd | undefined;
  #pendingEnd: LiveEnd | undefined;
  #consumed = 0n;
  #received = 0n;
  #phase: ConsumerPhase = "prepared";
  #cancelled = false;
  #errorReported = false;
  #waiter: (() => void) | undefined;
  #creditWaiter: (() => void) | undefined;
  readonly #endWaiters: Array<(end: LiveEnd) => void> = [];

  constructor(sessionId: string) {
    this.sessionId = sessionId;
  }

  /** Resolve the internal start gate; the first `next()` calls this. */
  notifyStart(): void {
    if (this.#startedResolved) return;
    this.#startedResolved = true;
    this.#started.resolve();
    this.start.resolve();
  }

  #startedResolved = false;

  /** Wait until the first iteration installs the activation path. */
  waitStart(): Promise<void> {
    return this.#started.promise;
  }

  get phase(): ConsumerPhase {
    return this.#phase;
  }

  setPhase(phase: ConsumerPhase): void {
    this.#phase = phase;
    this.wake();
  }

  get cancelled(): boolean {
    return this.#cancelled;
  }

  /** Admit one verified application item within the wire window. */
  admit(item: Admitted<T>): boolean {
    if (this.#queue.length + 1 > C.windowFrames) return false;
    if (this.#queuedBytes + item.encodedLen > C.windowBytes) return false;
    this.#queue.push(item);
    this.#queuedBytes += item.encodedLen;
    this.#received += 1n;
    this.wake();
    this.wakeCredit();
    return true;
  }

  /** Record one verified frame that is not exposed to the application. */
  releaseFiltered(): void {
    this.#received += 1n;
    this.#consumed += 1n;
    this.wakeCredit();
    this.wake();
  }

  hasQueued(): boolean {
    return this.#queue.length > 0;
  }

  consume(): Admitted<T> | undefined {
    const item = this.#queue.shift();
    if (item) {
      this.#queuedBytes -= item.encodedLen;
      this.#consumed += 1n;
      this.wakeCredit();
    }
    return item;
  }

  consumedSeq(): bigint {
    return this.#consumed;
  }

  receivedSeq(): bigint {
    return this.#received;
  }

  queuedBytes(): number {
    return this.#queuedBytes;
  }

  /** Record one verified remote normal end, pending until the queue drains. */
  setPendingEnd(end: LiveEnd): void {
    this.#pendingEnd ??= end;
    this.wake();
  }

  /** Register a one-shot terminal observer, called immediately if closed. */
  onEnd(callback: (end: LiveEnd) => void): void {
    const existing = this.#end;
    if (existing) {
      callback(existing);
      return;
    }
    this.#endWaiters.push(callback);
  }

  /** Commit one terminal outcome once; later calls are ignored. */
  commitEnd(end: LiveEnd): void {
    if (this.#end) return;
    this.#end = end;
    this.#phase = "closed";
    this.#pendingEnd = undefined;
    this.wake();
    this.start.resolve();
    this.#started.resolve();
    const waiters = this.#endWaiters.splice(0);
    for (const waiter of waiters) waiter(end);
  }

  committedEnd(): LiveEnd | undefined {
    return this.#end;
  }

  pendingEnd(): LiveEnd | undefined {
    return this.#pendingEnd;
  }

  discardQueue(): void {
    this.#queue = [];
    this.#queuedBytes = 0;
  }

  /** Resolve a pending normal end once the queue is empty. */
  drainComplete(): boolean {
    if (this.#end?.isComplete() !== true) return false;
    return this.#queue.length === 0;
  }

  /** Promote a pending normal end to the committed outcome once drained. */
  commitDrainIfComplete(): void {
    if (this.#pendingEnd && this.#queue.length === 0) {
      const pending = this.#pendingEnd;
      this.#pendingEnd = undefined;
      this.commitEnd(pending);
    }
  }

  cancel(): void {
    this.#cancelled = true;
    this.wake();
    this.start.resolve();
    this.#started.resolve();
  }

  reportedError(): boolean {
    return this.#errorReported;
  }

  markErrorReported(): void {
    this.#errorReported = true;
  }

  /** Wake the application's pending poll, if any. */
  wake(): void {
    const waiter = this.#waiter;
    this.#waiter = undefined;
    waiter?.();
  }

  /** Register the single pending application waiter. */
  setWaiter(waiter: () => void): void {
    this.#waiter = waiter;
  }

  /** Wake the control pump when consumption advances. */
  wakeCredit(): void {
    const waiter = this.#creditWaiter;
    this.#creditWaiter = undefined;
    waiter?.();
  }

  /** Wait for the next consumption advance. */
  waitCredit(): Promise<void> {
    return new Promise((resolve) => {
      this.#creditWaiter = resolve;
    });
  }
}

/**
 * Public owned live subscription.
 *
 * Implements the single-consumer iterator contract: `[Symbol.asyncIterator]`
 * returns this, one `next()` may be pending, `closed` resolves once and never
 * rejects, and `close()` returns a bounded receipt.
 */
export class LiveSubscription<T> implements AsyncIterableIterator<T> {
  readonly #core: ConsumerCore<T>;
  readonly #cancellation: LiveCancellation;
  readonly #closeFn: () => Promise<LiveCloseReceipt>;
  readonly #closed = Promise.withResolvers<LiveEnd>();
  #closedResolved = false;
  #nextPending = false;
  #activated = false;
  #permit: { [Symbol.dispose](): void } | undefined;
  #closePromise: Promise<LiveCloseReceipt> | undefined;

  constructor(
    core: ConsumerCore<T>,
    cancellation: LiveCancellation,
    closeFn: () => Promise<LiveCloseReceipt>,
    permit?: { [Symbol.dispose](): void },
  ) {
    this.#core = core;
    this.#cancellation = cancellation;
    this.#closeFn = closeFn;
    this.#permit = permit;
    this.#core.onEnd((end) => this.#resolveClosed(end));
  }

  #resolveClosed(end: LiveEnd): void {
    if (this.#closedResolved) return;
    this.#closedResolved = true;
    this.#permit?.[Symbol.dispose]();
    this.#permit = undefined;
    this.#closed.resolve(end);
  }

  /** The terminal outcome; resolves once and never rejects. */
  get closed(): Promise<LiveEnd> {
    return this.#closed.promise;
  }

  get activated(): boolean {
    return this.#activated;
  }

  [Symbol.asyncIterator](): AsyncIterableIterator<T> {
    return this;
  }

  async next(): Promise<IteratorResult<T>> {
    if (this.#nextPending) {
      throw new LiveUsageError("concurrent next() is not supported");
    }
    this.#nextPending = true;
    try {
      if (!this.#activated) {
        this.#activated = true;
        this.#core.notifyStart();
      }
      while (true) {
        if (this.#cancellation.aborted || this.#core.cancelled) {
          this.#core.discardQueue();
          this.#core.cancel();
          return { done: true, value: undefined };
        }
        const end = this.#core.committedEnd();
        if (end && !end.isComplete()) {
          this.#core.discardQueue();
          if (end.error && !this.#core.reportedError()) {
            this.#core.markErrorReported();
            throw end.error;
          }
          return { done: true, value: undefined };
        }
        if (this.#activePhase()) {
          const item = this.#core.consume();
          if (item) {
            return { done: false, value: item.value };
          }
        }
        this.#core.commitDrainIfComplete();
        if (this.#core.drainComplete()) {
          return { done: true, value: undefined };
        }
        await new Promise<void>((resolve) => {
          this.#core.setWaiter(resolve);
          if (
            (this.#activePhase() && this.#core.hasQueued()) ||
            this.#core.committedEnd()
          ) {
            resolve();
          }
        });
      }
    } finally {
      this.#nextPending = false;
    }
  }

  #activePhase(): boolean {
    const phase = this.#core.phase;
    return phase === "active" || phase === "draining";
  }

  /** Run the one bounded close exchange, shared by every close path. */
  #runClose(): Promise<LiveCloseReceipt> {
    this.#closePromise ??= this.#closeFn();
    return this.#closePromise;
  }

  #cancel(): void {
    this.#cancellation.cancel();
    this.#core.discardQueue();
    this.#core.cancel();
    this.#resolveClosed(
      this.#core.committedEnd() ?? this.#core.pendingEnd() ??
        new LiveEnd("cancelled"),
    );
  }

  /** Cancellation-safe iterator return; never hangs behind a pending next. */
  async return(value?: unknown): Promise<IteratorResult<T>> {
    this.#cancel();
    try {
      await this.#runClose();
    } catch {
      // Remote confirmation is best-effort and already bounded.
    }
    return { done: true, value: value as T };
  }

  async throw(error?: unknown): Promise<IteratorResult<T>> {
    return await this.return(error);
  }

  /** Explicitly close the observation and await the bounded close exchange. */
  async close(): Promise<AsyncResult<LiveCloseReceipt, UnexpectedError>> {
    this.#cancel();
    try {
      const receipt = await this.#runClose();
      return AsyncResult.ok(receipt);
    } catch (cause) {
      return AsyncResult.err(new UnexpectedError({ cause }));
    }
  }

  async [Symbol.asyncDispose](): Promise<void> {
    await this.close();
  }
}

/** Build one abnormal consumer terminal outcome. */
export function consumerFailure(
  code: string,
  message: string,
): LiveEnd {
  const reason = code === "consumer_slow"
    ? "consumer_slow"
    : code === "delivery_gap"
    ? "delivery_gap"
    : code === "authorization_unavailable"
    ? "authorization_lost"
    : code === "permission_denied" || code === "authorization_revoked" ||
        code === "authorization_expired"
    ? "authorization_lost"
    : code === "setup_timeout"
    ? "setup_timeout"
    : code === "peer_lost"
    ? "peer_lost"
    : code === "disconnected"
    ? "disconnected"
    : "protocol_error";
  return new LiveEnd(reason, new LiveStreamError(code, message));
}
