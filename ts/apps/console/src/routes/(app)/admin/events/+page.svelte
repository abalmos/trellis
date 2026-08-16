<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import EmptyState from "../../../../lib/components/EmptyState.svelte";
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
    EventLogMetricsOutput,
    EventLogQueryInput,
  } from "../../../../../../../../generated/packages/jsr/eventlog/mod.ts";

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
  type Focus = "exceptions" | "all" | "unresolved" | "malformed" | "largest" | EventVerificationStatus;
  type EventTypeRef = { ownerContractId: string; ownerEventName: string };
  type ConsumerManagedBy = "authority" | "platform" | "external";
  type ConsumerStatus = "current" | "processing" | "behind" | "saturated" | "inactive" | "failing" | "missing" | "orphaned" | "unmanaged";

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
    related: Array<{ eventId: string; eventTime: string; subject: string; matchedBy: string }>;
  };

  type ConsumerRow = {
    deploymentId?: string;
    contractId?: string;
    group?: string;
    stream: string;
    consumerName: string;
    filterSubjects: string[];
    managedBy: ConsumerManagedBy;
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

  const trellis = getTrellis();
  const rpcTimeout = 10_000;
  const pageLimit = 40;
  const windows: Array<{ value: WindowValue; label: string; minutes: number }> = [
    { value: "15m", label: "15m", minutes: 15 },
    { value: "1h", label: "1h", minutes: 60 },
    { value: "6h", label: "6h", minutes: 360 },
    { value: "24h", label: "24h", minutes: 1_440 },
    { value: "7d", label: "7d", minutes: 10_080 },
  ];
  const verificationIssues: EventVerificationStatus[] = [
    "missing-proof",
    "invalid-signature",
    "missing-session",
    "subject-denied",
    "outside-session-window",
    "auth-unavailable",
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
    unmanaged: 8,
  };

  let loading = $state(true);
  let refreshing = $state(false);
  let error = $state<string | null>(null);
  let unavailableMessage = $state<string | null>(null);
  let feedOnline = $state(false);
  let feedMessage = $state<string | null>(null);
  let rows = $state.raw<EventLogRow[]>([]);
  let consumers = $state.raw<ConsumerRow[]>([]);
  let metrics = $state.raw<EventLogMetricsOutput | null>(null);
  let selectedEvent = $state.raw<EventInspect | null>(null);
  let selectedConsumer = $state.raw<{ row: ConsumerRow; detail: Record<string, unknown> | null } | null>(null);
  let detailLoading = $state(false);
  let detailError = $state<string | null>(null);
  let focus = $state<Focus>("exceptions");
  let selectedEventType = $state.raw<EventTypeRef | null>(null);
  let attentionConsumersOnly = $state(false);
  let searchText = $state("");
  let ownerContractId = $state("");
  let publisherDeploymentId = $state("");
  let windowValue = $state<WindowValue>("1h");
  let offset = $state(0);
  let total = $state(0);
  let consumerTotal = $state(0);
  let lastUpdated = $state<Date | null>(null);

  let loadSequence = 0;
  let detailSequence = 0;
  let watchController: AbortController | null = null;
  let reloadTimer: ReturnType<typeof setTimeout> | null = null;

  const windowOption = $derived(windows.find((option) => option.value === windowValue) ?? windows[1]);
  const eventRate = $derived((metrics?.summary.total ?? 0) / windowOption.minutes);
  const averagePayload = $derived(metrics?.summary.total ? metrics.summary.payloadSizeBytes / metrics.summary.total : 0);
  const eventTypesByCount = $derived.by(() => [...(metrics?.summary.eventTypes ?? [])].sort((a, b) => b.count - a.count));
  const sortedConsumers = $derived.by(() => [...consumers].sort((a, b) => consumerSeverity[a.status] - consumerSeverity[b.status]));
  const attentionConsumers = $derived(sortedConsumers.filter(isAttentionConsumer));
  const displayedConsumers = $derived(attentionConsumersOnly ? attentionConsumers : sortedConsumers);
  const oldestLagConsumer = $derived.by(() =>
    consumers
      .filter((consumer) => consumer.oldestPendingAt)
      .sort((a, b) => new Date(a.oldestPendingAt ?? 0).getTime() - new Date(b.oldestPendingAt ?? 0).getTime())[0],
  );
  const eventPoints = $derived(chartPoints(metrics?.buckets.map((bucket) => bucket.total) ?? []));
  const exceptionPoints = $derived(chartPoints(metrics?.buckets.map((bucket) => bucket.integrityExceptions) ?? []));
  const focusTitle = $derived(`${selectedEventType ? `${selectedEventType.ownerEventName} · ` : ""}${focusLabel(focus)}`);
  const focusDescription = $derived(focusDetail(focus));

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

  function isResolution(value: unknown): value is EventResolution {
    return value === "resolved" || value === "unresolved" || value === "malformed";
  }

  function isVerificationStatus(value: unknown): value is EventVerificationStatus {
    return value === "verified" || verificationIssues.includes(value as EventVerificationStatus);
  }

  function isConsumerStatus(value: unknown): value is ConsumerStatus {
    return Object.hasOwn(consumerSeverity, String(value));
  }

  function isConsumerManagedBy(value: unknown): value is ConsumerManagedBy {
    return value === "authority" || value === "platform" || value === "external";
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
    const managedBy = isConsumerManagedBy(row.managedBy) ? row.managedBy : stringValue(row.deploymentId) ? "authority" : "external";
    return {
      deploymentId: stringValue(row.deploymentId),
      contractId: stringValue(row.contractId),
      group: stringValue(row.group),
      stream,
      consumerName,
      filterSubjects: stringArray(row.filterSubjects),
      managedBy,
      status: managedBy === "external" ? "unmanaged" : row.status,
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
      related,
    };
  }

  function isAttentionConsumer(consumer: ConsumerRow): boolean {
    return consumer.status !== "current" && consumer.status !== "processing" && consumer.status !== "unmanaged";
  }

  function statusBadgeClass(status: string): string {
    if (status === "verified" || status === "current") return "badge-success";
    if (status === "processing" || status === "behind" || status === "saturated" || status === "orphaned" || status === "unresolved") return "badge-warning";
    if (status === "missing-proof" || status === "auth-unavailable" || status === "inactive" || status === "unmanaged") return "badge-neutral";
    return "badge-error";
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${Math.round(bytes)} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  function ageLabel(value: string | undefined): string {
    if (!value) return "-";
    const time = new Date(value).getTime();
    return Number.isNaN(time) ? value : compactDuration(Date.now() - time);
  }

  function subjectOwner(row: EventLogRow): string {
    if (row.ownerContractId && row.ownerEventName) return `${row.ownerContractId} / ${row.ownerEventName}`;
    return row.ownerContractId ?? row.ownerEventName ?? row.resolution;
  }

  function publisherLabel(row: EventLogRow): string {
    return row.publisherDeploymentId ?? row.publisherContractId ?? row.publisherKind ?? "unverified";
  }

  function focusLabel(value: Focus): string {
    if (value === "exceptions") return "Recent integrity exceptions";
    if (value === "all") return "Recent event flow";
    if (value === "unresolved") return "Unresolved events";
    if (value === "malformed") return "Malformed events";
    if (value === "largest") return "Largest event payloads";
    return `${value.replaceAll("-", " ")} events`;
  }

  function focusDetail(value: Focus): string {
    if (value === "exceptions") return "Events that could not be fully attributed or verified.";
    if (value === "all") return "Newest projected events in the selected window.";
    if (value === "unresolved") return "Subjects that could not be associated with a catalog event.";
    if (value === "malformed") return "Event envelopes the projector could not interpret.";
    if (value === "largest") return "Highest payload sizes in the selected window.";
    return `Events with verification status ${value}.`;
  }

  function chartPoints(values: number[]): string {
    if (values.length === 0) return "";
    const maximum = Math.max(...values, 1);
    return values.map((value, index) => `${values.length === 1 ? 130 : index * 260 / (values.length - 1)},${52 - value / maximum * 46}`).join(" ");
  }

  function buildEventQuery(): EventLogQueryInput {
    const input: EventLogQueryInput = {
      includeEventTypes: selectedEventType ? [selectedEventType] : undefined,
      limit: pageLimit,
      offset,
      ownerContractId: ownerContractId.trim() || undefined,
      publisherDeploymentId: publisherDeploymentId.trim() || undefined,
      search: searchText.trim() || undefined,
      sort: { field: focus === "largest" ? "payloadSize" : "eventTime", direction: "desc" },
      window: windowValue,
    };
    if (focus === "exceptions") input.integrityExceptionOnly = true;
    else if (focus === "unresolved" || focus === "malformed") input.resolution = [focus];
    else if (focus === "verified") input.verificationStatus = [focus];
    return input;
  }

  function buildConsumerQuery(): EventLogConsumersQueryInput {
    return { limit: 100, offset: 0 };
  }

  function unavailableText(loadError: unknown): string | null {
    const message = errorMessage(loadError);
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
        trellis.eventLogQuery(buildEventQuery(), { timeout: rpcTimeout }).orThrow(),
        trellis.eventLogConsumersQuery(buildConsumerQuery(), { timeout: rpcTimeout }).orThrow(),
        trellis.eventLogMetrics(metricsInput, { timeout: rpcTimeout }).orThrow(),
      ]);
      if (sequence !== loadSequence) return;
      rows = eventData.events.filter(isEventLogRow);
      total = eventData.total;
      consumers = consumerData.consumers.map(toConsumerRow).filter((row): row is ConsumerRow => row !== null);
      consumerTotal = consumerData.total;
      metrics = metricData;
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
        if (showLoading) {
          rows = [];
          consumers = [];
          metrics = null;
          total = 0;
          consumerTotal = 0;
        }
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
    void load();
  }

  function selectFocus(value: Focus) {
    focus = value;
    resetAndLoad();
  }

  function selectEventType(eventType: EventTypeRef) {
    selectedEventType = selectedEventType?.ownerContractId === eventType.ownerContractId && selectedEventType.ownerEventName === eventType.ownerEventName ? null : eventType;
    resetAndLoad();
  }

  function clearEventFilters() {
    searchText = "";
    ownerContractId = "";
    publisherDeploymentId = "";
    selectedEventType = null;
    focus = "exceptions";
    resetAndLoad();
  }

  function showAttentionConsumers() {
    attentionConsumersOnly = !attentionConsumersOnly;
    document.querySelector(".consumer-health")?.scrollIntoView({ behavior: "smooth", block: "start" });
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
    const sequence = ++detailSequence;
    detailLoading = true;
    detailError = null;
    selectedConsumer = null;
    try {
      const input: EventLogInspectInput = row.eventId ? { eventId: row.eventId } : { streamSequence: row.streamSequence };
      const detail = toInspect(await trellis.eventLogInspect(input, { timeout: rpcTimeout }).orThrow());
      if (sequence !== detailSequence) return;
      selectedEvent = detail ?? { event: row, headers: {}, related: [] };
    } catch (inspectError) {
      if (sequence !== detailSequence) return;
      detailError = errorMessage(inspectError);
      selectedEvent = { event: row, headers: {}, related: [] };
    } finally {
      if (sequence === detailSequence) detailLoading = false;
    }
  }

  async function inspectConsumer(row: ConsumerRow) {
    const sequence = ++detailSequence;
    detailLoading = true;
    detailError = null;
    selectedEvent = null;
    try {
      const detail = await trellis.eventLogConsumersInspect({ consumerName: row.consumerName, stream: row.stream },
        { timeout: rpcTimeout },
      ).orThrow();
      if (sequence !== detailSequence) return;
      selectedConsumer = { row, detail: objectRecord(detail) };
    } catch (inspectError) {
      if (sequence !== detailSequence) return;
      detailError = errorMessage(inspectError);
      selectedConsumer = { row, detail: null };
    } finally {
      if (sequence === detailSequence) detailLoading = false;
    }
  }

  function selectOldestLag() {
    if (oldestLagConsumer) void inspectConsumer(oldestLagConsumer);
  }

  function goPrevious() {
    offset = Math.max(0, offset - pageLimit);
    void load();
  }

  function goNext() {
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
        const stream = await trellis.eventLogWatch({}, { signal: controller.signal }).orThrow();
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

<section class="events-page">
  <PageToolbar title="Events" description="Delivery health, event integrity, and recent exceptions.">
    {#snippet meta()}
      <span class={['badge badge-sm', feedOnline ? 'badge-success' : 'badge-neutral']}>{feedOnline ? "Live" : "Historical"}</span>
      {#if lastUpdated}<span class="text-xs text-base-content/50">Updated {lastUpdated.toLocaleTimeString()}</span>{/if}
    {/snippet}
    {#snippet actions()}
      <div class="window-switcher" aria-label="Metrics window">
        {#each windows as option (option.value)}
          <button class:active={windowValue === option.value} aria-pressed={windowValue === option.value} onclick={() => { windowValue = option.value; resetAndLoad(); }}>{option.label}</button>
        {/each}
      </div>
      <button class="btn btn-ghost btn-sm" onclick={() => { void load(false); }} disabled={loading || refreshing}>{refreshing ? "Refreshing" : "Refresh"}</button>
    {/snippet}
  </PageToolbar>

  {#if error}<Notice variant="error" role="alert">{error}</Notice>{/if}
  {#if unavailableMessage}<Notice variant="info" role="status">{unavailableMessage}</Notice>{/if}
  {#if feedMessage}<Notice variant="warning" role="status">{feedMessage}</Notice>{/if}

  {#if loading}
    <LoadingState label="Loading event health" />
  {:else if unavailableMessage}
    <EmptyState title="Event Log is unavailable" description="Event publishing and consuming continue without the optional visibility service." />
  {:else}
    <div class="event-ledger" aria-label="Event health summary">
      <button class:active={focus === "all"} aria-pressed={focus === "all"} onclick={() => selectFocus("all")}>
        <span><i class="dot blue"></i>Event flow</span><strong>{(metrics?.summary.total ?? 0).toLocaleString()}</strong><small>{eventRate.toLocaleString(undefined, { maximumFractionDigits: 1 })} per minute</small>
      </button>
      <button class:active={focus === "exceptions"} aria-pressed={focus === "exceptions"} onclick={() => selectFocus("exceptions")}>
        <span><i class="dot red"></i>Integrity exceptions</span><strong>{(metrics?.summary.integrityExceptions ?? 0).toLocaleString()}</strong><small>{metrics?.summary.total ? (metrics.summary.integrityExceptions / metrics.summary.total * 100).toFixed(2) : "0.00"}% of events</small>
      </button>
      <button class:active={focus === "unresolved"} aria-pressed={focus === "unresolved"} onclick={() => selectFocus("unresolved")}>
        <span><i class="dot amber"></i>Unresolved</span><strong>{(metrics?.summary.byResolution.unresolved ?? 0).toLocaleString()}</strong><small>owner not resolved</small>
      </button>
      <button class:active={attentionConsumersOnly} aria-pressed={attentionConsumersOnly} onclick={showAttentionConsumers}>
        <span><i class="dot red"></i>Consumers</span><strong>{attentionConsumers.length}</strong><small>need attention · {consumerTotal} total</small>
      </button>
      <button class:active={selectedConsumer?.row.consumerName === oldestLagConsumer?.consumerName} aria-pressed={selectedConsumer?.row.consumerName === oldestLagConsumer?.consumerName} onclick={selectOldestLag} disabled={!oldestLagConsumer}>
        <span><i class="dot amber"></i>Oldest lag</span><strong>{ageLabel(oldestLagConsumer?.oldestPendingAt)}</strong><small>{oldestLagConsumer?.consumerName ?? "no pending events"}</small>
      </button>
      <button class:active={focus === "largest"} aria-pressed={focus === "largest"} onclick={() => selectFocus("largest")}>
        <span><i class="dot green"></i>Payload</span><strong>{formatBytes(metrics?.summary.payloadSizeBytes ?? 0)}</strong><small>{formatBytes(averagePayload)} average</small>
      </button>
    </div>

    <div class="health-layout">
      <section class="consumer-health">
        <div class="section-heading">
          <div><h2>Consumer delivery health</h2><p>Known Trellis consumers first; external consumers remain neutral.</p></div>
          <span>{displayedConsumers.length} of {consumerTotal}</span>
        </div>
        {#if displayedConsumers.length === 0}
          <EmptyState title="No consumers need attention" description="All known Trellis consumers are current or processing." />
        {:else}
          <div class="consumer-matrix" aria-label="Consumer delivery health">
            <div class="consumer-header">
              <span>Consumer deployment / contract</span><span>Status</span><span>Pending</span><span>Ack</span><span>Pulls</span><span>Oldest</span><span>Redeliv.</span>
            </div>
            {#each displayedConsumers as consumer (`${consumer.stream}:${consumer.consumerName}`)}
              <button
                class:selected={selectedConsumer?.row.consumerName === consumer.consumerName && selectedConsumer.row.stream === consumer.stream}
                class="consumer-row"
                onclick={() => { void inspectConsumer(consumer); }}
              >
                <span class="consumer-identity"><strong>{consumer.deploymentId ?? consumer.consumerName}</strong><small>{consumer.contractId ?? consumer.managedBy}{consumer.group ? ` / ${consumer.group}` : ""}</small></span>
                <span class="consumer-status"><i class={['dot', isAttentionConsumer(consumer) ? consumer.status === 'behind' || consumer.status === 'saturated' ? 'amber' : 'red' : consumer.status === 'unmanaged' ? 'neutral' : 'green']}></i>{consumer.status}</span>
                <span class="consumer-value" class:pressure={consumer.pending > 0} data-label="Pending">{consumer.pending.toLocaleString()}</span>
                <span class="consumer-value" data-label="Ack">{consumer.ackPending.toLocaleString()}</span>
                <span class="consumer-value" data-label="Pulls">{consumer.waitingPulls.toLocaleString()}</span>
                <span class="consumer-value" class:pressure={isAttentionConsumer(consumer)} data-label="Oldest">{ageLabel(consumer.oldestPendingAt)}</span>
                <span class="consumer-value" class:pressure={(consumer.redelivered ?? 0) > 0} data-label="Redeliv.">{consumer.redelivered ?? 0}</span>
              </button>
            {/each}
          </div>
        {/if}
      </section>

      <aside class="event-rail" aria-label="Event flow and integrity">
        <section class="mini-chart">
          <div><h2>Event volume</h2><strong>{eventRate.toLocaleString(undefined, { maximumFractionDigits: 1 })}/min</strong></div>
          <p>{(metrics?.summary.total ?? 0).toLocaleString()} events · {windowOption.label}</p>
          <svg viewBox="0 0 260 52" preserveAspectRatio="none" role="img" aria-label={`Event volume over ${windowOption.label}`}>
            <path class="chart-grid" d="M0 46H260 M0 26H260" />
            <polyline class="event-line" points={eventPoints} />
          </svg>
        </section>
        <section class="mini-chart">
          <div><h2>Integrity exceptions</h2><strong class="text-error">{metrics?.summary.integrityExceptions ?? 0}</strong></div>
          <p>Verification and resolution failures</p>
          <svg viewBox="0 0 260 52" preserveAspectRatio="none" role="img" aria-label={`Integrity exceptions over ${windowOption.label}`}>
            <path class="chart-grid" d="M0 46H260 M0 26H260" />
            <polyline class="exception-line" points={exceptionPoints} />
          </svg>
          <div class="integrity-breakdown">
            {#each verificationIssues as status (status)}
              {#if (metrics?.summary.byVerificationStatus[status] ?? 0) > 0}
                <button aria-pressed={focus === status} onclick={() => selectFocus(status)}><span>{status.replaceAll("-", " ")}</span><strong>{metrics?.summary.byVerificationStatus[status] ?? 0}</strong></button>
              {/if}
            {/each}
            {#if (metrics?.summary.byResolution.malformed ?? 0) > 0}
              <button aria-pressed={focus === "malformed"} onclick={() => selectFocus("malformed")}><span>malformed</span><strong>{metrics?.summary.byResolution.malformed}</strong></button>
            {/if}
          </div>
        </section>
        <section class="event-types">
          <div class="section-heading"><div><h2>Highest-volume types</h2><p>Resolved owner/event pairs.</p></div></div>
          {#each eventTypesByCount.slice(0, 6) as eventType (`${eventType.ownerContractId}:${eventType.ownerEventName}`)}
            <button class:active={selectedEventType?.ownerContractId === eventType.ownerContractId && selectedEventType.ownerEventName === eventType.ownerEventName} aria-pressed={selectedEventType?.ownerContractId === eventType.ownerContractId && selectedEventType.ownerEventName === eventType.ownerEventName} onclick={() => selectEventType(eventType)}>
              <span><strong>{eventType.ownerEventName}</strong><small>{eventType.ownerContractId}</small></span><b>{eventType.count.toLocaleString()}</b>
              <i style={`--width: ${metrics?.summary.total ? eventType.count / metrics.summary.total * 100 : 0}%`}></i>
            </button>
          {:else}
            <p class="text-xs text-base-content/55">No resolved event types in this window.</p>
          {/each}
        </section>
      </aside>
    </div>

    <section class="focused-events">
      <div class="section-heading list-heading">
        <div><h2>{focusTitle}</h2><p>{focusDescription}</p></div>
        <div class="event-tools">
          <input class="input input-bordered input-sm" placeholder="Search event metadata" bind:value={searchText} onchange={resetAndLoad} />
          {#if selectedEventType || ownerContractId || publisherDeploymentId || searchText}<button class="btn btn-ghost btn-sm" onclick={clearEventFilters}>Clear scope</button>{/if}
          <span>{rows.length} shown from {total}</span>
        </div>
      </div>
      {#if selectedEventType || ownerContractId || publisherDeploymentId}
        <div class="active-scope" aria-label="Active event scope">
          {#if selectedEventType}<button onclick={() => { selectedEventType = null; resetAndLoad(); }}>type: {selectedEventType.ownerContractId} / {selectedEventType.ownerEventName} ×</button>{/if}
          {#if ownerContractId}<button onclick={() => { ownerContractId = ''; resetAndLoad(); }}>owner: {ownerContractId} ×</button>{/if}
          {#if publisherDeploymentId}<button onclick={() => { publisherDeploymentId = ''; resetAndLoad(); }}>publisher: {publisherDeploymentId} ×</button>{/if}
        </div>
      {/if}
      {#if rows.length === 0}
        <EmptyState title="No events match this operational view" description="Choose another status, widen the window, or clear the active scope." />
      {:else}
        <div class="event-list" aria-label={focusTitle}>
          <div class="event-header"><span>Time</span><span>Subject / event</span><span>Owner</span><span>Publisher</span><span>Integrity</span><span>Payload</span></div>
          {#each rows as row (`${row.streamSequence}:${row.eventId}`)}
            <div class="event-row">
              <span data-label="Time">{formatDate(row.eventTime)}</span>
              <button class="event-subject" onclick={() => { void inspectEvent(row); }}><strong>{row.subject}</strong><small>seq {row.streamSequence}</small></button>
              <button class="identifier" onclick={() => selectOwner(row.ownerContractId)}>{subjectOwner(row)}</button>
              <button class="identifier" onclick={() => selectPublisher(row.publisherDeploymentId)}>{publisherLabel(row)}</button>
              <span><i class={['badge badge-sm', statusBadgeClass(row.verificationStatus)]}>{row.verificationStatus}</i>{#if row.resolution !== "resolved"}<i class={['badge badge-sm', statusBadgeClass(row.resolution)]}>{row.resolution}</i>{/if}</span>
              <span data-label="Payload">{formatBytes(row.payloadSizeBytes)}</span>
            </div>
          {/each}
        </div>
        {#if offset > 0 || offset + pageLimit < total}
          <div class="pagination"><button class="btn btn-outline btn-xs" onclick={goPrevious} disabled={offset === 0}>Previous</button><span>Page {Math.floor(offset / pageLimit) + 1}</span><button class="btn btn-outline btn-xs" onclick={goNext} disabled={offset + pageLimit >= total}>Next</button></div>
        {/if}
      {/if}
    </section>

    {#if detailLoading}
      <Panel eyebrow="Detail" title="Loading detail"><LoadingState label="Loading detail" /></Panel>
    {:else if selectedEvent}
      <Panel eyebrow="Detail" title="Event inspect">
        {#if detailError}<Notice variant="warning" role="status">{detailError}</Notice>{/if}
        <div class="detail-grid">
          <div><h3>Event identity</h3><p><span>id</span><code>{selectedEvent.event.eventId}</code></p><p><span>time</span><code>{selectedEvent.event.eventTime}</code></p><p><span>subject</span><code>{selectedEvent.event.subject}</code></p><p><span>stream sequence</span><code>{selectedEvent.event.streamSequence}</code></p></div>
          <div><h3>Owner from subject/catalog</h3><p><span>contract</span><code>{selectedEvent.event.ownerContractId ?? "unresolved"}</code></p><p><span>event</span><code>{selectedEvent.event.ownerEventName ?? "-"}</code></p><p><span>resolution</span><code>{selectedEvent.event.resolution}</code></p></div>
          <div><h3>Publisher from verified session</h3><p><span>status</span><code>{selectedEvent.event.verificationStatus}</code></p><p><span>deployment</span><code>{selectedEvent.event.publisherDeploymentId ?? "-"}</code></p><p><span>instance</span><code>{selectedEvent.event.publisherInstanceId ?? "-"}</code></p><p><span>contract</span><code>{selectedEvent.event.publisherContractId ?? "-"}</code></p></div>
        </div>
        <div class="payload-grid"><div><h3>Headers</h3><pre>{jsonBlock(selectedEvent.headers)}</pre></div><div><h3>Payload</h3>{#if selectedEvent.decodeError}<p class="text-xs text-error">{selectedEvent.decodeError}</p>{/if}<pre>{selectedEvent.payloadText ?? jsonBlock(selectedEvent.payload)}</pre></div></div>
      </Panel>
    {:else if selectedConsumer}
      <Panel eyebrow="Consumer" title={selectedConsumer.row.deploymentId ?? selectedConsumer.row.consumerName}>
        {#if detailError}<Notice variant="warning" role="status">{detailError}</Notice>{/if}
        <div class="detail-grid">
          <div><h3>Ownership</h3><p><span>managed by</span><code>{selectedConsumer.row.managedBy}</code></p><p><span>deployment</span><code>{selectedConsumer.row.deploymentId ?? "-"}</code></p><p><span>contract</span><code>{selectedConsumer.row.contractId ?? "-"}</code></p><p><span>group</span><code>{selectedConsumer.row.group ?? "-"}</code></p></div>
          <div><h3>Live state</h3><p><span>status</span><code>{selectedConsumer.row.status}</code></p><p><span>pending</span><code>{selectedConsumer.row.pending}</code></p><p><span>ack pending</span><code>{selectedConsumer.row.ackPending}</code></p><p><span>waiting pulls</span><code>{selectedConsumer.row.waitingPulls}</code></p></div>
        </div>
        <pre>{jsonBlock(selectedConsumer.detail)}</pre>
      </Panel>
    {/if}
  {/if}
</section>

<style>
  .events-page { display: grid; gap: 1rem; }
  .window-switcher { display: flex; gap: 0.15rem; }
  .window-switcher button { border: 0; border-radius: 0.4rem; color: color-mix(in oklab, var(--color-base-content) 58%, transparent); cursor: pointer; font-size: 0.72rem; font-weight: 650; padding: 0.28rem 0.48rem; }
  .window-switcher button.active { background: var(--color-base-content); color: var(--color-base-100); }
  .event-ledger { border-block: 1px solid var(--color-base-300); display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); }
  .event-ledger button { background: transparent; border: 0; border-right: 1px solid var(--color-base-300); cursor: pointer; min-width: 0; padding: 0.75rem 0.85rem; text-align: left; }
  .event-ledger button:last-child { border-right: 0; }
  .event-ledger button:hover, .event-ledger button.active { background: color-mix(in oklab, var(--color-base-200) 72%, transparent); }
  .event-ledger button:disabled { cursor: default; opacity: 0.65; }
  .event-ledger span { align-items: center; color: color-mix(in oklab, var(--color-base-content) 58%, transparent); display: flex; font-size: 0.64rem; font-weight: 750; gap: 0.2rem; letter-spacing: 0.065em; text-transform: uppercase; }
  .event-ledger strong { display: block; font-size: 1.25rem; letter-spacing: -0.03em; margin: 0.22rem 0 0.08rem; }
  .event-ledger small { color: color-mix(in oklab, var(--color-base-content) 52%, transparent); display: block; font-size: 0.66rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dot { border-radius: 50%; display: inline-block; flex: 0 0 auto; height: 0.38rem; width: 0.38rem; }
  .dot.blue { background: var(--color-info); } .dot.red { background: var(--color-error); } .dot.amber { background: var(--color-warning); } .dot.green { background: var(--color-success); } .dot.neutral { background: color-mix(in oklab, var(--color-base-content) 42%, transparent); }
  .health-layout { display: grid; gap: 1.15rem; grid-template-columns: minmax(38rem, 1fr) 17.5rem; }
  .section-heading { align-items: end; display: flex; gap: 1rem; justify-content: space-between; margin-bottom: 0.55rem; }
  .section-heading h2 { font-size: 0.88rem; font-weight: 720; letter-spacing: -0.012em; margin: 0; }
  .section-heading p { color: color-mix(in oklab, var(--color-base-content) 56%, transparent); font-size: 0.71rem; margin: 0.08rem 0 0; }
  .section-heading > span, .event-tools > span { color: color-mix(in oklab, var(--color-base-content) 55%, transparent); font-size: 0.68rem; white-space: nowrap; }
  .consumer-matrix { border-bottom: 1px solid var(--color-base-300); }
  .consumer-header, .consumer-row { align-items: center; display: grid; gap: 0.6rem; grid-template-columns: minmax(12rem, 1.5fr) 5.5rem 4.8rem 4rem 4rem 5.1rem 4.5rem; }
  .consumer-header { border-bottom: 1px solid var(--color-base-300); color: color-mix(in oklab, var(--color-base-content) 52%, transparent); font-size: 0.58rem; font-weight: 750; letter-spacing: 0.06em; padding: 0.42rem 0.55rem; text-transform: uppercase; }
  .consumer-row { background: transparent; border: 0; border-top: 1px solid color-mix(in oklab, var(--color-base-300) 68%, transparent); cursor: pointer; font-size: 0.7rem; padding: 0.6rem 0.55rem; text-align: left; width: 100%; }
  .consumer-row:first-of-type { border-top: 0; }
  .consumer-row:hover, .consumer-row.selected { background: color-mix(in oklab, var(--color-base-200) 72%, transparent); }
  .consumer-identity { min-width: 0; }
  .consumer-identity strong, .consumer-identity small { display: block; font-family: var(--font-mono, ui-monospace, monospace); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .consumer-identity strong { font-size: 0.72rem; }
  .consumer-identity small { color: color-mix(in oklab, var(--color-base-content) 48%, transparent); font-size: 0.62rem; margin-top: 0.12rem; }
  .consumer-status { align-items: center; display: flex; font-size: 0.65rem; font-weight: 700; gap: 0.3rem; text-transform: capitalize; }
  .consumer-value { font-variant-numeric: tabular-nums; }
  .pressure { color: var(--color-error); font-weight: 700; }
  .event-rail { border-left: 1px solid var(--color-base-300); padding-left: 1.1rem; }
  .mini-chart { border-bottom: 1px solid var(--color-base-300); margin-bottom: 0.9rem; padding-bottom: 0.9rem; }
  .mini-chart > div:first-child { align-items: baseline; display: flex; justify-content: space-between; }
  .mini-chart h2 { font-size: 0.72rem; margin: 0; }
  .mini-chart strong { font-size: 0.8rem; }
  .mini-chart p { color: color-mix(in oklab, var(--color-base-content) 52%, transparent); font-size: 0.64rem; margin: 0.1rem 0 0.45rem; }
  .mini-chart svg { display: block; height: 3.25rem; width: 100%; }
  .chart-grid { fill: none; stroke: color-mix(in oklab, var(--color-base-300) 75%, transparent); stroke-width: 1; }
  .event-line, .exception-line { fill: none; stroke-linecap: round; stroke-linejoin: round; stroke-width: 2; }
  .event-line { stroke: var(--color-info); } .exception-line { stroke: var(--color-error); }
  .integrity-breakdown { display: grid; gap: 0.16rem; margin-top: 0.3rem; }
  .integrity-breakdown button { background: transparent; border: 0; cursor: pointer; display: flex; font-size: 0.63rem; justify-content: space-between; padding: 0.16rem 0; text-align: left; text-transform: capitalize; }
  .integrity-breakdown button:hover span { text-decoration: underline; }
  .event-types .section-heading { margin-bottom: 0.32rem; }
  .event-types button { background: transparent; border: 0; border-radius: 0.35rem; cursor: pointer; display: grid; gap: 0.15rem 0.5rem; grid-template-columns: minmax(0, 1fr) auto; margin: 0 -0.25rem; padding: 0.3rem 0.25rem; text-align: left; width: calc(100% + 0.5rem); }
  .event-types button:hover, .event-types button.active { background: color-mix(in oklab, var(--color-base-200) 72%, transparent); }
  .event-types button span { min-width: 0; }
  .event-types button strong, .event-types button small { display: block; font-family: var(--font-mono, ui-monospace, monospace); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .event-types button strong { font-size: 0.65rem; } .event-types button small { color: color-mix(in oklab, var(--color-base-content) 48%, transparent); font-size: 0.58rem; }
  .event-types button b { font-size: 0.64rem; font-variant-numeric: tabular-nums; }
  .event-types button i { background: color-mix(in oklab, var(--color-info) 70%, var(--color-base-300)); border-radius: 0.2rem; grid-column: 1 / -1; height: 0.18rem; width: var(--width); }
  .focused-events { margin-top: 0.4rem; }
  .list-heading { align-items: center; }
  .event-tools { align-items: center; display: flex; gap: 0.45rem; }
  .event-tools input { width: min(17rem, 28vw); }
  .active-scope { display: flex; flex-wrap: wrap; gap: 0.3rem; margin: -0.15rem 0 0.55rem; }
  .active-scope button { background: color-mix(in oklab, var(--color-base-200) 82%, transparent); border: 1px solid var(--color-base-300); border-radius: 999px; cursor: pointer; font-family: var(--font-mono, ui-monospace, monospace); font-size: 0.62rem; padding: 0.2rem 0.48rem; }
  .event-list { border-bottom: 1px solid var(--color-base-300); }
  .event-header, .event-row { align-items: center; display: grid; gap: 0.65rem; grid-template-columns: 8rem minmax(12rem, 1.3fr) minmax(10rem, 1fr) minmax(9rem, 0.9fr) 9rem 4.5rem; }
  .event-header { border-bottom: 1px solid var(--color-base-300); color: color-mix(in oklab, var(--color-base-content) 52%, transparent); font-size: 0.58rem; font-weight: 750; letter-spacing: 0.06em; padding: 0.42rem 0.55rem; text-transform: uppercase; }
  .event-row { background: transparent; border-bottom: 1px solid color-mix(in oklab, var(--color-base-300) 68%, transparent); font-size: 0.67rem; padding: 0.58rem 0.55rem; text-align: left; width: 100%; }
  .event-row:hover { background: color-mix(in oklab, var(--color-base-200) 72%, transparent); }
  .event-row > span, .event-row > button { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .event-row > button { background: transparent; border: 0; cursor: pointer; padding: 0; text-align: left; }
  .event-subject strong, .event-subject small { display: block; font-family: var(--font-mono, ui-monospace, monospace); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .event-subject small { color: color-mix(in oklab, var(--color-base-content) 45%, transparent); font-size: 0.58rem; }
  .identifier { font-family: var(--font-mono, ui-monospace, monospace); }
  .identifier:hover { text-decoration: underline; }
  .event-row .badge + .badge { margin-left: 0.18rem; }
  .pagination { align-items: center; display: flex; font-size: 0.68rem; gap: 0.55rem; justify-content: flex-end; padding-top: 0.6rem; }
  .detail-grid, .payload-grid { display: grid; gap: 0.75rem; grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr)); }
  h3 { font-size: 0.72rem; font-weight: 700; letter-spacing: 0.08em; margin-bottom: 0.35rem; text-transform: uppercase; }
  .detail-grid p { align-items: baseline; display: flex; gap: 0.45rem; margin: 0.15rem 0; }
  .detail-grid p span { color: color-mix(in oklab, var(--color-base-content) 52%, transparent); font-size: 0.72rem; min-width: 6rem; }
  code, pre { font-family: var(--font-mono, ui-monospace, monospace); }
  pre { background: color-mix(in oklab, var(--color-base-200) 80%, transparent); border: 1px solid var(--color-base-300); border-radius: var(--radius-box, 1rem); max-height: 24rem; overflow: auto; padding: 0.75rem; white-space: pre-wrap; }

  @media (max-width: 75rem) {
    .health-layout { grid-template-columns: 1fr; }
    .event-rail { border-left: 0; border-top: 1px solid var(--color-base-300); display: grid; gap: 1rem; grid-template-columns: repeat(3, 1fr); padding: 1rem 0 0; }
    .mini-chart { border-bottom: 0; border-right: 1px solid var(--color-base-300); margin: 0; padding: 0 1rem 0 0; }
  }

  @media (max-width: 56rem) {
    .event-ledger { grid-template-columns: repeat(3, 1fr); }
    .event-ledger button:nth-child(3n) { border-right: 0; }
    .event-ledger button:nth-child(-n + 3) { border-bottom: 1px solid var(--color-base-300); }
    .consumer-header, .event-header { display: none; }
    .consumer-row { grid-template-columns: repeat(4, 1fr); }
    .consumer-identity { grid-column: 1 / -1; }
    .consumer-row > :nth-child(6), .consumer-row > :nth-child(7) { display: none; }
    .consumer-row [data-label]::before { color: color-mix(in oklab, var(--color-base-content) 48%, transparent); content: attr(data-label); display: block; font-size: 0.54rem; font-weight: 700; letter-spacing: 0.05em; text-transform: uppercase; }
    .event-row { grid-template-columns: minmax(0, 1fr) auto; }
    .event-row > :nth-child(1), .event-row > :nth-child(3), .event-row > :nth-child(4) { display: none; }
    .event-row > :nth-child(6) { text-align: right; }
  }

  @media (max-width: 42rem) {
    .event-ledger { grid-template-columns: repeat(2, 1fr); }
    .event-ledger button:nth-child(3n) { border-right: 1px solid var(--color-base-300); }
    .event-ledger button:nth-child(2n) { border-right: 0; }
    .event-ledger button { border-bottom: 1px solid var(--color-base-300); }
    .event-ledger button:nth-last-child(-n + 2) { border-bottom: 0; }
    .event-rail { display: block; }
    .mini-chart { border-bottom: 1px solid var(--color-base-300); border-right: 0; margin-bottom: 0.9rem; padding: 0 0 0.9rem; }
    .list-heading { align-items: flex-start; flex-direction: column; }
    .event-tools { flex-wrap: wrap; width: 100%; }
    .event-tools input { flex: 1 1 12rem; width: auto; }
  }
</style>
