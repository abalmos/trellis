<script lang="ts">
  import { TrellisProvider } from "@qlever-llc/trellis-svelte";
  import type { Component, Snippet } from "svelte";
  import { onMount } from "svelte";
  import { setSelectedTrellisUrl, trellisApp } from "$lib/trellis-context.svelte";
  import AuthenticatedApp from "../../lib/components/AuthenticatedApp.svelte";
  import AdminAccountRedirect from "../../lib/components/AdminAccountRedirect.svelte";
  import { buildConsoleLoginUrl } from "../../lib/auth";
  import {
    buildAdminAccountLoginUrl,
    consumeAdminAccountToken,
  } from "../../lib/admin_account";
  import { APP_CONFIG } from "../../lib/config";

  type Props = {
    children: Snippet;
  };
  type ConsoleTrellisProviderProps = {
    trellisApp: typeof trellisApp;
    auth: { redirectTo(): string };
    onAuthRequired(loginUrl: string): void;
    onRecoverableAuthError(error: unknown): void | Promise<void>;
    children: Snippet;
    loading: Snippet;
    recoveringAuth: Snippet;
    error: Snippet<[unknown]>;
  };

  const ConsoleTrellisProvider = TrellisProvider as Component<ConsoleTrellisProviderProps>;

  let { children }: Props = $props();
  let initialized = $state(false);
  let adminAccountToken = $state<string | null>(null);

  function currentPath(): string {
    return window.location.pathname + window.location.search;
  }

  onMount(() => {
    const currentUrl = new URL(window.location.href);
    adminAccountToken = consumeAdminAccountToken(currentUrl);
    if (adminAccountToken) {
      window.history.replaceState(window.history.state, "", currentUrl);
    }
    if (!APP_CONFIG.authUrl) {
      window.location.href = buildConsoleLoginUrl({
        redirectTo: currentPath(),
        location: window.location,
      });
      return;
    }

    setSelectedTrellisUrl(APP_CONFIG.authUrl);
    initialized = true;
  });

  function redirectToLogin(loginUrl: string): void {
    if (loginUrl) {
      if (adminAccountToken) {
        const setupUrl = buildAdminAccountLoginUrl(loginUrl, adminAccountToken);
        if (setupUrl) {
          adminAccountToken = null;
          window.location.href = setupUrl;
          return;
        }
      }
      window.location.href = loginUrl;
      return;
    }

    window.location.href = buildConsoleLoginUrl({
      redirectTo: currentPath(),
      location: window.location,
      authError: "Your session ended. Sign in again.",
    });
  }

  type SerializableError = {
    message?: unknown;
    code?: unknown;
    hint?: unknown;
    context?: unknown;
  };

  function maybeSerializable(value: unknown): SerializableError | undefined {
    if (!value || typeof value !== "object" || !("toSerializable" in value)) return undefined;
    const fn = (value as { toSerializable: unknown }).toSerializable;
    if (typeof fn !== "function") return undefined;
    const s = fn.call(value);
    return s && typeof s === "object" ? (s as SerializableError) : undefined;
  }

  function connectionErrorDetails(error: unknown): string {
    const parts: string[] = [];
    const s = maybeSerializable(error);
    const msg =
      typeof s?.message === "string"
        ? s.message
        : error instanceof Error
          ? error.message
          : String(error ?? "Unknown error");
    parts.push(`Error: ${msg}`);
    if (typeof s?.code === "string") parts.push(`Code: ${s.code}`);
    if (typeof s?.hint === "string") parts.push(`Hint: ${s.hint}`);
    if (s?.context && typeof s.context === "object") {
      parts.push(`Context: ${JSON.stringify(s.context, null, 2)}`);
    } else if (error instanceof Error && error.stack) {
      parts.push(error.stack);
    }
    return parts.join("\n\n");
  }

  let recovering = $state(false);

  function recoverAuth(): void {
    if (recovering) return;
    recovering = true;
    window.location.href = buildConsoleLoginUrl({
      redirectTo: currentPath(),
      location: window.location,
    });
  }

  function goToConsole(): void {
    window.location.href = "/console";
  }
</script>

{#if initialized}
  <ConsoleTrellisProvider
    {trellisApp}
    auth={{ redirectTo: () => window.location.href }}
    onAuthRequired={redirectToLogin}
    onRecoverableAuthError={recoverAuth}
  >
    {#snippet loading()}
      <div class="flex min-h-screen items-center justify-center bg-base-200 px-4 py-10">
        <div class="card trellis-card w-full max-w-sm border border-base-300 bg-base-100 shadow-none">
          <div class="card-body text-center gap-3">
            <h1 class="text-lg font-semibold">Connecting</h1>
            <span class="loading loading-spinner loading-md mx-auto"></span>
          </div>
        </div>
      </div>
    {/snippet}

    {#snippet recoveringAuth()}
      <div class="flex min-h-screen items-center justify-center bg-base-200 px-4 py-10">
        <div class="card trellis-card w-full max-w-sm border border-base-300 bg-base-100 shadow-none">
          <div class="card-body text-center gap-3">
            <h1 class="text-lg font-semibold">Connecting</h1>
            <span class="loading loading-spinner loading-md mx-auto"></span>
          </div>
        </div>
      </div>
    {/snippet}

    {#snippet error(connectError)}
      <div class="flex min-h-screen items-center justify-center bg-base-200 px-4 py-10">
        <div class="flex flex-col items-center text-center">
          <svg class="mb-6 h-16 w-16 text-base-content/30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 10h-4V6" />
            <path d="M14 10l7.03-7.03a2.12 2.12 0 0 1 3 3L17 13" />
            <path d="M6 14h4v4" />
            <path d="M10 14l-7.03 7.03a2.12 2.12 0 0 1-3-3L7 11" />
          </svg>
          <h1 class="text-xl font-semibold">Something went wrong</h1>
          <p class="mt-2 max-w-sm text-sm text-base-content/60">We couldn't load the console. This might be a temporary issue.</p>
          <div class="mt-6 flex flex-col gap-3">
            <button class="btn btn-primary btn-sm" onclick={goToConsole}>Go to console</button>
            <details class="text-left">
              <summary class="cursor-pointer text-xs text-base-content/50 hover:text-base-content/70">Technical details</summary>
              <pre class="mt-2 max-w-md overflow-auto rounded bg-base-300/50 p-3 text-xs text-base-content/70">{connectionErrorDetails(connectError)}</pre>
            </details>
          </div>
        </div>
      </div>
    {/snippet}

    {#if adminAccountToken}
      <AdminAccountRedirect token={adminAccountToken} />
    {:else}
      <AuthenticatedApp>
        {@render children()}
      </AuthenticatedApp>
    {/if}
  </ConsoleTrellisProvider>
{:else}
  <div class="flex min-h-screen items-center justify-center bg-base-200 px-4 py-10">
    <div class="card trellis-card w-full max-w-sm border border-base-300 bg-base-100 shadow-none">
      <div class="card-body text-center gap-3">
        <h1 class="text-lg font-semibold">Redirecting to sign in</h1>
        <span class="loading loading-spinner loading-md mx-auto"></span>
      </div>
    </div>
  </div>
{/if}
