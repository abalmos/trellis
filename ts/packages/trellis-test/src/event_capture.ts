import type {
  CallerContract,
  CallerRuntime,
  ClientOpts,
  EventListenerContext,
  InferSchemaType,
} from "@qlever-llc/trellis";
import { defineAppContract } from "@qlever-llc/trellis";
import type {
  ActionDescriptor,
  DescriptorForAction,
} from "@qlever-llc/trellis/contracts";
import type { EventDesc } from "@qlever-llc/trellis/contracts";

import type { WaitForOptions } from "./types.ts";
import { unscopedCaseActionName } from "./integration/names.ts";

export type TrellisTestEventAction = ActionDescriptor<
  string,
  string,
  "event-subscribe",
  EventDesc,
  string
>;
type EventSubscribeAction = TrellisTestEventAction;
type EventName<TAction extends EventSubscribeAction> = TAction["name"];
type EventPayload<
  TAction extends EventSubscribeAction,
  TName extends EventName<TAction>,
> = Extract<TAction, { name: TName }> extends
  infer TSelected extends EventSubscribeAction
  ? DescriptorForAction<TSelected> extends EventDesc<infer TSchema>
    ? InferSchemaType<TSchema>
  : never
  : never;

type ConnectedClient = { connection: { close(): Promise<void> } };
type EventListener<TEvent> = (
  handler: (event: TEvent, context: EventListenerContext) => unknown,
  opts: { mode: "ephemeral"; signal: AbortSignal },
) => { orThrow(): Promise<void> };

type EventCaptureRuntime = {
  contracts: {
    approve(args: {
      contract: TrellisTestEventSourceContract;
      deployment?: string;
    }): Promise<unknown>;
  };
  connectClient<TContract extends CallerContract>(
    args: ClientOpts & { name: string; contract: TContract },
  ): Promise<CallerRuntime<TContract>>;
  waitFor<T>(
    fn: () =>
      | T
      | null
      | undefined
      | false
      | Promise<T | null | undefined | false>,
    opts?: WaitForOptions,
  ): Promise<T>;
};

/** Contract value accepted by `TrellisTestRuntime.captureEvents`. */
export type TrellisTestEventSourceContract = CallerContract;

/** Options for starting a live decoded contract event capture. */
export type TrellisTestEventCaptureOptions<
  TContract extends TrellisTestEventSourceContract,
  TEvents extends readonly EventSubscribeAction[],
> = ClientOpts & {
  name: string;
  contract: TContract;
  deployment?: string;
  events: TEvents;
};

/** Transport-neutral listener metadata captured with a test event. */
export type TrellisTestCapturedEventContext = {
  readonly id: string;
  readonly time: Date;
  readonly mode: "ephemeral";
};

/** A decoded contract event observed by a `TrellisTestEventCapture`. */
export type TrellisTestCapturedEvent<
  TAction extends EventSubscribeAction,
  TName extends EventName<TAction>,
> = TName extends EventName<TAction> ? {
    readonly event: TName;
    readonly payload: EventPayload<TAction, TName>;
    readonly context: TrellisTestCapturedEventContext;
    readonly receivedAt: Date;
  }
  : never;

/** Predicate used by `TrellisTestEventCapture.waitFor`. */
export type TrellisTestCapturedEventPredicate<
  TAction extends EventSubscribeAction,
  TName extends EventName<TAction>,
> = (
  event: TrellisTestCapturedEvent<TAction, TName>,
) => boolean | Promise<boolean>;

function captureContractName(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/gu, "-").replace(
    /^-+|-+$/gu,
    "",
  ) || "capture";
}

function selectedEvents<TEvents extends readonly EventSubscribeAction[]>(
  contract: TrellisTestEventSourceContract,
  events: TEvents,
): TEvents {
  if (events.length === 0) {
    throw new Error("Trellis event capture requires at least one event action");
  }
  const seen = new Set<string>();
  const apiId = contract.API.id;
  if (typeof apiId !== "string") {
    throw new Error("Native contract API is missing an id");
  }
  for (const event of events) {
    if (event.contractId !== apiId) {
      throw new Error(
        `Event '${event.name}' belongs to '${event.contractId}', not '${apiId}'`,
      );
    }
    if (seen.has(event.name)) {
      throw new Error(`Duplicate event '${event.name}' in capture options`);
    }
    seen.add(event.name);
  }
  return events;
}

type InternalCapturedEvent = {
  readonly event: string;
  readonly payload: unknown;
  readonly context: TrellisTestCapturedEventContext;
  readonly receivedAt: Date;
};

function isCapturedEvent(event: InternalCapturedEvent, name: string): boolean {
  return event.event === name;
}

/** Disposable live event capture for integration tests. */
export class TrellisTestEventCapture<
  TAction extends EventSubscribeAction,
