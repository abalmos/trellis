<script lang="ts">
  import { ulid } from "ulid";
  import { isErr } from "@qlever-llc/result";
  import { type apis } from "trellis-web-generated";
  import { resolve } from "$lib/console_paths";
  import { onMount } from "svelte";
  import BulkActionBar from "$lib/components/BulkActionBar.svelte";
  import BulkResult from "$lib/components/BulkResult.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import DataTable from "$lib/components/DataTable.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import Notice from "$lib/components/Notice.svelte";
  import PageToolbar from "$lib/components/PageToolbar.svelte";
  import Panel from "$lib/components/Panel.svelte";
  import SelectableRecordButton from "$lib/components/SelectableRecordButton.svelte";
  import SelectionRail from "$lib/components/SelectionRail.svelte";
  import StatusBadge from "$lib/components/StatusBadge.svelte";
  import { errorMessage, formatDate } from "$lib/format";
  import { bulkExpectedCount, bulkTargetDetails, runBulk, toggleAll, toggleId } from "$lib/bulk.ts";
  import { getTrellis } from "$lib/trellis";

  type DeviceDeployment = apis.auth.AuthDeploymentsListOutput["entries"][number];
  type DeviceInstance = apis.auth.AuthDevicesListOutput["entries"][number];
  type Activation = apis.auth.AuthDeviceUserAuthoritiesListOutput["entries"][number];
  type Review = apis.auth.AuthDeviceUserAuthoritiesReviewsListOutput["entries"][number];
  type Tab = "instances" | "activations" | "reviews";
  type StatusVariant = "healthy" | "degraded" | "unhealthy" | "offline";

  const trellis = getTrellis();
  const understoodMetadataKeys = ["name", "serialNumber", "modelNumber"] as const;
  const understoodMetadataKeySet = new Set<string>(understoodMetadataKeys);
  const tabs: Tab[] = ["instances", "activations", "reviews"];

  let loading = $state(true);
  let error = $state<string | null>(null);
  let deployments = $state.raw<DeviceDeployment[]>([]);
  let instances = $state.raw<DeviceInstance[]>([]);
  let activations = $state.raw<Activation[]>([]);
  let reviews = $state.raw<Review[]>([]);

  let selectedDeploymentId = $state("");
  let activeTab = $state<Tab>("instances");
  let search = $state("");
  let showMetadata = $state(false);
  let selectedReviewId = $state<string | null>(null);

  const selectedDeployment = $derived(deployments.find((deployment) => deployment.deploymentId === selectedDeploymentId) ?? null);
  const instancesById = $derived.by(() => new Map(instances.map((instance) => [instance.instanceId, instance])));
  const selectedInstances = $derived(instances.filter((instance) => instance.deploymentId === selectedDeploymentId));
  const selectedActivations = $derived(activations.filter((activation) => activation.device.deploymentId === selectedDeploymentId));
  const selectedReviews = $derived(reviews.filter((review) => review.deploymentId === selectedDeploymentId));
  const selectedPendingReviews = $derived(selectedReviews.filter((review) => review.state === "pending"));
  const filteredDeployments = $derived.by(() => {
    const term = search.trim().toLowerCase();
    if (!term) return deployments;
    return deployments.filter((deployment) => deployment.deploymentId.toLowerCase().includes(term));
  });
  const selectedReview = $derived(selectedReviews.find((review) => review.reviewId === selectedReviewId) ?? selectedReviews[0] ?? null);
  const activeInstanceCount = $derived(selectedInstances.filter((instance) => instance.state === "active").length);
  const revokedActivationCount = $derived(selectedActivations.filter((activation) => activation.device.delegationState === "revoked").length);

  function syncSelectedDeployment(nextDeployments: DeviceDeployment[]): string {
    const nextDeploymentId = nextDeployments.some((deployment) => deployment.deploymentId === selectedDeploymentId)
      ? selectedDeploymentId
      : nextDeployments[0]?.deploymentId ?? "";
    if (nextDeploymentId !== selectedDeploymentId) selectedReviewId = null;
    selectedDeploymentId = nextDeploymentId;
    return nextDeploymentId;
  }

  function selectDeployment(deploymentId: string) {
    selectedDeploymentId = deploymentId;
    selectedReviewId = null;
  }

  function selectTab(tab: Tab) {
    activeTab = tab;
  }

  function deploymentStatus(): StatusVariant {
    return "offline";
  }

  function instanceStatus(state: DeviceInstance["state"]): StatusVariant {
    if (state === "active") return "healthy";
    if (state === "pending") return "degraded";
    if (state === "revoked") return "unhealthy";
    return "offline";
  }

  function activationStatus(state: Activation["device"]["delegationState"]): StatusVariant {
    if (state === "active") return "healthy";
    if (state === "missing") return "degraded";
    return "unhealthy";
  }

  function reviewStatus(state: Review["state"]): StatusVariant {
    if (state === "approved") return "healthy";
    if (state === "pending") return "degraded";
    if (state === "rejected") return "unhealthy";
    return "offline";
  }


  function badgeClassForDeployment(): string {
    return "badge-neutral";
  }

  function dotClassForDeployment(): string {
    return "bg-base-content/30";
  }

  function deploymentInstances(deploymentId: string): DeviceInstance[] {
    return instances.filter((instance) => instance.deploymentId === deploymentId);
  }

  function pendingReviewsForDeployment(deploymentId: string): number {
    return reviews.filter((review) => review.deploymentId === deploymentId && review.state === "pending").length;
  }

  function metadataValue(instanceId: string, key: (typeof understoodMetadataKeys)[number]): string | null {
    return null;
  }

  function metadataEntries(instanceId: string): Array<[string, string]> {
    return [];
  }

  function instanceRowKey(instance: DeviceInstance): string {
    return `${instance.instanceId}:${instance.createdAt}:${instance.identityPublicKey ?? ""}`;
  }

  let selectedInstanceIds = $state(new Set<string>());
  let bulkBusy = $state(false);
  let bulkResult = $state<{ succeeded: number; failed: string[] } | null>(null);
  let failedInstances = $state.raw<DeviceInstance[]>([]);
  let confirmationModal: ConfirmationModal | undefined = $state();

  const disableableInstances = $derived(selectedInstances.filter((instance) => instance.state !== "disabled"));
  const selectableInstanceIds = $derived(disableableInstances.map((instance) => instance.instanceId));

  async function disableInstances(targets: DeviceInstance[]) {
    bulkBusy = true;
    bulkResult = null;
    const outcome = await runBulk(targets, async (instance) => {
      const response = await trellis.authDevicesDisable({
        expectedVersion: instance.version,
        idempotencyKey: ulid(),
        instanceId: instance.instanceId,
        reason: null,
      }).take();
      if (isErr(response)) throw new Error(errorMessage(response));
    });
    failedInstances = outcome.failed.map((failure) => failure.target);
    for (const instance of targets) selectedInstanceIds.delete(instance.instanceId);
    bulkResult = {
      succeeded: outcome.succeeded,
      failed: outcome.failed.map((failure) => `${failure.target.instanceId}: ${failure.reason}`),
    };
    bulkBusy = false;
    void load();
  }

  async function requestBulkDisable() {
    const targets = disableableInstances.filter((instance) => selectedInstanceIds.has(instance.instanceId));
    if (targets.length === 0) return;
    const confirmed = await confirmationModal?.confirm({
      title: `Disable ${targets.length} device instance${targets.length === 1 ? "" : "s"}?`,
      message: "Selected device instances stop operating. Each must complete activation again before use.",
      confirmLabel: `Disable ${targets.length}`,
      targetLabel: "Instances",
      targetName: `${targets.length} instances`,
      expectedValue: bulkExpectedCount(targets.length),
      details: bulkTargetDetails(targets.map((instance) => instance.instanceId)),
    });
    if (!confirmed) return;
    await disableInstances(targets);
  }

  function activationRowKey(activation: Activation): string {
    return `${activation.device.instanceId}:${activation.device.updatedAt}`;
  }

  function tabLabel(tab: Tab): string {
    return tab[0].toUpperCase() + tab.slice(1);
  }

  function tabId(tab: Tab): string {
    return `device-detail-tab-${tab}`;
  }

  function tabPanelId(tab: Tab): string {
    return `device-detail-panel-${tab}`;
  }

  async function load() {
    loading = true;
    error = null;
    try {
      const [deploymentsResponse, instancesResponse, activationsResponse, reviewsResponse] = await Promise.all([
        trellis.authDeploymentsList({ kind: "device", limit: 100 }).take(),
        trellis.authDevicesList({ limit: 100 }).take(),
        trellis.authDeviceUserAuthoritiesList({ limit: 100 }).take(),
        trellis.authDeviceUserAuthoritiesReviewsList({ limit: 100 }).take(),
      ]);

      if (isErr(deploymentsResponse)) { error = errorMessage(deploymentsResponse); return; }
      if (isErr(instancesResponse)) { error = errorMessage(instancesResponse); return; }
      if (isErr(activationsResponse)) { error = errorMessage(activationsResponse); return; }
      if (isErr(reviewsResponse)) { error = errorMessage(reviewsResponse); return; }

      deployments = (deploymentsResponse.entries ?? []).filter((deployment): deployment is DeviceDeployment => deployment.kind === "device");
      instances = instancesResponse.entries ?? [];
      activations = activationsResponse.entries ?? [];
      reviews = reviewsResponse.entries ?? [];
      syncSelectedDeployment(deployments);
      if (selectedReviewId && !reviews.some((review) => review.reviewId === selectedReviewId)) selectedReviewId = null;
    } catch (cause) {
      error = errorMessage(cause);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void load();
  });
