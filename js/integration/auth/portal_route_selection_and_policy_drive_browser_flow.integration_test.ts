import { assert, assertEquals } from "@std/assert";
import { base64urlEncode, createAuth } from "@qlever-llc/trellis/auth";
import { caseScopedName } from "../_support/names.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID = "auth.portal-route-selection-and-policy-drive-browser-flow";
const fixture = createAuthLocalLoginFixture(CASE_ID);
const portalId = caseScopedName("auth-portal-route-selection", CASE_ID);
const defaultPortalId = caseScopedName(
  "auth-portal-route-selection-default",
  CASE_ID,
);
const customEntryUrl = "https://custom.portal.example/_trellis/login";
const defaultEntryUrl = "https://default.portal.example/_trellis/login";
const filteredProviders = [
  { id: "local", displayName: "Username and password" },
  { id: "github", displayName: "GitHub" },
];
const defaultProviders = [
  ...filteredProviders,
  { id: "google", displayName: "Google" },
];

liveTrellisTest({
  name:
    "auth.portal-route-selection-and-policy-drive-browser-flow selects portal routes and exposes portal policy",
  scope: runtimeScopeForCase(CASE_ID),
  runtime: {
    oauthProviders: {
      github: {
        type: "github",
        clientId: "github-client",
        clientSecret: "github-secret",
        displayName: "GitHub",
      },
      google: {
        type: "oidc",
        issuer: "https://accounts.google.example",
        clientId: "google-client",
        clientSecret: "google-secret",
        displayName: "Google",
      },
    },
  },
  async fn(runtime) {
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const appOrigin = new URL(runtime.trellisUrl).origin;
    try {
      await admin.authPortalsPut({
        portalId: defaultPortalId,
        displayName: "Default Route Selection Portal",
        entryUrl: defaultEntryUrl,
      }).orThrow();
      await admin.authPortalsRoutesPut({
        portalId: defaultPortalId,
        contractId: null,
        origin: null,
      }).orThrow();
      await admin.authPortalsPut({
        portalId,
        displayName: "Custom Route Selection Portal",
        entryUrl: customEntryUrl,
      }).orThrow();
      await admin.authPortalsLoginSettingsUpdate({
        portalId,
        localRegistrationEnabled: false,
        federatedRegistrationEnabled: true,
        allowedFederatedProviders: ["github"],
        selfRegisteredAccountActive: false,
        defaultCapabilities: [],
        defaultCapabilityGroups: [],
      }).orThrow();
      await putCustomRoute(admin, appOrigin, false);

      const custom = await startBrowserFlow(runtime.trellisUrl, {
        redirectTo: `${appOrigin}/return`,
      });
      assertEquals(
        new URL(custom.loginUrl).origin,
        new URL(customEntryUrl).origin,
      );
      assertEquals(
        custom.loginUrl,
        `${customEntryUrl}?flowId=${custom.flowId}`,
      );
      await assertFlowUsesPortal(runtime.trellisUrl, custom.flowId, portalId, {
        providers: filteredProviders,
        localRegistration: false,
        federatedRegistration: true,
      });

      await putCustomRoute(admin, appOrigin, true);
      const disabled = await startBrowserFlow(runtime.trellisUrl, {
        redirectTo: `${appOrigin}/disabled`,
      });
      assertEquals(
        new URL(disabled.loginUrl).origin,
        new URL(defaultEntryUrl).origin,
      );
      await assertFlowUsesPortal(
        runtime.trellisUrl,
        disabled.flowId,
        defaultPortalId,
        { providers: defaultProviders },
      );

      await putCustomRoute(admin, appOrigin, false);
      await admin.authPortalsRoutesRemove({
        portalId,
        contractId: fixture.clientContract.CONTRACT.id,
        origin: appOrigin,
      }).orThrow();
      const removed = await startBrowserFlow(runtime.trellisUrl, {
        redirectTo: `${appOrigin}/removed`,
      });
      assertEquals(
        new URL(removed.loginUrl).origin,
        new URL(defaultEntryUrl).origin,
      );
      await assertFlowUsesPortal(
        runtime.trellisUrl,
        removed.flowId,
        defaultPortalId,
        { providers: defaultProviders },
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
});

async function putCustomRoute(
  admin: Awaited<ReturnType<typeof fixture.setupSessionAdmin>>,
  origin: string,
  disabled: boolean,
): Promise<void> {
  await admin.authPortalsRoutesPut({
    portalId,
    contractId: fixture.clientContract.CONTRACT.id,
    origin,
    disabled,
  }).orThrow();
}

async function startBrowserFlow(
  trellisUrl: string,
  args: { redirectTo: string },
): Promise<{ flowId: string; loginUrl: string }> {
  const seed = crypto.getRandomValues(new Uint8Array(32));
  const auth = await createAuth({ sessionKeySeed: base64urlEncode(seed) });
  const contract = fixture.clientContract.CONTRACT;
  const response = await fetch(`${trellisUrl}/auth/requests`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      redirectTo: args.redirectTo,
      sessionKey: auth.sessionKey,
      sig: await auth.oauthInitSig(
        args.redirectTo,
        undefined,
        undefined,
        contract,
      ),
      contract,
    }),
  });
  const body = await response.text();
  assertEquals(response.status, 200, body);
  const payload: unknown = JSON.parse(body);
  assertRecord(payload);
  assertEquals(payload.status, "flow_started");
  assertString(payload.flowId, "flowId");
  assertString(payload.loginUrl, "loginUrl");
  return { flowId: payload.flowId, loginUrl: payload.loginUrl };
}

async function assertFlowUsesPortal(
  trellisUrl: string,
  flowId: string,
  expectedPortalId: string,
  expected: {
    providers: Array<{ id: string; displayName: string }>;
    localRegistration?: boolean;
    federatedRegistration?: boolean;
  },
): Promise<void> {
  const state = await fetchJson(
    `${trellisUrl}/auth/flow/${encodeURIComponent(flowId)}`,
  );
  assertRecord(state);
  assertEquals(state.status, "choose_provider");
  assertRecord(state.portal);
  assertEquals(state.portal.portalId, expectedPortalId);
  assertEquals(state.providers, expected.providers);
  if (expected.localRegistration !== undefined) {
    assertRecord(state.registration);
    assertRecord(state.registration.localIdentity);
    assertRecord(state.registration.federatedIdentity);
    assertEquals(
      state.registration.localIdentity.available,
      expected.localRegistration,
    );
    assertEquals(
      state.registration.federatedIdentity.available,
      expected.federatedRegistration,
    );
    assertEquals(state.registration.federatedIdentity.providers, [
      { id: "github", displayName: "GitHub" },
    ]);
  }
}

async function fetchJson(url: string): Promise<unknown> {
  const response = await fetch(url);
  const body = await response.text();
  assertEquals(response.status, 200, body);
  return JSON.parse(body);
}

function assertRecord(
  value: unknown,
): asserts value is Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
}

function assertString(
  value: unknown,
  field: string,
): asserts value is string {
  assert(typeof value === "string", `expected ${field} string`);
}
