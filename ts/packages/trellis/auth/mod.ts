/**
 * Protocol-owned Trellis session keys and proof helpers.
 *
 * @module
 */

export {
  type AuthorizationClientState,
  type AuthorizationContextBundle,
  AuthorizationContextBundleSchema,
  AuthorizationContextCache,
  type AuthorizationContextPersistence,
  AuthorizationContextRefreshError,
  type AuthorizationContextStore,
  AuthorizationProviderCache,
  type AuthorizationProviderEvent,
  type AuthorizationProviderRequest,
  type AuthorizationRoutingMaterial,
  type AuthorizationSessionBinding,
  type AuthorizationTrustBundle,
  AuthorizationTrustBundleSchema,
  type AuthorizationTrustState,
  MemoryAuthorizationContextStore,
  refreshAuthorizationContext,
  refreshAuthorizationContextWithMetadata,
  startAuthorizationContextRefresh,
  type VerifiedAuthorizationContext,
} from "./authorization_context.ts";
export {
  clearSessionKey,
  generateSessionKey,
  getOrCreateSessionKey,
  getPublicSessionKey,
  hasSessionKey,
  loadSessionKey,
  type SessionKeyHandle,
  type SessionKeyOptions,
  type SessionKeyPersistenceMode,
  setSessionId,
  signBytes,
} from "./browser.ts";
export {
  type BrowserAuthRecoveryClassification,
  type BrowserAuthRecoveryKind,
  classifyBrowserAuthError,
  isRecoverableBrowserAuthError,
} from "./browser_recovery.ts";
export {
  buildDeviceActivationPayload,
  deriveDeviceConfirmationCode,
  deriveDeviceIdentity,
  waitForDeviceActivation,
} from "./device_activation.ts";
export {
  fetchPortalFlowState,
  portalFlowIdFromUrl,
  portalProviderLoginUrl,
  portalRedirectLocation,
  submitPortalApproval,
} from "./browser/portal.ts";
export type {
  AuthDeploymentAuthorityGetResponse,
  DeploymentAuthority,
  DeploymentAuthorityCapabilityNeed,
  DeploymentAuthorityContractNeed,
  DeploymentAuthorityKind,
  DeploymentAuthorityMaterialization,
  DeploymentAuthorityNeeds,
  DeploymentAuthorityPlan,
  DeploymentAuthorityPlanBreakingChange,
  DeploymentAuthorityResourceNeed,
  DeploymentAuthoritySurface,
  DeploymentAuthoritySurfaceNeed,
  PortalFlowInsufficientCapabilitiesState,
  PortalFlowState,
} from "./protocol.ts";
export type {
  AuthDeviceUserAuthoritiesResolveOutput,
  AuthDeviceUserAuthoritiesResolveProgress,
} from "../sdk/auth.ts";
// Context-bound proof helpers for local signing and signature verification.
export {
  buildEventProofInput,
  buildProofInput,
  createEventProof,
  createProof,
  type EventProofParams,
  type ProofParams,
  verifyEventProof,
  verifyProof,
} from "./proof.ts";
export {
  buildSessionProofTranscriptV1,
  parseSessionProofV1,
  SESSION_PROOF_FORMAT_V1,
  type SessionProofInputV1,
  type SessionProofPolicyV1,
  type SessionProofPurposeV1,
  sessionProofRequestDigestV1,
  type SessionProofV1,
  signSessionProofV1,
  verifySessionProofV1,
} from "./session_proof.ts";
export {
  createAuth,
  type NatsConnectOptions,
  type TrellisAuth,
} from "./session_auth.ts";
export { correctedIatSeconds, estimateMidpointClockOffsetMs } from "./time.ts";
export { trellisIdFromOriginId } from "./trellis_id.ts";
export {
  base64urlDecode,
  base64urlEncode,
  canonicalizeJsonValue,
  sha256,
  toArrayBuffer,
  utf8,
} from "./utils.ts";
