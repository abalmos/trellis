<script lang="ts">
  import { isErr } from "@qlever-llc/result";
  import { type apis } from "trellis-web-generated";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import DataTable from "$lib/components/DataTable.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import Notice from "$lib/components/Notice.svelte";
  import PageToolbar from "$lib/components/PageToolbar.svelte";
  import Panel from "$lib/components/Panel.svelte";
  import StatusBadge from "$lib/components/StatusBadge.svelte";
  import { errorMessage, formatDate } from "$lib/format";
  import { getTrellis } from "$lib/trellis";

  const trellis = getTrellis();
  const deploymentId = $derived(
    decodeURIComponent(page.url.pathname.split("/").filter(Boolean).at(-1) ?? ""),
  );
  let loading = $state(true);
  let error = $state<string | null>(null);
  let detail = $state.raw<apis.auth.AuthDeploymentsGetOutput | null>(null);

  async function load() {
    loading = true;
    error = null;
    const response = await trellis.authDeploymentsGet({ deploymentId }).take();
    if (isErr(response)) error = errorMessage(response);
    else detail = response;
    loading = false;
  }

  onMount(load);
</script>

<PageToolbar
  title={detail?.deployment.displayName ?? "Deployment"}
  description="Current deployment, participant-scoped grant, and physical resource evidence."
>
  {#snippet actions()}
    <button class="btn btn-ghost btn-sm" onclick={load}>Refresh</button>
  {/snippet}
</PageToolbar>

{#if error}<Notice variant="error">{error}</Notice>{/if}
{#if loading}
  <LoadingState />
{:else if detail}
  <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_22rem]">
    <Panel title="Resource evidence">
      {#if detail.resources.length === 0}
        <EmptyState
          title="No resource bindings"
          description="This installed participant has no physical resource evidence."
          class="m-5"
        />
      {:else}
        <DataTable size="sm">
          <thead>
            <tr><th>Resource</th><th>Kind</th><th>State</th><th>Binding</th><th>Observed</th></tr>
          </thead>
          <tbody>
            {#each detail.resources as resource (resource.bindingId)}
              <tr>
                <td class="trellis-identifier">{resource.localName}</td>
                <td>{resource.resourceKind}</td>
                <td>
                  <StatusBadge
                    label={resource.state}
                    status={resource.state === "available"
                      ? "healthy"
                      : resource.state === "stale"
                      ? "degraded"
                      : "unhealthy"}
                  />
                </td>
                <td class="trellis-identifier">{resource.bindingId}</td>
                <td>{formatDate(resource.materializedAt)}</td>
              </tr>
            {/each}
          </tbody>
        </DataTable>
      {/if}
    </Panel>

    <div class="space-y-4">
      <Panel title="Deployment">
        <dl class="grid grid-cols-[7rem_minmax(0,1fr)] gap-x-3 gap-y-2 text-sm">
          <dt class="text-base-content/60">State</dt>
          <dd><StatusBadge label={detail.deployment.state} status={detail.deployment.state === "active" ? "healthy" : "offline"} /></dd>
          <dt class="text-base-content/60">Deployment</dt>
          <dd class="trellis-identifier truncate">{detail.deployment.deploymentId}</dd>
          <dt class="text-base-content/60">Participant</dt>
          <dd class="trellis-identifier truncate">{detail.deployment.participantId ?? "Not installed"}</dd>
          <dt class="text-base-content/60">Version</dt><dd>{detail.deployment.version}</dd>
          <dt class="text-base-content/60">Updated</dt><dd>{formatDate(detail.deployment.updatedAt)}</dd>
        </dl>
      </Panel>

      <Panel title="GrantBinding">
        {#if detail.binding}
          <dl class="grid grid-cols-[7rem_minmax(0,1fr)] gap-x-3 gap-y-2 text-sm">
            <dt class="text-base-content/60">State</dt>
            <dd><StatusBadge label={detail.binding.state} status={detail.binding.state === "active" ? "healthy" : "offline"} /></dd>
            <dt class="text-base-content/60">Revision</dt><dd>{detail.binding.revision}</dd>
            <dt class="text-base-content/60">Installed</dt><dd>{detail.binding.installedRevision}</dd>
            <dt class="text-base-content/60">Permissions</dt><dd>{detail.binding.grants.permissions.length}</dd>
            <dt class="text-base-content/60">Privileges</dt><dd>{detail.binding.platformPrivileges.join(", ") || "None"}</dd>
          </dl>
        {:else}
          <EmptyState
            title="No grant binding"
            description="Apply this deployment to install its participant and create a binding."
          />
        {/if}
      </Panel>
    </div>
  </div>
{/if}
