<script lang="ts">
  import { resolve } from "$app/paths";
  import { afterNavigate } from "$app/navigation";
  import { page } from "$app/state";
  import { onDestroy, onMount } from "svelte";
  import DataTable from "../../../../../lib/components/DataTable.svelte";
  import EmptyState from "../../../../../lib/components/EmptyState.svelte";
  import LoadingState from "../../../../../lib/components/LoadingState.svelte";
  import Notice from "../../../../../lib/components/Notice.svelte";
  import PageToolbar from "../../../../../lib/components/PageToolbar.svelte";
  import Panel from "../../../../../lib/components/Panel.svelte";
  import StatusBadge from "../../../../../lib/components/StatusBadge.svelte";
  import { errorMessage, formatDate } from "../../../../../lib/format";
  import {
    cancelJob,
    dismissDlqJob,
    loadJobDetailData,
    replayDlqJob,
    retryJob,
    type JobInspection,
  } from "../../../../../lib/jobs_page.ts";
  import { getTrellis } from "../../../../../lib/trellis";

  const trellis = getTrellis();
  type Inspection = JobInspection;
  type Job = Inspection["job"];

  const jobId = $derived(page.params.jobid);
  const currentJobId = $derived(jobId ?? "");
  let loading = $state(true);
  let actionBusy = $state<string | null>(null);
  let error = $state<string | null>(null);
  let unavailableMessage = $state<string | null>(null);
  let inspection = $state.raw<Inspection | undefined>(undefined);
  let loadedJobId = $state<string | null>(null);
  let watchController: AbortController | undefined;
  let watchReloadTimer: ReturnType<typeof setTimeout> | undefined;

  const job = $derived(inspection?.job);
  const attempts = $derived(inspection?.attempts ?? []);
  const errors = $derived(inspection?.errors ?? []);
  const related = $derived(inspection?.related ?? []);
  const timeline = $derived(inspection?.timeline ?? []);
  const canCancel = $derived(job?.state === "pending" || job?.state === "retry" || job?.state === "active");
  const canRetry = $derived(job?.state === "failed");
  const canDlq = $derived(job?.state === "dead");

  function stateStatus(state: Job["state"]): "healthy" | "degraded" | "unhealthy" | "offline" {
    switch (state) {
      case "completed":
        return "healthy";
      case "failed":
      case "dead":
      case "expired":
        return "unhealthy";
      case "active":
        return "healthy";
      case "pending":
      case "retry":
        return "degraded";
      default:
        return "offline";
    }
  }

  function jsonBlock(value: unknown): string {
    if (value === undefined) return "-";
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }

  function durationLabel(start: string | undefined, end: string | undefined): string {
    if (!start) return "-";
    const startTime = new Date(start).getTime();
    const endTime = end ? new Date(end).getTime() : Date.now();
    if (Number.isNaN(startTime) || Number.isNaN(endTime) || endTime < startTime) return "-";
    const seconds = Math.floor((endTime - startTime) / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;
    return `${Math.floor(minutes / 60)}h`;
  }

  async function load(id = currentJobId, showLoading = true) {
    loadedJobId = id;
    stopJobsWatch();
    if (showLoading) loading = true;
    error = null;
    unavailableMessage = null;

    try {
      const data = await loadJobDetailData({
        inspect: (input) => trellis.request("Jobs.Inspect", input),
      }, id);
      unavailableMessage = data.available ? null : data.message ?? "Jobs admin runtime is unavailable.";
      inspection = data.inspection;
      if (data.available) startJobsWatch(id);
    } catch (e) {
      error = errorMessage(e);
      unavailableMessage = null;
      inspection = undefined;
    } finally {
      if (showLoading) loading = false;
    }
  }

  function loadCurrentJobIfNeeded() {
    if (!currentJobId || currentJobId === loadedJobId) return;
    void load(currentJobId);
  }

  async function runAction(name: "cancel" | "retry" | "replay" | "dismiss") {
    const actionJobId = job?.id ?? currentJobId;
    actionBusy = name;
    error = null;
    try {
      if (name === "cancel") {
        await cancelJob({ action: (input) => trellis.request("Jobs.Cancel", input) }, actionJobId);
      } else if (name === "retry") {
        await retryJob({ action: (input) => trellis.request("Jobs.Retry", input) }, actionJobId);
      } else if (name === "replay") {
        await replayDlqJob({ action: (input) => trellis.request("Jobs.ReplayDLQ", input) }, actionJobId);
      } else {
        await dismissDlqJob({ action: (input) => trellis.request("Jobs.DismissDLQ", input) }, actionJobId);
      }
      await load(actionJobId);
    } catch (e) {
      error = errorMessage(e);
    } finally {
      actionBusy = null;
    }
  }

  function clearWatchReload() {
    if (!watchReloadTimer) return;
    clearTimeout(watchReloadTimer);
    watchReloadTimer = undefined;
  }

  function scheduleWatchReload(id: string) {
    clearWatchReload();
    watchReloadTimer = setTimeout(() => {
      watchReloadTimer = undefined;
      void load(id, false);
    }, 350);
  }

  function stopJobsWatch() {
    watchController?.abort();
    watchController = undefined;
    clearWatchReload();
  }

  function startJobsWatch(id: string) {
    stopJobsWatch();
    const controller = new AbortController();
    watchController = controller;

    void (async () => {
      try {
        const stream = await trellis.feed.jobs.watch({ includeInitial: false, jobId: id }, { signal: controller.signal }).orThrow();
        for await (const event of stream) {
          if (controller.signal.aborted) return;
          if (event.kind !== "ready") scheduleWatchReload(id);
        }
      } catch {
        // Jobs.Watch is optional; manual refresh remains available.
      }
    })();
  }

  onMount(() => {
    loadCurrentJobIfNeeded();
  });

  afterNavigate(() => {
    loadCurrentJobIfNeeded();
  });

  onDestroy(() => {
    stopJobsWatch();
  });
</script>

<section class="space-y-4">
  <PageToolbar title="Job detail" description="Job identity, timings, payload, result and operator actions.">
    {#snippet meta()}
      {#if job}
        <StatusBadge label={job.state} status={stateStatus(job.state)} />
      {/if}
    {/snippet}
    {#snippet actions()}
      <a class="btn btn-ghost btn-sm" href={resolve("/admin/jobs")}>Back</a>
      <button class="btn btn-ghost btn-sm" onclick={() => load()} disabled={loading || actionBusy !== null}>Refresh</button>
    {/snippet}
  </PageToolbar>

  {#if error}
    <Notice variant="error" role="alert">{error}</Notice>
  {:else if unavailableMessage}
    <Notice variant="info" role="status">{unavailableMessage}</Notice>
  {/if}

  {#if loading}
    <LoadingState label="Loading job" />
  {:else if unavailableMessage}
    <p class="text-xs text-base-content/60">The console can still be used normally without jobs installed.</p>
  {:else if !job}
    <Panel title="Job" eyebrow="Primary">
      <EmptyState title="Job not found" description="No job exists for this id." />
    </Panel>
  {:else}
    <Panel title={job.id} eyebrow="Primary">
      {#snippet actions()}
        <button class="btn btn-outline btn-xs" onclick={() => runAction("cancel")} disabled={!canCancel || actionBusy !== null}>Cancel</button>
        <button class="btn btn-outline btn-xs" onclick={() => runAction("retry")} disabled={!canRetry || actionBusy !== null}>Retry</button>
        <button class="btn btn-outline btn-xs" onclick={() => runAction("replay")} disabled={!canDlq || actionBusy !== null}>Replay DLQ</button>
        <button class="btn btn-outline btn-xs" onclick={() => runAction("dismiss")} disabled={!canDlq || actionBusy !== null}>Dismiss DLQ</button>
      {/snippet}

      <div class="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,24rem)]">
        <div class="space-y-4 min-w-0">
          <DataTable>
              <tbody>
                <tr><th class="w-32">ID</th><td class="trellis-identifier">{job.id}</td></tr>
                <tr><th>Service</th><td class="trellis-identifier">{job.service}</td></tr>
                <tr><th>Type</th><td class="trellis-identifier">{job.type}</td></tr>
                <tr><th>State</th><td><StatusBadge label={job.state} status={stateStatus(job.state)} /></td></tr>
                <tr><th>Tries</th><td class="tabular-nums">{job.tries}/{job.maxTries}</td></tr>
                <tr><th>Last error</th><td class="text-error">{job.lastError ?? "-"}</td></tr>
              </tbody>
          </DataTable>

          <div>
            <h2 class="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-base-content/50">Logs</h2>
            {#if job.logs && job.logs.length > 0}
              <DataTable>
                  <thead><tr><th>Time</th><th>Level</th><th>Message</th></tr></thead>
                  <tbody>
                    {#each job.logs as log (`${log.timestamp}:${log.message}`)}
                      <tr>
                        <td class="text-xs text-base-content/60">{formatDate(log.timestamp)}</td>
                        <td><span class="badge badge-ghost badge-xs">{log.level}</span></td>
                        <td>{log.message}</td>
                      </tr>
                    {/each}
                  </tbody>
              </DataTable>
            {:else}
              <p class="text-sm text-base-content/60">No logs recorded.</p>
            {/if}
          </div>

          <div>
            <h2 class="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-base-content/50">Timeline</h2>
            {#if timeline.length > 0}
              <DataTable>
                  <thead><tr><th>Time</th><th>State</th><th>Message</th></tr></thead>
                  <tbody>
                    {#each timeline as event (event.sequence)}
                      <tr>
                        <td class="text-xs text-base-content/60">{formatDate(event.timestamp)}</td>
                        <td><StatusBadge label={event.state} status={stateStatus(event.state)} /></td>
                        <td>{event.message ?? event.reason ?? event.error ?? event.type}</td>
                      </tr>
                    {/each}
                  </tbody>
              </DataTable>
            {:else}
              <p class="text-sm text-base-content/60">No timeline entries recorded.</p>
            {/if}
          </div>
        </div>

        <div class="space-y-4">
          <DataTable>
              <tbody>
                <tr><th>Created</th><td>{formatDate(job.createdAt)}</td></tr>
                <tr><th>Updated</th><td>{formatDate(job.updatedAt)}</td></tr>
                <tr><th>Started</th><td>{formatDate(job.startedAt)}</td></tr>
                <tr><th>Completed</th><td>{formatDate(job.completedAt)}</td></tr>
                <tr><th>Deadline</th><td>{formatDate(job.deadline)}</td></tr>
                <tr><th>Duration</th><td>{durationLabel(job.startedAt, job.completedAt)}</td></tr>
              </tbody>
          </DataTable>

          <div>
            <h2 class="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-base-content/50">Progress</h2>
            <pre class="max-h-48 overflow-auto rounded-box bg-base-200 p-3 text-xs">{jsonBlock(job.progress)}</pre>
          </div>

          <div>
            <h2 class="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-base-content/50">Attempts</h2>
            {#if attempts.length > 0}
              <DataTable size="xs">
                  <tbody>
                    {#each attempts as attempt (attempt.try)}
                      <tr><th>Try {attempt.try}</th><td>{attempt.state ?? "-"}</td><td>{durationLabel(attempt.startedAt, attempt.endedAt)}</td></tr>
                    {/each}
                  </tbody>
              </DataTable>
            {:else}
              <p class="text-sm text-base-content/60">No attempts recorded.</p>
            {/if}
          </div>
        </div>
      </div>

      <div class="mt-4 grid gap-4 lg:grid-cols-2">
        <div>
          <h2 class="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-base-content/50">Payload</h2>
          <pre class="max-h-96 overflow-auto rounded-box bg-base-200 p-3 text-xs">{jsonBlock(job.payload)}</pre>
        </div>
        <div>
          <h2 class="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-base-content/50">Result</h2>
          <pre class="max-h-96 overflow-auto rounded-box bg-base-200 p-3 text-xs">{jsonBlock(job.result)}</pre>
        </div>
      </div>

      <div class="mt-4 grid gap-4 lg:grid-cols-2">
        <div>
          <h2 class="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-base-content/50">Errors</h2>
          <pre class="max-h-96 overflow-auto rounded-box bg-base-200 p-3 text-xs">{jsonBlock(errors)}</pre>
        </div>
        <div>
          <h2 class="mb-2 text-xs font-semibold uppercase tracking-[0.12em] text-base-content/50">Related</h2>
          <pre class="max-h-96 overflow-auto rounded-box bg-base-200 p-3 text-xs">{jsonBlock(related)}</pre>
        </div>
      </div>
    </Panel>
  {/if}
</section>
