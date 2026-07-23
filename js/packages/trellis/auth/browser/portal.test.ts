import { assertEquals, assertRejects } from "@std/assert";

import {
  fetchPortalFlowState,
  portalFlowIdFromUrl,
  portalProviderLoginUrl,
  portalRedirectLocation,
  submitPortalApproval,
} from "./portal.ts";

Deno.test("portalFlowIdFromUrl reads flowId from URL", () => {
  assertEquals(
    portalFlowIdFromUrl(
      new URL("https://portal.example.com/login?flowId=flow-1"),
    ),
    "flow-1",
  );
  assertEquals(
    portalFlowIdFromUrl(
      new URL("https://portal.example.com/login?redirectTo=%2F"),
    ),
    null,
  );
});

Deno.test("portalProviderLoginUrl keeps flowId on provider links", () => {
  assertEquals(
    portalProviderLoginUrl(
      { authUrl: "https://auth.example.com/" },
      "google",
      "flow-1",
    ),
    "https://auth.example.com/auth/login/google?flowId=flow-1",
  );
});

Deno.test("portalRedirectLocation returns auth-owned redirect locations", () => {
  assertEquals(
    portalRedirectLocation({
      status: "redirect",
      location: "https://app.example.com/callback?flowId=flow-1",
    }),
    "https://app.example.com/callback?flowId=flow-1",
  );
  assertEquals(
    portalRedirectLocation({
      status: "approval_denied",
      flowId: "flow-1",
      approval: {
        contractId: "trellis.console@v1",
        contractDigest: "digest",
        displayName: "Trellis Console",
        description: "Admin console",
        capabilities: {},
      },
      returnLocation:
        "https://app.example.com/callback?authError=approval_denied",
    }),
    "https://app.example.com/callback?authError=approval_denied",
  );
  assertEquals(portalRedirectLocation({ status: "expired" }), null);
  assertEquals(
    portalRedirectLocation({
      status: "expired",
      returnLocation: "https://app.example.com/callback",
    }),
    "https://app.example.com/callback",
  );
});

Deno.test("fetchPortalFlowState returns auth-owned portal state directly", async () => {
  const originalFetch = globalThis.fetch;
  try {
    globalThis.fetch = (async (input) => {
      assertEquals(String(input), "https://auth.example.com/auth/flow/flow-1");
      return new Response(JSON.stringify({
        flowId: "flow-1",
        state: "choose_provider",
        providers: ["github", "auth0"],
        registrationEnabled: false,
        federatedRegistrationEnabled: false,
        consentViewDigest: "consent-digest",
        consentView: {
          participant: {
            id: "trellis.portal-app@v1",
            digest: "digest",
            displayName: "Portal App",
            description: "User-facing auth portal",
          },
          required: { permissions: [], capabilities: [] },
        },
      }));
    }) as typeof fetch;

    const flow = await fetchPortalFlowState({
      authUrl: "https://auth.example.com",
    }, "flow-1");
    assertEquals(flow.status, "choose_provider");
    if (flow.status === "choose_provider") {
      assertEquals(flow.providers.length, 2);
      assertEquals(flow.app.displayName, "Portal App");
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("fetchPortalFlowState throws on non-success responses", async () => {
  const originalFetch = globalThis.fetch;
  try {
    globalThis.fetch = (async () =>
      new Response("missing", { status: 404 })) as typeof fetch;

    await assertRejects(
      () =>
        fetchPortalFlowState(
          { authUrl: "https://auth.example.com" },
          "missing",
        ),
      Error,
      "Failed to load portal flow (404)",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("submitPortalApproval posts decision and parses next state", async () => {
  const originalFetch = globalThis.fetch;
  try {
    let call = 0;
    globalThis.fetch = (async (input, init) => {
      call += 1;
      if (call !== 2) {
        return new Response(JSON.stringify({
          flowId: "flow-1",
          state: call === 1 ? "approval_required" : "approved",
          providers: ["local"],
          registrationEnabled: false,
          federatedRegistrationEnabled: false,
          consentViewDigest: "consent-digest",
          consentView: {
            participant: {
              id: "trellis.console@v1",
              digest: "digest",
              displayName: "Trellis Console",
              description: "Admin console",
            },
            required: { permissions: [], capabilities: [] },
          },
          user: {
            origin: "trellis",
            id: "usr-1",
            name: "Admin",
          },
          redirectTarget: "https://app.example.com/callback?flowId=flow-1",
        }));
      }
      assertEquals(
        String(input),
        "https://auth.example.com/auth/flow/flow-1/approval",
      );
      assertEquals(init?.method, "POST");
      assertEquals(init?.headers, { "content-type": "application/json" });
      const body = JSON.parse(String(init?.body));
      assertEquals(body.approved, true);
      assertEquals(body.consentViewDigest, "consent-digest");
      assertEquals(body.selectedOptionalBundles, []);
      assertEquals(typeof body.idempotencyKey, "string");

      return new Response(JSON.stringify({ state: "approved" }));
    }) as typeof fetch;

    const state = await submitPortalApproval(
      { authUrl: "https://auth.example.com/" },
      "flow-1",
      "approved",
    );
    assertEquals(state, {
      status: "redirect",
      location: "https://app.example.com/callback?flowId=flow-1",
    });
  } finally {
    globalThis.fetch = originalFetch;
  }
});
