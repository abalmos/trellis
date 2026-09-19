import { LiveCancellation, LiveEnd, LiveStreamError } from "./types.ts";

type Admitted<T> = { value: T; encodedLen: number };

/** Queue and cursors for one consumer session. */
export class ConsumerCore<T> {
  readonly sessionId: string;
  readonly start = Promise.withResolvers<void>();
  #started = false;
  #queue: Admitted<T>[] = [];
  #end: LiveEnd | undefined;
  #consumed = 0;
  #received = 0;
  #waker: (() => void) | undefined;
  #errorReported = false;

  constructor(sessionId: string) {
    this.sessionId = sessionId;
  }

  notifyStart(): void {
    if (this.#started) return;
    this.#started = true;
    this.start.resolve();
  }

  admit(item: Admitted<T>): boolean {
    if (this.#end && !this.#end.isComplete()) return false;
    this.#queue.push(item);
    this.#received += 1;
    this.#waker?.();
    return true;
  }

  hasQueued(): boolean {
    return this.#queue.length > 0;
  }

  consume(): Admitted<T> | undefined {
    const item = this.#queue.shift();
    if (item) this.#consumed += 1;
    return item;
  }

  consumedSeq(): number {
    return this.#consumed;
  }

  receivedSeq(): number {
    return this.#received;
  }

  commitEnd(end: LiveEnd): void {
    this.#end ??= end;
    this.#waker?.();
    this.start.resolve();
  }

  committedEnd(): LiveEnd | undefined {
    return this.#end;
  }

  discardQueue(): void {
    this.#queue = [];
  }

  drainComplete(): boolean {
    return this.#end?.isComplete() === true && this.#queue.length === 0;
  }

  setWaker(waker: () => void): void {
    this.#waker = waker;
  }

  reportedError(): boolean {
    return this.#errorReported;
  }

  markErrorReported(): void {
    this.#errorReported = true;
  }
}

/** Owned live subscription. First next() starts activation. */
export class LiveSubscription<T> implements AsyncIterable<T> {
  readonly #core: ConsumerCore<T>;
  readonly #cancellation: LiveCancellation;
  #activated = false;
  #permit: { [Symbol.dispose](): void } | undefined;
  readonly #closeExchange: Promise<void>;

  constructor(
    core: ConsumerCore<T>,
    cancellation: LiveCancellation,
    permit?: { [Symbol.dispose](): void },
    closeExchange: Promise<void> = Promise.resolve(),
  ) {
    this.#core = core;
    this.#cancellation = cancellation;
    this.#permit = permit;
    this.#closeExchange = closeExchange;
  }

  async *[Symbol.asyncIterator](): AsyncGenerator<T, void, unknown> {
    this.#activated = true;
    this.#core.notifyStart();
    try {
      while (true) {
        if (this.#cancellation.aborted) {
          this.#core.discardQueue();
          return;
        }
        const end = this.#core.committedEnd();
        if (end && !end.isComplete()) {
          this.#core.discardQueue();
          if (end.error && !this.#core.reportedError()) {
            this.#core.markErrorReported();
            throw end.error;
          }
          return;
        }
        if (this.#core.drainComplete()) return;
        const item = this.#core.consume();
        if (item) {
          yield item.value;
          continue;
        }
        if (this.#core.drainComplete()) return;
        await new Promise<void>((resolve) => {
          this.#core.setWaker(resolve);
          if (this.#core.hasQueued() || this.#core.committedEnd()) resolve();
        });
      }
    } finally {
      this.close();
      await this.#closeExchange;
    }
  }

  async closed(): Promise<LiveEnd> {
    while (true) {
      const end = this.#core.committedEnd();
      if (end) return end;
      await new Promise<void>((resolve) => this.#core.setWaker(resolve));
    }
  }

  close(): void {
    this.#cancellation.cancel();
    this.#core.commitEnd(new LiveEnd("cancelled"));
    this.#permit?.[Symbol.dispose]();
    this.#permit = undefined;
  }

  async [Symbol.asyncDispose](): Promise<void> {
    this.close();
    await this.#closeExchange;
  }

  get activated(): boolean {
    return this.#activated;
  }
}

export function consumerFailure(
  code: string,
  message: string,
): LiveEnd {
  return new LiveEnd("source_error", new LiveStreamError(code, message));
}
