<script lang="ts">
  import { isErr } from "@qlever-llc/result";
  import { loadSessionKey } from "@qlever-llc/trellis/auth/browser";
  import type { AuthSessionsRevokeInput, AuthUsersListOutput } from "@qlever-llc/trellis/sdk/auth";
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import ActionMenu from "$lib/components/ActionMenu.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import DataTable from "$lib/components/DataTable.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import Notice from "$lib/components/Notice.svelte";
  import PageToolbar from "$lib/components/PageToolbar.svelte";
  import Panel from "$lib/components/Panel.svelte";
  import { describeSessionPrincipal, formatShortKey, participantKindBadgeClass, participantKindLabel, type SessionRecord } from "../../../../lib/auth_display.ts";
  import { errorMessage, formatDate } from "../../../../lib/format";
  import { getNotifications } from "../../../../lib/notifications.svelte";
  import { getTrellis } from "../../../../lib/trellis";

  const trellis = getTrellis();
  const notifications = getNotifications();

  type UserView = AuthUsersListOutput["entries"][number];
  type IdentityView = UserView["identities"][number];
  type PasswordResetResult = {
    name: string | null;
    username: string | null;
    email: string | null;
    resetUrl: string;
    expiresAt: string;
  };

  function identityLabel(user: UserView) {
    return userDisplayName(user) ?? userDisplayEmail(user) ?? userDisplayUsername(user) ?? "Unnamed user";
  }

  function userKey(user: Pick<UserView, "userId">) {
    return user.userId;
  }

  function primaryIdentity(user: UserView): IdentityView | null {
    return user.identities[0] ?? null;
  }

  function localIdentity(user: UserView): IdentityView | null {
    return user.identities.find((identity) => identity.provider.trim().toLowerCase() === "local") ?? null;
  }

  function userDisplayName(user: UserView): string | null {
    return user.name?.trim() || primaryIdentity(user)?.displayName?.trim() || null;
  }

  function userDisplayEmail(user: UserView): string | null {
    return user.email?.trim() || primaryIdentity(user)?.email?.trim() || null;
  }

  function userDisplayUsername(user: UserView): string | null {
    return localIdentity(user)?.subject.trim() || null;
  }

  function identitySummary(user: UserView): string {
    const identity = primaryIdentity(user);
    if (!identity) return "No linked identity";
    return `${identity.provider}:${identity.subject}`;
  }

  function identityCountLabel(count: number): string {
    return `${count} ${count === 1 ? "identity" : "identities"}`;
  }

  function identityProvidersLabel(user: UserView): string {
    const providers = Array.from(new Set(user.identities.map((identity) => identity.provider.trim()).filter(Boolean)));
    if (providers.length === 0) return "No providers";
    return providers.join(", ");
  }

  function sessionsForUser(userId: string): SessionRecord[] {
    return sessions.filter((session) => session.principal.type === "user" && session.principal.userId === userId);
  }

  let loading = $state(true);
  let error = $state<string | null>(null);
  let sessionsWarning = $state<string | null>(null);
  let users = $state<UserView[]>([]);
  let sessions = $state<SessionRecord[]>([]);
  let userLastAuth = $state<Record<string, string>>({});
  let selectedUserId = $state<string | null>(null);
  let currentSessionKey = $state<string | null>(null);
  let revokePending = $state(false);
  let confirmationModal: ConfirmationModal | undefined = $state();
  let resetPendingUserId = $state<string | null>(null);
  let resetResult = $state<PasswordResetResult | null>(null);
  let resetDialog = $state<HTMLDialogElement | null>(null);

  const activeUserCount = $derived(users.filter((user) => user.active).length);
  const inactiveUserCount = $derived(users.length - activeUserCount);
  const selectedUser = $derived(users.find((user) => user.userId === selectedUserId) ?? null);
  const selectedUserSessions = $derived(selectedUserId
    ? sessions.filter((session) => session.principal.type === "user" && session.principal.userId === selectedUserId)
    : []);

  async function load() {
    loading = true;
    error = null;
    sessionsWarning = null;
    try {
      const usersResponse = await trellis.request("Auth.Users.List", { limit: 500, offset: 0 }).take();
      if (isErr(usersResponse)) { error = errorMessage(usersResponse); return; }
      users = usersResponse.entries ?? [];
      currentSessionKey = (await loadSessionKey())?.sessionKey ?? null;

      const sessionsResponse = await trellis.request("Auth.Sessions.List", { limit: 500, offset: 0 }).take();
      if (isErr(sessionsResponse)) {
        sessionsWarning = `Last-auth metadata unavailable: ${errorMessage(sessionsResponse)}`;
        userLastAuth = {};
        sessions = [];
        return;
      }
      sessions = sessionsResponse.entries ?? [];

      const lastAuthByUser: Record<string, string> = {};
      for (const session of sessions) {
        if (session.principal.type !== "user") continue;
        const key = session.principal.userId;
        if (!lastAuthByUser[key] || session.lastAuth > lastAuthByUser[key]) {
          lastAuthByUser[key] = session.lastAuth;
        }
      }
      userLastAuth = lastAuthByUser;
    } catch (e) { error = errorMessage(e); }
    finally { loading = false; }
  }

  async function revokeSessions(targetSessions: SessionRecord[], title: string, targetUser = selectedUser) {
    if (revokePending || targetSessions.length === 0) return;
    const includesCurrentSession = targetSessions.some((session) => session.sessionKey === currentSessionKey);
    const confirmed = await confirmationModal?.confirm({
      title,
      message: includesCurrentSession
        ? "This includes your current console session. Continuing will force this app to sign in again."
        : `This immediately invalidates ${targetSessions.length} active user session${targetSessions.length !== 1 ? "s" : ""}.`,
      confirmLabel: targetSessions.length === 1 ? "Revoke session" : "Revoke sessions",
      targetLabel: "User",
      targetName: targetUser ? identityLabel(targetUser) : undefined,
      expectedValue: targetSessions.length === 1 ? targetSessions[0].sessionKey : "REVOKE",
      details: targetSessions.length === 1
        ? "Type the session key to confirm."
        : "Type REVOKE to confirm revoking all sessions shown for this user.",
    });
    if (!confirmed) return;

    revokePending = true;
    try {
      const orderedSessions = [...targetSessions].sort((left, right) => Number(left.sessionKey === currentSessionKey) - Number(right.sessionKey === currentSessionKey));
      for (const session of orderedSessions) {
        const response = await trellis.request("Auth.Sessions.Revoke", { sessionKey: session.sessionKey } satisfies AuthSessionsRevokeInput).take();
        if (isErr(response)) {
          notifications.error(errorMessage(response), "Revoke failed");
          return;
        }
      }
      notifications.success(`Revoked ${targetSessions.length} session${targetSessions.length !== 1 ? "s" : ""}.`, "Sessions revoked");
      await load();
    } catch (e) {
      notifications.error(errorMessage(e), "Revoke failed");
    } finally {
      revokePending = false;
    }
  }

  async function createPasswordReset(user: UserView) {
    if (resetPendingUserId) return;
    resetPendingUserId = user.userId;
    try {
      const response = await trellis.request("Auth.Users.PasswordReset.Create", {
        userId: user.userId,
      }).take();
      if (isErr(response)) {
        notifications.error(errorMessage(response), "Password reset failed");
        return;
      }

      resetResult = {
        name: userDisplayName(user),
        username: userDisplayUsername(user),
        email: userDisplayEmail(user),
        resetUrl: response.url,
        expiresAt: response.expiresAt,
      };
      resetDialog?.showModal();
      notifications.success(`Created password reset link for ${identityLabel(user)}.`, "Password reset ready");
    } catch (e) {
      notifications.error(errorMessage(e), "Password reset failed");
    } finally {
      resetPendingUserId = null;
    }
  }

  async function copyResetUrl() {
    if (!resetResult) return;
    if (typeof navigator === "undefined" || !navigator.clipboard) {
      notifications.error("Clipboard access is unavailable in this browser.", "Copy failed");
      return;
    }

    try {
      await navigator.clipboard.writeText(resetResult.resetUrl);
      notifications.success("Password reset URL copied to clipboard.", "Copied");
    } catch (e) {
      notifications.error(errorMessage(e), "Copy failed");
    }
  }

  function resetDialogAttachment(element: HTMLDialogElement) {
    resetDialog = element;
    return () => {
      if (resetDialog === element) resetDialog = null;
    };
  }

  onMount(() => { void load(); });
