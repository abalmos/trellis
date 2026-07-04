<script lang="ts">
  import { resolve } from "$app/paths";
  import { onDestroy, onMount } from "svelte";
  import DataTable from "../../../../lib/components/DataTable.svelte";
  import EmptyState from "../../../../lib/components/EmptyState.svelte";
  import LoadingState from "../../../../lib/components/LoadingState.svelte";
  import Notice from "../../../../lib/components/Notice.svelte";
  import PageToolbar from "../../../../lib/components/PageToolbar.svelte";
  import Panel from "../../../../lib/components/Panel.svelte";
  import StatusBadge from "../../../../lib/components/StatusBadge.svelte";
  import { errorMessage, formatDate } from "../../../../lib/format";
  import {
    cancelJob,
    dismissDlqJob,
    loadJobsPageData,
    replayDlqJob,
    retryJob,
  } from "../../../../lib/jobs_page.ts";
  import { getTrellis } from "../../../../lib/trellis";
  import type {
    JobsListServicesOutput,
    JobsQueryInput,
    JobsQueryOutput,
  } from "@qlever-llc/trellis/sdk/jobs";

  type Job = JobsQueryOutput["entries"][number];
  type JobState = Job["state"];
  type ServiceInfo = JobsListServicesOutput["entries"][number];
  type QueryGroup = JobsQueryOutput["groups"][number];
  type QueryStats = JobsQueryOutput["stats"];
  type JobPathname = `/admin/jobs/${string}` & {};
  type RuntimeFilter = "" | "running" | "slow" | "queued" | "terminal";
  type GroupBy = "service" | "type" | "state";
  type SortMode = "queueAge" | "failRate" | "runtime" | "depth" | "recent";

  const trellis = getTrellis();
  const pageLimit = 100;

  const stateOptions: Array<{ value: "" | JobState; label: string }> = [
    { value: "", label: "All states" },
    { value: "pending", label: "Pending" },
    { value: "active", label: "Active" },
    { value: "retry", label: "Retry" },
    { value: "completed", label: "Completed" },
    { value: "failed", label: "Failed" },
    { value: "cancelled", label: "Cancelled" },
    { value: "skipped", label: "Skipped" },
    { value: "stale", label: "Stale" },
    { value: "expired", label: "Expired" },
    { value: "dead", label: "Dead" },
    { value: "dismissed", label: "Dismissed" },
  ];

  const runtimeOptions: Array<{ value: RuntimeFilter; label: string }> = [
    { value: "", label: "Any runtime" },
    { value: "running", label: "Running now" },
    { value: "slow", label: "Slow or stuck" },
    { value: "queued", label: "Queued" },
    { value: "terminal", label: "Terminal" },
  ];

  let loading = $state(true);
  let refreshing = $state(false);
  let actionBusy = $state<string | null>(null);
  let error = $state<string | null>(null);
  let unavailableMessage = $state<string | null>(null);
  let services = $state.raw<ServiceInfo[]>([]);
  let jobs = $state.raw<Job[]>([]);
  let groups = $state.raw<QueryGroup[]>([]);
  let stats = $state.raw<QueryStats>({ byState: {}, total: 0 });
  let selectedJobId = $state<string | null>(null);
  let selectedService = $state("");
  let selectedState = $state<"" | JobState>("");
  let runtimeFilter = $state<RuntimeFilter>("");
  let typeFilter = $state("");
  let searchText = $state("");
  let groupBy = $state<GroupBy>("service");
  let sortMode = $state<SortMode>("queueAge");
  let offset = $state(0);
  let offsetStack = $state.raw<number[]>([]);
  let nextOffset = $state<number | undefined>(undefined);
  let autoRefresh = $state(false);
  let lastUpdated = $state<Date | null>(null);
  let refreshInterval: ReturnType<typeof setInterval> | undefined;
  let watchController: AbortController | undefined;
  let watchReloadTimer: ReturnType<typeof setTimeout> | undefined;
  let loadSequence = 0;

  const pageNumber = $derived(offsetStack.length + 1);
  const pageTypeFilter = $derived(typeFilter.trim());
  const query = $derived(searchText.trim());
  const selectedJob = $derived.by(() => {
    if (jobs.length === 0) return undefined;
    return jobs.find((job) => job.id === selectedJobId) ?? jobs[0];
  });
  const workerCount = $derived.by(() => services.reduce((sum, service) => sum + service.workers.length, 0));
  const filterSummary = $derived.by(() => {
    const parts = [
      selectedService || "all services",
      selectedState || "all states",
      runtimeOptions.find((option) => option.value === runtimeFilter)?.label.toLowerCase() ?? "any runtime",
      pageTypeFilter ? `type ${pageTypeFilter}` : "all types",
      query ? "server search" : "no search",
    ];
    return parts.join(" · ");
  });

  function stateStatus(state: JobState): "healthy" | "degraded" | "unhealthy" | "offline" {
    switch (state) {
      case "completed":
        return "healthy";
      case "failed":
      case "dead":
      case "expired":
      case "stale":
        return "unhealthy";
      case "active":
        return "healthy";
      case "retry":
      case "pending":
        return "degraded";
      default:
        return "offline";
    }
  }

  function ageLabel(value: string | undefined): string {
    if (!value) return "-";
    const time = new Date(value).getTime();
    if (Number.isNaN(time)) return "-";
    return compactDuration(Date.now() - time);
  }

  function compactDuration(ms: number): string {
    if (!Number.isFinite(ms) || ms < 0) return "-";
    const seconds = Math.floor(ms / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 48) return `${hours}h`;
    return `${Math.floor(hours / 24)}d`;
  }

  function durationLabel(job: Job): string {
    return compactDuration(job.runtimeMs ?? 0);
  }

  function jsonBlock(value: unknown): string {
    if (value === undefined) return "-";
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }

  function jobRoute(id: string): JobPathname {
    return `/admin/jobs/${encodeURIComponent(id)}` as JobPathname;
  }

  function sortField(): NonNullable<JobsQueryInput["sort"]>["field"] {
    if (sortMode === "failRate") return "failureRate";
    if (sortMode === "recent") return "updatedAt";
    return sortMode;
  }

  function buildFilter(): JobsQueryInput {
    return {
      groupBy,
      limit: pageLimit,
      offset,
      runtimeBand: runtimeFilter || undefined,
      search: query || undefined,
      service: selectedService || undefined,
      sort: { field: sortField(), direction: "desc" },
      state: selectedState ? [selectedState] : undefined,
      type: pageTypeFilter || undefined,
    };
  }

  async function load(showLoading = true) {
    const sequence = ++loadSequence;
    const filter = buildFilter();
    stopJobsWatch();
    if (showLoading) {
      loading = true;
    } else {
      refreshing = true;
    }
    error = null;
    unavailableMessage = null;

    try {
      const data = await loadJobsPageData({
        listServices: (input) => trellis.request("Jobs.ListServices", input),
        queryJobs: (filter) => trellis.request("Jobs.Query", filter),
      }, filter);

      if (sequence !== loadSequence) return;

      unavailableMessage = data.available ? null : data.message ?? "Jobs admin runtime is unavailable.";
      services = data.services;
      jobs = data.jobs;
      groups = data.groups;
      stats = data.stats;
      selectedJobId = data.jobs.some((job) => job.id === selectedJobId) ? selectedJobId : data.jobs[0]?.id ?? null;
      nextOffset = data.nextOffset;
      lastUpdated = new Date();
      if (data.available) startJobsWatch(filter);
    } catch (e) {
      if (sequence !== loadSequence) return;
      error = errorMessage(e);
      unavailableMessage = null;
      jobs = [];
      groups = [];
      stats = { byState: {}, total: 0 };
      services = [];
      selectedJobId = null;
      nextOffset = undefined;
    } finally {
      if (sequence === loadSequence) {
        loading = false;
        refreshing = false;
      }
    }
  }

  function resetPagination() {
    offset = 0;
    offsetStack = [];
  }

  function applyFilters() {
    resetPagination();
    void load();
  }

  function isJobStateFilter(value: string): value is "" | JobState {
    return stateOptions.some((option) => option.value === value);
  }

  function isRuntimeFilter(value: string): value is RuntimeFilter {
    return runtimeOptions.some((option) => option.value === value);
  }

  function handleServiceFilterChange(event: Event) {
    selectedService = event.currentTarget instanceof HTMLSelectElement ? event.currentTarget.value : "";
    resetPagination();
    void load();
  }

  function handleStateFilterChange(event: Event) {
    const value = event.currentTarget instanceof HTMLSelectElement ? event.currentTarget.value : "";
    selectedState = isJobStateFilter(value) ? value : "";
    resetPagination();
    void load();
  }

  function handleRuntimeFilterChange(event: Event) {
    const value = event.currentTarget instanceof HTMLSelectElement ? event.currentTarget.value : "";
    runtimeFilter = isRuntimeFilter(value) ? value : "";
  }

  function selectJob(job: Job) {
    selectedJobId = job.id;
  }

  function selectJobFromKeyboard(event: KeyboardEvent, job: Job) {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    selectJob(job);
  }

  function goNext() {
    if (nextOffset === undefined) return;
    offsetStack = [...offsetStack, offset];
    offset = nextOffset;
    void load();
  }

  function goPrevious() {
    if (offsetStack.length === 0) return;
    const previous = offsetStack[offsetStack.length - 1];
    offsetStack = offsetStack.slice(0, -1);
    offset = previous ?? 0;
    void load();
  }

  function refreshNow() {
    void load(false);
  }

  function clearAutoRefresh() {
    if (!refreshInterval) return;
    clearInterval(refreshInterval);
    refreshInterval = undefined;
  }

  function clearWatchReload() {
    if (!watchReloadTimer) return;
    clearTimeout(watchReloadTimer);
    watchReloadTimer = undefined;
  }

  function scheduleWatchReload() {
    clearWatchReload();
    watchReloadTimer = setTimeout(() => {
      watchReloadTimer = undefined;
      void load(false);
    }, 350);
  }

  function stopJobsWatch() {
    watchController?.abort();
    watchController = undefined;
    clearWatchReload();
  }

  function startJobsWatch(filter: JobsQueryInput) {
    stopJobsWatch();
    const controller = new AbortController();
    watchController = controller;

    void (async () => {
      try {
        const stream = await trellis.feed.jobs.watch({ includeInitial: false, query: filter }, { signal: controller.signal }).orThrow();
        for await (const event of stream) {
          if (controller.signal.aborted) return;
          if (event.kind !== "ready") scheduleWatchReload();
        }
      } catch {
        // Jobs.Watch is optional; manual refresh remains available.
      }
    })();
  }

  function handleAutoRefreshChange(event: Event) {
    const checked = event.currentTarget instanceof HTMLInputElement ? event.currentTarget.checked : false;
    autoRefresh = checked;
    clearAutoRefresh();
    if (!checked) return;
    refreshInterval = setInterval(() => {
      void load(false);
    }, 10000);
  }

  async function runAction(name: "cancel" | "retry" | "replay" | "dismiss") {
    if (!selectedJob) return;
    actionBusy = name;
    error = null;
    try {
      if (name === "cancel") {
        await cancelJob({ action: (input) => trellis.request("Jobs.Cancel", input) }, selectedJob.id);
      } else if (name === "retry") {
        await retryJob({ action: (input) => trellis.request("Jobs.Retry", input) }, selectedJob.id);
      } else if (name === "replay") {
        await replayDlqJob({ action: (input) => trellis.request("Jobs.ReplayDLQ", input) }, selectedJob.id);
      } else {
        await dismissDlqJob({ action: (input) => trellis.request("Jobs.DismissDLQ", input) }, selectedJob.id);
      }
      await load(false);
    } catch (e) {
      error = errorMessage(e);
    } finally {
      actionBusy = null;
    }
  }

  onMount(() => {
    void load();
  });

  onDestroy(() => {
    clearAutoRefresh();
    stopJobsWatch();
  });
