/** Return whether an auth error asks the browser bind to retry convergence. */
export function isAuthorizationPending(value: unknown): boolean {
  if (value === null || typeof value !== "object" || !("error" in value)) {
    return false;
  }
  const error = value.error;
  return error !== null && typeof error === "object" && "code" in error &&
    error.code === "authorization_pending";
}
