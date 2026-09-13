<script lang="ts">
  import { goto, afterNavigate } from "$app/navigation";
  import { base, resolve } from "$lib/console_paths";
  import { page } from "$app/state";
  import { type apis } from "trellis-web-generated";
  import type { Snippet } from "svelte";
  import { onDestroy, onMount } from "svelte";
  import { buildConsoleLoginUrl } from "../auth";
  import {
    canAccessRoute,
    getVisibleNavSections,
    type Authority,
    type NavSection,
  } from "../control-panel.ts";
  import { errorMessage } from "../format";
  import { NotificationsController, setNotifications } from "../notifications.svelte";
  import { getAuthenticatedUser, getConnection, getTrellis, type ConnectionStatus } from "../trellis";
  import AppShell from "./AppShell.svelte";

  type Props = {
    children: Snippet;
  };

  let { children }: Props = $props();

  const connection = getConnection();
  const trellis = getTrellis();
  const notifications = setNotifications(new NotificationsController());

  let authFailure = $state<string | null>(null);
  const connectionStatus = $derived<ConnectionStatus["phase"]>(connection.status.phase);
  let authority = $state<Authority | null>(null);
  let navSections = $state<NavSection[]>(getVisibleNavSections(null));
  let profile = $state<apis.auth.SessionsMeOutput["user"] | null>(null);
  let profileLoaded = $state(false);

  function toRoutePath(pathname: string): string {
    if (base && pathname === base) {
      return "/";
    }

    if (base && pathname.startsWith(`${base}/`)) {
      return pathname.slice(base.length);
    }

    return pathname;
  }

  function enforceAuthorityAccess(pathname: string): void {
    if (!profileLoaded || canAccessRoute(pathname, authority)) {
      return;
    }

    authFailure = "Your account does not have access to this operations page.";
    void goto(resolve("/profile"));
  }

  async function authMe() {
    return await getAuthenticatedUser(trellis);
  }

  async function signOut(): Promise<void> {
    try {
      await trellis.logout();
    } finally {
      window.location.href = buildConsoleLoginUrl({ redirectTo: "/profile" });
    }
  }

  afterNavigate(({ to }) => {
    if (!to) return;
    enforceAuthorityAccess(toRoutePath(to.url.pathname));
  });

  onMount(() => {
    let active = true;

    void (async () => {
      try {
        const me = await authMe();
        if (!active) return;

        authority = {
          platformPrivileges: me.connection.platformPrivileges,
          grants: me.connection.grants,
        };
        navSections = getVisibleNavSections(authority);
        if (me.user) {
          profile = me.user;
        }
      } catch (error) {
        if (!active) return;
        authFailure = errorMessage(error);
      } finally {
        if (active) {
          profileLoaded = true;
          enforceAuthorityAccess(toRoutePath(page.url.pathname));
        }
      }
    })();

    return () => {
      active = false;
    };
  });

  onDestroy(() => {
    notifications.clear();
  });
</script>

<AppShell
  {profile}
  {authority}
  {profileLoaded}
  {navSections}
  {connectionStatus}
  {authFailure}
  onSignOut={signOut}
>
  {@render children()}
</AppShell>
