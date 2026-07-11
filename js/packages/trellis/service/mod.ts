/**
 * Trellis service authoring entry point.
 *
 * This subpath exposes the service wrapper and service-side helpers without the
 * low-level runtime used by the internal server package.
 *
 * @module
 */

export {
  createHealthHeartbeatSample,
  type HealthCheckResult,
  runAllServiceHealthChecks,
  runServiceHealthCheck,
  ServiceHealth,
  type ServiceHealthCheck,
  type ServiceHealthCheckFn,
  type ServiceHealthInfo,
  type ServiceHealthInfoFn,
} from "../server/health.ts";
export {
  HealthCheckResultSchema,
  HealthInfoSchema,
} from "../server/health_schemas.ts";
export {
  type HealthHeartbeatSample,
  HealthHeartbeatSampleSchema,
} from "../health.ts";
export {
  type HealthCheckHandler,
  type HealthInfoHandler,
  type JobQueue,
  type JobsFacadeOf,
  type OperationHandler,
  type OperationRegistration,
  type RpcHandler,
  type ServiceContract,
  type ServiceEventHandler,
  type SqlOutbox,
  type SqlOutboxEventEnqueueFacade,
  type SqlOutboxJobEnqueueFacade,
  type SqlOutboxJobSubmission,
  type SqlOutboxTransactionContext,
  type SqlOutboxTransactionRunner,
  StoreHandle,
  type Trellis,
  TrellisService,
  type TrellisServiceConnectOpts,
  type TrellisServiceConnectTelemetryOpts,
  type TrellisServiceSqlOutboxCommonOptions,
  type TrellisServiceSqlOutboxExecutorOptions,
  type TrellisServiceSqlOutboxOptions,
} from "../server/service.ts";
export {
  createPostgresOutboxSchema,
  createSqliteOutboxSchema,
  createSqlOutboxAdapter,
  defaultSqlOutboxTables,
  dispatchOutbox,
  getSqlOutboxMigrations,
  type InboxRepository,
  type KvOutboxRecord,
  KvOutboxRecordSchema,
  MemoryInboxRepository,
  MemoryOutboxRepository,
  NatsKvInboxRepository,
  NatsKvOutboxRepository,
  OutboxDispatcher,
  type OutboxDispatcherOptions,
  type OutboxDispatchResult,
  type OutboxDispatchRuntime,
  type OutboxJobDispatchOutcome,
  type OutboxKvEntry,
  type OutboxKvStore,
  type OutboxMessage,
  type OutboxMessageState,
  outboxMessageToPreparedEvent,
  type OutboxRecordKind,
  type OutboxRepository,
  type PreparedOutboxRecord,
  preparedTrellisEventToOutboxRecord,
  type SqlDialect,
  type SqlExecutor,
  SqlInboxRepository,
  type SqlOutboxAdapter,
  type SqlOutboxMigration,
  type SqlOutboxMigrationOptions,
  SqlOutboxRepository,
  type SqlOutboxTables,
  type SqlRow,
} from "./outbox_inbox.ts";
export {
  createEventContext,
  type EventContext,
  type EventHandler,
  type GroupedSubscription,
  isGroupedSubscription,
  type MultiEventSubscription,
  type MultiSubscribeOpts,
  type OrderingGroup,
  type SingleSubscription,
  type SubscribeOpts,
} from "../server/subscription.ts";
