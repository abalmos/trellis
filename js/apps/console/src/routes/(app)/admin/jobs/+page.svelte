<script lang="ts">
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import type {
    JobsListServicesOutput,
    JobsMetricsInput,
    JobsMetricsOutput,
    JobsQueryInput,
    JobsQueryOutput,
  } from "@qlever-llc/trellis/sdk/jobs";
  import EmptyState from "../../../../lib/components/EmptyState.svelte";
  import JobsHealthMatrix from "../../../../lib/components/JobsHealthMatrix.svelte";
  import JobsScopedCharts from "../../../../lib/components/JobsScopedCharts.svelte";
  import LoadingState from "../../../../lib/components/LoadingState.svelte";
  import Notice from "../../../../lib/components/Notice.svelte";
  import PageToolbar from "../../../../lib/components/PageToolbar.svelte";
  import { compactDuration, errorMessage } from "../../../../lib/format";
  import { loadJobsMetrics } from "../../../../lib/jobs_metrics.ts";
  import { loadJobsPageData } from "../../../../lib/jobs_page.ts";
  import { getTrellis } from "../../../../lib/trellis";

  type Job = JobsQueryOutput["entries"][number];
  type JobState = Job["state"];
  type ServiceInfo = JobsListServicesOutput["entries"][number];
  type MetricsWindow = JobsMetricsInput["window"];
  type Focus = "running-risk" | "running" | "action" | "completed" | "failed" | "dead" | "backlog";
  type JobPathname = `/admin/jobs/${string}` & {};

  const trellis = getTrellis();
  const rpcTimeout = 10_000;
  const windows: Array<{ value: MetricsWindow; label: string; title: string }> = [
    { value: "15m", label: "15m", title: "Last 15 minutes" },
    { value: "1h", label: "1h", title: "Last hour" },
    { value: "6h", label: "6h", title: "Last 6 hours" },
    { value: "24h", label: "24h", title: "Last 24 hours" },
    { value: "7d", label: "7d", title: "Last 7 days" },
  ];

  let loading = $state(true);
  let metricsLoading = $state(true);
  let refreshing = $state(false);
  let error = $state<string | null>(null);
  let metricsError = $state<string | null>(null);
  let unavailableMessage = $state<string | null>(null);
  let services = $state.raw<ServiceInfo[]>([]);
  let jobs = $state.raw<Job[]>([]);
  let jobCount = $state(0);
  let metrics = $state.raw<JobsMetricsOutput | null>(null);
  let metricsWindow = $state<MetricsWindow>("1h");
  let selectedJobType = $state<string | null>(null);
  let focus = $state<Focus>("running-risk");
  let lastUpdated = $state<Date | null>(null);
  let jobsSequence = 0;
  let metricsSequence = 0;

  const workerCount = $derived(services.reduce((sum, service) => sum + service.workers.length, 0));
  const windowLabel = $derived(windows.find((option) => option.value === metricsWindow)?.title ?? metricsWindow);
  const focusedJobs = $derived.by(() =>
    focus === "running-risk"
      ? [...jobs].sort((left, right) => riskPriority(left) - riskPriority(right) || (right.runtimeMs ?? 0) - (left.runtimeMs ?? 0))
      : jobs,
  );
  const overview = $derived.by(() => {
    const byState: Record<string, number> = {};
    let backlog = 0;
    let running = 0;
    let slow = 0;
    for (const group of metrics?.summary ?? []) {
      backlog += group.queued ?? 0;
      running += group.running ?? 0;
      slow += group.slow ?? 0;
      for (const [state, count] of Object.entries(group.byState)) byState[state] = (byState[state] ?? 0) + count;
    }
    const processed = metrics?.buckets.reduce(
      (total, bucket) => total + bucket.groups.reduce((sum, group) => sum + group.completed, 0),
      0,
    ) ?? 0;
    const failed = byState.failed ?? 0;
    const dead = byState.dead ?? 0;
    const retrying = byState.retry ?? 0;
    const stale = byState.stale ?? 0;
    return {
      action: failed + dead + retrying + stale,
      backlog,
      dead,
      failed,
      failureRate: processed > 0 ? ((failed + dead) / processed) * 100 : 0,
      processed,
      retrying,
      running,
      slow,
      stale,
    };
  });

  function resolveMetricsStep(window: MetricsWindow): JobsMetricsInput["step"] {
    if (window === "15m" || window === "1h") return "1m";
    if (window === "6h") return "5m";
    if (window === "24h") return "15m";
    return "1h";
  }

  function focusStates(value: Focus): JobState[] {
    if (value === "running-risk") return ["active", "retry"];
    if (value === "running") return ["active"];
    if (value === "action") return ["retry", "failed", "dead", "stale"];
    if (value === "completed") return ["completed"];
    if (value === "failed") return ["failed"];
    if (value === "dead") return ["dead"];
    return ["pending", "retry"];
  }

  function focusTitle(): string {
    const prefix = selectedJobType ? `${selectedJobType} · ` : "";
    if (focus === "running-risk") return `${prefix}Running and at risk`;
    if (focus === "running") return `${prefix}Running jobs`;
    if (focus === "action") return `${prefix}Jobs needing action`;
    if (focus === "completed") return `${prefix}Completed jobs`;
    if (focus === "failed") return `${prefix}Failed jobs`;
    if (focus === "dead") return `${prefix}Dead-lettered jobs`;
    return `${prefix}Backlog`;
  }

  function focusDescription(): string {
    if (focus === "running-risk") return "Waiting, slow, and retrying work first";
    if (focus === "running") return "Current active execution";
    if (focus === "action") return "Retrying, failed, dead, and stale work";
    if (focus === "completed") return "Most recently completed retained work";
    if (focus === "failed") return "Non-retryable failures available for inspection";
    if (focus === "dead") return "Exhausted deliveries awaiting replay or dismissal";
    return "Pending and retrying work, oldest first";
  }

  function buildQuery(): JobsQueryInput {
    return {
      limit: 40,
      state: focusStates(focus),
      type: selectedJobType ?? undefined,
      sort: {
        direction: "desc",
        field: focus === "backlog" ? "queueAge" : focus === "running" || focus === "running-risk" ? "runtime" : "updatedAt",
      },
    };
  }

  async function loadJobs(showLoading = true) {
    const sequence = ++jobsSequence;
    if (showLoading) loading = true;
    error = null;
    unavailableMessage = null;
    try {
      const data = await loadJobsPageData({
        listServices: (input) => trellis.request("Jobs.ListServices", input, { timeout: rpcTimeout }),
        queryJobs: (input) => trellis.request("Jobs.Query", input, { timeout: rpcTimeout }),
      }, buildQuery());
      if (sequence !== jobsSequence) return;
      unavailableMessage = data.available ? null : data.message ?? "Jobs admin runtime is unavailable.";
      services = data.services;
      jobs = data.jobs;
      jobCount = data.count;
    } catch (cause) {
      if (sequence !== jobsSequence) return;
      error = errorMessage(cause);
      jobs = [];
      jobCount = 0;
      services = [];
    } finally {
      if (sequence === jobsSequence) loading = false;
    }
  }

  async function loadMetrics() {
    const sequence = ++metricsSequence;
    metricsLoading = true;
    metricsError = null;
    try {
      const payload = await loadJobsMetrics(
        { metrics: (input) => trellis.request("Jobs.Metrics", input, { timeout: rpcTimeout }) },
        { groupBy: "type", step: resolveMetricsStep(metricsWindow), window: metricsWindow },
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
        void loadJobs(false);
      }
    } catch (cause) {
      if (sequence !== metricsSequence) return;
      metrics = null;
      metricsError = errorMessage(cause);
    } finally {
      if (sequence === metricsSequence) metricsLoading = false;
    }
  }

  async function refresh(showLoading = false) {
    if (!showLoading) refreshing = true;
    await Promise.all([loadJobs(showLoading), loadMetrics()]);
    lastUpdated = new Date();
    refreshing = false;
  }

  function selectFocus(value: Focus) {
    focus = value;
    void loadJobs(false);
  }

  function selectJobType(value: string | null) {
    selectedJobType = value;
    void loadJobs(false);
  }

  function selectWindow(value: MetricsWindow) {
    metricsWindow = value;
    void loadMetrics();
  }

  function jobRoute(id: string): JobPathname {
    return `/admin/jobs/${encodeURIComponent(id)}` as JobPathname;
  }

  function jobStateLabel(job: Job): string {
    if (job.state === "active" && (job.waitingOn?.length ?? 0) > 0) return "waiting";
    if (job.state === "active" && job.runtimeBand === "slow") return "slow";
    return job.state;
  }

  function riskPriority(job: Job): number {
    if ((job.waitingOn?.length ?? 0) > 0) return 0;
    if (job.state === "retry") return 1;
    if (job.runtimeBand === "slow") return 2;
    return 3;
  }

  function jobNarrative(job: Job): string {
    const wait = job.waitingOn?.[0];
    if (wait) return wait.label ?? wait.target.label ?? `Waiting on ${wait.target.kind}`;
    return job.progress?.message ?? job.progress?.step ?? job.lastError ?? "Execution in progress";
  }

  function jobDuration(job: Job): string {
    const value = job.state === "pending" || job.state === "retry" ? job.queueAgeMs : job.runtimeMs;
    return compactDuration(value ?? 0);
  }

  function jobProgress(job: Job): number {
    const current = job.progress?.current;
    const total = job.progress?.total;
    if (current === undefined || total === undefined || total <= 0) return 0;
    return Math.min(100, Math.max(0, (current / total) * 100));
  }

  onMount(() => {
    void refresh(true);
  });
