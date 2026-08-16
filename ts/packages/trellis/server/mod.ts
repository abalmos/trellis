/**
 * Trellis service package entry point.
 *
 * This package is service-side glue: it re-exports the RPC/event hosting runtime
 * plus common service helpers (health checks, subscription types).
 *
 * @module
 */

export { TrellisServiceRuntime } from "../server.ts";
export type { TrellisServiceRuntimeFor } from "../server.ts";
// Re-export health types
export {
  createHealthHeartbeatSample,
  type HealthCheckFn,
  type HealthCheckResult,
  type HealthResponse,
  runAllHealthChecks,
  runAllServiceHealthChecks,
  runHealthCheck,
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
  HealthRpcSchema,
} from "./health_schemas.ts";
export type { HealthHeartbeatSample } from "../sdk/_generated/health/types.ts";
export { HealthHeartbeatSampleSchema } from "../sdk/_generated/health/schemas.ts";
export {
  type FeedHandler,
  type HealthCheckHandler,
  type HealthInfoHandler,
  type JobHandler,
  type JobQueue,
  type JobsFacadeOf,
  type OperationHandler,
  type OperationRegistration,
  type RpcHandler,
  type ServiceContract,
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
