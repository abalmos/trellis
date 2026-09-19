/** One per-connection live session manager. */
export class LiveSessionManager {
  #stopped = false;
  #suspended = false;
  #generation = 1;
  #consumerSessions = 0;
  readonly #maxConsumers: number;

  constructor(maxConsumers = 128) {
    this.#maxConsumers = maxConsumers;
  }

  generation(): number {
    return this.#generation;
  }

  stop(): void {
    this.#stopped = true;
    this.#generation += 1;
  }

  suspend(): void {
    this.#suspended = true;
    this.#generation += 1;
  }

  resume(): void {
    this.#suspended = false;
  }

  isAvailable(): boolean {
    return !this.#stopped && !this.#suspended;
  }

  admitConsumer(): ConsumerPermit {
    if (!this.isAvailable() || this.#consumerSessions >= this.#maxConsumers) {
      throw new Error("trellis.live.resource_exhausted");
    }
    this.#consumerSessions += 1;
    return new ConsumerPermit(() => {
      this.#consumerSessions = Math.max(0, this.#consumerSessions - 1);
    });
  }

  consumerCount(): number {
    return this.#consumerSessions;
  }
}

/** Releases one consumer admission slot on dispose. */
export class ConsumerPermit {
  #release: (() => void) | undefined;

  constructor(release: () => void) {
    this.#release = release;
  }

  [Symbol.dispose](): void {
    this.#release?.();
    this.#release = undefined;
  }
}