</script>

<svelte:head><title>Jobs · Trellis Console</title></svelte:head>

<section class="jobs-page">
  <PageToolbar title="Jobs" description="Execution health across services and job types.">
    {#snippet actions()}
      <div class="jobs-window" aria-label="Metrics window">
        {#each windows as option (option.value)}
          <button
            type="button"
            class:active={metricsWindow === option.value}
            aria-pressed={metricsWindow === option.value}
            onclick={() => selectWindow(option.value)}
          >{option.label}</button>
        {/each}
      </div>
      <button class="btn btn-outline btn-sm" onclick={() => void refresh()} disabled={loading || refreshing}>
        {refreshing ? "Refreshing" : "Refresh"}
      </button>
    {/snippet}
  </PageToolbar>

  {#if lastUpdated}
    <p class="jobs-updated">Updated {lastUpdated.toLocaleTimeString()}</p>
  {/if}

  {#if error}
    <Notice variant="error" role="alert">Jobs could not be loaded. {error}</Notice>
  {:else if unavailableMessage}
    <Notice variant="info" role="status">{unavailableMessage} Job processing can continue while visibility is unavailable.</Notice>
  {/if}

  {#if metricsError}
    <Notice variant="info" role="status">{metricsError}</Notice>
  {/if}

  {#if metricsLoading && !metrics}
    <LoadingState label="Loading job health" />
  {:else if metrics}
    <div class="jobs-ledger" aria-label="Jobs status summary">
      <button class:active={focus === "action"} class="attention" onclick={() => selectFocus("action")}>
        <span><i></i>Action needed</span><strong>{overview.action}</strong><small>{overview.failed} failed · {overview.dead} dead · {overview.retrying} retrying</small>
      </button>
      <button class:active={focus === "running" || focus === "running-risk"} onclick={() => selectFocus("running")}>
        <span><i class="healthy"></i>Running</span><strong>{overview.running}</strong><small>{overview.slow} slow · {workerCount} workers</small>
      </button>
      <button class:active={focus === "completed"} onclick={() => selectFocus("completed")}>
        <span><i class="healthy"></i>Processed</span><strong>{overview.processed.toLocaleString()}</strong><small>completed · {windowLabel.toLowerCase()}</small>
      </button>
      <button class:active={focus === "failed"} onclick={() => selectFocus("failed")}>
        <span><i></i>Failed</span><strong>{overview.failed}</strong><small>{overview.failureRate.toFixed(2)}% of completed</small>
      </button>
      <button class:active={focus === "dead"} onclick={() => selectFocus("dead")}>
        <span><i></i>Dead</span><strong>{overview.dead}</strong><small>requires replay or dismissal</small>
      </button>
      <button class:active={focus === "backlog"} onclick={() => selectFocus("backlog")}>
        <span><i class="warning"></i>Backlog</span><strong>{overview.backlog}</strong><small>pending + retrying</small>
      </button>
    </div>

    <div class="jobs-overview">
      <section class="jobs-matrix-section">
        <header class="jobs-section-heading">
          <div><h2>Job-type health</h2><p>Pressure and latency by execution contract. Select a type to scope live work.</p></div>
          <span>{metrics.summary.length} types reporting</span>
        </header>
        <JobsHealthMatrix summary={metrics.summary} buckets={metrics.buckets} selectedKey={selectedJobType} onSelect={selectJobType} />
      </section>
      <JobsScopedCharts buckets={metrics.buckets} selectedKey={selectedJobType} {windowLabel} />
    </div>
  {/if}

  {#if !unavailableMessage}
    <section class="jobs-focused">
      <header class="jobs-section-heading">
        <div><h2>{focusTitle()}</h2><p>{focusDescription()}</p></div>
        <div class="jobs-focused-meta">
          <span>{jobCount} matching</span>
          {#if selectedJobType}
            <button type="button" onclick={() => selectJobType(null)}>Clear type</button>
          {/if}
        </div>
      </header>

      {#if loading}
        <LoadingState label="Loading focused jobs" class="min-h-32" />
      {:else if jobs.length === 0}
        <EmptyState title="No matching jobs" description="No retained jobs match this operational view." />
      {:else}
        <div class="jobs-list">
          {#each focusedJobs as job (job.id)}
            {@const progress = jobProgress(job)}
            <a class="jobs-row" href={resolve(jobRoute(job.id))}>
              <span class="jobs-row-identity"><strong>{job.type}</strong><small>{job.id} · {job.service}</small></span>
              <span class="jobs-row-key">{job.queueKey ?? "Unkeyed"}</span>
              <span class={["jobs-row-state", `state-${jobStateLabel(job)}`]}>{jobStateLabel(job)}</span>
              <span class="jobs-row-narrative">
                <strong>{jobNarrative(job)}</strong>
                {#if job.progress?.step}<small>{job.progress.step}</small>{/if}
                {#if progress > 0}<span class="jobs-progress"><i style:--job-progress={`${progress}%`}></i></span>{/if}
              </span>
              <span class="jobs-row-duration"><small>{job.state === "pending" || job.state === "retry" ? "Queue age" : "Runtime"}</small>{jobDuration(job)}</span>
              <span class="jobs-row-tries"><small>Attempt</small>{job.tries}/{job.maxTries}</span>
            </a>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</section>

<style>
  .jobs-page {
    display: grid;
    gap: 1.5rem;
  }

  .jobs-page :global(.mb-5) {
    margin-bottom: 0;
  }

  .jobs-updated {
    color: color-mix(in oklab, var(--color-base-content) 45%, transparent);
    font-size: 0.68rem;
    margin: -1.15rem 0 -0.75rem;
    text-align: right;
  }

  .jobs-window {
    background: color-mix(in oklab, var(--color-base-200) 75%, var(--color-base-100));
    border: 1px solid color-mix(in oklab, var(--color-base-300) 85%, transparent);
    border-radius: var(--radius-field, 0.5rem);
    display: flex;
    padding: 0.18rem;
  }

  .jobs-window button {
    border: 0;
    border-radius: calc(var(--radius-field, 0.5rem) - 0.15rem);
    color: color-mix(in oklab, var(--color-base-content) 58%, transparent);
    cursor: pointer;
    font-size: 0.7rem;
    font-weight: 650;
    min-height: 1.8rem;
    padding: 0 0.65rem;
  }

  .jobs-window button.active {
    background: var(--color-base-100);
    box-shadow: 0 1px 2px color-mix(in oklab, var(--color-base-content) 12%, transparent);
    color: var(--color-base-content);
  }

  .jobs-window button:focus-visible {
    box-shadow: var(--trellis-focus-ring);
    outline: none;
  }

  .jobs-ledger {
    border-bottom: 1px solid color-mix(in oklab, var(--color-base-300) 85%, transparent);
    border-top: 1px solid color-mix(in oklab, var(--color-base-300) 85%, transparent);
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
  }

  .jobs-ledger > button {
    background: transparent;
    border: 0;
    border-right: 1px solid color-mix(in oklab, var(--color-base-300) 75%, transparent);
    display: grid;
    gap: 0.2rem;
    min-width: 0;
    padding: 0.8rem 1rem;
    text-align: left;
  }

  .jobs-ledger > :last-child {
    border-right: 0;
  }

  .jobs-ledger > button {
    cursor: pointer;
    transition: background-color 150ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .jobs-ledger > button:hover {
    background: color-mix(in oklab, var(--color-base-200) 55%, transparent);
  }

  .jobs-ledger > button.active {
    background: color-mix(in oklab, var(--color-primary) 9%, var(--color-base-100));
    box-shadow: inset 0 -2px color-mix(in oklab, var(--color-primary) 75%, var(--color-base-content));
  }

  .jobs-ledger > button.attention.active {
    background: color-mix(in oklab, var(--color-error) 9%, var(--color-base-100));
    box-shadow: inset 0 -2px color-mix(in oklab, var(--color-error) 75%, var(--color-base-content));
  }

  .jobs-ledger > button:focus-visible {
    box-shadow: var(--trellis-focus-ring);
    outline: none;
  }

  .jobs-ledger span {
    align-items: center;
    color: color-mix(in oklab, var(--color-base-content) 58%, transparent);
    display: flex;
    font-size: 0.62rem;
    font-weight: 720;
    gap: 0.35rem;
    letter-spacing: 0.055em;
    text-transform: uppercase;
  }

  .jobs-ledger i {
    background: var(--color-error);
    border-radius: 50%;
    height: 0.35rem;
    width: 0.35rem;
  }

  .jobs-ledger i.healthy {
    background: var(--color-success);
  }

  .jobs-ledger i.warning {
    background: var(--color-warning);
  }

  .jobs-ledger strong {
    font-size: 1.35rem;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.035em;
    line-height: 1.15;
  }

  .jobs-ledger small {
    color: color-mix(in oklab, var(--color-base-content) 45%, transparent);
    font-size: 0.62rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .jobs-overview {
    display: grid;
    gap: 1.5rem;
    grid-template-columns: minmax(0, 1.6fr) minmax(14rem, 0.52fr);
  }

  .jobs-matrix-section,
  .jobs-focused {
    min-width: 0;
  }

  .jobs-section-heading {
    align-items: flex-end;
    display: flex;
    gap: 1rem;
    justify-content: space-between;
    margin-bottom: 0.7rem;
  }

  .jobs-section-heading h2 {
    font-size: 0.88rem;
    font-weight: 720;
    margin: 0;
  }

  .jobs-section-heading p,
  .jobs-section-heading > span,
  .jobs-focused-meta {
    color: color-mix(in oklab, var(--color-base-content) 52%, transparent);
    font-size: 0.68rem;
    margin: 0.15rem 0 0;
  }

  .jobs-focused-meta {
    align-items: center;
    display: flex;
    gap: 0.75rem;
  }

  .jobs-focused-meta button {
    background: transparent;
    border: 0;
    color: color-mix(in oklab, var(--color-primary) 75%, var(--color-base-content));
    cursor: pointer;
    font-size: inherit;
    font-weight: 700;
    padding: 0;
  }

  .jobs-list {
    border-top: 1px solid color-mix(in oklab, var(--color-base-300) 85%, transparent);
  }

  .jobs-row {
    align-items: center;
    border-bottom: 1px solid color-mix(in oklab, var(--color-base-300) 75%, transparent);
    color: inherit;
    display: grid;
    gap: 0.85rem;
    grid-template-columns: minmax(11rem, 1.4fr) minmax(6rem, 0.7fr) 5rem minmax(11rem, 1.25fr) 5rem 3.25rem;
    min-height: 3.7rem;
    padding: 0.45rem 0.5rem;
    text-decoration: none;
    transition: background-color 150ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .jobs-row:hover {
    background: color-mix(in oklab, var(--color-base-200) 55%, transparent);
  }

  .jobs-row:focus-visible {
    box-shadow: var(--trellis-focus-ring);
    outline: none;
  }

  .jobs-row-identity,
  .jobs-row-narrative,
  .jobs-row-duration,
  .jobs-row-tries {
    min-width: 0;
  }

  .jobs-row-identity strong {
    display: block;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .jobs-row small {
    color: color-mix(in oklab, var(--color-base-content) 43%, transparent);
    display: block;
    font-size: 0.6rem;
    margin-top: 0.15rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .jobs-row-key {
    color: color-mix(in oklab, var(--color-base-content) 58%, transparent);
    font-size: 0.68rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .jobs-row-state {
    align-items: center;
    display: flex;
    font-size: 0.68rem;
    font-weight: 680;
    gap: 0.35rem;
  }

  .jobs-row-state::before {
    background: var(--color-primary);
    border-radius: 50%;
    content: "";
    height: 0.35rem;
    width: 0.35rem;
  }

  .jobs-row-state.state-waiting::before,
  .jobs-row-state.state-slow::before,
  .jobs-row-state.state-pending::before {
    background: var(--color-warning);
  }

  .jobs-row-state.state-retry::before,
  .jobs-row-state.state-failed::before,
  .jobs-row-state.state-dead::before,
  .jobs-row-state.state-stale::before {
    background: var(--color-error);
  }

  .jobs-row-narrative > strong {
    display: block;
    font-size: 0.68rem;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .jobs-progress {
    background: color-mix(in oklab, var(--color-base-300) 70%, transparent);
    border-radius: 999px;
    display: block;
    height: 0.22rem;
    margin-top: 0.35rem;
    overflow: hidden;
  }

  .jobs-progress i {
    background: color-mix(in oklab, var(--color-primary) 75%, var(--color-base-content));
    display: block;
    height: 100%;
    width: var(--job-progress);
  }

  .jobs-row-duration,
  .jobs-row-tries {
    font-size: 0.68rem;
    font-variant-numeric: tabular-nums;
  }

  @media (max-width: 1100px) {
    .jobs-ledger {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .jobs-ledger > :nth-child(3) {
      border-right: 0;
    }

    .jobs-ledger > :nth-child(-n + 3) {
      border-bottom: 1px solid color-mix(in oklab, var(--color-base-300) 75%, transparent);
    }

    .jobs-overview {
      grid-template-columns: 1fr;
    }

    .jobs-row {
      grid-template-columns: minmax(10rem, 1.35fr) 5rem minmax(10rem, 1fr) 4.5rem 3rem;
    }

    .jobs-row-key {
      display: none;
    }
  }

  @media (max-width: 700px) {
    .jobs-page {
      gap: 1.25rem;
    }

    .jobs-updated {
      display: none;
    }

    .jobs-window {
      overflow-x: auto;
      width: 100%;
    }

    .jobs-window button {
      flex: 1;
      min-width: 2.75rem;
    }

    .jobs-ledger {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .jobs-ledger > :nth-child(3) {
      border-right: 1px solid color-mix(in oklab, var(--color-base-300) 75%, transparent);
    }

    .jobs-ledger > :nth-child(2n) {
      border-right: 0;
    }

    .jobs-ledger > :nth-child(-n + 4) {
      border-bottom: 1px solid color-mix(in oklab, var(--color-base-300) 75%, transparent);
    }

    .jobs-ledger > button {
      padding: 0.75rem;
    }

    .jobs-section-heading {
      align-items: flex-start;
      flex-direction: column;
      gap: 0.4rem;
    }

    .jobs-row {
      gap: 0.6rem;
      grid-template-columns: 1fr auto;
      padding: 0.75rem 0.25rem;
    }

    .jobs-row-identity,
    .jobs-row-narrative {
      grid-column: 1;
    }

    .jobs-row-state,
    .jobs-row-duration {
      grid-column: 2;
      justify-self: end;
    }

    .jobs-row-narrative {
      grid-row: 2;
    }

    .jobs-row-duration {
      grid-row: 2;
    }

    .jobs-row-tries {
      display: none;
    }
  }
</style>
