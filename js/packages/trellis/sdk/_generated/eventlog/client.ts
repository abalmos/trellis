// Generated from ./generated/contracts/manifests/trellis.eventlog@v1.json
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
import type * as AuthSdk from "../auth/mod.ts";
import type * as HealthSdk from "../health/mod.ts";

type EventCallback<TMessage> = {
  bivarianceHack(
    message: TMessage,
    context: EventListenerContext,
  ): MaybeAsync<void, BaseError>;
}["bivarianceHack"];

type DependencyServiceEventHandler<TEvent> = (
  args: { event: TEvent; context: EventListenerContext; client: HandlerClient },
) => MaybeAsync<void, BaseError>;

export type TrellisEventlogState = {};

export interface TrellisEventlogClient {
  readonly name: string;
  readonly timeout: number;
  readonly stream: string;
  readonly api: Api;
  readonly state: TrellisEventlogState;
  readonly connection: TrellisConnection;
  transfer(grant: SendTransferGrant): SendTransferHandle;
  transfer(grant: ReceiveTransferGrant): ReceiveTransferHandle;
  readonly rpc: {
    readonly auth: {
      eventConsumersList(
        input: AuthSdk.AuthEventConsumersListInput,
        opts?: RequestOpts,
      ): AsyncResult<AuthSdk.AuthEventConsumersListOutput, BaseError>;
    };
    readonly eventLog: {
      consumersInspect(
        input: Types.EventLogConsumersInspectInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.EventLogConsumersInspectOutput, BaseError>;
      consumersQuery(
        input: Types.EventLogConsumersQueryInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.EventLogConsumersQueryOutput, BaseError>;
      inspect(
        input: Types.EventLogInspectInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.EventLogInspectOutput, BaseError>;
      metrics(
        input: Types.EventLogMetricsInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.EventLogMetricsOutput, BaseError>;
      query(
        input: Types.EventLogQueryInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.EventLogQueryOutput, BaseError>;
    };
  };
  readonly event: {
    readonly health: {
      heartbeat: {
        publish(
          event: HealthSdk.HealthHeartbeatEvent,
        ): AsyncResult<void, ValidationError | UnexpectedError>;
        prepare(
          event: HealthSdk.HealthHeartbeatEvent,
        ): Result<
          PreparedTrellisEvent<HealthSdk.HealthHeartbeatEvent>,
          ValidationError | UnexpectedError
        >;
        listen(
          handler: EventCallback<HealthSdk.HealthHeartbeatEvent>,
          subjectData?: Record<string, unknown>,
          opts?: EventOpts,
        ): AsyncResult<void, ValidationError | UnexpectedError>;
      };
    };
  };
  readonly feed: {
    readonly eventLog: {
      watch(
        input: Types.EventLogWatchInput,
        opts?: FeedSubscribeOpts,
      ): AsyncResult<FeedSubscription<Types.EventLogWatchEvent>, BaseError>;
    };
  };
  readonly operation: {};
  wait(): AsyncResult<void, BaseError>;
}

export interface Service extends TrellisEventlogClient {
  readonly handle: ServiceHandle;
}

export interface ServiceEventSurface {
  readonly health: {
    heartbeat: {
      publish(
        event: HealthSdk.HealthHeartbeatEvent,
      ): AsyncResult<void, ValidationError | UnexpectedError>;
      prepare(
        event: HealthSdk.HealthHeartbeatEvent,
      ): Result<
        PreparedTrellisEvent<HealthSdk.HealthHeartbeatEvent>,
        ValidationError | UnexpectedError
      >;
      listen(
        handler: DependencyServiceEventHandler<HealthSdk.HealthHeartbeatEvent>,
        subjectData?: Record<string, unknown>,
        opts?: EventOpts,
      ): AsyncResult<void, ValidationError | UnexpectedError>;
    };
  };
}

export interface ServiceHandle {
  readonly rpc: {
    readonly eventLog: {
      consumersInspect(
        handler: Types.EventLogConsumersInspectHandler,
      ): Promise<void>;
      consumersQuery(
        handler: Types.EventLogConsumersQueryHandler,
      ): Promise<void>;
      inspect(handler: Types.EventLogInspectHandler): Promise<void>;
      metrics(handler: Types.EventLogMetricsHandler): Promise<void>;
      query(handler: Types.EventLogQueryHandler): Promise<void>;
    };
  };
  readonly feed: {
    readonly eventLog: {
      watch(handler: Types.EventLogWatchFeedHandler): Promise<void>;
    };
  };
  readonly operation: {};
}

export type HandlerClient = HandlerTrellis<Api>;
export type Client = TrellisEventlogClient;
