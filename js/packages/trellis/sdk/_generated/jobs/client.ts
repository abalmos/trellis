// Generated from ./generated/contracts/manifests/trellis.jobs@v1.json
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

export type TrellisJobsState = {};

export interface TrellisJobsClient {
  readonly name: string;
  readonly timeout: number;
  readonly stream: string;
  readonly api: Api;
  readonly state: TrellisJobsState;
  readonly connection: TrellisConnection;
  transfer(grant: SendTransferGrant): SendTransferHandle;
  transfer(grant: ReceiveTransferGrant): ReceiveTransferHandle;
  readonly rpc: {
    readonly jobs: {
      cancel(
        input: Types.JobsCancelInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsCancelOutput, BaseError>;
      dismissDLQ(
        input: Types.JobsDismissDLQInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsDismissDLQOutput, BaseError>;
      getKey(
        input: Types.JobsGetKeyInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsGetKeyOutput, BaseError>;
      inspect(
        input: Types.JobsInspectInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsInspectOutput, BaseError>;
      listDLQ(
        input: Types.JobsListDLQInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsListDLQOutput, BaseError>;
      listServices(
        input: Types.JobsListServicesInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsListServicesOutput, BaseError>;
      metrics(
        input: Types.JobsMetricsInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsMetricsOutput, BaseError>;
      query(
        input: Types.JobsQueryInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsQueryOutput, BaseError>;
      replayDLQ(
        input: Types.JobsReplayDLQInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsReplayDLQOutput, BaseError>;
      retry(
        input: Types.JobsRetryInput,
        opts?: RequestOpts,
      ): AsyncResult<Types.JobsRetryOutput, BaseError>;
    };
  };
  readonly event: {};
  readonly feed: {
    readonly jobs: {
      watch(
        input: Types.JobsWatchInput,
        opts?: FeedSubscribeOpts,
      ): AsyncResult<FeedSubscription<Types.JobsWatchEvent>, BaseError>;
    };
  };
  readonly operation: {};
  wait(): AsyncResult<void, BaseError>;
}

export interface Service extends TrellisJobsClient {
  readonly handle: ServiceHandle;
}

export type ServiceEventSurface = {};

export interface ServiceHandle {
  readonly rpc: {
    readonly jobs: {
      cancel(handler: Types.JobsCancelHandler): Promise<void>;
      dismissDLQ(handler: Types.JobsDismissDLQHandler): Promise<void>;
      getKey(handler: Types.JobsGetKeyHandler): Promise<void>;
      inspect(handler: Types.JobsInspectHandler): Promise<void>;
      listDLQ(handler: Types.JobsListDLQHandler): Promise<void>;
      listServices(handler: Types.JobsListServicesHandler): Promise<void>;
      metrics(handler: Types.JobsMetricsHandler): Promise<void>;
      query(handler: Types.JobsQueryHandler): Promise<void>;
      replayDLQ(handler: Types.JobsReplayDLQHandler): Promise<void>;
      retry(handler: Types.JobsRetryHandler): Promise<void>;
    };
  };
  readonly feed: {
    readonly jobs: {
      watch(handler: Types.JobsWatchFeedHandler): Promise<void>;
    };
  };
  readonly operation: {};
}

export type HandlerClient = HandlerTrellis<Api>;
export type Client = TrellisJobsClient;
