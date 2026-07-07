<script lang="ts" module>
  type JsonValue =
    | string
    | number
    | boolean
    | null
    | undefined
    | JsonValue[]
    | { [key: string]: JsonValue };

  type Summary = {
    kind: "object" | "array" | "string" | "number" | "boolean" | "null" | "empty";
    length?: number;
    preview?: string;
  };

  export function summarize(value: unknown): Summary {
    if (value === null) return { kind: "null" };
    if (value === undefined) return { kind: "empty" };
    if (Array.isArray(value)) {
      if (value.length === 0) return { kind: "array", length: 0 };
      return { kind: "array", length: value.length };
    }
    const t = typeof value;
    if (t === "object") {
      const keys = Object.keys(value as Record<string, unknown>);
      if (keys.length === 0) return { kind: "object", length: 0 };
      return { kind: "object", length: keys.length };
    }
    if (t === "string") {
      const s = value as string;
      return {
        kind: "string",
        preview: s.length > 80 ? `${s.slice(0, 77)}…` : s,
      };
    }
    if (t === "number") return { kind: "number" };
    if (t === "boolean") return { kind: "boolean" };
    return { kind: "empty" };
  }
</script>

<script lang="ts">
  import Icon from "./Icon.svelte";
  import JsonTree from "./JsonTree.svelte";

  type Props = {
    value: unknown;
    label?: string;
    initiallyExpanded?: boolean;
    maxDepth?: number;
  };

  let {
    value,
    label,
    initiallyExpanded = true,
    maxDepth = 4,
  }: Props = $props();

  const summary = $derived(summarize(value));
  const canExpand = $derived(
    summary.kind === "object" || summary.kind === "array",
  );

  let expanded = $state(false);

  $effect(() => {
    expanded = initiallyExpanded;
  });

  function toggle() {
    if (!canExpand) return;
    expanded = !expanded;
  }

  function copyChild() {
    if (typeof navigator === "undefined" || !navigator.clipboard) return;
    try {
      navigator.clipboard.writeText(JSON.stringify(value, null, 2));
    } catch {
      // clipboard denied; ignore
    }
  }

  function asArray(value: unknown): unknown[] {
    if (!Array.isArray(value)) return [];
    return value;
  }

  function asObject(value: unknown): Array<[string, unknown]> {
    if (value === null || typeof value !== "object" || Array.isArray(value)) return [];
    return Object.entries(value as Record<string, unknown>);
  }
</script>

<div class="json-tree" data-depth={0}>
  <div class="json-tree-row">
    {#if canExpand}
      <button
        type="button"
        class="json-tree-toggle"
        aria-expanded={expanded}
        aria-label={expanded ? "Collapse" : "Expand"}
        onclick={toggle}
      >
        <span class="json-tree-caret" class:open={expanded}>▸</span>
      </button>
    {:else}
      <span class="json-tree-toggle json-tree-toggle-empty" aria-hidden="true"></span>
    {/if}
    {#if label !== undefined}
      <span class="json-tree-key">{label}</span>
      <span class="json-tree-punct">:</span>
    {/if}
    <span class={["json-tree-pill", `json-tree-pill-${summary.kind}`]}>
      {#if summary.kind === "object"}
        {`{ ${summary.length ?? 0} ${summary.length === 1 ? "key" : "keys"} }`}
      {:else if summary.kind === "array"}
        {`[ ${summary.length ?? 0} ${summary.length === 1 ? "item" : "items"} ]`}
      {:else if summary.kind === "string"}
        "{summary.preview ?? ""}"
      {:else if summary.kind === "number"}
        {String(value)}
      {:else if summary.kind === "boolean"}
        {String(value)}
      {:else if summary.kind === "null"}
        null
      {/if}
    </span>
    {#if canExpand && initiallyExpanded}
      <button
        type="button"
        class="json-tree-copy"
        aria-label="Copy this value as JSON"
        onclick={copyChild}
      >
        <Icon name="clipboard" size={10} />
      </button>
    {/if}
  </div>
  {#if canExpand && expanded}
    <div class="json-tree-children">
      {#if summary.kind === "array"}
        {#each asArray(value) as item, index (index)}
          <JsonTree
            value={item}
            label={`${index}`}
            initiallyExpanded={initiallyExpanded && maxDepth > 1}
            maxDepth={maxDepth - 1}
          />
        {/each}
      {:else}
        {#each asObject(value) as [key, child] (key)}
          <JsonTree
            value={child}
            label={key}
            initiallyExpanded={initiallyExpanded && maxDepth > 1}
            maxDepth={maxDepth - 1}
          />
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .json-tree {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 0.78rem;
    line-height: 1.5;
  }

  .json-tree-row {
    align-items: center;
    display: flex;
    gap: 0.3rem;
    min-width: 0;
    padding: 0.1rem 0;
  }

  .json-tree-toggle {
    align-items: center;
    background: transparent;
    border: none;
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    cursor: pointer;
    display: inline-flex;
    justify-content: center;
    padding: 0;
    width: 1rem;
  }

  .json-tree-toggle-empty {
    cursor: default;
  }

  .json-tree-caret {
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    display: inline-block;
    font-size: 0.65rem;
    transition: transform 150ms ease-out;
  }

  .json-tree-caret.open {
    transform: rotate(90deg);
  }

  .json-tree-key {
    color: var(--color-base-content);
    word-break: break-all;
  }

  .json-tree-punct {
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
  }

  .json-tree-pill {
    align-items: baseline;
    color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
    display: inline-flex;
    font-size: 0.72rem;
    gap: 0.25rem;
  }

  .json-tree-pill-string {
    color: oklch(0.6 0.16 145);
  }

  .json-tree-pill-number,
  .json-tree-pill-boolean {
    color: oklch(0.62 0.16 245);
  }

  .json-tree-pill-null,
  .json-tree-pill-empty {
    color: oklch(0.6 0.18 25);
  }

  .json-tree-copy {
    background: transparent;
    border: none;
    color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
    cursor: pointer;
    display: inline-flex;
    margin-left: 0.2rem;
    opacity: 0;
    padding: 0;
    transition: opacity 150ms ease-out, color 150ms ease-out;
  }

  .json-tree-row:hover .json-tree-copy,
  .json-tree-copy:focus-visible {
    opacity: 1;
  }

  .json-tree-copy:hover {
    color: var(--color-base-content);
  }

  .json-tree-copy:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .json-tree-children {
    border-left: 1px solid color-mix(in oklab, var(--color-base-300) 50%, transparent);
    margin-left: 0.5rem;
    padding-left: 0.65rem;
  }
</style>
