/**
 * @module
 * Browser-based authentication utilities for session-key based authentication.
 * Uses WebCrypto API and IndexedDB for secure key storage.
 */

export {
  completeSessionLogout,
  type CompleteSessionLogoutArgs,
  logoutSession,
} from "./browser/logout.ts";
export {
  type ApprovalDecision,
  fetchPortalFlowState,
  portalFlowIdFromUrl,
  type PortalFlowState,
  type PortalFlowState as BrowserPortalFlowState,
  portalProviderLoginUrl,
  portalRedirectLocation,
  submitPortalApproval,
} from "./browser/portal.ts";
export {
  clearSessionKey,
  createRpcProof,
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
} from "./browser/session.ts";
export {
  BrowserAuthorizationContextStore,
  deleteKeyPair,
  hasKeyPair,
} from "./browser/storage.ts";
export {
  classifyBrowserAuthError,
  isRecoverableBrowserAuthError,
} from "./browser_recovery.ts";
export type {
  BrowserAuthRecoveryClassification,
  BrowserAuthRecoveryKind,
} from "./browser_recovery.ts";
export {
  base64urlDecode,
  base64urlEncode,
  sha256,
  toArrayBuffer,
  utf8,
} from "./utils.ts";
