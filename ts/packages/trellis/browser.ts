/**
 * Browser-safe Trellis client entrypoint.
 */

export {
  classifyBrowserAuthError,
  clearSessionKey,
  createAuth,
  generateSessionKey,
  getOrCreateSessionKey,
  getPublicSessionKey,
  hasSessionKey,
  isRecoverableBrowserAuthError,
  loadSessionKey,
  signBytes,
} from "./auth.ts";
export type {
  BrowserAuthRecoveryClassification,
  BrowserAuthRecoveryKind,
  NatsConnectOptions,
  SessionKeyHandle,
} from "./auth.ts";
export {
  canonicalizeJson,
  digestJson,
  isJsonValue,
  schema,
  unwrapSchema,
} from "./contracts.ts";
export type { InferSchemaType, JsonValue } from "./contracts.ts";
export { TrellisConnection } from "./connection.ts";
export type { TrellisConnectionStatus } from "./connection.ts";
export { defineError, withTrellisValidation } from "./contract_support/mod.ts";
export {
  buildCursorPage,
  buildPageResponse,
  CursorPageInfoSchema,
  CursorPageSchema,
  CursorQuerySchema,
  normalizeCursorQuery,
  normalizePageQuery,
  PageRequestSchema,
  PageResponseSchema,
} from "./contract_support/mod.ts";
export type {
  CursorPage,
  CursorPageInfo,
  CursorPageResponseSchema,
  CursorQuery,
  CursorQueryOptions,
  ErrorClass,
  InferRuntimeRpcError,
  NormalizedCursorQuery,
  PageRequest,
  PageResponse,
  RpcErrorClass,
  RuntimeRpcErrorDesc,
  SerializableErrorData,
  TrellisValidationExtension,
  TrellisValidationIssueHint,
} from "./contract_support/mod.ts";
export {
  defineAgentContract,
  defineAppContract,
  defineDeviceContract,
  defineServiceContract,
} from "./contract.ts";
export type { DefineContractInput } from "./contract.ts";
export {
  AsyncResult,
  BaseError,
  err,
  isErr,
  isOk,
  ok,
  Result,
} from "@qlever-llc/result";
export type { MaybeAsync } from "@qlever-llc/result";
export type { ClientOpts } from "./client.ts";
export type {
  ClientAuthContinuation,
  ClientAuthOptions,
  ClientAuthRequiredContext,
  TrellisClientConnectArgs,
} from "./client_connect.ts";
export { ClientAuthHandledError, TrellisClient } from "./client_connect.ts";
export type { TrellisErrorInstance } from "./errors/index.ts";
export {
  AuthError,
  KVError,
  OperationAlreadyTerminalError,
  OperationMismatchError,
  OperationNotFoundError,
  RemoteError,
  StoreError,
  TransferError,
  TransportError,
  TrellisError,
  UnexpectedError,
  ValidationError,
} from "./errors/index.ts";
export { TypedKVEntry } from "./kv.ts";
export type { WatchEvent, WatchOptions } from "./kv.ts";
export { TypedStoreEntry } from "./store_entry.ts";
export type {
  StoreBody,
  StoreInfo,
  StoreOpenOptions,
  StorePutOptions,
  StoreStatus,
  StoreWaitOptions,
} from "./store.ts";
export { FileInfoSchema } from "./transfer.ts";
export type {
  FileInfo,
  ReceiveTransferGrant,
  ReceiveTransferHandle,
  SendTransferGrant,
  SendTransferHandle,
  TransferBody,
  TransferGrant,
} from "./transfer.ts";
export type {
  AcceptedOperationEvent,
  CancelledOperationEvent,
  CompletedOperationEvent,
  CompletedTransfer,
  FailedOperationEvent,
  OperationControlError,
  OperationEvent,
  OperationInputBuilder,
  OperationLifecycleError,
  OperationObserverCallbacks,
  OperationRef,
  OperationRefData,
  OperationSignalAck,
  OperationSnapshot,
  OperationState,
  OperationTransferProgress,
  OperationWatchOptions,
  ProgressOperationEvent,
  ProgressOperationSnapshot,
  StartedOperationEvent,
  StartedTransfer,
  TerminalOperation,
  TransferCapableOperationInputBuilder,
  TransferOperationBuilder,
  TransferOperationEvent,
  TransferOperationSnapshot,
  UpdateOperationEvent,
} from "./operations.ts";
export { controlSubject, OperationInvoker } from "./operations.ts";
export type {
  AcceptedOperation,
  EventListenerContext,
  EventName,
  EventOpts,
  EventPayload,
  EventType,
  FeedInputBuilder,
  FeedSubscribeOpts,
  FeedSubscription,
  HandlerJobQueue,
  HandlerJobsFacade,
  HandlerKvFacade,
  HandlerStoreHandle,
  HandlerTrellis,
  HandlerTrellisForContract,
  InternalCaller,
  MapStateStoreClient,
  OperationHandlerContext,
  OperationHandlerErrorOf,
  OperationRegistration,
  OperationRuntimeHandle,
  OperationTransferContextOf,
  OperationTransferHandle,
  OperationUpdateOf,
  PreparedTrellisEvent,
  RequestOpts,
  RpcHandlerContext,
  RpcInputOf,
  RpcMethodNameOf,
  RpcOutputOf,
  RpcRequestErrorOf,
  RuntimeStateStoresForContract,
  RuntimeStateStoreShape,
  SessionCaller,
  StateFacade,
  ValueStateStoreClient,
  VerifiedCaller,
} from "./session.ts";
export type { TrellisAuth } from "./session.ts";
