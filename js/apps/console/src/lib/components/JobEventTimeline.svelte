<script lang="ts">
  import { formatDate, compactDuration, jobStateStatus } from "../format";

  type TimelineEvent = {
    sequence: number;
    timestamp: string;
    state: string;
    type: string;
    message?: string;
    reason?: string;
    error?: string;
    workerInstanceId?: string;
    tries?: number;
  };

  type Props = {
    events: TimelineEvent[];
  };

  let { events }: Props = $props();

  let showRaw = $state(false);

  type TimelineStep = {
    kind: "start" | "progress" | "terminal";
    label: string;
    detail?: string;
    timestamp: string;
    state: string;
    type: string;
    attempt?: number;
    workerInstanceId?: string;
    rawEvents: TimelineEvent[];
  };

  const sortedEvents = $derived(
    [...events].sort((a, b) => a.sequence - b.sequence)
  );

  const steps = $derived(buildSteps(sortedEvents));

  function buildSteps(sorted: TimelineEvent[]): TimelineStep[] {
    if (sorted.length === 0) return [];

    const result: TimelineStep[] = [];
    let createdTime: number | null = null;
    let startTime: number | null = null;

    for (const event of sorted) {
      const t = new Date(event.timestamp).getTime();
      if (Number.isNaN(t)) continue;

      if (event.state === "created" || event.state === "pending" || event.type === "CREATED" || event.type === "Created") {
        createdTime = t;
        const attempt = event.tries;
        result.push({
          kind: "start",
          label: "Created",
          detail: createdTime != null ? `Entered queue` : undefined,
          timestamp: event.timestamp,
          state: event.state,
          type: event.type,
          attempt,
          rawEvents: [event],
        });
        continue;
      }

      if (event.type === "PROGRESS" || event.type === "Progress" || (event.message && event.message.length > 0)) {
        const attempt = event.tries;
        result.push({
          kind: "progress",
          label: event.message ?? event.reason ?? event.type,
          timestamp: event.timestamp,
          state: event.state,
          type: event.type,
          attempt,
          workerInstanceId: event.workerInstanceId,
          rawEvents: [event],
        });
        continue;
      }

      if (event.type === "STARTED" || event.type === "Started" || event.state === "started" || event.state === "active") {
        startTime = t;
        const waitDuration = createdTime != null ? compactDuration(t - createdTime) : null;
        const attempt = event.tries;
        result.push({
          kind: "start",
          label: "Started",
          detail: waitDuration ? `Picked up after ${waitDuration}` : undefined,
          timestamp: event.timestamp,
          state: event.state,
          type: event.type,
          attempt,
          workerInstanceId: event.workerInstanceId,
          rawEvents: [event],
        });
        continue;
      }

      if (event.state === "completed" || event.type === "COMPLETED" || event.type === "Completed") {
        const runDuration = startTime != null ? compactDuration(t - startTime) : null;
        const attempt = event.tries;
        result.push({
          kind: "terminal",
          label: "Completed",
          detail: runDuration ? `Ran for ${runDuration}` : undefined,
          timestamp: event.timestamp,
          state: event.state,
          type: event.type,
          attempt,
          rawEvents: [event],
        });
        continue;
      }

      if (event.state === "failed" || event.type === "FAILED" || event.type === "Failed") {
        const runDuration = startTime != null ? compactDuration(t - startTime) : null;
        const errorMsg = parseErrorMessage(event);
        const attempt = event.tries;
        result.push({
          kind: "terminal",
          label: errorMsg ?? "Failed",
          detail: runDuration ? `Ran for ${runDuration}` : undefined,
          timestamp: event.timestamp,
          state: event.state,
          type: event.type,
          attempt,
          workerInstanceId: event.workerInstanceId,
          rawEvents: [event],
        });
        continue;
      }

      if (event.state === "dead" || event.type === "DEAD" || event.type === "Dead") {
        const attempt = event.tries;
        result.push({
          kind: "terminal",
          label: "Dead letter queue",
          timestamp: event.timestamp,
          state: event.state,
          type: event.type,
          attempt,
          rawEvents: [event],
        });
        continue;
      }

      if (event.state === "retry" || event.type === "RETRY" || event.type === "Retry") {
        const attempt = event.tries;
        result.push({
          kind: "terminal",
          label: "Retrying",
          timestamp: event.timestamp,
          state: event.state,
          type: event.type,
          attempt,
          rawEvents: [event],
        });
        continue;
      }

      if (event.message || event.reason) {
        const attempt = event.tries;
        result.push({
          kind: "progress",
          label: event.message ?? event.reason ?? event.type,
          timestamp: event.timestamp,
          state: event.state,
          type: event.type,
          attempt,
          workerInstanceId: event.workerInstanceId,
          rawEvents: [event],
        });
      }
    }

    return result;
  }

  function parseErrorMessage(event: TimelineEvent): string | null {
    if (!event.error) return null;
    try {
      const parsed = JSON.parse(event.error);
      return parsed.message ?? parsed.type ?? null;
    } catch {
      return event.error.length > 80 ? event.error.slice(0, 80) + "..." : event.error;
    }
  }

  function stepColor(kind: TimelineStep["kind"], state: string): string {
    if (kind === "terminal") {
      switch (state) {
        case "completed": return "step-success";
        case "failed":
        case "dead": return "step-error";
        case "retry": return "step-warning";
        default: return "step-neutral";
      }
    }
    if (kind === "start") return "step-info";
    return "step-progress";
  }
