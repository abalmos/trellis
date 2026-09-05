/**
 * Trellis service authoring entry point.
 *
 * This subpath exposes the service wrapper and service-side helpers without the
 * internal service runtime implementation.
 *
 * @module
 */

export {
  type HealthCheckResult,
  runAllServiceHealthChecks,
  runServiceHealthCheck,
  type ServiceHealth,
  type ServiceHealthCheck,
  type ServiceHealthCheckFn,
  type ServiceHealthInfo,
  type ServiceHealthInfoFn,
} from "./runtime/health.ts";
export {
  HealthCheckResultSchema,
  HealthInfoSchema,
} from "./runtime/health_schemas.ts";
export {
  type ConnectedTrellisService,
  type FeedHandler,
  type GeneratedServiceParticipant,
  type HealthCheckHandler,
  type HealthInfoHandler,
  type JobHandler,
  type JobQueue,
  type JobsFacadeOf,
  type OperationHandler,
  type OperationRegistration,
  type RpcHandler,
  type ServiceEventHandler,
  type SqlOutbox,
  type SqlOutboxEventEnqueueFacade,
  type SqlOutboxJobEnqueueFacade,
  type SqlOutboxJobSubmission,
  type SqlOutboxTransactionContext,
  type SqlOutboxTransactionRunner,
  TrellisService,
  type TrellisServiceConnectOpts,
  type TrellisServiceConnectTelemetryOpts,
  type TrellisServiceSqlOutboxCommonOptions,
  type TrellisServiceSqlOutboxExecutorOptions,
  type TrellisServiceSqlOutboxOptions,
} from "./runtime/service.ts";
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
} from "./runtime/subscription.ts";
