/**
 * Trellis service package entry point.
 *
 * This package is service-side glue: it re-exports the RPC/event hosting runtime
 * plus common service helpers (health checks, subscription types).
 *
 * @module
 */

export { TrellisServiceRuntime } from "./core.ts";
export type { TrellisServiceRuntimeFor } from "./core.ts";
// Re-export health types
export {
  createHealthHeartbeatSample,
  type HealthCheckResult,
  type HealthResponse,
  runAllServiceHealthChecks,
  runServiceHealthCheck,
  type ServiceHealth,
  type ServiceHealthCheck,
  type ServiceHealthCheckFn,
  type ServiceHealthInfo,
  type ServiceHealthInfoFn,
} from "./health.ts";
export {
  HealthCheckResultSchema,
  HealthInfoSchema,
  HealthResponseSchema,
} from "./health_schemas.ts";
export type { HealthHeartbeatSample } from "../../internal_sdk/generated/health/types.ts";
export { HealthHeartbeatSampleSchema } from "../../internal_sdk/generated/health/schemas.ts";
export {
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
  StoreHandle,
  type Trellis,
  TrellisService,
  type TrellisServiceConnectOpts,
} from "./service.ts";

// Re-export subscription types
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
} from "./subscription.ts";