</script>

<section class="space-y-4">
  <PageToolbar title="Jobs" description="Grouped service-private work for local triage, queue review, and runtime timeline.">
    {#snippet meta()}
      <span class="badge badge-ghost badge-sm">Page {pageNumber}</span>
      <span class="badge badge-neutral badge-sm">{jobs.length} loaded</span>
      {#if lastUpdated}
        <span class="text-xs text-base-content/50">Updated {lastUpdated.toLocaleTimeString()}</span>
      {/if}
    {/snippet}
    {#snippet actions()}
      <label class="flex items-center gap-2 text-xs text-base-content/70">
        <input class="toggle toggle-xs" type="checkbox" checked={autoRefresh} onchange={handleAutoRefreshChange} />
        Auto refresh
      </label>
      <button class="btn btn-ghost btn-sm" onclick={refreshNow} disabled={loading || refreshing || !!unavailableMessage}>
        {refreshing ? "Refreshing" : "Refresh"}
      </button>
    {/snippet}
  </PageToolbar>

  <form class="trellis-filterbar" onsubmit={(event) => { event.preventDefault(); applyFilters(); }}>
    <div class="trellis-filterbar-controls grow">
      <label class="trellis-field min-w-[min(100%,28rem)] grow">
        <span class="trellis-field-label">Command search</span>
        <input
          class="input input-bordered input-sm"
          placeholder="Paste error, job id, trigger, service, or job type"
          bind:value={searchText}
          disabled={loading || !!unavailableMessage}
        />
        <span class="trellis-field-help">Sent to Jobs.Query as server-side search.</span>
      </label>
      <label class="trellis-field">
        <span class="trellis-field-label">Status</span>
        <select class="select select-bordered select-sm" value={selectedState} onchange={handleStateFilterChange} disabled={loading || !!unavailableMessage}>
          {#each stateOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </label>
      <label class="trellis-field">
        <span class="trellis-field-label">Runtime</span>
        <select class="select select-bordered select-sm" value={runtimeFilter} onchange={handleRuntimeFilterChange} disabled={loading || !!unavailableMessage}>
          {#each runtimeOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </label>
      <label class="trellis-field">
        <span class="trellis-field-label">Service</span>
        <select class="select select-bordered select-sm" value={selectedService} onchange={handleServiceFilterChange} disabled={loading || !!unavailableMessage}>
          <option value="">All services</option>
          {#each services as service (service.name)}
            <option value={service.name}>{service.name}</option>
          {/each}
        </select>
      </label>
      <label class="trellis-field">
        <span class="trellis-field-label">Job type</span>
        <input class="input input-bordered input-sm" placeholder="Exact type" bind:value={typeFilter} disabled={loading || !!unavailableMessage} />
      </label>
      <label class="trellis-field">
        <span class="trellis-field-label">Queue/key</span>
        <input class="input input-bordered input-sm" placeholder="Use command search" disabled />
      </label>
      <label class="trellis-field">
        <span class="trellis-field-label">Trigger/context</span>
        <input class="input input-bordered input-sm" placeholder="Use command search" disabled />
      </label>
    </div>
    <div class="trellis-filterbar-actions">
      <label class="trellis-field">
        <span class="trellis-field-label">Group</span>
        <select class="select select-bordered select-sm" bind:value={groupBy} disabled={loading || !!unavailableMessage}>
          <option value="service">Service</option>
          <option value="type">Job type</option>
          <option value="state">State</option>
        </select>
      </label>
      <label class="trellis-field">
        <span class="trellis-field-label">Sort</span>
        <select class="select select-bordered select-sm" bind:value={sortMode} disabled={loading || !!unavailableMessage}>
          <option value="queueAge">Queue age</option>
          <option value="failRate">Failure rate</option>
          <option value="runtime">Runtime</option>
          <option value="depth">Depth</option>
          <option value="recent">Recent</option>
        </select>
      </label>
      <button class="btn btn-outline btn-sm" disabled={loading || !!unavailableMessage}>Apply server filters</button>
    </div>
  </form>

  {#if error}
    <Notice variant="error" role="alert">{error}</Notice>
  {:else if unavailableMessage}
    <Notice variant="info" role="status">{unavailableMessage}</Notice>
  {/if}

  <div class="jobs-workbench">
    <div class="space-y-4 min-w-0">
      <Panel title="Queue topology" eyebrow="Primary">
        {#snippet actions()}
          <span class="badge badge-ghost badge-sm">{filterSummary}</span>
        {/snippet}

        {#if loading}
          <LoadingState label="Loading jobs" />
        {:else if unavailableMessage}
          <p class="text-xs text-base-content/60">The console can still be used normally without jobs installed.</p>
        {:else if jobs.length === 0}
          <EmptyState title="No jobs found" description="Jobs will appear here when the Jobs API reports active or retained work." />
        {:else}
          <div class="mb-3 grid gap-2 text-xs text-base-content/70 sm:grid-cols-4">
            <div class="ops-stat"><span>Total</span><strong>{stats.total}</strong></div>
            <div class="ops-stat"><span>Queued</span><strong>{stats.queued ?? 0}</strong></div>
            <div class="ops-stat"><span>Running</span><strong>{stats.running ?? 0}</strong></div>
            <div class="ops-stat"><span>Workers seen</span><strong>{workerCount}</strong></div>
          </div>
          <DataTable fixed wrapperClass="jobs-topology-table">
            <thead>
              <tr>
                <th>Family / job</th>
                <th class="w-32">Status</th>
                <th class="w-24">Runtime</th>
                <th class="w-24">Queue age</th>
                <th class="w-20">Depth</th>
                <th class="w-24">Tries</th>
              </tr>
            </thead>
            <tbody>
              {#each groups as group (group.key)}
                <tr class="group-row">
                  <td>
                    <div class="flex items-center gap-2">
                      <span class="badge badge-ghost badge-xs">{groupBy}</span>
                      <span class="font-semibold">{group.label}</span>
                      <span class="text-xs text-base-content/50">{group.count} jobs</span>
                    </div>
                  </td>
                  <td>
                    <span class="text-xs tabular-nums text-base-content/70">{group.failureRate === undefined ? "-" : `${(group.failureRate * 100).toFixed(1)}% fail`}</span>
                  </td>
                  <td class="text-xs tabular-nums text-base-content/70">-</td>
                  <td class="text-xs tabular-nums text-base-content/70">{group.oldestCreatedAt ? ageLabel(group.oldestCreatedAt) : "-"}</td>
                  <td class="text-xs tabular-nums text-base-content/70">{group.depth ?? group.count}</td>
                  <td></td>
                </tr>
              {/each}
              {#each jobs as job (job.id)}
                <tr
                  class={['job-row hover', selectedJob?.id === job.id && 'selected']}
                  tabindex="0"
                  onkeydown={(event) => selectJobFromKeyboard(event, job)}
                  onclick={() => selectJob(job)}
                >
                  <td class="min-w-0">
                    <div class="flex min-w-0 flex-col gap-1">
                      <div class="flex min-w-0 items-center gap-2">
                        <button class="btn btn-ghost btn-xs px-1" type="button" onclick={(event) => { event.stopPropagation(); selectJob(job); }}>Inspect</button>
                        <a class="link link-hover trellis-identifier truncate" href={resolve(jobRoute(job.id))} onclick={(event) => event.stopPropagation()}>{job.type}</a>
                      </div>
                      <span class="trellis-identifier text-base-content/45">{job.id}</span>
                    </div>
                  </td>
                  <td><StatusBadge label={job.state} status={stateStatus(job.state)} /></td>
                  <td class="text-xs tabular-nums text-base-content/70">{durationLabel(job)}</td>
                  <td class="text-xs tabular-nums text-base-content/70">{compactDuration(job.queueAgeMs ?? 0)}</td>
                  <td class="text-xs tabular-nums text-base-content/70">{job.queueKey ? "keyed" : "1"}</td>
                  <td class="text-xs tabular-nums text-base-content/70">{job.tries}/{job.maxTries}</td>
                </tr>
              {/each}
            </tbody>
          </DataTable>
        {/if}

        {#snippet footer()}
          <div class="flex items-center justify-between gap-3">
            <span>{jobs.length} shown from {stats.total} jobs</span>
            <div class="join">
              <button class="btn btn-outline btn-xs join-item" onclick={goPrevious} disabled={loading || offsetStack.length === 0}>Previous</button>
              <button class="btn btn-outline btn-xs join-item" onclick={goNext} disabled={loading || nextOffset === undefined}>Next</button>
            </div>
          </div>
        {/snippet}
      </Panel>

    </div>

    <aside class="job-inspector">
      <Panel title={selectedJob?.type ?? "Job inspector"} eyebrow={selectedJob ? selectedJob.service : "Select a job"}>
        {#snippet actions()}
          {#if selectedJob}
            <a class="btn btn-ghost btn-xs" href={resolve(jobRoute(selectedJob.id))}>Full detail</a>
          {/if}
        {/snippet}

        {#if !selectedJob}
          <EmptyState title="Select a job" description="Choose a job row to inspect logs, context, trigger, retries, and controls." />
        {:else}
          <div class="space-y-4">
            <div class="flex flex-wrap items-center gap-2">
              <StatusBadge label={selectedJob.state} status={stateStatus(selectedJob.state)} />
              <span class="badge badge-ghost badge-sm">runtime {durationLabel(selectedJob)}</span>
              <span class="badge badge-ghost badge-sm">tries {selectedJob.tries}/{selectedJob.maxTries}</span>
            </div>

            <DataTable size="xs" overflow="none">
              <tbody>
                <tr><th>Service</th><td class="trellis-identifier">{selectedJob.service}</td></tr>
                <tr><th>Job ID</th><td class="trellis-identifier break-anywhere">{selectedJob.id}</td></tr>
                <tr><th>Trigger</th><td class="trellis-identifier break-anywhere">{selectedJob.trigger?.id ?? selectedJob.context?.requestId ?? "-"}</td></tr>
                <tr><th>Trace</th><td class="trellis-identifier break-anywhere">{selectedJob.context?.traceId ?? selectedJob.trigger?.traceId ?? "-"}</td></tr>
                <tr><th>Queue key</th><td class="trellis-identifier break-anywhere">{selectedJob.queueKey ?? "unkeyed"}</td></tr>
              </tbody>
            </DataTable>

            {#if selectedJob.lastError}
              <div class="timeline-box error-box">
                <div class="flex items-center justify-between gap-3">
                  <h3>Error</h3>
                  <span class="badge badge-error badge-xs">captured</span>
                </div>
                <p class="trellis-identifier break-anywhere">{selectedJob.lastError}</p>
              </div>
            {/if}

            <div class="action-panel">
              <h3>Action controls</h3>
              <button class="btn btn-primary btn-sm" onclick={() => runAction("retry")} disabled={selectedJob.state !== "failed" || actionBusy !== null}>
                {actionBusy === "retry" ? "Retrying" : "Restart job"}
              </button>
              <p class="text-xs text-base-content/55">{selectedJob.state === "failed" ? "Creates a new run with the same job identity." : "Available only for failed jobs."}</p>
              <button class="btn btn-outline btn-sm" onclick={() => runAction("cancel")} disabled={!(selectedJob.state === "pending" || selectedJob.state === "retry" || selectedJob.state === "active") || actionBusy !== null}>Cancel job</button>
              <p class="text-xs text-base-content/55">{selectedJob.state === "failed" ? "Cannot cancel a job that already failed" : "Cancels pending, retrying, or active work."}</p>
              <button class="btn btn-outline btn-sm" disabled>Stop job</button>
              <p class="text-xs text-base-content/55">{selectedJob.state === "active" ? "Stop is not exposed by the current Jobs API." : "Cannot stop a job that is not running"}</p>
              {#if selectedJob.state === "dead"}
                <div class="join w-full">
                  <button class="btn btn-outline btn-xs join-item flex-1" onclick={() => runAction("replay")} disabled={actionBusy !== null}>Replay DLQ</button>
                  <button class="btn btn-outline btn-xs join-item flex-1" onclick={() => runAction("dismiss")} disabled={actionBusy !== null}>Dismiss DLQ</button>
                </div>
              {/if}
            </div>

            <div class="inspector-grid">
              <section class="timeline-section">
                <h3>Input / context</h3>
                <pre>{jsonBlock({ context: selectedJob.context, progress: selectedJob.progress, queueKey: selectedJob.queueKey, trigger: selectedJob.trigger })}</pre>
              </section>
              <section class="timeline-section">
                <h3>Retry history</h3>
                <DataTable size="xs" overflow="none">
                  <tbody>
                    <tr><th>Attempt</th><td>{selectedJob.tries}</td></tr>
                    <tr><th>Max</th><td>{selectedJob.maxTries}</td></tr>
                    <tr><th>Last update</th><td>{formatDate(selectedJob.updatedAt)}</td></tr>
                  </tbody>
                </DataTable>
              </section>
            </div>

          </div>
        {/if}
      </Panel>
    </aside>
  </div>
</section>

<style>
  .jobs-workbench {
    display: grid;
    gap: 1rem;
    grid-template-columns: minmax(0, 1fr);
  }

  @media (min-width: 1280px) {
    .jobs-workbench {
      grid-template-columns: minmax(0, 1fr) minmax(24rem, 32rem);
    }

    .job-inspector {
      position: sticky;
      top: 1rem;
      align-self: start;
    }
  }

  .ops-stat {
    background: color-mix(in oklab, var(--color-base-100) 78%, var(--color-base-200));
    border: 1px solid color-mix(in oklab, var(--color-base-300) 78%, transparent);
    border-radius: 0.85rem;
    display: grid;
    gap: 0.1rem;
    padding: 0.65rem 0.75rem;
  }

  .ops-stat span {
    color: color-mix(in oklab, var(--color-base-content) 54%, transparent);
    font-size: 0.72rem;
    font-style: normal;
  }

  .ops-stat strong {
    font-size: 1rem;
    font-variant-numeric: tabular-nums;
    line-height: 1.2;
  }

  .group-row {
    background: color-mix(in oklab, var(--color-base-content) 3%, transparent);
  }

  .job-row {
    cursor: pointer;
  }

  .job-row.selected {
    background: color-mix(in oklab, var(--color-accent) 10%, transparent);
    outline: 1px solid color-mix(in oklab, var(--color-accent) 46%, transparent);
    outline-offset: -1px;
  }

  .job-row:focus-visible {
    box-shadow: var(--trellis-focus-ring);
    outline: none;
  }

  .timeline-box,
  .action-panel,
  .timeline-section {
    border: 1px solid color-mix(in oklab, var(--color-base-300) 76%, transparent);
    border-radius: 0.85rem;
    padding: 0.8rem;
  }

  .error-box {
    background: color-mix(in oklab, var(--color-error) 8%, var(--color-base-100));
    border-color: color-mix(in oklab, var(--color-error) 36%, var(--color-base-300));
  }

  .action-panel {
    display: grid;
    gap: 0.55rem;
  }

  .action-panel h3,
  .timeline-section h3,
  .timeline-box h3 {
    color: color-mix(in oklab, var(--color-base-content) 74%, transparent);
    font-size: 0.72rem;
    font-weight: 800;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .inspector-grid {
    display: grid;
    gap: 0.75rem;
  }

  .timeline-section pre {
    background: color-mix(in oklab, var(--color-base-200) 64%, var(--color-base-100));
    border-radius: 0.65rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 0.72rem;
    line-height: 1.5;
    margin-top: 0.65rem;
    max-height: 18rem;
    overflow: auto;
    padding: 0.75rem;
    white-space: pre-wrap;
  }

  .break-anywhere {
    overflow-wrap: anywhere;
    white-space: normal;
  }
</style>
