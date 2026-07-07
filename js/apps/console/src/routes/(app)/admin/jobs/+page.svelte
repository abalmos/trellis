<script lang="ts">
  import { resolve } from "$app/paths";
  import { onDestroy, onMount } from "svelte";
  import DataTable from "../../../../lib/components/DataTable.svelte";
  import EmptyState from "../../../../lib/components/EmptyState.svelte";
  import JobsHealthMatrix from "../../../../lib/components/JobsHealthMatrix.svelte";
  import JobsScopedCharts from "../../../../lib/components/JobsScopedCharts.svelte";
  import LoadingState from "../../../../lib/components/LoadingState.svelte";
  import Notice from "../../../../lib/components/Notice.svelte";
  import PageToolbar from "../../../../lib/components/PageToolbar.svelte";
  import Panel from "../../../../lib/components/Panel.svelte";
  import InlineMetricsStrip from "../../../../lib/components/InlineMetricsStrip.svelte";
  import { errorMessage, jobStateStatus, compactDuration } from "../../../../lib/format";
  import {
    loadJobsPageData,
  } from "../../../../lib/jobs_page.ts";
  import { loadJobsMetrics, type JobsMetrics } from "../../../../lib/jobs_metrics.ts";
  import { getTrellis } from "../../../../lib/trellis";
  import type {
    JobsListServicesOutput,
    JobsMetricsInput,
    JobsQueryInput,
    JobsQueryOutput,
  } from "@qlever-llc/trellis/sdk/jobs";

  type Job = JobsQueryOutput["entries"][number];
  type JobState = Job["state"];
  type ServiceInfo = JobsListServicesOutput["entries"][number];
  type QueryStats = JobsQueryOutput["stats"];
  type JobPathname = `/admin/jobs/${string}` & {};
  type RuntimeFilter = "" | "running" | "slow" | "queued" | "terminal";
  type GroupBy = "service" | "type" | "state";
  type SortMode = "queueAge" | "failRate" | "runtime" | "depth" | "recent";

  const trellis = getTrellis();
  const pageLimit = 100;

  const statePills: Array<{ value: "" | JobState; label: string }> = [
    { value: "", label: "All" },
    { value: "active", label: "Active" },
    { value: "failed", label: "Failed" },
    { value: "dead", label: "Dead" },
    { value: "stale", label: "Stale" },
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
  let error = $state<string | null>(null);
  let unavailableMessage = $state<string | null>(null);
  let services = $state.raw<ServiceInfo[]>([]);
  let jobs = $state.raw<Job[]>([]);
  let stats = $state.raw<QueryStats>({ byState: {}, total: 0 });
  let metrics = $state.raw<JobsMetrics | null>(null);
  let metricsError = $state<string | null>(null);
  let metricsWindow = $state<"15m" | "1h" | "6h" | "24h" | "7d">("1h");
  let selectedJobType = $state<string | null>(null);
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
  let lastUpdated = $state<Date | null>(null);
  let metricsSequence = 0;
  let loadSequence = 0;

  const pageNumber = $derived(offsetStack.length + 1);
  const query = $derived(searchText.trim());
  const workerCount = $derived.by(() => services.reduce((sum, service) => sum + service.workers.length, 0));
  const stateStatus = jobStateStatus;

  function stateBadgeClass(state: JobState): string {
    switch (state) {
      case "completed":
      case "active":
        return "badge-success";
      case "failed":
      case "dead":
      case "expired":
        return "badge-error";
      case "stale":
      case "dismissed":
        return "badge-warning";
      case "retry":
      case "pending":
        return "badge-info";
      default:
        return "badge-ghost";
    }
  }

  function jobRuntimeLabel(job: Job): string {
    return compactDuration(job.runtimeMs ?? 0);
  }

  function jobQueueAgeLabel(job: Job): string {
    return compactDuration(job.queueAgeMs ?? 0);
  }

  function sortField(): NonNullable<JobsQueryInput["sort"]>["field"] {
    if (sortMode === "failRate") return "failureRate";
    if (sortMode === "recent") return "updatedAt";
    return sortMode;
  }

  function resolveMetricsStep(window: typeof metricsWindow): JobsMetricsInput["step"] {
    if (window === "15m" || window === "1h") return "1m";
    if (window === "6h") return "5m";
    if (window === "24h") return "15m";
    return "1h";
  }

  function buildMetricsInput(): JobsMetricsInput {
    return {
      groupBy: "type",
      service: selectedService || undefined,
      state: selectedState ? [selectedState] : undefined,
      step: resolveMetricsStep(metricsWindow),
      window: metricsWindow,
    };
  }

  async function loadMetrics() {
    const sequence = ++metricsSequence;
    metricsError = null;
    try {
      const payload = await loadJobsMetrics(
        { metrics: (request) => trellis.request("Jobs.Metrics", request) },
        buildMetricsInput(),
      );
      if (sequence !== metricsSequence) return;
      if (!payload.available) {
        metrics = null;
        metricsError = payload.message ?? "Jobs metrics are unavailable.";
        return;
      }
      metrics = payload.metrics ?? null;
      if (selectedJobType && metrics && !metrics.summary.some((group) => group.key === selectedJobType)) {
        selectedJobType = null;
      }
    } catch (e) {
      if (sequence !== metricsSequence) return;
      metrics = null;
      metricsError = errorMessage(e);
    }
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
      type: typeFilter.trim() || undefined,
    };
  }

  async function load(showLoading = true) {
    const sequence = ++loadSequence;
    const filter = buildFilter();
    if (showLoading) loading = true;
    else refreshing = true;
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
      stats = data.stats;
      nextOffset = data.nextOffset;
      lastUpdated = new Date();
    } catch (e) {
      if (sequence !== loadSequence) return;
      error = errorMessage(e);
      unavailableMessage = null;
      jobs = [];
      stats = { byState: {}, total: 0 };
      services = [];
      nextOffset = undefined;
    } finally {
      if (sequence === loadSequence) {
        loading = false;
        refreshing = false;
      }
    }
    void loadMetrics();
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
    return statePills.some((option) => option.value === value);
  }

  function isRuntimeFilter(value: string): value is RuntimeFilter {
    return runtimeOptions.some((option) => option.value === value);
  }

  function selectState(value: "" | JobState) {
    selectedState = value;
    resetPagination();
    void load();
  }

  function handleServiceFilterChange(event: Event) {
    selectedService = event.currentTarget instanceof HTMLSelectElement ? event.currentTarget.value : "";
    resetPagination();
    void load();
  }

  function handleStateFilterChange(event: Event) {
    const value = event.currentTarget instanceof HTMLSelectElement ? event.currentTarget.value : "";
    selectState(isJobStateFilter(value) ? value : "");
  }

  function handleRuntimeFilterChange(event: Event) {
    const value = event.currentTarget instanceof HTMLSelectElement ? event.currentTarget.value : "";
    runtimeFilter = isRuntimeFilter(value) ? value : "";
  }

  function handleMetricsWindowChange(event: Event) {
    const value = event.currentTarget instanceof HTMLSelectElement ? event.currentTarget.value : "1h";
    if (value === "15m" || value === "1h" || value === "6h" || value === "24h" || value === "7d") {
      metricsWindow = value;
      void loadMetrics();
    }
  }

  function selectJobType(key: string | null) {
    if (key === null) {
      selectedJobType = null;
      return;
    }
    selectedJobType = key;
    if (typeFilter !== key) {
      typeFilter = key;
      resetPagination();
      void load();
    }
  }

  function jobRoute(id: string): JobPathname {
    return `/admin/jobs/${encodeURIComponent(id)}` as JobPathname;
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

  function clearAll() {
    selectedJobType = null;
    selectedState = "";
    selectedService = "";
    runtimeFilter = "";
    typeFilter = "";
    searchText = "";
    resetPagination();
    void load();
  }

  onMount(() => {
    void load();
  });

  onDestroy(() => {
    // load() now uses straight RPCs; no streaming controller to tear down.
  });
</script>

<section class="space-y-4">
  <PageToolbar title="Jobs" description="Trellis job-type health, scoped diagnostics, and individual job triage.">
    {#snippet meta()}
      <span class="badge badge-ghost badge-sm">Page {pageNumber}</span>
      <span class="badge badge-neutral badge-sm">{jobs.length} loaded</span>
      {#if lastUpdated}
        <span class="text-xs text-base-content/50">Updated {lastUpdated.toLocaleTimeString()}</span>
      {/if}
    {/snippet}
    {#snippet actions()}
      <button class="btn btn-ghost btn-sm" onclick={refreshNow} disabled={loading || refreshing || !!unavailableMessage}>
        {refreshing ? "Refreshing" : "Refresh"}
      </button>
    {/snippet}
  </PageToolbar>

  {#if !loading && !error && !unavailableMessage}
    {@const metricsStrip = [
      { label: 'Total', value: stats.total, badge: `${stats.byState.active ?? 0} active`, badgeClass: 'badge-neutral' as const },
      { label: 'Queued', value: stats.queued ?? 0, badge: `${stats.byState.pending ?? 0} pending`, badgeClass: 'badge-neutral' as const },
      { label: 'Running', value: stats.running ?? 0, badge: `${stats.byState.slow ?? 0} slow`, badgeClass: 'badge-neutral' as const },
      { label: 'Failed', value: stats.byState.failed ?? 0, badge: `${stats.byState.dead ?? 0} dead`, badgeClass: (stats.byState.failed ?? 0) > 0 ? 'badge-error' as const : 'badge-neutral' as const },
      { label: 'Workers', value: workerCount, detail: 'services' },
    ]}
    <InlineMetricsStrip metrics={metricsStrip} />
  {/if}

  <div class="jobs-filterbar">
    <div class="jobs-filterbar-primary">
      <input
        class="input input-bordered input-sm jobs-search-input"
        placeholder="Search error, job id, trigger, service, or type"
        bind:value={searchText}
        onchange={applyFilters}
        disabled={loading || !!unavailableMessage}
      />

      <div class="jobs-state-pills">
        {#each statePills as pill (pill.value)}
          <button
            type="button"
            class={['jobs-state-pill', selectedState === pill.value && 'active']}
            onclick={() => selectState(pill.value)}
            disabled={loading || !!unavailableMessage}
          >
            {pill.label}
          </button>
        {/each}
      </div>

      <label class="jobs-inline-select">
        <span class="jobs-inline-label">Service</span>
        <select
          class="select select-bordered select-xs"
          value={selectedService}
          onchange={handleServiceFilterChange}
          disabled={loading || !!unavailableMessage}
        >
          <option value="">All</option>
          {#each services as service (service.name)}
            <option value={service.name}>{service.name}</option>
          {/each}
        </select>
      </label>

      <label class="jobs-inline-select">
        <span class="jobs-inline-label">Window</span>
        <select
          class="select select-bordered select-xs"
          value={metricsWindow}
          onchange={handleMetricsWindowChange}
          disabled={!!unavailableMessage}
        >
          <option value="15m">15m</option>
          <option value="1h">1h</option>
          <option value="6h">6h</option>
          <option value="24h">24h</option>
          <option value="7d">7d</option>
        </select>
      </label>

      <details class="jobs-more-filters">
        <summary class="jobs-more-filters-trigger">More filters</summary>
        <div class="jobs-more-filters-popover">
          <label class="jobs-inline-select">
            <span class="jobs-inline-label">Type</span>
            <input
              class="input input-bordered input-xs jobs-type-input"
              placeholder="Exact type"
              bind:value={typeFilter}
              onchange={applyFilters}
              disabled={loading || !!unavailableMessage}
            />
          </label>
          <label class="jobs-inline-select">
            <span class="jobs-inline-label">Group</span>
            <select
              class="select select-bordered select-xs"
              bind:value={groupBy}
              onchange={applyFilters}
              disabled={loading || !!unavailableMessage}
            >
              <option value="service">Service</option>
              <option value="type">Job type</option>
              <option value="state">State</option>
            </select>
          </label>
          <label class="jobs-inline-select">
            <span class="jobs-inline-label">Sort</span>
            <select
              class="select select-bordered select-xs"
              bind:value={sortMode}
              onchange={applyFilters}
              disabled={loading || !!unavailableMessage}
            >
              <option value="queueAge">Queue age</option>
              <option value="failRate">Failure rate</option>
              <option value="runtime">Runtime</option>
              <option value="depth">Depth</option>
              <option value="recent">Recent</option>
            </select>
          </label>
          <label class="jobs-inline-select">
            <span class="jobs-inline-label">Runtime</span>
            <select
              class="select select-bordered select-xs"
              value={runtimeFilter}
              onchange={handleRuntimeFilterChange}
              disabled={loading || !!unavailableMessage}
            >
              {#each runtimeOptions as option (option.value)}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </label>
        </div>
      </details>
    </div>

    {#if selectedJobType || selectedState || selectedService || runtimeFilter || typeFilter.trim() || query}
      <div class="jobs-active-filters">
        {#if selectedJobType}
          <button type="button" class="badge badge-sm badge-primary cursor-pointer" onclick={() => selectJobType(null)}>
            Type drill: {selectedJobType} ×
          </button>
        {/if}
        {#if selectedState}
          <button type="button" class="badge badge-sm badge-outline cursor-pointer" onclick={() => selectState("")}>
            State: {selectedState} ×
          </button>
        {/if}
        {#if selectedService}
          <button type="button" class="badge badge-sm badge-outline cursor-pointer" onclick={() => { selectedService = ''; resetPagination(); void load(); }}>
            Service: {selectedService} ×
          </button>
        {/if}
        {#if runtimeFilter}
          <button type="button" class="badge badge-sm badge-outline cursor-pointer" onclick={() => { runtimeFilter = ''; resetPagination(); void load(); }}>
            Runtime: {runtimeFilter} ×
          </button>
        {/if}
        {#if typeFilter.trim() && typeFilter.trim() !== selectedJobType}
          <button type="button" class="badge badge-sm badge-outline cursor-pointer" onclick={() => { typeFilter = ''; resetPagination(); void load(); }}>
            Type: {typeFilter} ×
          </button>
        {/if}
        {#if query}
          <button type="button" class="badge badge-sm badge-outline cursor-pointer" onclick={() => { searchText = ''; resetPagination(); void load(); }}>
            Search: {query} ×
          </button>
        {/if}
        <button type="button" class="text-xs text-base-content/50 underline" onclick={clearAll}>
          Clear all
        </button>
      </div>
    {/if}
  </div>

  {#if error}
    <Notice variant="error" role="alert">{error}</Notice>
  {:else if unavailableMessage}
    <Notice variant="info" role="status">{unavailableMessage}</Notice>
  {/if}

  {#if metricsError}
    <Notice variant="info" role="status">{metricsError}</Notice>
  {/if}

  {#if metrics}
    <JobsHealthMatrix
      summary={metrics.summary}
      buckets={metrics.buckets}
      selectedKey={selectedJobType}
      onSelect={selectJobType}
    />
    {#if selectedJobType}
      <JobsScopedCharts buckets={metrics.buckets} selectedKey={selectedJobType} />
    {/if}
  {/if}

  <Panel eyebrow="Primary" title="Jobs">
    {#if loading}
      <LoadingState label="Loading jobs" />
    {:else if unavailableMessage}
      <p class="text-xs text-base-content/60">The console can still be used normally without jobs installed.</p>
    {:else if jobs.length === 0}
      <EmptyState title="No jobs found" description="Jobs will appear here when the Jobs API reports active or retained work." />
    {:else}
      <DataTable fixed wrapperClass="jobs-topology-table">
        <thead>
          <tr>
            <th>Job</th>
            <th class="w-32">Status</th>
            <th class="w-24">Runtime</th>
            <th class="w-24">Queue age</th>
            <th class="w-20">Tries</th>
          </tr>
        </thead>
        <tbody>
          {#each jobs as job (job.id)}
            <tr class="job-row hover">
              <td class="min-w-0">
                <div class="flex min-w-0 flex-col gap-1">
                  <a class="link link-hover trellis-identifier truncate" href={resolve(jobRoute(job.id))}>{job.type}</a>
                  <span class="trellis-identifier text-base-content/45">{job.id}</span>
                </div>
              </td>
              <td><span class={['badge badge-sm', stateBadgeClass(job.state)]}>{job.state}</span></td>
              <td class="text-xs tabular-nums text-base-content/70">{jobRuntimeLabel(job)}</td>
              <td class="text-xs tabular-nums text-base-content/70">{jobQueueAgeLabel(job)}</td>
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
</section>

  <style>
    .jobs-filterbar {
    background: color-mix(in oklab, var(--color-base-100) 78%, var(--color-base-200));
    border: 1px solid color-mix(in oklab, var(--color-base-300) 82%, transparent);
    border-radius: var(--radius-box, 1rem);
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 0.65rem 0.85rem;
  }

  .jobs-filterbar-primary {
    align-items: center;
    display: flex;
    flex-wrap: wrap;
    gap: 0.55rem;
  }

  .jobs-search-input {
    flex: 1 1 16rem;
    min-width: 12rem;
  }

  .jobs-state-pills {
    display: inline-flex;
    align-items: center;
    gap: 0.1rem;
    border-left: 1px solid color-mix(in oklab, var(--color-base-300) 60%, transparent);
    border-right: 1px solid color-mix(in oklab, var(--color-base-300) 60%, transparent);
    padding: 0 0.45rem;
  }

  .jobs-state-pill {
    background: transparent;
    border: none;
    border-radius: var(--radius-field, 0.5rem);
    color: color-mix(in oklab, var(--color-base-content) 65%, transparent);
    cursor: pointer;
    font-size: 0.78rem;
    padding: 0.25rem 0.55rem;
  }

  .jobs-state-pill:hover:not(:disabled) {
    background: color-mix(in oklab, var(--color-base-300) 30%, transparent);
    color: var(--color-base-content);
  }

  .jobs-state-pill.active {
    background: var(--color-base-content);
    color: var(--color-base-100);
  }

  .jobs-state-pill:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .jobs-inline-select {
    align-items: center;
    display: inline-flex;
    gap: 0.35rem;
  }

  .jobs-inline-label {
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    font-size: 0.7rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .jobs-type-input {
    width: 8rem;
  }

  .jobs-more-filters {
    position: relative;
  }

  .jobs-more-filters summary {
    list-style: none;
    cursor: pointer;
  }

  .jobs-more-filters summary::-webkit-details-marker {
    display: none;
  }

  .jobs-more-filters-trigger {
    color: color-mix(in oklab, var(--color-base-content) 65%, transparent);
    font-size: 0.78rem;
    padding: 0.3rem 0.55rem;
    border-radius: var(--radius-field, 0.5rem);
    border: 1px solid color-mix(in oklab, var(--color-base-300) 70%, transparent);
  }

  .jobs-more-filters-trigger:hover {
    background: color-mix(in oklab, var(--color-base-300) 30%, transparent);
    color: var(--color-base-content);
  }

  .jobs-more-filters[open] > .jobs-more-filters-trigger {
    background: color-mix(in oklab, var(--color-base-300) 30%, transparent);
    color: var(--color-base-content);
  }

  .jobs-more-filters-popover {
    position: absolute;
    top: calc(100% + 0.35rem);
    right: 0;
    z-index: 30;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 16rem;
    max-width: min(28rem, 90vw);
    background: color-mix(in oklab, var(--color-base-100) 95%, var(--color-base-200));
    border: 1px solid color-mix(in oklab, var(--color-base-300) 75%, transparent);
    border-radius: var(--radius-box, 0.75rem);
    box-shadow: 0 8px 24px -12px color-mix(in oklab, var(--color-base-content) 25%, transparent);
    padding: 0.65rem 0.75rem;
  }

  .jobs-active-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    align-items: center;
  }

  .job-row {
    cursor: pointer;
  }

  .job-row:hover {
    background: color-mix(in oklab, var(--color-accent) 6%, transparent);
  }

  .job-row:focus-visible {
    box-shadow: var(--trellis-focus-ring);
    outline: none;
  }

  @media (max-width: 640px) {
    .jobs-filterbar-primary {
      align-items: stretch;
    }

    .jobs-search-input {
      width: 100%;
    }
  }
</style>