</script>

<section class="space-y-4">
  <PageToolbar
    title="Devices"
    description="Manage device deployments, provisioned identities, activation state, and review decisions from one operator surface."
  >
    {#snippet actions()}
      <button class="btn btn-ghost btn-sm" onclick={load} disabled={loading}>Refresh</button>
      <a class="btn btn-outline btn-sm" href={resolve("/admin/devices/profiles/new")}>Create deployment</a>
      <a class="btn btn-outline btn-sm" href={resolve("/admin/devices/instances/provision")}>Provision device</a>
    {/snippet}
  </PageToolbar>

  {#if error}
    <Notice variant="error">{error}</Notice>
  {/if}

  {#if loading}
    <Panel><LoadingState label="Loading devices" /></Panel>
  {:else}
    <div class="grid min-h-[calc(100vh-12rem)] items-stretch gap-4 xl:grid-cols-[22rem_minmax(0,1fr)]">
      <SelectionRail title="Deployments" eyebrow={`${deployments.length} deployment${deployments.length === 1 ? "" : "s"}`}>
        <div class="mb-3">
          <label class="input input-bordered input-sm flex items-center gap-2">
            <Icon name="search" size={14} class="text-base-content/50" />
            <input bind:value={search} class="grow" placeholder="Search ID or review mode" aria-label="Search deployments" />
          </label>
        </div>

        {#if deployments.length === 0}
          <EmptyState title="No device deployments" description="Create a deployment before provisioning device identities." />
        {:else}
          <div class="space-y-2">
            {#each filteredDeployments as deployment (deployment.deploymentId)}
              {@const deploymentDeviceInstances = deploymentInstances(deployment.deploymentId)}
              {@const activeDevices = deploymentDeviceInstances.filter((instance) => instance.state === "active")}
              {@const pendingReviewCount = pendingReviewsForDeployment(deployment.deploymentId)}
              <SelectableRecordButton
                selected={selectedDeploymentId === deployment.deploymentId}
                onclick={() => selectDeployment(deployment.deploymentId)}
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <div class="flex items-center gap-2">
                      <span class={["h-2.5 w-2.5 rounded-full", dotClassForDeployment()]}></span>
                      <span class="trellis-identifier truncate font-medium">{deployment.deploymentId}</span>
                    </div>
                    <div class="mt-1 text-xs text-base-content/60">{activeDevices.length}/{deploymentDeviceInstances.length} activated instances</div>
                    <div class="mt-1 flex flex-wrap gap-1">
                      <span class="badge badge-outline badge-xs">review {deployment.reviewMode}</span>
                      <span class="badge badge-outline badge-xs">delegation {deployment.requiresDeviceDelegation ? "required" : "none"}</span>
                      {#if pendingReviewCount > 0}<span class="badge badge-warning badge-xs">{pendingReviewCount} review</span>{/if}
                    </div>
                  </div>
                  <span class={["badge badge-sm", badgeClassForDeployment()]}>{deployment.state === "disabled" ? "Disabled" : "Enabled"}</span>
                </div>
              </SelectableRecordButton>
            {:else}
              <EmptyState title="No matches" description="Try a different deployment ID or review mode." class="py-4" />
            {/each}
          </div>
        {/if}

        {#snippet footer()}
          <span>{deployments.filter((deployment) => deployment.state === "disabled").length} disabled / archived</span>
        {/snippet}
      </SelectionRail>

      <div class="flex min-w-0 flex-col gap-4">
        {#if !selectedDeployment}
          <Panel><EmptyState title="Select a deployment" description="Choose a device deployment from the left rail to inspect instances, activations, and reviews." /></Panel>
        {:else}
          <Panel class="flex min-w-0 flex-1 flex-col [&>.trellis-section-body]:flex-1">
            <div class="flex flex-wrap items-start justify-between gap-3 border-b border-base-300 pb-3">
              <div class="flex min-w-0 items-start gap-3">
                <div class="rounded-box bg-primary/10 p-2.5 text-primary"><Icon name="phone" size={22} /></div>
                <div class="min-w-0">
                  <div class="flex flex-wrap items-center gap-2">
                    <h2 class="trellis-identifier truncate text-lg font-semibold">{selectedDeployment.deploymentId}</h2>
                    <StatusBadge label={selectedDeployment.state === "disabled" ? "Disabled" : "Enabled"} status={deploymentStatus()} />
                  </div>
                  <div class="mt-1 flex flex-wrap gap-1 text-sm text-base-content/60">
                    <span>Review: <span class="badge badge-outline badge-sm">{selectedDeployment.reviewMode}</span></span>
                    <span>Delegation: <span class="badge badge-outline badge-sm">{selectedDeployment.requiresDeviceDelegation ? "required" : "none"}</span></span>
                  </div>
                </div>
              </div>
              <div class="flex flex-wrap gap-2">
                {#if selectedDeployment.state !== "disabled"}
                  <a class="btn btn-error btn-outline btn-sm" href={resolve(`/admin/devices/profiles/disable?deployment=${encodeURIComponent(selectedDeployment.deploymentId)}`)}>Disable deployment</a>
                {/if}
              </div>
            </div>

            <div class="mt-3 flex flex-wrap items-center gap-2 text-sm">
              <span class="badge badge-outline badge-sm">{activeInstanceCount}/{selectedInstances.length} activated instances</span>
              <span class="badge badge-outline badge-sm">{selectedPendingReviews.length} pending review{selectedPendingReviews.length === 1 ? "" : "s"}</span>
              <span class="badge badge-outline badge-sm">{selectedActivations.length} activation{selectedActivations.length === 1 ? "" : "s"}</span>
              <span class="badge badge-outline badge-sm">{revokedActivationCount} revoked</span>
            </div>

            {#if selectedPendingReviews.length > 0}
              <div class="mt-3 rounded-box border border-warning/30 bg-warning/10 px-3 py-2 text-sm">
                <div class="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <div class="font-medium">Activation review required</div>
                    <div class="mt-1 text-xs text-base-content/70">Pending device activations need an approve or reject decision.</div>
                  </div>
                  <button type="button" class="btn btn-ghost btn-xs" onclick={() => selectTab("reviews")}>{selectedPendingReviews.length} pending review{selectedPendingReviews.length === 1 ? "" : "s"}</button>
                </div>
              </div>
            {/if}


            <div class="tabs tabs-box tabs-sm mt-4 w-fit bg-base-200/70 p-1" role="tablist" aria-label="Deployment detail sections">
              {#each tabs as tab (tab)}
                <button type="button" id={tabId(tab)} role="tab" aria-selected={activeTab === tab} aria-controls={tabPanelId(tab)} class={["tab rounded-field px-4", activeTab === tab && "tab-active bg-base-100 shadow-sm"]} onclick={() => selectTab(tab)}>{tabLabel(tab)}</button>
              {/each}
            </div>

            <div id={tabPanelId(activeTab)} class="mt-4 flex-1" role="tabpanel" aria-labelledby={tabId(activeTab)}>
              {#if activeTab === "instances"}
                {#if bulkResult}
                  <BulkResult
                    succeeded={bulkResult.succeeded}
                    failed={bulkResult.failed}
                    pastTense="instances disabled"
                    onRetry={failedInstances.length > 0 ? () => void disableInstances(failedInstances) : undefined}
                    onDismiss={() => { bulkResult = null; }}
                  />
                {:else if selectedInstanceIds.size > 0}
                  <BulkActionBar count={selectedInstanceIds.size} noun="instance" onClear={() => selectedInstanceIds.clear()}>
                    {#snippet actions()}
                      <button class="btn btn-error btn-outline btn-sm" disabled={bulkBusy} onclick={() => void requestBulkDisable()}>
                        {bulkBusy ? "Disabling…" : "Disable selected"}
                      </button>
                    {/snippet}
                  </BulkActionBar>
                {/if}
                <div class="mb-2 flex justify-end">
                  <label class="label cursor-pointer gap-2 py-0">
                    <span class="label-text text-sm">Metadata</span>
                    <input class="toggle toggle-sm" type="checkbox" bind:checked={showMetadata} />
                  </label>
                </div>
                {#if selectedInstances.length === 0}
                  <EmptyState title="No device instances" description="Provisioned device identities for this deployment appear here." />
                {:else}
                  <DataTable>
                      <thead><tr>
                        <th>
                          <span class="sr-only">Select all instances</span>
                          <input
                            type="checkbox"
                            class="checkbox checkbox-xs"
                            aria-label="Select all instances"
                            disabled={bulkBusy || selectableInstanceIds.length === 0}
                            checked={selectableInstanceIds.length > 0 && selectableInstanceIds.every((id) => selectedInstanceIds.has(id))}
                            indeterminate={selectableInstanceIds.some((id) => selectedInstanceIds.has(id)) && !selectableInstanceIds.every((id) => selectedInstanceIds.has(id))}
                            onchange={() => toggleAll(selectedInstanceIds, selectableInstanceIds)}
                          />
                        </th>
                        <th>Instance</th><th>Identity key</th><th>Name</th><th>Serial</th><th>Model</th>{#if showMetadata}<th>Metadata</th>{/if}<th>State</th><th>Created</th><th>Actions</th></tr></thead>
                      <tbody>
                        {#each selectedInstances as instance (instanceRowKey(instance))}
                          <tr>
                            <td>
                              {#if instance.state !== "disabled"}
                                <input
                                  type="checkbox"
                                  class="checkbox checkbox-xs"
                                  aria-label={`Select instance {instance.instanceId}`}
                                  disabled={bulkBusy}
                                  checked={selectedInstanceIds.has(instance.instanceId)}
                                  onchange={() => toggleId(selectedInstanceIds, instance.instanceId)}
                                />
                              {:else}
                                <span class="text-xs text-base-content/50">—</span>
                              {/if}
                            </td>
                            <td class="trellis-identifier font-medium">{instance.instanceId}</td>
                            <td class="trellis-identifier text-base-content/60">{instance.identityPublicKey ?? "—"}</td>
                            <td class="text-base-content/60">{metadataValue(instance.instanceId, "name") ?? "—"}</td>
                            <td class="text-base-content/60">{metadataValue(instance.instanceId, "serialNumber") ?? "—"}</td>
                            <td class="text-base-content/60">{metadataValue(instance.instanceId, "modelNumber") ?? "—"}</td>
                            {#if showMetadata}
                              <td class="text-xs text-base-content/60">
                                {#if metadataEntries(instance.instanceId).length > 0}
                                  <div class="space-y-1">
                                    {#each metadataEntries(instance.instanceId) as [key, value] (key)}
                                      <div><span class="font-medium text-base-content">{key}</span>=<span class="trellis-identifier">{value}</span></div>
                                    {/each}
                                  </div>
                                {:else}
                                  —
                                {/if}
                              </td>
                            {/if}
                            <td><StatusBadge label={instance.state} status={instanceStatus(instance.state)} /></td>
                            <td class="text-base-content/60">{formatDate(instance.createdAt)}</td>
                            <td>
                              {#if instance.state === "disabled"}
                                <span class="text-xs text-base-content/40">—</span>
                              {:else}
                                <a class="btn btn-error btn-outline btn-xs" href={resolve(`/admin/devices/instances/disable?instance=${encodeURIComponent(instance.instanceId)}`)}>Disable</a>
                              {/if}
                            </td>
                          </tr>
                        {/each}
                      </tbody>
                  </DataTable>
                {/if}
              {:else if activeTab === "activations"}
                {#if selectedActivations.length === 0}
                  <EmptyState title="No device activations" description="Activation records for this deployment appear here." />
                {:else}
                  <DataTable>
                      <thead><tr><th>Instance</th><th>Principal</th><th>Delegation</th><th>Created</th><th>Revoked</th><th>Actions</th></tr></thead>
                      <tbody>
                        {#each selectedActivations as activation (activationRowKey(activation))}
                          <tr>
                            <td><div class="trellis-identifier font-medium">{activation.device.instanceId}</div></td>
                            <td class="trellis-identifier text-base-content/60">{activation.device.principalId}</td>
                            <td><StatusBadge label={activation.device.delegationState} status={activationStatus(activation.device.delegationState)} /></td>
                            <td class="text-base-content/60">{formatDate(activation.device.createdAt)}</td>
                            <td class="text-base-content/60">{activation.device.delegationState === "revoked" ? formatDate(activation.device.updatedAt) : "—"}</td>
                            <td>
                              {#if activation.device.delegationState !== "active"}
                                <span class="text-xs text-base-content/40">—</span>
                              {:else}
                                <a class="btn btn-error btn-outline btn-xs" href={resolve(`/admin/devices/activations/revoke?instance=${encodeURIComponent(activation.device.instanceId)}`)}>Revoke</a>
                              {/if}
                            </td>
                          </tr>
                        {/each}
                      </tbody>
                  </DataTable>
                {/if}
              {:else if activeTab === "reviews"}
                <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_22rem]">
                  <div class="min-w-0">
                    {#if selectedReviews.length === 0}
                      <EmptyState title="No device reviews" description="Activation reviews for this deployment appear here." />
                    {:else}
                      <DataTable>
                          <thead><tr><th>Review</th><th>Instance</th><th>State</th><th>Requested</th><th>Actions</th></tr></thead>
                          <tbody>
                            {#each selectedReviews as review (review.reviewId)}
                              <tr class={{ "bg-base-200/60": selectedReview?.reviewId === review.reviewId }}>
                                <td><button class="trellis-identifier text-left hover:underline" onclick={() => (selectedReviewId = review.reviewId)}>{review.reviewId}</button></td>
                                <td><div class="trellis-identifier">{review.instanceId}</div><div class="trellis-identifier text-xs text-base-content/60">{review.devicePrincipalId}</div></td>
                                <td><StatusBadge label={review.state} status={reviewStatus(review.state)} /></td>
                                <td class="text-base-content/60">{formatDate(review.requestedAt)}</td>
                                <td>
                                  {#if review.state === "pending"}
                                    <a class="btn btn-ghost btn-xs" href={resolve(`/admin/devices/reviews/decide?review=${encodeURIComponent(review.reviewId)}`)}>Decide</a>
                                  {:else}
                                    <span class="text-xs text-base-content/40">—</span>
                                  {/if}
                                </td>
                              </tr>
                            {/each}
                          </tbody>
                      </DataTable>
                    {/if}
                  </div>
                  <div class="rounded-box border border-base-300 bg-base-200/30 p-3">
                    {#if selectedReview}
                      <div class="space-y-3 text-sm">
                        <div class="flex items-center justify-between gap-3">
                          <span class="trellis-identifier font-medium">{selectedReview.reviewId}</span>
                          <StatusBadge label={selectedReview.state} status={reviewStatus(selectedReview.state)} />
                        </div>
                        <div>
                          <p class="text-[0.65rem] font-semibold uppercase tracking-wider text-base-content/50">Instance</p>
                          <p class="trellis-identifier">{selectedReview.instanceId}</p>
                          <p class="trellis-identifier text-base-content/60">{selectedReview.devicePrincipalId}</p>
                        </div>
                        <div class="grid grid-cols-2 gap-2 text-xs">
                          <div><span class="text-base-content/50">Requested</span><div>{formatDate(selectedReview.requestedAt)}</div></div>
                          <div><span class="text-base-content/50">Decided</span><div>{selectedReview.decidedAt ? formatDate(selectedReview.decidedAt) : "—"}</div></div>
                          <div class="col-span-2"><span class="text-base-content/50">Reason</span><div>{selectedReview.reason ?? "—"}</div></div>
                        </div>
                        <div class="space-y-0.5 text-xs text-base-content/60">
                          <div><span class="font-medium text-base-content">Name</span>: {metadataValue(selectedReview.instanceId, "name") ?? "—"}</div>
                          <div><span class="font-medium text-base-content">Serial</span>: {metadataValue(selectedReview.instanceId, "serialNumber") ?? "—"}</div>
                          <div><span class="font-medium text-base-content">Model</span>: {metadataValue(selectedReview.instanceId, "modelNumber") ?? "—"}</div>
                        </div>
                        {#if selectedReview.state === "pending"}
                          <a class="btn btn-outline btn-sm w-full" href={resolve(`/admin/devices/reviews/decide?review=${encodeURIComponent(selectedReview.reviewId)}`)}>Decide review</a>
                        {/if}
                      </div>
                    {:else}
                      <EmptyState title="Select a review" description="Choose a review to inspect activation metadata." class="py-4" />
                    {/if}
                  </div>
                </div>
              {/if}
            </div>
          </Panel>
        {/if}
      </div>
    </div>
  {/if}
</section>

<ConfirmationModal bind:this={confirmationModal} />
