<script lang="ts">
  import { formatDate, compactDuration, jobStateStatus } from "../format";
  import StatusBadge from "./StatusBadge.svelte";

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

  const sortedEvents = $derived(
    [...events].sort((a, b) => a.sequence - b.sequence)
  );

  const stateStatus = jobStateStatus;

  function eventDotColor(state: string): string {
    switch (state) {
      case "created": return "bg-neutral";
      case "started": return "bg-info";
      case "completed": return "bg-success";
      case "failed": return "bg-error";
      case "retry": return "bg-warning";
      case "cancelled": return "bg-neutral";
      case "dead": return "bg-error";
      case "dismissed": return "bg-neutral";
      default: return "bg-base-300";
    }
  }

  function eventDurationLabel(prev: TimelineEvent, current: TimelineEvent): string | null {
    const prevTime = new Date(prev.timestamp).getTime();
    const currTime = new Date(current.timestamp).getTime();
    if (Number.isNaN(prevTime) || Number.isNaN(currTime) || currTime <= prevTime) return null;
    return compactDuration(currTime - prevTime);
  }
</script>

<div class="job-event-timeline">
  {#if sortedEvents.length === 0}
    <p class="text-sm text-base-content/60">No timeline events recorded.</p>
  {:else}
    <div class="timeline-list">
      {#each sortedEvents as event, index (event.sequence)}
        {@const prevEvent = index > 0 ? sortedEvents[index - 1] : null}
        {@const gap = prevEvent ? eventDurationLabel(prevEvent, event) : null}
        <div class="timeline-entry">
          {#if gap}
            <div class="timeline-gap">{gap}</div>
          {/if}
          <div class="timeline-row">
            <div class="timeline-marker">
              <div class={["timeline-dot", eventDotColor(event.state)]}></div>
              {#if index < sortedEvents.length - 1}
                <div class="timeline-line"></div>
              {/if}
            </div>
            <div class="timeline-content">
              <div class="timeline-header">
                <time class="timeline-time">{formatDate(event.timestamp)}</time>
                <StatusBadge label={event.state} status={stateStatus(event.state)} />
                <span class="timeline-type">{event.type}</span>
                {#if event.workerInstanceId}
                  <span class="timeline-worker trellis-identifier">{event.workerInstanceId.slice(0, 12)}</span>
                {/if}
                {#if event.tries !== undefined}
                  <span class="timeline-tries">attempt {event.tries}</span>
                {/if}
              </div>
              {#if !event.error && (event.message || event.reason)}
                <div class="timeline-message">
                  {event.message ?? event.reason ?? ''}
                </div>
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
    gap: 0.25rem;
  }

  .timeline-list {
    display: flex;
    flex-direction: column;
  }

  .timeline-entry {
    display: flex;
    flex-direction: column;
  }

  .timeline-gap {
    color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
    font-size: 0.65rem;
    font-variant-numeric: tabular-nums;
    margin-left: 0.4rem;
    padding: 0.2rem 0 0.2rem 1.1rem;
    position: relative;
  }

  .timeline-gap::before {
    border-left: 1px dashed color-mix(in oklab, var(--color-base-300) 60%, transparent);
    content: '';
    height: 100%;
    left: 0;
    position: absolute;
    top: 0;
  }

  .timeline-row {
    display: flex;
    gap: 0.75rem;
    padding: 0.4rem 0;
  }

  .timeline-marker {
    align-items: center;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    width: 0.8rem;
  }

  .timeline-dot {
    border-radius: 50%;
    flex-shrink: 0;
    height: 0.55rem;
    width: 0.55rem;
  }

  .timeline-line {
    background: color-mix(in oklab, var(--color-base-300) 50%, transparent);
    flex: 1;
    margin-top: 0.15rem;
    width: 1px;
  }

  .timeline-content {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
    padding-bottom: 0.5rem;
  }

  .timeline-header {
    align-items: center;
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    min-width: 0;
  }

  .timeline-time {
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    font-size: 0.72rem;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .timeline-type {
    background: color-mix(in oklab, var(--color-base-300) 30%, transparent);
    border-radius: 0.25rem;
    color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
    font-size: 0.65rem;
    font-weight: 600;
    padding: 0.05rem 0.3rem;
    text-transform: uppercase;
  }

  .timeline-worker {
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    font-size: 0.72rem;
  }

  .timeline-tries {
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    font-size: 0.72rem;
    font-variant-numeric: tabular-nums;
  }

  .timeline-message {
    color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
    font-size: 0.78rem;
    line-height: 1.4;
    word-break: break-word;
  }
</style>