</script>

<section class="space-y-4">
  <PageToolbar title="Users" description="Manage user activation and capabilities.">
    {#snippet actions()}
      <a class="btn btn-outline btn-sm" href={resolve("/admin/users/new")}>New user</a>
      <button class="btn btn-ghost btn-sm" onclick={load} disabled={loading}>Refresh</button>
    {/snippet}
  </PageToolbar>

  {#if error}
    <Notice variant="error">{error}</Notice>
  {/if}
  {#if sessionsWarning}
    <Notice variant="info">{sessionsWarning}</Notice>
  {/if}

  {#if loading}
    <Panel><LoadingState label="Loading users" /></Panel>
  {:else if users.length === 0}
    <EmptyState title="No users" description="No users have been registered yet." />
  {:else}
    <div class="space-y-2">
      <div class="flex flex-col gap-3 border-y border-base-300 bg-base-100/45 px-3 py-3 sm:flex-row sm:items-center sm:justify-between sm:px-4">
        <div class="min-w-0">
          <p class="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-base-content/45">User registry</p>
          <p class="mt-1 text-sm text-base-content/60">Account identity, activity, and activation state.</p>
        </div>
        <div class="flex shrink-0 flex-wrap items-center gap-2">
          <span class="badge badge-success badge-sm">{activeUserCount} active</span>
          {#if inactiveUserCount > 0}
            <span class="badge badge-neutral badge-sm">{inactiveUserCount} inactive</span>
          {/if}
        </div>
      </div>

      <DataTable class="users-table border-b border-base-300 bg-base-100/30" overflow="visible">
        <thead>
          <tr>
            <th class="w-[28%]">Name</th>
            <th class="hidden w-[18%] sm:table-cell">Username</th>
            <th class="hidden w-[30%] md:table-cell">Email</th>
            <th class="hidden w-[14%] lg:table-cell">Last active</th>
            <th class="w-[6rem]">Status</th>
            <th class="w-[5rem] text-right">Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each users as user (user.userId)}
            <tr class={["users-row", selectedUserId === user.userId && "users-row-selected", !user.active && "users-row-inactive"]}>
              <td class="max-w-0 align-top">
                <div class="flex min-w-0 items-start gap-3">
                  <span class={["mt-1.5 size-2 rounded-full", user.active ? "bg-success" : "bg-base-content/25"]} aria-hidden="true"></span>
                  <div class="min-w-0 space-y-1">
                    <div class="flex min-w-0 items-center gap-2">
                       <span class="truncate font-medium" title={identityLabel(user)}>{identityLabel(user)}</span>
                    </div>
                  </div>
                </div>
              </td>
              <td class="hidden max-w-0 align-top sm:table-cell">
                <span class="trellis-identifier block truncate text-xs text-base-content/70" title={userDisplayUsername(user) ?? identitySummary(user)}>{userDisplayUsername(user) ?? "Not set"}</span>
              </td>
              <td class="hidden max-w-0 align-top md:table-cell">
                <span class="block truncate text-xs text-base-content/70" title={userDisplayEmail(user) ?? "Not set"}>{userDisplayEmail(user) ?? "Not set"}</span>
                <span class="mt-1 block truncate text-[0.68rem] uppercase tracking-[0.08em] text-base-content/40" title={identityProvidersLabel(user)}>{identityCountLabel(user.identities.length)} · {identityProvidersLabel(user)}</span>
              </td>
              <td class="hidden w-36 align-top text-xs text-base-content/60 lg:table-cell">
                {#if userLastAuth[userKey(user)]}
                  <span title={userLastAuth[userKey(user)]}>{formatDate(userLastAuth[userKey(user)])}</span>
                {:else}
                  <span>Not active</span>
                {/if}
              </td>
              <td class="w-28 align-top">
                {#if user.active}
                  <span class="badge badge-success badge-sm">Active</span>
                {:else}
                  <span class="badge badge-neutral badge-sm">Inactive</span>
                {/if}
              </td>
              <td class="w-24 whitespace-nowrap text-right align-top">
                <ActionMenu widthClass="w-44">
                    <li><button type="button" onclick={() => { selectedUserId = user.userId; }}>View sessions</button></li>
                    <li><button class="text-error" type="button" onclick={() => { selectedUserId = user.userId; void revokeSessions(sessionsForUser(user.userId), "Revoke all sessions for this user?", user); }} disabled={revokePending || sessionsForUser(user.userId).length === 0}>Revoke sessions</button></li>
                    <li><a href={resolve(`/admin/users/edit?userId=${encodeURIComponent(user.userId)}`)}>Edit</a></li>
                    <li><button type="button" onclick={() => void createPasswordReset(user)} disabled={resetPendingUserId !== null}>{resetPendingUserId === user.userId ? "Creating reset..." : "Create reset link"}</button></li>
                </ActionMenu>
              </td>
            </tr>
          {/each}
        </tbody>
      </DataTable>
      <p class="text-xs text-base-content/50">{users.length} user{users.length !== 1 ? "s" : ""}</p>

      {#if selectedUser}
        <Panel title="Active sessions" eyebrow="Selected user">
          <div class="space-y-3">
            <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
              <div class="min-w-0">
                <div class="truncate text-sm font-medium" title={identityLabel(selectedUser)}>{identityLabel(selectedUser)}</div>
                <div class="trellis-identifier truncate text-xs text-base-content/60" title={selectedUser.userId}>{selectedUser.userId}</div>
              </div>
              <button class="btn btn-error btn-sm" type="button" onclick={() => void revokeSessions(selectedUserSessions, "Revoke all sessions for this user?")} disabled={revokePending || selectedUserSessions.length === 0}>{revokePending ? "Revoking..." : "Revoke all sessions"}</button>
            </div>

            {#if selectedUserSessions.length === 0}
              <EmptyState title="No active sessions" description="This user does not currently have active sessions." />
            {:else}
              <DataTable fixed tableClass="w-full" overflow="visible">
                <colgroup>
                  <col class="w-[40%]" />
                  <col class="w-28" />
                  <col class="w-36" />
                  <col class="w-40" />
                  <col class="w-24" />
                </colgroup>
                <thead>
                  <tr>
                    <th>Principal</th>
                    <th>Kind</th>
                    <th>Session Key</th>
                    <th>Activity</th>
                    <th class="text-right">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {#each selectedUserSessions as session (session.key)}
                    {@const summary = describeSessionPrincipal(session)}
                    <tr>
                      <td class="min-w-0">
                        <div class="truncate font-medium" title={summary.title}>{summary.title}</div>
                        {#if summary.details}
                          <div class="truncate text-xs text-base-content/60" title={summary.details}>{summary.details}</div>
                        {/if}
                      </td>
                      <td>
                        <span class={["badge badge-sm", participantKindBadgeClass(session.participantKind)]}>{participantKindLabel(session.participantKind)}</span>
                      </td>
                      <td class="trellis-identifier text-base-content/60">{formatShortKey(session.sessionKey)}</td>
                      <td class="text-xs text-base-content/60">
                        <div>Last auth {formatDate(session.lastAuth)}</div>
                        <div>Created {formatDate(session.createdAt)}</div>
                      </td>
                      <td class="text-right">
                        <button class="btn btn-ghost btn-xs text-error" type="button" onclick={() => void revokeSessions([session], "Revoke this session?")} disabled={revokePending}>Revoke</button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </DataTable>
            {/if}
          </div>
        </Panel>
      {/if}
    </div>
  {/if}
</section>

<dialog class="modal" {@attach resetDialogAttachment}>
  <div class="modal-box max-w-2xl border border-base-300 bg-base-100 p-0">
    {#if resetResult}
      <form method="dialog" class="absolute right-3 top-3">
        <button class="btn btn-ghost btn-xs btn-square" aria-label="Close password reset link dialog">x</button>
      </form>
      <div class="border-b border-base-300 px-5 py-4">
        <p class="text-[0.65rem] font-semibold uppercase tracking-[0.12em] text-base-content/45">Password reset link</p>
        <dl class="mt-3 grid gap-3 text-sm sm:grid-cols-3">
          <div class="min-w-0">
            <dt class="text-xs font-medium uppercase tracking-wide text-base-content/50">Name</dt>
            <dd class="truncate font-medium" title={resetResult.name ?? "Not set"}>{resetResult.name ?? "Not set"}</dd>
          </div>
          <div class="min-w-0">
            <dt class="text-xs font-medium uppercase tracking-wide text-base-content/50">Username</dt>
            <dd class="trellis-identifier truncate" title={resetResult.username ?? "Not set"}>{resetResult.username ?? "Not set"}</dd>
          </div>
          <div class="min-w-0">
            <dt class="text-xs font-medium uppercase tracking-wide text-base-content/50">Email</dt>
            <dd class="truncate" title={resetResult.email ?? "Not set"}>{resetResult.email ?? "Not set"}</dd>
          </div>
        </dl>
        <p class="trellis-field-help mt-1">Expires {formatDate(resetResult.expiresAt)}.</p>
      </div>
      <div class="px-5 py-4">
        <p class="trellis-field-help">Copy and send this URL to the user.</p>
        <input class="input input-bordered input-sm mt-3 w-full trellis-identifier" readonly value={resetResult.resetUrl} aria-label="Password reset URL" />
        <div class="mt-4 flex flex-wrap justify-end gap-2">
          <button class="btn btn-outline btn-sm" type="button" onclick={copyResetUrl}>Copy</button>
        </div>
      </div>
    {/if}
  </div>
  <form method="dialog" class="modal-backdrop">
    <button>close</button>
  </form>
</dialog>

<ConfirmationModal bind:this={confirmationModal} />

<style>
  :global(.users-table) {
    min-width: 0;
    table-layout: fixed;
    width: 100%;
  }

  :global(.users-table thead) {
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 3.5%,
      transparent
    );
  }

  .users-row {
    transition: background-color 160ms ease-out;
  }

  .users-row-inactive {
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 2.5%,
      transparent
    );
  }

  .users-row-selected {
    background-color: color-mix(
      in oklab,
      var(--color-primary) 8%,
      transparent
    );
  }

</style>
