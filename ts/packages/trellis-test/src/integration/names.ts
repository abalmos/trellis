/**
 * Returns a stable ASCII slug safe for contract IDs, subjects, deployment IDs,
 * and participant or resource names.
 *
 * The slug keeps ASCII letters, digits, `_`, and `-`; replaces `.` with `-`;
 * and replaces all other characters with `-`.
 */
export function integrationSlug(caseId: string): string {
  return caseId.replaceAll(".", "-").replaceAll(/[^a-zA-Z0-9_-]/g, "-");
}

/** Returns a deterministic deployment ID for one case in a shared runtime run. */
export function caseDeploymentId(runId: string, caseId: string): string {
  return `js-it-${runId}-${integrationSlug(caseId)}`;
}
