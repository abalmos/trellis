<script lang="ts">
  import PortalBrand from "$lib/components/PortalBrand.svelte";
</script>

<svelte:head><title>Authentication complete | Trellis</title></svelte:head>

<main class="flex min-h-screen items-center justify-center bg-base-200 p-6">
  <section class="card w-full max-w-lg border border-base-300 bg-base-100 shadow-sm">
    <div class="card-body gap-5">
      <PortalBrand subtitle="Account" />
      <div>
        <p class="text-sm font-semibold uppercase tracking-wide text-success">Account ready</p>
        <h1 class="mt-2 text-3xl font-semibold text-base-content">Authentication complete</h1>
      </div>
      <p class="text-base-content/70">
        Your account change was saved. You can close this tab and return to the application that
        started the request.
      </p>
      <div class="card-actions">
        <a class="btn btn-primary" href="/console">Open Console</a>
      </div>
    </div>
  </section>
</main>
