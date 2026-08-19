/** Default total worker budget for live integration tests. */
export function defaultLiveJobs(
  logicalCpus = navigator.hardwareConcurrency,
): number {
  const cpus = Number.isFinite(logicalCpus)
    ? Math.max(1, Math.floor(logicalCpus))
    : 1;
  return Math.max(1, Math.ceil(cpus / 2));
}