> implements AsyncDisposable {
  readonly #client: ConnectedClient;
  readonly #waitFor: EventCaptureRuntime["waitFor"];
  readonly #onStop: (
    client: ConnectedClient,
    capture: TrellisTestEventCapture<TAction>,
  ) => void;
  readonly #controller = new AbortController();
  readonly #events: InternalCapturedEvent[] = [];
  #stopped = false;

  protected constructor(args: {
    client: ConnectedClient;
    waitFor: EventCaptureRuntime["waitFor"];
    onStop: (
      client: ConnectedClient,
      capture: TrellisTestEventCapture<TAction>,
    ) => void;
  }) {
    this.#client = args.client;
    this.#waitFor = args.waitFor;
    this.#onStop = args.onStop;
  }

  protected get listenerSignal(): AbortSignal {
    return this.#controller.signal;
  }

  protected record(
    event: InternalCapturedEvent,
  ): void {
    this.#events.push(event);
  }

  /** Returns all captured events, optionally filtered by event name. */
  all(): ReadonlyArray<
    TrellisTestCapturedEvent<TAction, EventName<TAction>>
  >;
  all<TName extends EventName<TAction>>(
    name: TName,
  ): ReadonlyArray<TrellisTestCapturedEvent<TAction, TName>>;
  all<TName extends EventName<TAction>>(
    name?: TName,
  ): ReadonlyArray<InternalCapturedEvent> {
    return name === undefined
      ? [...this.#events]
      : this.#events.filter((event) => isCapturedEvent(event, name));
  }

  /** Removes all captured events without stopping listeners. */
  clear(): void {
    this.#events.length = 0;
  }

  /** Waits for a matching captured event. */
  async waitFor<TName extends EventName<TAction>>(
    name: TName,
    predicate?: TrellisTestCapturedEventPredicate<TAction, TName>,
    opts?: WaitForOptions,
  ): Promise<TrellisTestCapturedEvent<TAction, TName>> {
    return await this.#waitFor(async () => {
      for (const event of this.#events) {
        if (!isCapturedEvent(event, name)) continue;
        const typed = event as TrellisTestCapturedEvent<TAction, TName>;
        if (predicate === undefined || await predicate(typed)) return typed;
      }
      return false;
    }, opts);
  }

  /** Stops listeners and closes the synthetic client. */
  async stop(): Promise<void> {
    if (this.#stopped) return;
    this.#controller.abort();
    await this.#client.connection.close();
    this.#stopped = true;
    this.#onStop(this.#client, this);
  }

  /** Stops the capture when used with `await using`. */
  [Symbol.asyncDispose](): Promise<void> {
    return this.stop();
  }
}

class StartedTrellisTestEventCapture<
  TAction extends EventSubscribeAction,
> extends TrellisTestEventCapture<TAction> {
  constructor(args: {
    client: ConnectedClient;
    waitFor: EventCaptureRuntime["waitFor"];
    onStop: (
      client: ConnectedClient,
      capture: TrellisTestEventCapture<TAction>,
    ) => void;
  }) {
    super(args);
  }

  get signal(): AbortSignal {
    return this.listenerSignal;
  }

  recordCaptured(
    event: InternalCapturedEvent,
  ): void {
    this.record(event);
  }
}

/** @internal Starts a capture through runtime-owned client helpers. */
export async function startTrellisTestEventCapture<
  TContract extends TrellisTestEventSourceContract,
  const TEvents extends readonly EventSubscribeAction[],
>(args: {
  runtime: EventCaptureRuntime;
  options: TrellisTestEventCaptureOptions<TContract, TEvents>;
  onStop: (
    client: ConnectedClient,
    capture: TrellisTestEventCapture<TEvents[number]>,
  ) => void;
}): Promise<TrellisTestEventCapture<TEvents[number]>> {
  const events = selectedEvents(args.options.contract, args.options.events);
  await args.runtime.contracts.approve({
    contract: args.options.contract,
    deployment: args.options.deployment,
  });

  const appContract = defineAppContract(() => ({
    id: `trellis.test.event-capture.${
      captureContractName(args.options.name)
    }@v1`,
    displayName: `Trellis Test Event Capture: ${args.options.name}`,
    description: "Synthetic app participant for live test event capture.",
    uses: events,
  }));
  const {
    contract: _sourceContract,
    deployment: _deployment,
    events: _events,
    ...clientOptions
  } = args.options;
  const client = await args.runtime.connectClient({
    ...clientOptions,
    contract: appContract,
  });
  const capture = new StartedTrellisTestEventCapture<TEvents[number]>({
    client,
    waitFor: args.runtime.waitFor.bind(args.runtime),
    onStop: args.onStop,
  });

  try {
    for (const event of events) {
      const listener = Reflect.get(
        client,
        event.connectedName,
      ) as EventListener<
        EventPayload<TEvents[number], typeof event.name>
      >;
      await listener((decoded, context) => {
        capture.recordCaptured({
          event: unscopedCaseActionName(args.options.contract, event.name),
          payload: decoded,
          context: { id: context.id, time: context.time, mode: "ephemeral" },
          receivedAt: new Date(),
        });
      }, { mode: "ephemeral", signal: capture.signal }).orThrow();
    }
  } catch (error) {
    await capture.stop().catch(() => undefined);
    throw error;
  }

  return capture;
}
