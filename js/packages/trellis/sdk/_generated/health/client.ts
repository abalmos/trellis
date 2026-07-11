// Generated from ./generated/contracts/manifests/trellis.health@v1.json
import type {
  AcceptedOperation,
  AsyncResult,
  BaseError,
  EventListenerContext,
  EventOpts,
  FeedSubscribeOpts,
  FeedSubscription,
  HandlerTrellis,
  MapStateStoreClient,
  MaybeAsync,
  OperationInputBuilder,
  OperationObserverCallbacks,
  OperationRef,
  OperationRefData,
  OperationRuntimeHandle,
  PreparedTrellisEvent,
  ReceiveTransferGrant,
  ReceiveTransferHandle,
  RequestOpts,
  Result,
  SendTransferGrant,
  SendTransferHandle,
  TerminalOperation,
  TransferCapableOperationInputBuilder,
  TrellisConnection,
  UnexpectedError,
  ValidationError,
  ValueStateStoreClient,
} from "../../../index.ts";
import type { API, Api } from "./api.ts";
import type * as Types from "./types.ts";

type EventCallback<TMessage> = {
  bivarianceHack(
    message: TMessage,
    context: EventListenerContext,
  ): MaybeAsync<void, BaseError>;
}["bivarianceHack"];

type DependencyServiceEventHandler<TEvent> = (
  args: { event: TEvent; context: EventListenerContext; client: HandlerClient },
) => MaybeAsync<void, BaseError>;

export type TrellisHealthState = {};

export interface TrellisHealthClient {
  readonly name: string;
  readonly timeout: number;
  readonly stream: string;
  readonly api: Api;
  readonly state: TrellisHealthState;
  readonly connection: TrellisConnection;
  transfer(grant: SendTransferGrant): SendTransferHandle;
  transfer(grant: ReceiveTransferGrant): ReceiveTransferHandle;
  readonly rpc: {
    readonly health: {
      inspect(
        input: Types.HealthInspectInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.HealthInspectOutput, BaseError>;
      metrics(
        input: Types.HealthMetricsInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.HealthMetricsOutput, BaseError>;
      query(
        input: Types.HealthQueryInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.HealthQueryOutput, BaseError>;
    };
  };
  readonly event: {
    readonly health: {
      statusChanged: {
        publish(
          event: Types.HealthStatusChangedEvent,
        ): AsyncResult<void, ValidationError | UnexpectedError>;
        prepare(
          event: Types.HealthStatusChangedEvent,
        ): Result<
          PreparedTrellisEvent<Types.HealthStatusChangedEvent>,
          ValidationError | UnexpectedError
        >;
        listen(
          handler: EventCallback<Types.HealthStatusChangedEvent>,
          subjectData?: Record<string, unknown>,
          opts?: EventOpts,
        ): AsyncResult<void, ValidationError | UnexpectedError>;
      };
    };
  };
  readonly feed: {
    readonly health: {
      watch(
        input: Types.HealthWatchInput,
        opts?: FeedSubscribeOpts,
      ): AsyncResult<FeedSubscription<Types.HealthWatchEvent>, BaseError>;
    };
  };
  readonly operation: {};
  wait(): AsyncResult<void, BaseError>;
}

export interface Service extends TrellisHealthClient {
  readonly handle: ServiceHandle;
}

export interface ServiceEventSurface {
  readonly health: {
    statusChanged: {
      publish(
        event: Types.HealthStatusChangedEvent,
      ): AsyncResult<void, ValidationError | UnexpectedError>;
      prepare(
        event: Types.HealthStatusChangedEvent,
      ): Result<
        PreparedTrellisEvent<Types.HealthStatusChangedEvent>,
        ValidationError | UnexpectedError
      >;
      listen(
        handler: Types.HealthStatusChangedEventHandler,
        subjectData?: Record<string, unknown>,
        opts?: EventOpts,
      ): AsyncResult<void, ValidationError | UnexpectedError>;
    };
  };
}

export interface ServiceHandle {
  readonly rpc: {
    readonly health: {
      inspect(handler: Types.HealthInspectHandler): Promise<void>;
      metrics(handler: Types.HealthMetricsHandler): Promise<void>;
      query(handler: Types.HealthQueryHandler): Promise<void>;
    };
  };
  readonly feed: {
    readonly health: {
      watch(handler: Types.HealthWatchFeedHandler): Promise<void>;
    };
  };
  readonly operation: {};
}

export type HandlerClient = HandlerTrellis<Api>;
export type Client = TrellisHealthClient;
