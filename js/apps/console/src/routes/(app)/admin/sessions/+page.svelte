<script lang="ts">
  import { isErr } from "@qlever-llc/result";
  import { loadSessionKey } from "@qlever-llc/trellis/auth/browser";
  import type { AuthSessionsRevokeInput } from "@qlever-llc/trellis/sdk/auth";
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import {
    describeSessionPrincipal,
    formatShortKey,
    participantKindBadgeClass,
    participantKindLabel,
    type ConnectionRecord,
    type SessionRecord,
  } from "../../../../lib/auth_display.ts";
  import { errorMessage, formatDate } from "../../../../lib/format";
  import ActionMenu from "$lib/components/ActionMenu.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import DataTable from "$lib/components/DataTable.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import Notice from "$lib/components/Notice.svelte";
  import PageToolbar from "$lib/components/PageToolbar.svelte";
  import Panel from "$lib/components/Panel.svelte";
  import { getNotifications } from "../../../../lib/notifications.svelte";
  import { getTrellis } from "../../../../lib/trellis";

  const trellis = getTrellis();
  const notifications = getNotifications();

  let activeTab = $state<"sessions" | "connections">("sessions");
  let loading = $state(true);
  let error = $state<string | null>(null);

  let sessions = $state<SessionRecord[]>([]);
  let sessionFilterUser = $state("");
  let currentSessionKey = $state<string | null>(null);
  let revokePending = $state(false);
  let confirmationModal: ConfirmationModal | undefined = $state();

  let connections = $state<ConnectionRecord[]>([]);
  let connFilterUser = $state("");
  let connFilterSessionKey = $state("");

  async function loadSessions() {
    loading = true;
    error = null;
    try {
      currentSessionKey = (await loadSessionKey())?.sessionKey ?? null;
      const response = await trellis.request("Auth.Sessions.List", {
        user: sessionFilterUser.trim() || undefined,
        limit: 500,
        offset: 0,
      }).take();
      if (isErr(response)) { error = errorMessage(response); return; }
      sessions = response.entries ?? [];
    } catch (e) {
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  }

  async function loadConnections() {
    loading = true;
    error = null;
    try {
      const response = await trellis.request("Auth.Connections.List", {
        user: connFilterUser.trim() || undefined,
        sessionKey: connFilterSessionKey.trim() || undefined,
        limit: 500,
        offset: 0,
      }).take();
      if (isErr(response)) { error = errorMessage(response); return; }
      connections = response.entries ?? [];
    } catch (e) {
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  }

  function loadActive() {
    if (activeTab === "sessions") void loadSessions();
    else void loadConnections();
  }

  function isCurrentSession(session: SessionRecord): boolean {
    return !!currentSessionKey && session.sessionKey === currentSessionKey;
  }

  async function revokeUserSessions() {
    if (revokePending) return;
    const userSessions = sessions.filter((session) => session.principal.type === "user");
    if (userSessions.length === 0) return;
    const includesCurrentSession = userSessions.some((session) => isCurrentSession(session));
    const confirmed = await confirmationModal?.confirm({
      title: sessionFilterUser.trim() ? "Revoke filtered user sessions?" : "Revoke all user sessions?",
      message: includesCurrentSession
        ? "This includes your current console session. Continuing will force this app to sign in again."
        : `This immediately invalidates ${userSessions.length} active user session${userSessions.length !== 1 ? "s" : ""}.`,
      confirmLabel: "Revoke sessions",
      targetLabel: "Scope",
      targetName: sessionFilterUser.trim() ? `Filter: ${sessionFilterUser.trim()}` : "All listed user sessions",
      expectedValue: "REVOKE",
      details: `Type REVOKE to confirm revoking ${userSessions.length} active user session${userSessions.length !== 1 ? "s" : ""}.`,
    });
    if (!confirmed) return;

    revokePending = true;
    try {
      const orderedSessions = [...userSessions].sort((left, right) => Number(isCurrentSession(left)) - Number(isCurrentSession(right)));
      for (const session of orderedSessions) {
        const response = await trellis.request("Auth.Sessions.Revoke", { sessionKey: session.sessionKey } satisfies AuthSessionsRevokeInput).take();
        if (isErr(response)) {
          notifications.error(errorMessage(response), "Revoke failed");
          return;
        }
      }
      notifications.success(`Revoked ${userSessions.length} user session${userSessions.length !== 1 ? "s" : ""}.`, "Sessions revoked");
      await loadSessions();
    } catch (e) {
      notifications.error(errorMessage(e), "Revoke failed");
    } finally {
      revokePending = false;
    }
  }

  onMount(() => { void loadSessions(); });
</script>

<section class="space-y-4">
  <PageToolbar title="Sessions" description="Inspect active sessions and connections and disconnect compromised principals.">
    {#snippet actions()}
      <button class="btn btn-ghost btn-sm" onclick={loadActive} disabled={loading}>Refresh</button>
      <ActionMenu buttonBaseClass="btn btn-outline btn-sm" menuClass="z-10" widthClass="w-56">
        {#snippet summary()}
          Actions <Icon name="chevronDown" size={14} />
        {/snippet}
        <li><a href={resolve("/admin/sessions/revoke")}>Revoke a session</a></li>
        <li><button class="text-error" type="button" onclick={() => void revokeUserSessions()} disabled={activeTab !== "sessions" || loading || revokePending || sessions.every((session) => session.principal.type !== "user")}>Revoke user sessions</button></li>
        <li><a href={resolve("/admin/sessions/kick")}>Kick a connection</a></li>
      </ActionMenu>
    {/snippet}
  </PageToolbar>

  <div class="flex items-center justify-between">
    <div role="tablist" class="tabs tabs-bordered">
      <button
        role="tab"
        class={["tab", activeTab === "sessions" && "tab-active"]}
        onclick={() => { activeTab = "sessions"; void loadSessions(); }}
      >Sessions</button>
      <button
        role="tab"
        class={["tab", activeTab === "connections" && "tab-active"]}
        onclick={() => { activeTab = "connections"; void loadConnections(); }}
      >Connections</button>
    </div>
  </div>

  {#if error}
    <Notice variant="error">{error}</Notice>
  {/if}

  {#if activeTab === "sessions"}
    <form class="flex gap-2 items-end" onsubmit={(e) => { e.preventDefault(); void loadSessions(); }}>
      <input class="input input-bordered input-sm w-60" placeholder="Filter by principal…" bind:value={sessionFilterUser} />
      <button type="submit" class="btn btn-outline btn-sm" disabled={loading}>Apply</button>
      {#if sessionFilterUser.trim()}
        <button type="button" class="btn btn-ghost btn-sm" onclick={() => { sessionFilterUser = ""; void loadSessions(); }}>Clear</button>
      {/if}
    </form>

    {#if loading}
      <Panel><LoadingState label="Loading sessions" /></Panel>
    {:else if sessions.length === 0}
      <EmptyState title="No sessions" description="No sessions match the current filter." />
    {:else}
      <Panel title="Sessions" eyebrow="Primary table">
        <DataTable fixed tableClass="sessions-table w-full">
          <colgroup>
            <col class="w-[44%]" />
            <col class="w-24" />
            <col class="w-32" />
            <col class="w-44" />
            <col class="w-24" />
          </colgroup>
          <thead>
            <tr>
              <th>Principal</th>
              <th>Kind</th>
              <th>Session Key</th>
              <th>Activity</th>
              <th class="sessions-actions-cell text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each sessions as session (session.key)}
              {@const summary = describeSessionPrincipal(session)}
              <tr>
                <td class="min-w-0">
                  <div class="truncate font-medium" title={summary.title}>{summary.title}</div>
                  {#if summary.details}
                    <div class="truncate text-xs text-base-content/60" title={summary.details}>{summary.details}</div>
                  {/if}
                </td>
                <td>
                  <span class={["badge badge-sm", participantKindBadgeClass(session.participantKind)]}>
                    {participantKindLabel(session.participantKind)}
                  </span>
                </td>
                <td class="trellis-identifier text-base-content/60">{formatShortKey(session.sessionKey)}</td>
                <td class="text-xs text-base-content/60">
                  <div>Last auth {formatDate(session.lastAuth)}</div>
                  <div>Created {formatDate(session.createdAt)}</div>
                </td>
                <td class="sessions-actions-cell text-right">
                  <div class="flex items-center justify-end gap-2">
                    {#if isCurrentSession(session)}
                      <span class="badge badge-info badge-sm">Current</span>
                    {/if}
                    <ActionMenu menuClass="z-10" widthClass="w-48">
                      <li><a class="text-error" href={resolve(`/admin/sessions/revoke?sessionKey=${encodeURIComponent(session.sessionKey)}`)}>Revoke</a></li>
                    </ActionMenu>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </DataTable>
      <p class="text-xs text-base-content/50">{sessions.length} session{sessions.length !== 1 ? "s" : ""}</p>
      </Panel>
    {/if}

  {:else}
    <form class="flex gap-2 items-end" onsubmit={(e) => { e.preventDefault(); void loadConnections(); }}>
      <input class="input input-bordered input-sm w-48" placeholder="Filter by principal…" bind:value={connFilterUser} />
      <input class="input input-bordered input-sm w-48" placeholder="Filter by session key…" bind:value={connFilterSessionKey} />
      <button type="submit" class="btn btn-outline btn-sm" disabled={loading}>Apply</button>
      {#if connFilterUser.trim() || connFilterSessionKey.trim()}
        <button type="button" class="btn btn-ghost btn-sm" onclick={() => { connFilterUser = ""; connFilterSessionKey = ""; void loadConnections(); }}>Clear</button>
      {/if}
    </form>

    {#if loading}
      <Panel><LoadingState label="Loading connections" /></Panel>
    {:else if connections.length === 0}
      <EmptyState title="No connections" description="No active connections match the current filter." />
    {:else}
      <Panel title="Connections" eyebrow="Primary table">
      <DataTable>
          <thead>
            <tr>
              <th>Principal</th>
              <th>Kind</th>
              <th>Session Key</th>
              <th>User NKey</th>
              <th>Server</th>
              <th>Connected</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each connections as connection (connection.key)}
              {@const summary = describeSessionPrincipal(connection)}
              <tr>
                <td>
                  <div class="font-medium">{summary.title}</div>
                  {#if summary.details}
                    <div class="text-xs text-base-content/60">{summary.details}</div>
                  {/if}
                </td>
                <td>
                  <span class={["badge badge-sm", participantKindBadgeClass(connection.participantKind)]}>
                    {participantKindLabel(connection.participantKind)}
                  </span>
                </td>
                <td class="trellis-identifier text-base-content/60">{formatShortKey(connection.sessionKey)}</td>
                <td class="trellis-identifier text-base-content/60">{formatShortKey(connection.userNkey)}</td>
                <td>
                  <span class="text-sm">{connection.serverId}</span>
                  <span class="text-xs text-base-content/50 block">client {connection.clientId}</span>
                </td>
                <td class="text-base-content/60">{formatDate(connection.connectedAt)}</td>
                <td class="text-right">
                  <ActionMenu menuClass="z-10" widthClass="w-48">
                      <li><a class="text-error" href={resolve(`/admin/sessions/kick?userNkey=${encodeURIComponent(connection.userNkey)}`)}>Kick</a></li>
                  </ActionMenu>
                </td>
              </tr>
            {/each}
          </tbody>
      </DataTable>
      <p class="text-xs text-base-content/50">{connections.length} connection{connections.length !== 1 ? "s" : ""}</p>
      </Panel>
    {/if}
  {/if}
</section>

<ConfirmationModal bind:this={confirmationModal} />

<style>
  :global(.sessions-table .sessions-actions-cell) {
    position: sticky;
    right: 0;
    z-index: 1;
    background-color: var(--color-base-100);
  }

  :global(.sessions-table thead .sessions-actions-cell) {
    z-index: 2;
  }
</style>