</script>

<div class="job-event-timeline">
  {#if steps.length === 0}
    <p class="text-sm text-base-content/60">No timeline events recorded.</p>
  {:else}
    <div class="timeline-list">
      {#each steps as step, index (step.rawEvents[0]?.sequence ?? index)}
        <div class="timeline-step">
          <div class="timeline-marker">
            <div class={["timeline-dot", stepColor(step.kind, step.state)]}></div>
            {#if index < steps.length - 1}
              <div class="timeline-line"></div>
            {/if}
          </div>
          <div class="timeline-content">
            <div class="timeline-type-row">
              <span class="timeline-type">{step.type}</span>
            </div>
            <div class="timeline-label">{step.label}</div>
            {#if step.detail}
              <div class="timeline-detail">{step.detail}</div>
            {/if}
            <div class="timeline-meta">
              <time class="timeline-time">{formatDate(step.timestamp)}</time>
              {#if step.workerInstanceId}
                <span class="timeline-worker trellis-identifier">{step.workerInstanceId.slice(0, 12)}</span>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .job-event-timeline {
    display: grid;
    gap: 0;
  }

  .timeline-list {
    display: flex;
    flex-direction: column;
  }

  .timeline-step {
    display: flex;
    gap: 0.75rem;
    padding: 0.4rem 0;
  }

  .timeline-marker {
    align-items: center;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    width: 1rem;
  }

  .timeline-dot {
    border-radius: 50%;
    flex-shrink: 0;
    height: 0.6rem;
    width: 0.6rem;
  }

  .step-info {
    background: oklch(0.6 0.12 240);
  }

  .step-progress {
    background: oklch(0.65 0.12 145);
  }

  .step-success {
    background: oklch(0.6 0.15 145);
  }

  .step-error {
    background: oklch(0.55 0.18 25);
  }

  .step-warning {
    background: oklch(0.7 0.15 85);
  }

  .step-neutral {
    background: color-mix(in oklab, var(--color-base-content) 30%, transparent);
  }

  .timeline-line {
    background: color-mix(in oklab, var(--color-base-300) 40%, transparent);
    flex: 1;
    margin-top: 0.15rem;
    width: 1.5px;
  }

  .timeline-content {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
    padding-bottom: 0.5rem;
  }

  .timeline-type-row {
    align-items: center;
    display: flex;
    gap: 0.35rem;
  }

  .timeline-type {
    background: color-mix(in oklab, var(--color-base-300) 30%, transparent);
    border-radius: 0.25rem;
    color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
    font-size: 0.55rem;
    font-weight: 600;
    padding: 0.1rem 0.4rem;
    text-transform: uppercase;
  }

  .timeline-label {
    color: var(--color-base-content);
    font-size: 0.8rem;
    font-weight: 500;
    line-height: 1.3;
  }

  .timeline-detail {
    color: color-mix(in oklab, var(--color-base-content) 55%, transparent);
    font-size: 0.7rem;
    line-height: 1.4;
  }

  .timeline-meta {
    align-items: center;
    display: flex;
    gap: 0.5rem;
  }

  .timeline-time {
    color: color-mix(in oklab, var(--color-base-content) 45%, transparent);
    font-size: 0.65rem;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .timeline-worker {
    color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
    font-size: 0.6rem;
  }
</style>
