/**
 * Authorization context facade.
 *
 * The implementation is split by responsibility under `auth/authorization/`;
 * this module preserves the existing import surface for Trellis clients.
 */

export {
  AuthorizationContextCache,
  authorizationContextVerificationPolicy,
  verifyAuthorizationContext,
} from "./authorization/client_context.ts";
export {
  AuthorizationProviderCache,
  type AuthorizationProviderCacheHealth,
  type AuthorizationProviderCacheOptions,
  type AuthorizationProviderIoCounters,
} from "./authorization/provider_cache.ts";
export {
  AuthorizationContextRefreshError,
  refreshAuthorizationContext,
  refreshAuthorizationContextWithMetadata,
  startAuthorizationContextRefresh,
} from "./authorization/refresh.ts";
export {
  MemoryAuthorizationContextStore,
  validateAuthorizationClientStateTransition,
} from "./authorization/store.ts";
export type {
  AuthorizationClientState,
  AuthorizationContextBundle,
  AuthorizationContextRefreshResponse,
  AuthorizationContextRefreshResult,
  AuthorizationContextVerificationMaterial,
  AuthorizationProviderEventV2,
  AuthorizationProviderRequestV2,
  AuthorizationRoutingMaterial,
  AuthorizationRuntimeBinding,
  AuthorizationRuntimeTransports,
  AuthorizationSessionBinding,
  AuthorizationTrustBundle,
  AuthorizationTrustState,
  VerifiedAuthorizationContext,
} from "./authorization/types.ts";
export type {
  AuthorizationContextPersistence,
  AuthorizationContextStore,
} from "./authorization/store.ts";
export {
  AuthorizationContextBundleSchema,
  AuthorizationContextRefreshResponseSchema,
  AuthorizationTrustBundleSchema,
} from "./authorization/types.ts";
