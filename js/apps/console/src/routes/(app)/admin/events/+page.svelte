<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import DataTable from "../../../../lib/components/DataTable.svelte";
  import EmptyState from "../../../../lib/components/EmptyState.svelte";
  import InlineMetricsStrip from "../../../../lib/components/InlineMetricsStrip.svelte";
  import LoadingState from "../../../../lib/components/LoadingState.svelte";
  import Notice from "../../../../lib/components/Notice.svelte";
  import PageToolbar from "../../../../lib/components/PageToolbar.svelte";
  import Panel from "../../../../lib/components/Panel.svelte";
  import { compactDuration, errorMessage, formatDate, jsonBlock } from "../../../../lib/format";
  import { getTrellis } from "../../../../lib/trellis";
  import type {
    EventLogConsumersQueryInput,
    EventLogInspectInput,
    EventLogMetricsInput,
    EventLogQueryInput,
  } from "../../../../../../../../generated/packages/jsr/eventlog/mod.ts";

  type Mode = "events" | "consumers";
  type WindowValue = "15m" | "1h" | "6h" | "24h" | "7d";
  type EventResolution = "resolved" | "unresolved" | "malformed";
  type EventVerificationStatus =
    | "verified"
    | "missing-proof"
    | "invalid-signature"
    | "missing-session"
    | "subject-denied"
    | "outside-session-window"
    | "auth-unavailable";
  type EventTypeRef = {
    ownerContractId: string;
    ownerEventName: string;
  };
  type EventTypeMetric = EventTypeRef & { count: number };
  type ConsumerStatus =
    | "current"
    | "processing"
    | "behind"
    | "saturated"
    | "inactive"
    | "failing"
    | "missing"
    | "orphaned";

  type EventLogRow = {
    eventId: string;
    eventTime: string;
    streamSequence: number;
    subject: string;
    ownerContractId?: string;
    ownerEventName?: string;
    resolution: EventResolution;
    verificationStatus: EventVerificationStatus;
    publisherKind?: "service" | "device" | "user";
    publisherDeploymentId?: string;
    publisherInstanceId?: string;
    publisherContractId?: string;
    publisherContractDigest?: string;
    traceId?: string;
    payloadSizeBytes: number;
    headerCount: number;
  };

  type EventInspect = {
    event: EventLogRow;
    headers: Record<string, string>;
    payload?: unknown;
    payloadText?: string;
    decodeError?: string;
    proof?: Record<string, unknown>;
    owner?: Record<string, unknown>;
    publisher?: Record<string, unknown>;
    related: Array<{ eventId: string; eventTime: string; subject: string; matchedBy: string }>;
  };

  type ConsumerRow = {
    deploymentId?: string;
    contractId?: string;
    group?: string;
    stream: string;
    consumerName: string;
    filterSubjects: string[];
    status: ConsumerStatus;
    pending: number;
    ackPending: number;
    waitingPulls: number;
    redelivered?: number;
    concurrency?: number;
    ackWaitMs?: number;
    maxDeliver?: number;
    oldestPendingAt?: string;
    oldestPendingEventId?: string;
  };

  type EventMetrics = {
    eventRatePerMinute?: number;
    eventsInWindow?: number;
    uniqueSubjects?: number;
    unresolvedCount?: number;
    malformedCount?: number;
    consumersBehind?: number;
    oldestLagMs?: number;
    eventTypes: EventTypeMetric[];
  };

  const trellis = getTrellis();
  const pageLimit = 100;
  const windowOptions: WindowValue[] = ["15m", "1h", "6h", "24h", "7d"];
  const verificationOptions: Array<{ value: "" | EventVerificationStatus; label: string }> = [
    { value: "", label: "Any proof" },
    { value: "verified", label: "Verified" },
    { value: "missing-proof", label: "Missing proof" },
    { value: "invalid-signature", label: "Invalid signature" },
    { value: "missing-session", label: "Missing session" },
    { value: "subject-denied", label: "Subject denied" },
    { value: "outside-session-window", label: "Outside session" },
    { value: "auth-unavailable", label: "Auth unavailable" },
  ];
  const consumerSeverity: Record<ConsumerStatus, number> = {
    missing: 0,
    saturated: 1,
    inactive: 2,
    failing: 3,
    behind: 4,
    processing: 5,
    current: 6,
    orphaned: 7,
  };

  let mode = $state<Mode>("events");
  let loading = $state(true);
  let refreshing = $state(false);
  let error = $state<string | null>(null);
  let unavailableMessage = $state<string | null>(null);
  let feedOnline = $state(false);
  let feedMessage = $state<string | null>(null);
  let rows = $state.raw<EventLogRow[]>([]);
  let consumers = $state.raw<ConsumerRow[]>([]);
  let metrics = $state.raw<EventMetrics | null>(null);
  let selectedEvent = $state.raw<EventInspect | null>(null);
  let selectedConsumer = $state.raw<{ row: ConsumerRow; detail: Record<string, unknown> | null } | null>(null);
  let detailLoading = $state(false);
  let detailError = $state<string | null>(null);
  let searchText = $state("");
  let ownerContractId = $state("");
  let publisherDeploymentId = $state("");
  let consumerDeploymentId = $state("");
  let verificationStatus = $state<"" | EventVerificationStatus>("");
  let includeEventTypes = $state.raw<EventTypeRef[]>([]);
  let excludeEventTypes = $state.raw<EventTypeRef[]>([]);
  let windowValue = $state<WindowValue>("1h");
  let offset = $state(0);
  let total = $state(0);
  let consumerTotal = $state(0);
  let lastUpdated = $state<Date | null>(null);

  let loadSequence = 0;
  let watchController: AbortController | null = null;
  let reloadTimer: ReturnType<typeof setTimeout> | null = null;

  const query = $derived(searchText.trim());
  const pageNumber = $derived(Math.floor(offset / pageLimit) + 1);
  const includeEventTypeKeys = $derived(new Set(includeEventTypes.map(eventTypeKey)));
  const excludeEventTypeKeys = $derived(new Set(excludeEventTypes.map(eventTypeKey)));
  const eventTypesByCount = $derived.by(() => [...(metrics?.eventTypes ?? [])].sort((a, b) => b.count - a.count || eventTypeKey(a).localeCompare(eventTypeKey(b))));
  const includeEventTypeOptions = $derived(selectedEventTypesFirst(eventTypesByCount, includeEventTypeKeys));
  const excludeEventTypeOptions = $derived(selectedEventTypesFirst(eventTypesByCount, excludeEventTypeKeys));
  const sortedConsumers = $derived.by(() => [...consumers].sort((a, b) => consumerSeverity[a.status] - consumerSeverity[b.status]));
  const unhealthyConsumers = $derived(consumers.filter((consumer) => consumer.status !== "current" && consumer.status !== "processing"));
  const metricsStrip = $derived([
    { label: "Rate", value: numberLabel(metrics?.eventRatePerMinute), detail: "/ min" },
    { label: "Events", value: metrics?.eventsInWindow ?? total, detail: windowValue },
    { label: "Subjects", value: metrics?.uniqueSubjects ?? "-" },
    { label: "Unresolved", value: (metrics?.unresolvedCount ?? 0) + (metrics?.malformedCount ?? 0), badge: "owner", badgeClass: "badge-neutral" },
    { label: "Consumers", value: metrics?.consumersBehind ?? unhealthyConsumers.length, detail: "behind" },
  ]);

  function objectRecord(value: unknown): Record<string, unknown> {
    return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
  }

  function stringValue(value: unknown): string | undefined {
    return typeof value === "string" && value.length > 0 ? value : undefined;
  }

  function numberValue(value: unknown): number | undefined {
    return typeof value === "number" && Number.isFinite(value) ? value : undefined;
  }

  function stringArray(value: unknown): string[] {
    return Array.isArray(value) ? value.filter((entry): entry is string => typeof entry === "string") : [];
  }

  function numberLabel(value: number | undefined): string {
    return value === undefined ? "-" : value.toLocaleString(undefined, { maximumFractionDigits: 1 });
  }

  function eventTypeKey(eventType: EventTypeRef): string {
    return `${eventType.ownerContractId}\u0000${eventType.ownerEventName}`;
  }

  function selectedEventTypesFirst(eventTypes: EventTypeMetric[], selected: Set<string>): EventTypeMetric[] {
    return [...eventTypes].sort((a, b) => Number(selected.has(eventTypeKey(b))) - Number(selected.has(eventTypeKey(a))));
  }

  function isResolution(value: unknown): value is EventResolution {
    return value === "resolved" || value === "unresolved" || value === "malformed";
  }

  function isVerificationStatus(value: unknown): value is EventVerificationStatus {
    return verificationOptions.some((option) => option.value === value);
  }

  function isConsumerStatus(value: unknown): value is ConsumerStatus {
    return Object.hasOwn(consumerSeverity, String(value));
  }

  function isEventLogRow(value: unknown): value is EventLogRow {
    const row = objectRecord(value);
    return typeof row.eventId === "string" &&
      typeof row.eventTime === "string" &&
      typeof row.streamSequence === "number" &&
      typeof row.subject === "string" &&
      isResolution(row.resolution) &&
      isVerificationStatus(row.verificationStatus) &&
      typeof row.payloadSizeBytes === "number" &&
      typeof row.headerCount === "number";
  }

  function toConsumerRow(value: unknown): ConsumerRow | null {
    const row = objectRecord(value);
    const stream = stringValue(row.stream);
    const consumerName = stringValue(row.consumerName);
    if (!stream || !consumerName || !isConsumerStatus(row.status)) return null;
    return {
      deploymentId: stringValue(row.deploymentId),
      contractId: stringValue(row.contractId),
      group: stringValue(row.group),
      stream,
      consumerName,
      filterSubjects: stringArray(row.filterSubjects),
      status: row.status,
      pending: numberValue(row.pending) ?? 0,
      ackPending: numberValue(row.ackPending) ?? 0,
      waitingPulls: numberValue(row.waitingPulls) ?? 0,
      redelivered: numberValue(row.redelivered),
      concurrency: numberValue(row.concurrency),
      ackWaitMs: numberValue(row.ackWaitMs),
      maxDeliver: numberValue(row.maxDeliver),
      oldestPendingAt: stringValue(row.oldestPendingAt),
      oldestPendingEventId: stringValue(row.oldestPendingEventId),
    };
  }

  function toMetrics(value: unknown): EventMetrics | null {
    const record = objectRecord(value);
    const summary = objectRecord(record.summary);
    if (Object.keys(summary).length === 0) return null;
    const byResolution = objectRecord(summary.byResolution);
    const rawEventTypes = summary.eventTypes;
    const eventTypes = Array.isArray(rawEventTypes)
      ? rawEventTypes.flatMap((value) => {
        const eventType = objectRecord(value);
        const ownerContractId = stringValue(eventType.ownerContractId);
        const ownerEventName = stringValue(eventType.ownerEventName);
        const count = numberValue(eventType.count);
        return ownerContractId && ownerEventName && count !== undefined ? [{ ownerContractId, ownerEventName, count }] : [];
      })
      : [];
    return {
      eventRatePerMinute: numberValue(summary.eventRatePerMinute),
      eventsInWindow: numberValue(summary.total),
      uniqueSubjects: numberValue(summary.uniqueSubjects),
      unresolvedCount: numberValue(byResolution.unresolved),
      malformedCount: numberValue(byResolution.malformed),
      consumersBehind: numberValue(record.consumersBehind),
      oldestLagMs: numberValue(record.oldestLagMs),
      eventTypes,
    };
  }

  function toInspect(value: unknown): EventInspect | null {
    const record = objectRecord(value);
    if (!isEventLogRow(record.event)) return null;
    const headers: Record<string, string> = {};
    for (const [key, headerValue] of Object.entries(objectRecord(record.headers))) {
      if (typeof headerValue === "string") headers[key] = headerValue;
    }
    const related = Array.isArray(record.related)
      ? record.related.map((entry) => {
        const item = objectRecord(entry);
        return {
          eventId: stringValue(item.eventId) ?? "",
          eventTime: stringValue(item.eventTime) ?? "",
          subject: stringValue(item.subject) ?? "",
          matchedBy: stringValue(item.matchedBy) ?? "related",
        };
      }).filter((entry) => entry.eventId && entry.subject)
      : [];
    return {
      event: record.event,
      headers,
      payload: record.payload,
      payloadText: stringValue(record.payloadText),
      decodeError: stringValue(record.decodeError),
      proof: Object.keys(objectRecord(record.proof)).length ? objectRecord(record.proof) : undefined,
      owner: Object.keys(objectRecord(record.owner)).length ? objectRecord(record.owner) : undefined,
      publisher: Object.keys(objectRecord(record.publisher)).length ? objectRecord(record.publisher) : undefined,
      related,
    };
  }

  function statusBadgeClass(status: string): string {
    if (status === "verified" || status === "current") return "badge-success";
    if (status === "processing" || status === "behind") return "badge-warning";
    if (status === "missing-proof" || status === "auth-unavailable" || status === "inactive" || status === "orphaned") return "badge-neutral";
    return "badge-error";
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function ageLabel(value: string | undefined): string {
    if (!value) return "-";
    const time = new Date(value).getTime();
    if (Number.isNaN(time)) return value;
    return compactDuration(Date.now() - time);
  }

  function subjectOwner(row: EventLogRow): string {
    if (row.ownerContractId && row.ownerEventName) return `${row.ownerContractId} / ${row.ownerEventName}`;
    return row.ownerContractId ?? row.ownerEventName ?? row.resolution;
  }

  function publisherLabel(row: EventLogRow): string {
    return row.publisherDeploymentId ?? row.publisherContractId ?? row.publisherKind ?? "unverified";
  }

  function buildEventQuery(): EventLogQueryInput {
    return {
      excludeEventTypes: excludeEventTypes.length > 0 ? excludeEventTypes : undefined,
      includeEventTypes: includeEventTypes.length > 0 ? includeEventTypes : undefined,
      limit: pageLimit,
      offset,
      ownerContractId: ownerContractId.trim() || undefined,
      publisherDeploymentId: publisherDeploymentId.trim() || undefined,
      search: query || undefined,
      sort: { field: "eventTime", direction: "desc" },
      verificationStatus: verificationStatus ? [verificationStatus] : undefined,
      window: windowValue,
    };
  }

  function buildConsumerQuery(): EventLogConsumersQueryInput {
    return {
      deploymentId: consumerDeploymentId.trim() || undefined,
      limit: pageLimit,
      offset: 0,
      ownerContractId: ownerContractId.trim() || undefined,
      subject: query || undefined,
    };
  }

  function unavailableText(error: unknown): string | null {
    const message = errorMessage(error);
    const normalized = message.toLowerCase();
    if (normalized.includes("no responders") || normalized.includes("not currently reachable") || normalized.includes("inactive contract")) {
      return "Event Log service is optional; events continue to publish without it.";
    }
    if (message.includes("Permissions Violation") && message.includes("EventLog")) {
      return "Your current session is not approved for Event Log access. Sign out and sign back in to refresh permissions.";
    }
    return null;
  }

  async function load(showLoading = true) {
    const sequence = ++loadSequence;
    if (showLoading) loading = true;
    else refreshing = true;
    error = null;
    unavailableMessage = null;

    try {
      const metricsInput: EventLogMetricsInput = { window: windowValue };
      const [eventData, consumerData, metricData] = await Promise.all([
        trellis.request("EventLog.Query", buildEventQuery()).orThrow(),
        trellis.request("EventLog.Consumers.Query", buildConsumerQuery()).orThrow(),
        trellis.request("EventLog.Metrics", metricsInput).orThrow(),
      ]);
      if (sequence !== loadSequence) return;
      rows = eventData.events.filter(isEventLogRow);
      total = eventData.total;
      consumers = consumerData.consumers.map(toConsumerRow).filter((row): row is ConsumerRow => row !== null);
      consumerTotal = consumerData.total;
      metrics = toMetrics(metricData);
      lastUpdated = new Date();
    } catch (loadError) {
      if (sequence !== loadSequence) return;
      const unavailable = unavailableText(loadError);
      if (unavailable) {
        unavailableMessage = unavailable;
        rows = [];
        consumers = [];
        metrics = null;
        total = 0;
        consumerTotal = 0;
      } else {
        error = errorMessage(loadError);
      }
    } finally {
      if (sequence === loadSequence) {
        loading = false;
        refreshing = false;
      }
    }
  }

  function resetAndLoad() {
    offset = 0;
    selectedEvent = null;
    selectedConsumer = null;
    void load();
  }

  function refreshNow() {
    void load(false);
  }

  function clearFilters() {
    searchText = "";
    ownerContractId = "";
    publisherDeploymentId = "";
    consumerDeploymentId = "";
    verificationStatus = "";
    includeEventTypes = [];
    excludeEventTypes = [];
    resetAndLoad();
  }

  function updateEventTypeFilter(eventType: EventTypeRef, filter: "include" | "exclude", checked: boolean) {
    const key = eventTypeKey(eventType);
    const ref = { ownerContractId: eventType.ownerContractId, ownerEventName: eventType.ownerEventName };
    if (filter === "include") {
      includeEventTypes = checked ? [...includeEventTypes.filter((item) => eventTypeKey(item) !== key), ref] : includeEventTypes.filter((item) => eventTypeKey(item) !== key);
      if (checked) excludeEventTypes = excludeEventTypes.filter((item) => eventTypeKey(item) !== key);
    } else {
      excludeEventTypes = checked ? [...excludeEventTypes.filter((item) => eventTypeKey(item) !== key), ref] : excludeEventTypes.filter((item) => eventTypeKey(item) !== key);
      if (checked) includeEventTypes = includeEventTypes.filter((item) => eventTypeKey(item) !== key);
    }
    resetAndLoad();
  }

  function selectOwner(owner: string | undefined) {
    if (!owner) return;
    ownerContractId = owner;
    resetAndLoad();
  }

  function selectPublisher(deployment: string | undefined) {
    if (!deployment) return;
    publisherDeploymentId = deployment;
    resetAndLoad();
  }

  async function inspectEvent(row: EventLogRow) {
    detailLoading = true;
    detailError = null;
    selectedConsumer = null;
    try {
      const input: EventLogInspectInput = row.eventId ? { eventId: row.eventId } : { streamSequence: row.streamSequence };
      const detail = toInspect(await trellis.request("EventLog.Inspect", input).orThrow());
      selectedEvent = detail ?? { event: row, headers: {}, related: [] };
    } catch (inspectError) {
      detailError = errorMessage(inspectError);
      selectedEvent = { event: row, headers: {}, related: [] };
    } finally {
      detailLoading = false;
    }
  }

  async function inspectConsumer(row: ConsumerRow) {
    detailLoading = true;
    detailError = null;
    selectedEvent = null;
    try {
      const detail = await trellis.request("EventLog.Consumers.Inspect", { consumerName: row.consumerName, stream: row.stream }).orThrow();
      selectedConsumer = { row, detail: objectRecord(detail) };
    } catch (inspectError) {
      detailError = errorMessage(inspectError);
      selectedConsumer = { row, detail: null };
    } finally {
      detailLoading = false;
    }
  }

  function scopeConsumer(row: ConsumerRow) {
    mode = "events";
    consumerDeploymentId = row.deploymentId ?? "";
    searchText = (row.filterSubjects[0] ?? "").replaceAll("*", "").replaceAll(">", "");
    resetAndLoad();
  }

  function goPrevious() {
    if (offset === 0) return;
    offset = Math.max(0, offset - pageLimit);
    void load();
  }

  function goNext() {
    if (offset + pageLimit >= total) return;
    offset += pageLimit;
    void load();
  }

  function scheduleReload() {
    if (reloadTimer) clearTimeout(reloadTimer);
    reloadTimer = setTimeout(() => {
      reloadTimer = null;
      void load(false);
    }, 750);
  }

  function startWatch() {
    stopWatch();
    const controller = new AbortController();
    watchController = controller;
    void (async () => {
      try {
        const stream = await trellis.feed.eventLog.watch({}, { signal: controller.signal }).orThrow();
        for await (const frame of stream) {
          if (controller.signal.aborted) return;
          const record = objectRecord(frame);
          if (record.kind === "ready") feedOnline = true;
          if (record.kind !== "ready") scheduleReload();
        }
      } catch (watchError) {
        if (!controller.signal.aborted) {
          feedOnline = false;
          feedMessage = `Live feed offline: ${errorMessage(watchError)}`;
        }
      }
    })();
  }

  function stopWatch() {
    watchController?.abort();
    watchController = null;
  }

  onMount(() => {
    void load();
    startWatch();
  });

  onDestroy(() => {
    stopWatch();
    if (reloadTimer) clearTimeout(reloadTimer);
  });
</script>

<section class="events-page space-y-4">
  <PageToolbar title="Events" description="Event flow, publisher verification, and durable consumer health.">
    {#snippet meta()}
      <span class={["badge badge-sm", feedOnline ? "badge-success" : "badge-neutral"]}>{feedOnline ? "Live" : "Historical"}</span>
      <span class="badge badge-ghost badge-sm">Page {pageNumber}</span>
      {#if lastUpdated}
        <span class="text-xs text-base-content/50">Updated {lastUpdated.toLocaleTimeString()}</span>
      {/if}
    {/snippet}
    {#snippet actions()}
      <button class="btn btn-ghost btn-sm" onclick={refreshNow} disabled={loading || refreshing}>{refreshing ? "Refreshing" : "Refresh"}</button>
    {/snippet}
  </PageToolbar>

  <InlineMetricsStrip metrics={metricsStrip} />

  <div class="consumer-strip">
    <div class="consumer-strip-main">
      <span class="text-xs font-semibold uppercase tracking-wide text-base-content/50">Consumer health</span>
      <span class="badge badge-sm badge-neutral">{consumerTotal} total</span>
      <span class={["badge badge-sm", unhealthyConsumers.length > 0 ? "badge-warning" : "badge-success"]}>{unhealthyConsumers.length} attention</span>
      {#if metrics?.oldestLagMs !== undefined}
        <span class="text-xs tabular-nums text-base-content/60">oldest lag {compactDuration(metrics.oldestLagMs)}</span>
      {/if}
    </div>
    <div class="consumer-strip-items">
      {#each unhealthyConsumers.slice(0, 6) as consumer (`${consumer.stream}:${consumer.consumerName}`)}
        <button type="button" class="badge badge-outline badge-sm" onclick={() => { mode = 'consumers'; void inspectConsumer(consumer); }}>
          {consumer.status}: {consumer.consumerName}
        </button>
      {:else}
        <span class="text-xs text-base-content/55">No consumer health warnings.</span>
      {/each}
    </div>
  </div>

  <div class="events-filterbar">
    <div class="join">
      <button type="button" class={["btn btn-xs join-item", mode === "events" ? "btn-neutral" : "btn-outline"]} onclick={() => { mode = 'events'; selectedConsumer = null; }}>Events</button>
      <button type="button" class={["btn btn-xs join-item", mode === "consumers" ? "btn-neutral" : "btn-outline"]} onclick={() => { mode = 'consumers'; selectedEvent = null; }}>Consumers</button>
    </div>
    <input class="input input-bordered input-sm events-search" placeholder="Subject or search" bind:value={searchText} onchange={resetAndLoad} />
    <input class="input input-bordered input-sm events-contract" placeholder="Owner contract" bind:value={ownerContractId} onchange={resetAndLoad} />
    <input class="input input-bordered input-sm events-contract" placeholder="Publisher deployment" bind:value={publisherDeploymentId} onchange={resetAndLoad} />
    {#if mode === "consumers"}
      <input class="input input-bordered input-sm events-contract" placeholder="Consumer deployment" bind:value={consumerDeploymentId} onchange={resetAndLoad} />
    {/if}
    <select class="select select-bordered select-sm" bind:value={verificationStatus} onchange={resetAndLoad} disabled={mode === "consumers"}>
      {#each verificationOptions as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
    {#if mode === "consumers"}
      <button type="button" class="btn btn-outline btn-sm" disabled aria-label={`Include event types, ${includeEventTypes.length} selected`}>Include types ({includeEventTypes.length})</button>
      <button type="button" class="btn btn-outline btn-sm" disabled aria-label={`Hide event types, ${excludeEventTypes.length} selected`}>Hide types ({excludeEventTypes.length})</button>
    {:else}
      <details class="dropdown dropdown-start sm:dropdown-end">
        <summary class="btn btn-outline btn-sm" aria-label={`Include event types, ${includeEventTypes.length} selected`}>Include types ({includeEventTypes.length})</summary>
        <ul class="menu menu-sm dropdown-content z-30 mt-2 max-h-80 w-[min(22rem,calc(100vw-2rem))] flex-nowrap overflow-y-auto rounded-box border border-base-300 bg-base-100 p-2 shadow" aria-label="Include event types">
          {#each includeEventTypeOptions as eventType (eventTypeKey(eventType))}
            <li>
              <label class="flex-row items-start gap-2">
                <input
                  type="checkbox"
                  class="checkbox checkbox-xs mt-0.5"
                  checked={includeEventTypeKeys.has(eventTypeKey(eventType))}
                  onchange={(event) => updateEventTypeFilter(eventType, "include", event.currentTarget.checked)}
                />
                <span class="min-w-0 flex-1">
                  <span class="block truncate font-medium">{eventType.ownerEventName}</span>
                  <span class="trellis-identifier block truncate text-xs text-base-content/50">{eventType.ownerContractId}</span>
                </span>
                <span class="badge badge-ghost badge-sm tabular-nums">{eventType.count}</span>
              </label>
            </li>
          {:else}
            <li><span class="text-xs text-base-content/55">No event types in this window.</span></li>
          {/each}
        </ul>
      </details>
      <details class="dropdown dropdown-start sm:dropdown-end">
        <summary class="btn btn-outline btn-sm" aria-label={`Hide event types, ${excludeEventTypes.length} selected`}>Hide types ({excludeEventTypes.length})</summary>
        <ul class="menu menu-sm dropdown-content z-30 mt-2 max-h-80 w-[min(22rem,calc(100vw-2rem))] flex-nowrap overflow-y-auto rounded-box border border-base-300 bg-base-100 p-2 shadow" aria-label="Hide event types">
          {#each excludeEventTypeOptions as eventType (eventTypeKey(eventType))}
            <li>
              <label class="flex-row items-start gap-2">
                <input
                  type="checkbox"
                  class="checkbox checkbox-xs mt-0.5"
                  checked={excludeEventTypeKeys.has(eventTypeKey(eventType))}
                  onchange={(event) => updateEventTypeFilter(eventType, "exclude", event.currentTarget.checked)}
                />
                <span class="min-w-0 flex-1">
                  <span class="block truncate font-medium">{eventType.ownerEventName}</span>
                  <span class="trellis-identifier block truncate text-xs text-base-content/50">{eventType.ownerContractId}</span>
                </span>
                <span class="badge badge-ghost badge-sm tabular-nums">{eventType.count}</span>
              </label>
            </li>
          {:else}
            <li><span class="text-xs text-base-content/55">No event types in this window.</span></li>
          {/each}
        </ul>
      </details>
    {/if}
    <select class="select select-bordered select-sm" bind:value={windowValue} onchange={resetAndLoad}>
      {#each windowOptions as option (option)}
        <option value={option}>{option}</option>
      {/each}
    </select>
    <button type="button" class="btn btn-ghost btn-sm" onclick={clearFilters}>Clear</button>
  </div>

  {#if mode === "events" && (includeEventTypes.length > 0 || excludeEventTypes.length > 0)}
    <div class="flex flex-wrap items-center gap-x-3 gap-y-1 rounded-box border border-base-300 bg-base-100 px-3 py-2" aria-label="Active event type filters">
      {#if includeEventTypes.length > 0}
        <div class="flex min-w-0 flex-wrap items-center gap-1.5">
          <span class="text-xs font-semibold uppercase tracking-wide text-base-content/50">Include</span>
          {#each includeEventTypes as eventType (eventTypeKey(eventType))}
            <button
              type="button"
              class="badge badge-outline badge-sm h-auto max-w-full cursor-pointer gap-1 py-1"
              title={`${eventType.ownerContractId} / ${eventType.ownerEventName}`}
              aria-label={`Remove included event type ${eventType.ownerEventName} from ${eventType.ownerContractId}`}
              onclick={() => updateEventTypeFilter(eventType, "include", false)}
            >
              <span class="trellis-identifier max-w-[min(24rem,calc(100vw-6rem))] truncate">{eventType.ownerContractId} / {eventType.ownerEventName}</span>
              <span aria-hidden="true">&times;</span>
            </button>
          {/each}
        </div>
      {/if}
      {#if excludeEventTypes.length > 0}
        <div class="flex min-w-0 flex-wrap items-center gap-1.5">
          <span class="text-xs font-semibold uppercase tracking-wide text-base-content/50">Hide</span>
          {#each excludeEventTypes as eventType (eventTypeKey(eventType))}
            <button
              type="button"
              class="badge badge-outline badge-sm h-auto max-w-full cursor-pointer gap-1 py-1"
              title={`${eventType.ownerContractId} / ${eventType.ownerEventName}`}
              aria-label={`Remove hidden event type ${eventType.ownerEventName} from ${eventType.ownerContractId}`}
              onclick={() => updateEventTypeFilter(eventType, "exclude", false)}
            >
              <span class="trellis-identifier max-w-[min(24rem,calc(100vw-6rem))] truncate">{eventType.ownerContractId} / {eventType.ownerEventName}</span>
              <span aria-hidden="true">&times;</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if error}
    <Notice variant="error" role="alert">{error}</Notice>
  {:else if unavailableMessage}
    <Notice variant="info" role="status">{unavailableMessage}</Notice>
  {:else if feedMessage}
    <Notice variant="warning" role="status">{feedMessage}</Notice>
  {/if}

  <Panel eyebrow="Primary" title={mode === "events" ? "Event log" : "Consumers"}>
    {#if loading}
      <LoadingState label="Loading events" />
    {:else if unavailableMessage}
      <p class="text-xs text-base-content/60">Event publishing and consuming continue without the optional Event Log service.</p>
    {:else if mode === "events"}
      {#if rows.length === 0}
        <EmptyState title="No events in this window" description="Widen the time range or clear filters." />
      {:else}
        <DataTable fixed wrapperClass="events-table-wrap">
          <thead>
            <tr>
              <th class="w-36">Time</th>
              <th>Subject / event</th>
              <th>Owner contract / event</th>
              <th>Publisher deployment</th>
              <th class="w-32">Verification</th>
              <th class="w-28">Trace</th>
              <th class="w-24">Payload</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as row (`${row.streamSequence}:${row.eventId}`)}
              <tr class="hover cursor-pointer" onclick={() => { void inspectEvent(row); }}>
                <td class="text-xs tabular-nums text-base-content/70">{formatDate(row.eventTime)}</td>
                <td class="min-w-0">
                  <button type="button" class="trellis-identifier link link-hover truncate" onclick={(event) => { event.stopPropagation(); searchText = row.subject; resetAndLoad(); }}>{row.subject}</button>
                  <div class="text-xs text-base-content/45">seq {row.streamSequence}</div>
                </td>
                <td class="min-w-0">
                  <button type="button" class="trellis-identifier link link-hover truncate" onclick={(event) => { event.stopPropagation(); selectOwner(row.ownerContractId); }}>{subjectOwner(row)}</button>
                </td>
                <td class="min-w-0">
                  <button type="button" class="trellis-identifier link link-hover truncate" onclick={(event) => { event.stopPropagation(); selectPublisher(row.publisherDeploymentId); }}>{publisherLabel(row)}</button>
                  {#if row.publisherInstanceId}<div class="trellis-identifier text-xs text-base-content/45">{row.publisherInstanceId}</div>{/if}
                </td>
                <td><span class={["badge badge-sm", statusBadgeClass(row.verificationStatus)]}>{row.verificationStatus}</span></td>
                <td class="trellis-identifier text-xs text-base-content/60">{row.traceId ?? "-"}</td>
                <td class="text-xs tabular-nums text-base-content/70">{formatBytes(row.payloadSizeBytes)}</td>
              </tr>
            {/each}
          </tbody>
        </DataTable>
      {/if}
    {:else if sortedConsumers.length === 0}
      <EmptyState title="No durable consumers match these filters" description="Clear filters or wait for Event Log consumer sampling." />
    {:else}
      <DataTable fixed wrapperClass="events-table-wrap">
        <thead>
          <tr>
            <th class="w-28">Status</th>
            <th>Consumer deployment / contract / group</th>
            <th>Consumer name</th>
            <th>Filter subjects</th>
            <th class="w-20">Pending</th>
            <th class="w-24">Ack</th>
            <th class="w-24">Pulls</th>
            <th class="w-28">Oldest</th>
            <th class="w-24">Redeliv.</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedConsumers as consumer (`${consumer.stream}:${consumer.consumerName}`)}
            <tr class="hover cursor-pointer" onclick={() => { void inspectConsumer(consumer); }}>
              <td><span class={["badge badge-sm", statusBadgeClass(consumer.status)]}>{consumer.status}</span></td>
              <td class="min-w-0">
                <div class="trellis-identifier truncate">{consumer.deploymentId ?? "unattributed"}</div>
                <div class="trellis-identifier text-xs text-base-content/45">{consumer.contractId ?? "-"}{consumer.group ? ` / ${consumer.group}` : ""}</div>
              </td>
              <td class="trellis-identifier truncate">{consumer.consumerName}</td>
              <td class="min-w-0">
                <button type="button" class="trellis-identifier link link-hover truncate" onclick={(event) => { event.stopPropagation(); scopeConsumer(consumer); }}>{consumer.filterSubjects.join(", ") || "-"}</button>
              </td>
              <td class="text-xs tabular-nums text-base-content/70">{consumer.pending}</td>
              <td class="text-xs tabular-nums text-base-content/70">{consumer.ackPending}</td>
              <td class="text-xs tabular-nums text-base-content/70">{consumer.waitingPulls}</td>
              <td class="text-xs tabular-nums text-base-content/70">{ageLabel(consumer.oldestPendingAt)}</td>
              <td class="text-xs tabular-nums text-base-content/70">{consumer.redelivered ?? 0}</td>
            </tr>
          {/each}
        </tbody>
      </DataTable>
    {/if}
    {#snippet footer()}
      <div class="flex items-center justify-between gap-3">
        <span>{mode === "events" ? `${rows.length} shown from ${total} events` : `${sortedConsumers.length} shown from ${consumerTotal} consumers`}</span>
        {#if mode === "events"}
          <div class="join">
            <button class="btn btn-outline btn-xs join-item" onclick={goPrevious} disabled={loading || offset === 0}>Previous</button>
            <button class="btn btn-outline btn-xs join-item" onclick={goNext} disabled={loading || offset + pageLimit >= total}>Next</button>
          </div>
        {/if}
      </div>
    {/snippet}
  </Panel>

  {#if detailLoading}
    <Panel eyebrow="Detail" title="Loading detail"><LoadingState label="Loading detail" /></Panel>
  {:else if selectedEvent}
    <Panel eyebrow="Detail" title="Event inspect">
      {#if detailError}<Notice variant="warning" role="status">{detailError}</Notice>{/if}
      <div class="detail-grid">
        <div>
          <h3>Event identity</h3>
          <p><span>id</span><code>{selectedEvent.event.eventId}</code></p>
          <p><span>time</span><code>{selectedEvent.event.eventTime}</code></p>
          <p><span>subject</span><code>{selectedEvent.event.subject}</code></p>
          <p><span>stream sequence</span><code>{selectedEvent.event.streamSequence}</code></p>
        </div>
        <div>
          <h3>Owner from subject/catalog</h3>
          <p><span>contract</span><code>{selectedEvent.event.ownerContractId ?? "unresolved"}</code></p>
          <p><span>event</span><code>{selectedEvent.event.ownerEventName ?? "-"}</code></p>
          <p><span>resolution</span><code>{selectedEvent.event.resolution}</code></p>
        </div>
        <div>
          <h3>Publisher from verified session</h3>
          <p><span>status</span><code>{selectedEvent.event.verificationStatus}</code></p>
          <p><span>deployment</span><code>{selectedEvent.event.publisherDeploymentId ?? "-"}</code></p>
          <p><span>instance</span><code>{selectedEvent.event.publisherInstanceId ?? "-"}</code></p>
          <p><span>contract</span><code>{selectedEvent.event.publisherContractId ?? "-"}</code></p>
        </div>
      </div>
      <div class="payload-grid">
        <div>
          <h3>Headers</h3>
          <pre>{jsonBlock(selectedEvent.headers)}</pre>
        </div>
        <div>
          <h3>Payload</h3>
          {#if selectedEvent.decodeError}<p class="text-xs text-error">{selectedEvent.decodeError}</p>{/if}
          <pre>{selectedEvent.payloadText ?? jsonBlock(selectedEvent.payload)}</pre>
        </div>
      </div>
      {#if selectedEvent.related.length > 0}
        <div class="related-row">
          {#each selectedEvent.related as related (`${related.matchedBy}:${related.eventId}`)}
            <span class="badge badge-outline badge-sm">{related.matchedBy}: {related.subject}</span>
          {/each}
        </div>
      {/if}
    </Panel>
  {:else if selectedConsumer}
    <Panel eyebrow="Detail" title="Consumer inspect">
      {#if detailError}<Notice variant="warning" role="status">{detailError}</Notice>{/if}
      <div class="detail-grid">
        <div>
          <h3>Consumer deployment</h3>
          <p><span>deployment</span><code>{selectedConsumer.row.deploymentId ?? "-"}</code></p>
          <p><span>contract</span><code>{selectedConsumer.row.contractId ?? "-"}</code></p>
          <p><span>group</span><code>{selectedConsumer.row.group ?? "-"}</code></p>
        </div>
        <div>
          <h3>Live state</h3>
          <p><span>status</span><code>{selectedConsumer.row.status}</code></p>
          <p><span>pending</span><code>{selectedConsumer.row.pending}</code></p>
          <p><span>ack pending</span><code>{selectedConsumer.row.ackPending}</code></p>
          <p><span>waiting pulls</span><code>{selectedConsumer.row.waitingPulls}</code></p>
        </div>
      </div>
      <pre>{jsonBlock(selectedConsumer.detail)}</pre>
    </Panel>
  {/if}
</section>

<style>
  .consumer-strip,
  .events-filterbar {
    background: color-mix(in oklab, var(--color-base-100) 78%, var(--color-base-200));
    border: 1px solid color-mix(in oklab, var(--color-base-300) 82%, transparent);
    border-radius: var(--radius-box, 1rem);
    padding: 0.65rem 0.85rem;
  }

  .consumer-strip {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .consumer-strip-main,
  .consumer-strip-items,
  .events-filterbar {
    align-items: center;
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .events-search {
    flex: 1 1 15rem;
    min-width: 12rem;
  }

  .events-contract {
    flex: 0 1 13rem;
  }

  :global(.events-table-wrap td),
  :global(.events-table-wrap th) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail-grid,
  .payload-grid {
    display: grid;
    gap: 0.75rem;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
  }

  h3 {
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    margin-bottom: 0.35rem;
    text-transform: uppercase;
  }

  p {
    align-items: baseline;
    display: flex;
    gap: 0.45rem;
    margin: 0.15rem 0;
  }

  p span {
    color: color-mix(in oklab, var(--color-base-content) 52%, transparent);
    font-size: 0.72rem;
    min-width: 6rem;
  }

  code,
  pre {
    font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace);
  }

  pre {
    background: color-mix(in oklab, var(--color-base-200) 80%, transparent);
    border: 1px solid color-mix(in oklab, var(--color-base-300) 82%, transparent);
    border-radius: var(--radius-box, 1rem);
    max-height: 24rem;
    overflow: auto;
    padding: 0.75rem;
    white-space: pre-wrap;
  }

  .related-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-top: 0.75rem;
  }
</style>
