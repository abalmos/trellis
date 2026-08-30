import {
  type CallerRuntime,
  TrellisClient,
  TrellisDevice,
} from "@qlever-llc/trellis";
import {
  base64urlEncode,
  buildEventProofInput,
  MemoryAuthorizationContextStore,
  sha256,
  utf8,
  waitForDeviceActivation,
} from "@qlever-llc/trellis/auth";
import type {
  AuthConnectionsListOutput,
  AuthSessionsListOutput,
} from "@trellis/apis/trellis.auth";
import {
  assertEventCaptured,
  TrellisControlPlaneSqlite,
} from "@qlever-llc/trellis-test";
import { headers as natsHeaders } from "@nats-io/nats-core";
import { connect as connectNats } from "@nats-io/transport-deno";
import {
  assert,
  assertEquals,
  assertNotEquals,
  assertRejects,
} from "@std/assert";
import {
  extname,
  isAbsolute,
  join,
  normalize,
  relative,
  resolve,
} from "node:path";
import type { Browser, Page } from "playwright";
import { chromium } from "playwright";
import { ulid } from "ulid";

import { integrationSlug } from "../integration/_support/names.ts";
import {
  type LiveTrellisRuntime,
  withTrellisRuntime,
} from "../integration/_support/runtime.ts";
import { startTestOidcProvider } from "../packages/trellis-test/src/integration/oidc_provider.ts";
import { createAuthLocalLoginFixture } from "../integration/auth/_fixture.ts";
import { createAuth } from "../packages/trellis/auth/session_auth.ts";
import {
  createDeviceActivationFixture,
  requireDeviceAuthorityList,
} from "../integration/device-activation/_fixture.ts";

const buildDir = resolve("portals/login/build");
const coverageDir = resolve("coverage/browser");
const liveLocalLoginCaseId = "browser.login-portal-live-local-login";
const liveLocalLoginFixture = createAuthLocalLoginFixture(
  liveLocalLoginCaseId,
);
const liveLocalLoginPortalId = `browser-login-portal-${
  integrationSlug(liveLocalLoginCaseId)
}`;
const liveLocalLoginUsername = `browser-login-portal-user-${
  integrationSlug(liveLocalLoginCaseId)
}`;
const liveLocalLoginPassword =
  `trellis-integration-${liveLocalLoginCaseId}-password-2026`;
const liveSessionRefreshCaseId =
  "browser.login-portal-live-existing-session-refreshes-authority";
const liveSessionRefreshFixture = createAuthLocalLoginFixture(
  liveSessionRefreshCaseId,
);
const liveSessionRefreshPortalId = `browser-login-portal-${
  integrationSlug(liveSessionRefreshCaseId)
}`;
const liveSessionRefreshUsername = `browser-login-portal-user-${
  integrationSlug(liveSessionRefreshCaseId)
}`;
const liveSessionRefreshPassword =
  `trellis-integration-${liveSessionRefreshCaseId}-password-2026`;
const liveInvalidLocalLoginCaseId =
  "browser.login-portal-live-invalid-local-credentials";
const liveInvalidLocalLoginFixture = createAuthLocalLoginFixture(
  liveInvalidLocalLoginCaseId,
);
const liveInvalidLocalLoginPortalId = `browser-login-portal-${
  integrationSlug(liveInvalidLocalLoginCaseId)
}`;
const liveInvalidLocalLoginUsername = `browser-login-portal-user-${
  integrationSlug(liveInvalidLocalLoginCaseId)
}`;
const liveInvalidLocalLoginPassword =
  `trellis-integration-${liveInvalidLocalLoginCaseId}-password-2026`;
const liveInactiveLocalLoginCaseId =
  "browser.login-portal-live-inactive-local-user";
const liveInactiveLocalLoginFixture = createAuthLocalLoginFixture(
  liveInactiveLocalLoginCaseId,
);
const liveInactiveLocalLoginPortalId = `browser-login-portal-${
  integrationSlug(liveInactiveLocalLoginCaseId)
}`;
const liveInactiveLocalLoginUsername = `browser-login-portal-user-${
  integrationSlug(liveInactiveLocalLoginCaseId)
}`;
const liveInactiveLocalLoginPassword =
  `trellis-integration-${liveInactiveLocalLoginCaseId}-password-2026`;
const liveDeniedConsentCaseId = "browser.login-portal-live-denied-consent";
const liveDeniedConsentFixture = createAuthLocalLoginFixture(
  liveDeniedConsentCaseId,
);
const liveDeniedConsentPortalId = `browser-login-portal-${
  integrationSlug(liveDeniedConsentCaseId)
}`;
const liveDeniedConsentUsername = `browser-login-portal-user-${
  integrationSlug(liveDeniedConsentCaseId)
}`;
const liveDeniedConsentPassword =
  `trellis-integration-${liveDeniedConsentCaseId}-password-2026`;
const liveInsufficientCapabilitiesCaseId =
  "browser.login-portal-live-insufficient-capabilities";
const liveInsufficientCapabilitiesFixture = createAuthLocalLoginFixture(
  liveInsufficientCapabilitiesCaseId,
  { eventProbe: true, optionalPing: true },
);
const liveInsufficientCapabilitiesPortalId = `browser-login-portal-${
  integrationSlug(liveInsufficientCapabilitiesCaseId)
}`;
const liveInsufficientCapabilitiesUsername = `browser-login-portal-user-${
  integrationSlug(liveInsufficientCapabilitiesCaseId)
}`;
const liveInsufficientCapabilitiesPassword =
  `trellis-integration-${liveInsufficientCapabilitiesCaseId}-password-2026`;
const liveTrustedRegistrationUsername = `browser-login-portal-trusted-user-${
  integrationSlug(liveInsufficientCapabilitiesCaseId)
}`;
const liveTrustedRegistrationPassword =
  `trellis-integration-${liveInsufficientCapabilitiesCaseId}-trusted-password-2026`;
const liveOidcRoleCaseId = "browser.login-portal-live-oidc-role-mapping";
const liveOidcRoleFixture = createAuthLocalLoginFixture(liveOidcRoleCaseId, {
  identityLink: true,
  optionalPing: true,
});
const liveOidcRolePortalId = `browser-login-portal-${
  integrationSlug(liveOidcRoleCaseId)
}`;
const liveAccountLinkUsername = `browser-account-link-user-${
  integrationSlug(liveOidcRoleCaseId)
}`;
const liveAccountLinkPassword =
  `trellis-integration-${liveOidcRoleCaseId}-linked-password-2026`;
const liveAccountLinkDuplicateExistingUsername =
  `browser-account-link-existing-user-${integrationSlug(liveOidcRoleCaseId)}`;
const liveAccountLinkDuplicatePassword =
  `trellis-integration-${liveOidcRoleCaseId}-duplicate-password-2026`;
const liveAccountPasswordCaseId = "browser.login-portal-live-account-password";
const liveAccountPasswordFixture = createAuthLocalLoginFixture(
  liveAccountPasswordCaseId,
);
const liveAccountPasswordPortalId = `browser-login-portal-${
  integrationSlug(liveAccountPasswordCaseId)
}`;
const liveAccountPasswordUsername = `browser-account-password-user-${
  integrationSlug(liveAccountPasswordCaseId)
}`;
const liveAccountPasswordInitialPassword =
  `trellis-integration-${liveAccountPasswordCaseId}-initial-password-2026`;
const liveAccountPasswordNewPassword =
  `trellis-integration-${liveAccountPasswordCaseId}-new-password-2026`;
const liveAccountPasswordTooShortCaseId =
  "browser.login-portal-live-account-password-too-short";
const liveAccountPasswordTooShortFixture = createAuthLocalLoginFixture(
  liveAccountPasswordTooShortCaseId,
);
const liveAccountPasswordTooShortPortalId = `browser-login-portal-${
  integrationSlug(liveAccountPasswordTooShortCaseId)
}`;
const liveAccountPasswordTooShortUsername =
  `browser-account-password-short-user-${
    integrationSlug(liveAccountPasswordTooShortCaseId)
  }`;
const liveAccountPasswordTooShortInitialPassword =
  `trellis-integration-${liveAccountPasswordTooShortCaseId}-initial-password-2026`;
const liveMissingAccountFlowCaseId =
  "browser.login-portal-live-missing-account-flow";
const liveMissingAccountFlowFixture = createAuthLocalLoginFixture(
  liveMissingAccountFlowCaseId,
);
const liveMissingAccountFlowUsername = `browser-missing-account-flow-user-${
  integrationSlug(liveMissingAccountFlowCaseId)
}`;
const liveReusedAccountFlowCaseId =
  "browser.login-portal-live-reused-account-flow";
const liveReusedAccountFlowFixture = createAuthLocalLoginFixture(
  liveReusedAccountFlowCaseId,
);
const liveReusedAccountFlowUsername = `browser-reused-account-flow-user-${
  integrationSlug(liveReusedAccountFlowCaseId)
}`;
const liveReusedAccountFlowInitialPassword =
  `trellis-integration-${liveReusedAccountFlowCaseId}-initial-password-2026`;
const liveReusedAccountFlowNewPassword =
  `trellis-integration-${liveReusedAccountFlowCaseId}-new-password-2026`;
const liveDeviceActivationCaseId =
  "browser.login-portal-live-device-activation";
const liveDeviceActivationFixture = createDeviceActivationFixture(
  liveDeviceActivationCaseId,
);
const liveDeviceActivationLoginFixture = createAuthLocalLoginFixture(
  `${liveDeviceActivationCaseId}.login`,
);
const liveDeviceActivationPortalId = `browser-device-activation-portal-${
  integrationSlug(liveDeviceActivationCaseId)
}`;
const liveDeviceActivationUsername = `browser-device-activation-user-${
  integrationSlug(liveDeviceActivationCaseId)
}`;
const liveDeviceActivationPassword =
  `trellis-integration-${liveDeviceActivationCaseId}-password-2026`;
const liveDeviceActivationPendingCaseId =
  "browser.login-portal-live-device-activation-pending-review";
const liveDeviceActivationPendingFixture = createDeviceActivationFixture(
  liveDeviceActivationPendingCaseId,
);
const liveDeviceActivationPendingLoginFixture = createAuthLocalLoginFixture(
  `${liveDeviceActivationPendingCaseId}.login`,
);
const liveDeviceActivationPendingPortalId =
  `browser-device-activation-pending-portal-${
    integrationSlug(liveDeviceActivationPendingCaseId)
  }`;
const liveDeviceActivationPendingUsername =
  `browser-device-activation-pending-user-${
    integrationSlug(liveDeviceActivationPendingCaseId)
  }`;
const liveDeviceActivationPendingPassword =
  `trellis-integration-${liveDeviceActivationPendingCaseId}-password-2026`;
const liveDeviceActivationRejectedCaseId =
  "browser.login-portal-live-device-activation-rejected";
const liveDeviceActivationRejectedFixture = createDeviceActivationFixture(
  liveDeviceActivationRejectedCaseId,
);
const liveDeviceActivationRejectedLoginFixture = createAuthLocalLoginFixture(
  `${liveDeviceActivationRejectedCaseId}.login`,
);
const liveDeviceActivationRejectedPortalId =
  `browser-device-activation-rejected-portal-${
    integrationSlug(liveDeviceActivationRejectedCaseId)
  }`;
const liveDeviceActivationRejectedUsername =
  `browser-device-activation-rejected-user-${
    integrationSlug(liveDeviceActivationRejectedCaseId)
  }`;
const liveDeviceActivationRejectedPassword =
  `trellis-integration-${liveDeviceActivationRejectedCaseId}-password-2026`;
const deviceActivationPortalContractId = "trellis.portal.activation@v1";
type AuthLocalLoginFixture = ReturnType<typeof createAuthLocalLoginFixture>;
type DeviceActivationFixture = ReturnType<typeof createDeviceActivationFixture>;
type SessionAdminClient = Awaited<
  ReturnType<AuthLocalLoginFixture["setupSessionAdmin"]>
>;
type DeviceActivationAdmin = Awaited<
  ReturnType<DeviceActivationFixture["setupDeviceDeployment"]>
>["admin"];
type DeviceActivationIdentity = Awaited<
  ReturnType<DeviceActivationFixture["setupProvisionedDevice"]>
>["identity"];
type AppSession = AuthSessionsListOutput["entries"][number];
type AppConnection = AuthConnectionsListOutput["entries"][number];

Deno.test("browser.login-portal static routes render browser-only states", async () => {
  let server: ReturnType<typeof Deno.serve> | undefined;

  try {
    server = Deno.serve(
      { hostname: "127.0.0.1", port: 0, onListen() {} },
      (request) => serveStatic(request, buildDir),
    );
    const portalOrigin = `http://127.0.0.1:${server.addr.port}`;

    await withCoveredPage(
      "browser.login-portal static routes render browser-only states",
      async ({ page }) => {
        for (
          const route of [
            {
              pathname: "/_trellis/portal/users/login",
              text: "Session expired",
            },
            {
              pathname: "/_trellis/portal/admin/bootstrap",
              text: "Missing bootstrap flow id.",
            },
            {
              pathname: "/_trellis/portal/admin/invite",
              text: "Missing invitation flow id.",
            },
            {
              pathname: "/_trellis/portal/account/link",
              text: "Missing account-link flow id.",
            },
            {
              pathname: "/_trellis/portal/account/password",
              text: "Missing password flow id.",
            },
            {
              pathname: "/_trellis/portal/devices/activate",
              text: "Missing flow id.",
            },
          ] as const
        ) {
          const response = await page.goto(portalOrigin + route.pathname, {
            waitUntil: "networkidle",
          });

          assertEquals(response?.status(), 200);
          await page.locator("body").waitFor({ state: "visible" });
          await page.locator("script").first().waitFor({ state: "attached" });
          await page.getByText(route.text).waitFor();
        }
      },
    );
  } finally {
    await server?.shutdown();
  }
});

withLivePortalPage(
  "browser.login-portal live local login binds approved client",
  async ({ page, portalOrigin, runtime }) => {
    const service = await liveLocalLoginFixture.setupService(runtime);
    const admin = await liveLocalLoginFixture.setupSessionAdmin(runtime);
    const { clientAuth } = await liveLocalLoginFixture.setupClientRegistration(
      runtime,
    );
    let authRequired = false;
    let client:
      | CallerRuntime<typeof liveLocalLoginFixture.clientContract>
      | undefined;

    try {
      const user = await admin.authUsersCreate({
        name: "Browser Login Portal User",
        email: `${liveLocalLoginUsername}@example.test`,
        image: null,
        idempotencyKey: crypto.randomUUID(),
      }).orThrow();
      const reset = await admin.authUsersPasswordResetCreate({
        idempotencyKey: crypto.randomUUID(),
        returnTarget: null,
        userId: user.user.userId,
      }).orThrow();
      await completeLocalPasswordAccountFlow({
        completionUrl: reset.flow.completionUrl,
        username: liveLocalLoginUsername,
        password: liveLocalLoginPassword,
      });
      await admin.authPortalsPut({
        disabled: false,
        portalId: liveLocalLoginPortalId,
        displayName: "Browser Login Portal",
        entryUrl: `${portalOrigin}/_trellis/portal/users/login`,
        expectedVersion: null,
        idempotencyKey: crypto.randomUUID(),
        loginSettings: {
          federatedRegistration: false,
          localLogin: true,
          localRegistration: false,
          providers: ["local"],
        },
      }).orThrow();
      await admin.authPortalsRoutesPut({
        deploymentId: null,
        expectedVersion: null,
        idempotencyKey: crypto.randomUUID(),
        portalId: liveLocalLoginPortalId,
        participantId: liveLocalLoginFixture.clientContract.CONTRACT_ID,
        origin: portalOrigin,
        priority: 0,
        routeId: null,
      }).orThrow();

      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: liveLocalLoginFixture.clientName,
        contract: liveLocalLoginFixture.clientContract,
        participant: {
          id: liveLocalLoginFixture.clientContract.CONTRACT_ID,
          artifactDigest: liveLocalLoginFixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...clientAuth.auth,
          redirectTo: `${portalOrigin}/_trellis/test/client-auth`,
        },
        onAuthRequired: async (ctx) => {
          authRequired = true;
          const flowId = flowIdFromUrl(ctx.loginUrl);
          const response = await page.goto(
            portalPageUrl(ctx.loginUrl, portalOrigin),
            { waitUntil: "networkidle" },
          );
          assertEquals(response?.status(), 200);

          await page.getByLabel("Username").fill(liveLocalLoginUsername);
          await page.getByLabel("Password").fill(liveLocalLoginPassword);
          await page.getByRole("button", { name: "Sign in" }).last().click();
          const approve = page.getByRole("button", { name: "Approve" });
          await page.waitForFunction(
            ({ callbackPrefix }) =>
              globalThis.location.href.startsWith(callbackPrefix) ||
              [...document.querySelectorAll("button")].some((button) =>
                button.textContent?.trim() === "Approve" &&
                button.getBoundingClientRect().height > 0
              ),
            { callbackPrefix: `${portalOrigin}/_trellis/test/client-auth` },
          );
          if (
            !page.url().startsWith(`${portalOrigin}/_trellis/test/client-auth`)
          ) {
            await Promise.all([
              page.waitForURL(`${portalOrigin}/_trellis/test/client-auth**`),
              approve.click(),
            ]);
          }

          return { status: "bound", flowId };
        },
      }).orThrow();

      assert(authRequired, "expected local-login flow to require auth");
      const me = await client.authSessionsMe({}).orThrow();
      assertEquals(me.session.participantKind, "app");
      assert(me.user !== null, "expected Auth.Sessions.Me to return a user");
      assertEquals(me.user.state, "active");
      assertEquals(me.user.userId.length > 0, true);

      const ping = await client.authLoginPing({
        message: liveLocalLoginFixture.pingMessage,
      }).orThrow();
      assertEquals(ping, {
        message: liveLocalLoginFixture.pingMessage,
        accepted: true,
      });
    } finally {
      await client?.connection.close();
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live existing session reconnects without login",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveSessionRefreshFixture;
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientKey } = await fixture.setupClientRegistration(
      runtime,
    );
    const contextStore = new MemoryAuthorizationContextStore();
    let originalClient:
      | CallerRuntime<typeof fixture.clientContract>
      | undefined;
    let reboundClient:
      | CallerRuntime<typeof fixture.clientContract>
      | undefined;

    try {
      await setupLocalLoginPortalUser({
        admin,
        runtime,
        fixture,
        portalOrigin,
        portalId: liveSessionRefreshPortalId,
        username: liveSessionRefreshUsername,
        password: liveSessionRefreshPassword,
        name: "Browser Login Portal Refresh User",
      });

      originalClient = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          mode: "session_key",
          sessionKeySeed: clientKey.seed,
          authorizationContextStore: contextStore,
          redirectTo: `${portalOrigin}/callback`,
        },
        onAuthRequired: async (ctx) => {
          const flowId = flowIdFromUrl(ctx.loginUrl);
          await completeLocalLoginByFetch({
            trellisUrl: runtime.trellisUrl,
            origin: portalOrigin,
            flowId,
            username: liveSessionRefreshUsername,
            password: liveSessionRefreshPassword,
          });
          return { status: "bound", flowId };
        },
      }).orThrow();
      await originalClient.authLoginPing({
        message: fixture.pingMessage,
      }).orThrow();

      const beforeSession = await appSessionFor(admin, clientKey.sessionKey);
      const beforeConnection = await singleConnectionFor(
        admin,
        clientKey.sessionKey,
      );

      let authRequired = false;
      reboundClient = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          mode: "session_key",
          sessionKeySeed: clientKey.seed,
          sessionId: beforeSession.sessionId,
          redirectTo: `${portalOrigin}/_trellis/test/client-auth`,
          authorizationContextStore: contextStore,
        },
        onAuthRequired: async (ctx) => {
          authRequired = true;
          const flowId = await completeBrowserLocalLogin({
            page,
            loginUrl: ctx.loginUrl,
            portalOrigin,
            username: liveSessionRefreshUsername,
            password: liveSessionRefreshPassword,
          });
          return { status: "bound", flowId };
        },
      }).orThrow();

      assertEquals(authRequired, false);
      const afterSession = await appSessionFor(admin, clientKey.sessionKey);
      assertEquals(afterSession.createdAt, beforeSession.createdAt);
      assertEquals(
        afterSession.principalId,
        beforeSession.principalId,
      );
      assertEquals(
        afterSession.participantArtifactDigest,
        fixture.clientContract.CONTRACT_DIGEST,
      );

      const afterConnection = await runtime.waitFor(async () => {
        const connections = await admin.authConnectionsList({
          sessionId: afterSession.sessionId,
          limit: 100,
        }).orThrow();
        return connections.entries.find((entry) =>
          entry.userNkey !== beforeConnection.userNkey
        );
      });
      assert(afterConnection, "expected rebound app connection");
      assertNotEquals(afterConnection.userNkey, beforeConnection.userNkey);
    } finally {
      await reboundClient?.connection.close().catch(() => undefined);
      await originalClient?.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live invalid local credentials show error",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveInvalidLocalLoginFixture;
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientKey, clientAuth } = await fixture.setupClientRegistration(
      runtime,
    );

    try {
      await setupLocalLoginPortalUser({
        admin,
        runtime,
        fixture,
        portalOrigin,
        portalId: liveInvalidLocalLoginPortalId,
        username: liveInvalidLocalLoginUsername,
        password: liveInvalidLocalLoginPassword,
        name: "Browser Login Portal Invalid User",
      });

      let authRequired = false;
      const connectResult = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...clientAuth.auth,
          redirectTo: `${portalOrigin}/_trellis/test/client-auth`,
        },
        onAuthRequired: async (ctx) => {
          authRequired = true;
          const response = await page.goto(
            portalPageUrl(ctx.loginUrl, portalOrigin),
            { waitUntil: "networkidle" },
          );
          assertEquals(response?.status(), 200);
          await page.getByLabel("Username").fill(liveInvalidLocalLoginUsername);
          await page.getByLabel("Password").fill("not-the-password");
          await page.getByRole("button", { name: "Sign in" }).last().click();
          await page.getByText("Invalid username or password.").waitFor();
          return { status: "handled" };
        },
      });

      assert(authRequired, "expected invalid login to require auth");
      assert(
        connectResult.isErr(),
        "expected invalid browser login to stop connect",
      );
      assertEquals(
        (await appSessionsFor(admin, clientKey.sessionKey)).length,
        0,
      );
      const connections = await admin.authConnectionsList({
        sessionId: clientKey.sessionKey,
        limit: 100,
      }).orThrow();
      assertEquals(connections.entries.length, 0);
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live inactive local user shows error",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveInactiveLocalLoginFixture;
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientKey, clientAuth } = await fixture.setupClientRegistration(
      runtime,
    );

    try {
      const user = await createLocalPasswordUser({
        admin,
        runtime,
        username: liveInactiveLocalLoginUsername,
        password: liveInactiveLocalLoginPassword,
        name: "Browser Inactive Login Portal User",
        capabilities: [fixture.pingCapability],
      });
      await admin.authUsersUpdate({
        email: user.user.email,
        expectedVersion: user.user.version,
        idempotencyKey: crypto.randomUUID(),
        image: user.user.image,
        name: user.user.name,
        state: "disabled",
        userId: user.user.userId,
      }).orThrow();
      await configureLocalLoginPortal({
        admin,
        fixture,
        portalOrigin,
        portalId: liveInactiveLocalLoginPortalId,
      });

      let authRequired = false;
      const connectResult = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...clientAuth.auth,
          redirectTo: `${portalOrigin}/_trellis/test/client-auth`,
        },
        onAuthRequired: async (ctx) => {
          authRequired = true;
          const response = await page.goto(
            portalPageUrl(ctx.loginUrl, portalOrigin),
            { waitUntil: "networkidle" },
          );
          assertEquals(response?.status(), 200);
          await page.getByLabel("Username").fill(
            liveInactiveLocalLoginUsername,
          );
          await page.getByLabel("Password").fill(
            liveInactiveLocalLoginPassword,
          );
          await page.getByRole("button", { name: "Sign in" }).last().click();
          await page.getByText(
            "This account is inactive. Contact an administrator for access.",
          ).waitFor();
          return { status: "handled" };
        },
      });

      assert(authRequired, "expected inactive login to require auth");
      assert(connectResult.isErr(), "expected inactive browser login to fail");
      await assertNoAppSession(admin, clientKey.sessionKey);
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live denied consent does not grant authority",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveDeniedConsentFixture;
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientKey, clientAuth } = await fixture.setupClientRegistration(
      runtime,
    );

    try {
      await setupLocalLoginPortalUser({
        admin,
        runtime,
        fixture,
        portalOrigin,
        portalId: liveDeniedConsentPortalId,
        username: liveDeniedConsentUsername,
        password: liveDeniedConsentPassword,
        name: "Browser Denied Consent User",
      });

      let authRequired = false;
      const connectResult = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...clientAuth.auth,
          redirectTo: `${portalOrigin}/_trellis/test/client-auth`,
        },
        onAuthRequired: async (ctx) => {
          authRequired = true;
          const response = await page.goto(
            portalPageUrl(ctx.loginUrl, portalOrigin),
            { waitUntil: "networkidle" },
          );
          assertEquals(response?.status(), 200);
          await page.getByLabel("Username").fill(liveDeniedConsentUsername);
          await page.getByLabel("Password").fill(liveDeniedConsentPassword);
          await page.getByRole("button", { name: "Sign in" }).last().click();
          await page.getByRole("heading", { name: "Approve access" }).waitFor();
          await page.getByRole("button", { name: "Deny" }).click();
          await page.getByRole("heading", { name: "Access denied" }).waitFor();
          await page.getByText(
            `You denied access for ${fixture.clientDisplayName}.`,
          )
            .waitFor();
          return { status: "handled" };
        },
      });

      assert(authRequired, "expected denied consent to require auth");
      assert(connectResult.isErr(), "expected denied consent to stop connect");
      await assertNoAppSession(admin, clientKey.sessionKey);
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live refresh persists and reconnects after authority change",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveInsufficientCapabilitiesFixture;
    const service = await fixture.setupService(runtime);
    await runtime.services.createInstance({
      name: `auth-old-event-proof-provider-${
        integrationSlug(fixture.clientName)
      }`,
      contract: fixture.eventContract,
    });
    const eventCapture = await runtime.captureEvents({
      name: `auth-old-event-proof-capture-${
        integrationSlug(fixture.clientName)
      }`,
      contract: fixture.eventContract,
      events: [fixture.probeEvent.subscribe],
    });
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientAuth } = await fixture.setupClientRegistration(
      runtime,
    );
    let client: CallerRuntime<typeof fixture.clientContract> | undefined;
    let sameSessionClient:
      | CallerRuntime<typeof fixture.clientContract>
      | undefined;
    let secondClient: CallerRuntime<typeof fixture.clientContract> | undefined;
    let oldNats: Awaited<ReturnType<typeof connectNats>> | undefined;
    let freshNats: Awaited<ReturnType<typeof connectNats>> | undefined;

    try {
      const configuredPortal = await configureLocalLoginPortal({
        admin,
        fixture,
        localRegistration: true,
        portalOrigin,
        portalId: liveInsufficientCapabilitiesPortalId,
        routeOrigin: new URL(runtime.trellisUrl).origin,
      });

      let authRequired = false;
      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...clientAuth.auth,
          redirectTo: `${runtime.trellisUrl}/_trellis/test/client-auth`,
        },
        onAuthRequired: async (ctx) => {
          authRequired = true;
          const flowId = flowIdFromUrl(ctx.loginUrl);
          const response = await page.goto(
            portalPageUrl(ctx.loginUrl, portalOrigin),
            { waitUntil: "networkidle" },
          );
          assertEquals(response?.status(), 200);
          const flow = await fetchJson(
            `${runtime.trellisUrl}/auth/flow/${encodeURIComponent(flowId)}`,
          );
          assert(isRecord(flow.consentView));
          assert(isRecord(flow.consentView.required));
          assert(Array.isArray(flow.consentView.optionalBundles));
          assertEquals(flow.consentView.required.capabilities, [
            fixture.publishProbeCapability,
          ]);
          assertEquals(flow.consentView.optionalBundles.length, 1);
          assert(isRecord(flow.consentView.optionalBundles[0]));
          assertEquals(
            flow.consentView.optionalBundles[0].apiId,
            fixture.serviceContractId,
          );
          const registration = await fetch(
            `${runtime.trellisUrl}/auth/flow/${
              encodeURIComponent(flowId)
            }/register/local`,
            {
              method: "POST",
              headers: {
                "content-type": "application/json",
                origin: portalOrigin,
              },
              body: JSON.stringify({
                username: liveInsufficientCapabilitiesUsername,
                password: liveInsufficientCapabilitiesPassword,
                name: "Browser Missing Capability User",
                email: `${liveInsufficientCapabilitiesUsername}@example.test`,
              }),
            },
          );
          const registrationBody = await registration.text();
          assertEquals(registration.status, 200, registrationBody);
          await page.reload({ waitUntil: "networkidle" });
          await page.getByRole("button", { name: "Approve" }).click();
          await page.waitForURL(
            `${runtime.trellisUrl}/_trellis/test/client-auth**`,
          );
          return { status: "bound", flowId };
        },
      }).orThrow();

      assert(authRequired, "expected missing capability to require auth");
      const denied = await client.authLoginPing({
        message: fixture.pingMessage,
      });
      assert(denied.isErr(), "expected runtime to deny the missing capability");

      await admin.authPortalsGrantOverridesPut({
        portalId: liveInsufficientCapabilitiesPortalId,
        participantId: fixture.clientContract.CONTRACT_ID,
        directCapabilities: [fixture.pingCapability],
        capabilityGroupKeys: [],
        roleMappings: [],
        expectedVersion: null,
        idempotencyKey: crypto.randomUUID(),
      }).orThrow();
      await client.connection.close();
      client = undefined;
      const { clientAuth: trustedClientAuth } = await fixture
        .setupClientRegistration(runtime);
      if (trustedClientAuth.auth.mode !== "session_key") {
        throw new Error("test client auth must use a session key");
      }
      const trustedSessionKeySeed = trustedClientAuth.auth.sessionKeySeed;
      const trustedContextStore = new MemoryAuthorizationContextStore();
      let trustedAuthRequired = false;
      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          mode: "session_key",
          sessionKeySeed: trustedSessionKeySeed,
          authorizationContextStore: trustedContextStore,
          redirectTo: `${runtime.trellisUrl}/_trellis/test/client-auth`,
        },
        onAuthRequired: async (ctx) => {
          trustedAuthRequired = true;
          const flowId = flowIdFromUrl(ctx.loginUrl);
          const response = await page.goto(
            portalPageUrl(ctx.loginUrl, portalOrigin),
            { waitUntil: "networkidle" },
          );
          assertEquals(response?.status(), 200);
          const registration = await fetch(
            `${runtime.trellisUrl}/auth/flow/${
              encodeURIComponent(flowId)
            }/register/local`,
            {
              method: "POST",
              headers: {
                "content-type": "application/json",
                origin: portalOrigin,
              },
              body: JSON.stringify({
                username: liveTrustedRegistrationUsername,
                password: liveTrustedRegistrationPassword,
                name: "Browser Trusted Registration User",
                email: `${liveTrustedRegistrationUsername}@example.test`,
              }),
            },
          );
          const registrationBody = await registration.json();
          assertEquals(
            registration.status,
            200,
            JSON.stringify(registrationBody),
          );
          assertEquals(registrationBody.state, "approved");
          await page.reload({ waitUntil: "networkidle" });
          await page.waitForURL(
            `${runtime.trellisUrl}/_trellis/test/client-auth**`,
          );
          assertEquals(
            await page.getByRole("button", { name: "Approve" }).count(),
            0,
          );
          return { status: "bound", flowId };
        },
      }).orThrow();
      assert(
        trustedAuthRequired,
        "expected trusted login to require browser auth",
      );
      const ping = await client.authLoginPing({
        message: fixture.pingMessage,
      }).orThrow();
      assertEquals(ping, { message: fixture.pingMessage, accepted: true });

      const me = await client.authSessionsMe({}).orThrow();
      const connectRetainedSession = (store: MemoryAuthorizationContextStore) =>
        TrellisClient.connect({
          trellisUrl: runtime.trellisUrl,
          name: fixture.clientName,
          contract: fixture.clientContract,
          participant: {
            id: fixture.clientContract.CONTRACT_ID,
            artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
          },
          auth: {
            mode: "session_key",
            sessionKeySeed: trustedSessionKeySeed,
            authorizationContextStore: store,
            sessionId: me.session.sessionId,
            redirectTo: `${runtime.trellisUrl}/_trellis/test/client-auth`,
          },
          onAuthRequired: () => {
            throw new Error(
              "existing session unexpectedly required browser auth",
            );
          },
        }).orThrow();
      const connectRetainedSessionEventually = async (
        store: MemoryAuthorizationContextStore,
      ) => {
        const deadline = Date.now() + 20_000;
        let lastError: unknown;
        while (Date.now() < deadline) {
          try {
            return await connectRetainedSession(store);
          } catch (cause) {
            lastError = cause;
            await new Promise((resolve) => setTimeout(resolve, 250));
          }
        }
        throw lastError;
      };
      const trustedState = await trustedContextStore.load();
      assert(trustedState);
      assert(trustedState.contextDigest);
      assert(trustedState.routing);
      const staleAuth = await createAuth({
        sessionKeySeed: trustedSessionKeySeed,
        contextDigest: trustedState.contextDigest,
      });
      const staleNatsOptions = await staleAuth.natsConnectOptions({
        sessionId: me.session.sessionId,
        contextDigest: trustedState.contextDigest,
        jwt: trustedState.routing.bootstrapJwt,
      });
      oldNats = await connectNats({
        servers: runtime.natsUrl,
        authenticator: staleNatsOptions.authenticator,
        inboxPrefix: staleNatsOptions.inboxPrefix,
        maxReconnectAttempts: 0,
        timeout: 10_000,
        waitOnFirstConnect: false,
      });
      const oldEventPayload = utf8(
        JSON.stringify({ message: fixture.pingMessage }),
      );
      const oldEventId = ulid();
      const oldEventTime = new Date().toISOString();
      const oldEventProof = base64urlEncode(
        await staleAuth.sign(
          await sha256(
            buildEventProofInput(
              trustedState.contextDigest,
              fixture.probeEvent.publish.subject,
              await sha256(oldEventPayload),
              oldEventId,
              oldEventTime,
            ),
          ),
        ),
      );
      const oldEventHeaders = natsHeaders();
      oldEventHeaders.set("session-key", staleAuth.sessionKey);
      oldEventHeaders.set("Nats-Msg-Id", oldEventId);
      oldEventHeaders.set("Trellis-Event-Time", oldEventTime);
      oldEventHeaders.set("proof", oldEventProof);
      oldEventHeaders.set(
        "authorization-context",
        trustedState.contextDigest,
      );
      oldNats.publish(fixture.probeEvent.publish.subject, oldEventPayload, {
        headers: oldEventHeaders,
      });
      await oldNats.flush();
      await assertEventCaptured(
        eventCapture,
        "AuthLogin.Probe",
        (event) => event.context.id === oldEventId,
      );
      eventCapture.clear();
      const sameSessionContextStore = new MemoryAuthorizationContextStore();
      await sameSessionContextStore.commit(trustedState);
      sameSessionClient = await connectRetainedSession(sameSessionContextStore);
      assertEquals(
        (await sameSessionClient.authSessionsMe({}).orThrow()).session
          .sessionId,
        me.session.sessionId,
      );
      await sameSessionClient.authLoginPing({ message: fixture.pingMessage })
        .orThrow();

      const { clientAuth: secondClientAuth } = await fixture
        .setupClientRegistration(runtime);
      secondClient = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...secondClientAuth.auth,
          redirectTo: `${runtime.trellisUrl}/_trellis/test/client-auth`,
        },
        onAuthRequired: async (ctx) => {
          const flowId = flowIdFromUrl(ctx.loginUrl);
          const response = await page.goto(
            portalPageUrl(ctx.loginUrl, portalOrigin),
            { waitUntil: "networkidle" },
          );
          assertEquals(response?.status(), 200);
          await page.getByLabel("Username").fill(
            liveTrustedRegistrationUsername,
          );
          await page.getByLabel("Password").fill(
            liveTrustedRegistrationPassword,
          );
          await Promise.all([
            page.waitForURL(
              `${runtime.trellisUrl}/_trellis/test/client-auth**`,
            ),
            page.getByRole("button", { name: "Sign in" }).last().click(),
          ]);
          assertEquals(
            await page.getByRole("button", { name: "Approve" }).count(),
            0,
          );
          return { status: "bound", flowId };
        },
      }).orThrow();
      await secondClient.authLoginPing({ message: fixture.pingMessage })
        .orThrow();

      const [policy] = (await admin.authPortalsGrantOverridesList({
        portalId: liveInsufficientCapabilitiesPortalId,
        participantId: fixture.clientContract.CONTRACT_ID,
        offset: 0,
        limit: 1,
      }).orThrow()).entries;
      assert(policy, "expected trusted portal policy");
      const reduced = await admin.authPortalsGrantOverridesPut({
        portalId: policy.portalId,
        participantId: policy.participantId,
        directCapabilities: [],
        capabilityGroupKeys: [],
        roleMappings: [],
        expectedVersion: policy.version,
        idempotencyKey: crypto.randomUUID(),
      }).orThrow();
      await Promise.all([
        waitForPingAuthority(client, fixture.pingMessage, false, "first"),
        waitForPingAuthority(
          sameSessionClient,
          fixture.pingMessage,
          false,
          "same-session second connection",
        ),
        waitForPingAuthority(
          secondClient,
          fixture.pingMessage,
          false,
          "second",
        ),
      ]);
      await assertRejects(
        () =>
          connectNats({
            servers: runtime.natsUrl,
            authenticator: staleNatsOptions.authenticator,
            inboxPrefix: staleNatsOptions.inboxPrefix,
            maxReconnectAttempts: 0,
            timeout: 10_000,
            waitOnFirstConnect: false,
          }),
        Error,
        undefined,
        "old authorization context unexpectedly passed NATS admission",
      );
      await client.connection.close();
      client = undefined;
      await sameSessionClient.connection.close();
      sameSessionClient = undefined;
      await secondClient.connection.close();
      secondClient = undefined;

      await admin.authPortalsGrantOverridesPut({
        portalId: policy.portalId,
        participantId: policy.participantId,
        directCapabilities: [fixture.pingCapability],
        capabilityGroupKeys: [],
        roleMappings: [],
        expectedVersion: reduced.policy.version,
        idempotencyKey: crypto.randomUUID(),
      }).orThrow();
      client = await connectRetainedSessionEventually(trustedContextStore);
      await waitForPingAuthority(client, fixture.pingMessage, true, "fresh");
      const refreshedState = await trustedContextStore.load();
      assert(refreshedState);
      assert(refreshedState.contextDigest);
      assert(refreshedState.routing);
      const freshAuth = await createAuth({
        sessionKeySeed: trustedSessionKeySeed,
        contextDigest: refreshedState.contextDigest,
      });
      const freshNatsOptions = await freshAuth.natsConnectOptions({
        sessionId: me.session.sessionId,
        contextDigest: refreshedState.contextDigest,
        jwt: refreshedState.routing.bootstrapJwt,
      });
      freshNats = await connectNats({
        servers: runtime.natsUrl,
        authenticator: freshNatsOptions.authenticator,
        inboxPrefix: freshNatsOptions.inboxPrefix,
        maxReconnectAttempts: 0,
        timeout: 10_000,
        waitOnFirstConnect: false,
      });
      freshNats.publish(fixture.probeEvent.publish.subject, oldEventPayload, {
        headers: oldEventHeaders,
      });
      await freshNats.flush();
      await new Promise((resolve) => setTimeout(resolve, 1_000));
      assertEquals(eventCapture.all("AuthLogin.Probe").length, 0);
      const refreshedSecondStore = new MemoryAuthorizationContextStore();
      await refreshedSecondStore.commit(refreshedState);
      sameSessionClient = await connectRetainedSession(refreshedSecondStore);
      await sameSessionClient.authLoginPing({ message: fixture.pingMessage })
        .orThrow();

      const portal = configuredPortal.portal;
      const disabledPortal = await admin.authPortalsPut({
        disabled: true,
        portalId: portal.portalId,
        displayName: portal.displayName,
        entryUrl: portal.entryUrl,
        expectedVersion: portal.version,
        idempotencyKey: crypto.randomUUID(),
        loginSettings: portal.loginSettings,
      }).orThrow();
      const sqlite = new TrellisControlPlaneSqlite(
        join(runtime.workdir, "trellis", "trellis.sqlite.platform"),
      );
      const revocationDeadline = Date.now() + 20_000;
      let authorityState: unknown;
      while (Date.now() < revocationDeadline) {
        const [authority] = await sqlite.query(
          "SELECT state FROM auth_identity_authorities WHERE principal_id = ? AND participant_id = ?",
          [me.session.principalId, fixture.clientContract.CONTRACT_ID],
        );
        authorityState = authority?.state;
        if (authorityState === "revoked") break;
        await new Promise((resolve) => setTimeout(resolve, 250));
      }
      assertEquals(authorityState, "revoked");
      await Promise.all([
        waitForPingAuthority(client, fixture.pingMessage, false, "first"),
        waitForPingAuthority(
          sameSessionClient,
          fixture.pingMessage,
          false,
          "same-session second connection",
        ),
      ]);
      await client.connection.close();
      client = undefined;
      await sameSessionClient.connection.close();
      sameSessionClient = undefined;
      await admin.authPortalsPut({
        disabled: false,
        portalId: portal.portalId,
        displayName: portal.displayName,
        entryUrl: portal.entryUrl,
        expectedVersion: disabledPortal.portal.version,
        idempotencyKey: crypto.randomUUID(),
        loginSettings: portal.loginSettings,
      }).orThrow();
      await assertRejects(
        () => connectRetainedSessionEventually(trustedContextStore),
        Error,
        undefined,
        "restored policy unexpectedly admitted a session without fresh login",
      );
      const { clientAuth: restorationClientAuth } = await fixture
        .setupClientRegistration(runtime);
      let restorationLoginRequired = false;
      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...restorationClientAuth.auth,
          redirectTo: `${runtime.trellisUrl}/_trellis/test/client-auth`,
        },
        onAuthRequired: async (ctx) => {
          restorationLoginRequired = true;
          const flowId = flowIdFromUrl(ctx.loginUrl);
          const response = await page.goto(
            portalPageUrl(ctx.loginUrl, portalOrigin),
            { waitUntil: "networkidle" },
          );
          assertEquals(response?.status(), 200);
          await page.getByLabel("Username").fill(
            liveTrustedRegistrationUsername,
          );
          await page.getByLabel("Password").fill(
            liveTrustedRegistrationPassword,
          );
          await Promise.all([
            page.waitForURL(
              `${runtime.trellisUrl}/_trellis/test/client-auth**`,
            ),
            page.getByRole("button", { name: "Sign in" }).last().click(),
          ]);
          return { status: "bound", flowId };
        },
      }).orThrow();
      assert(restorationLoginRequired, "expected restoration to require login");
      assertEquals(
        (await client.authSessionsMe({}).orThrow()).session.principalId,
        me.session.principalId,
      );
      await waitForPingAuthority(
        client,
        fixture.pingMessage,
        true,
        "restored portal",
      );
    } finally {
      await freshNats?.close().catch(() => undefined);
      await oldNats?.close().catch(() => undefined);
      await eventCapture.stop();
      await secondClient?.connection.close().catch(() => undefined);
      await sameSessionClient?.connection.close().catch(() => undefined);
      await client?.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

async function waitForPingAuthority(
  client: {
    authLoginPing(input: { message: string }): PromiseLike<{ isOk(): boolean }>;
    connection: { status: { phase: string } };
  },
  message: string,
  allowed: boolean,
  label: string,
): Promise<void> {
  const deadline = Date.now() + 30_000;
  let observed = "no response";
  while (Date.now() < deadline) {
    const result = await Promise.race([
      Promise.resolve(client.authLoginPing({ message })).catch(() => null),
      new Promise<null>((resolve) => setTimeout(() => resolve(null), 5_000)),
    ]);
    observed = result === null
      ? "no response"
      : result.isOk()
      ? "allowed"
      : "denied";
    if (result?.isOk() === allowed) return;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(
    `${label} client timed out waiting for ping authority=${allowed}; last observed ${observed}; connection=${client.connection.status.phase}`,
  );
}

Deno.test("browser.login-portal live OIDC role mapping", async () => {
  const oidc = await startTestOidcProvider({ roles: ["direct"] });
  let trellisUrl = "";
  const portalServer = Deno.serve(
    { hostname: "127.0.0.1", port: 0, onListen() {} },
    (request) => serveStatic(request, buildDir, trellisUrl),
  );
  const portalOrigin = `http://127.0.0.1:${portalServer.addr.port}`;
  try {
    await withTrellisRuntime(async (runtime) => {
      trellisUrl = runtime.trellisUrl;
      await withCoveredPage(liveOidcRoleCaseId, async ({ page }) => {
        const fixture = liveOidcRoleFixture;
        const service = await fixture.setupService(runtime);
        const admin = await fixture.setupSessionAdmin(runtime);
        const clients: CallerRuntime<typeof fixture.clientContract>[] = [];
        try {
          await admin.authPortalsPut({
            disabled: false,
            portalId: liveOidcRolePortalId,
            displayName: "OIDC Role Portal",
            entryUrl: `${portalOrigin}/_trellis/portal/users/login`,
            expectedVersion: null,
            idempotencyKey: crypto.randomUUID(),
            loginSettings: {
              federatedRegistration: true,
              localLogin: false,
              localRegistration: false,
              providers: ["test-oidc", "other-oidc"],
            },
          }).orThrow();
          await admin.authPortalsRoutesPut({
            deploymentId: null,
            expectedVersion: null,
            idempotencyKey: crypto.randomUUID(),
            portalId: liveOidcRolePortalId,
            participantId: fixture.clientContract.CONTRACT_ID,
            origin: new URL(runtime.trellisUrl).origin,
            priority: 0,
            routeId: null,
          }).orThrow();
          const putPolicy = async (
            role: string | string[],
            group = false,
            providerId = "test-oidc",
          ) => {
            const current = (await admin.authPortalsGrantOverridesList({
              portalId: liveOidcRolePortalId,
              participantId: fixture.clientContract.CONTRACT_ID,
              offset: 0,
              limit: 1,
            }).orThrow()).entries[0];
            await admin.authPortalsGrantOverridesPut({
              portalId: liveOidcRolePortalId,
              participantId: fixture.clientContract.CONTRACT_ID,
              directCapabilities: [],
              capabilityGroupKeys: [],
              roleMappings: (Array.isArray(role) ? role : [role]).map(
                (mappedRole) => ({
                  providerId,
                  role: mappedRole,
                  directCapabilities: group && mappedRole === "group"
                    ? []
                    : [fixture.pingCapability],
                  capabilityGroupKeys: group && mappedRole === "group"
                    ? ["oidc-parent"]
                    : [],
                }),
              ),
              expectedVersion: current?.version ?? null,
              idempotencyKey: crypto.randomUUID(),
            }).orThrow();
          };
          const connect = async (loginPage = page) => {
            const { clientAuth } = await fixture.setupClientRegistration(
              runtime,
            );
            const connected = await TrellisClient.connect({
              trellisUrl: runtime.trellisUrl,
              name: fixture.clientName,
              contract: fixture.clientContract,
              participant: {
                id: fixture.clientContract.CONTRACT_ID,
                artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
              },
              auth: {
                ...clientAuth.auth,
                redirectTo: `${runtime.trellisUrl}/_trellis/test/client-auth`,
              },
              onAuthRequired: async (ctx) => {
                const flowId = flowIdFromUrl(ctx.loginUrl);
                await loginPage.goto(
                  portalPageUrl(ctx.loginUrl, portalOrigin),
                  {
                    waitUntil: "networkidle",
                  },
                );
                const providerHref = await loginPage.getByRole("link").filter({
                  hasText: "test-oidc",
                }).getAttribute("href");
                assert(providerHref);
                const providerUrl = new URL(providerHref, portalOrigin);
                await loginPage.goto(
                  `${runtime.trellisUrl}${providerUrl.pathname}${providerUrl.search}`,
                  { waitUntil: "networkidle" },
                );
                assertEquals(
                  await loginPage.getByRole("button", { name: "Approve" })
                    .count(),
                  0,
                );
                return { status: "bound", flowId };
              },
            });
            const client = connected.orThrow();
            clients.push(client);
            return client;
          };

          await putPolicy("direct");
          const oidcUser = await connect();
          await oidcUser.authLoginPing({
            message: fixture.pingMessage,
          }).orThrow();
          const concurrentBrowser = await chromium.launch({ headless: true });
          try {
            const concurrentClients = await Promise.all([
              connect(),
              connect(await concurrentBrowser.newPage()),
            ]);
            await Promise.all(
              concurrentClients.map((client) =>
                client.authLoginPing({ message: fixture.pingMessage }).orThrow()
              ),
            );
          } finally {
            await concurrentBrowser.close();
          }
          await admin.authCapabilityGroupsPut({
            groupKey: "oidc-leaf",
            displayName: "OIDC leaf",
            description: "Grants the live role-mapping capability.",
            capabilities: [fixture.pingCapability],
            includedGroups: [],
            expectedVersion: null,
            idempotencyKey: crypto.randomUUID(),
          }).orThrow();
          await admin.authCapabilityGroupsPut({
            groupKey: "oidc-parent",
            displayName: "OIDC parent",
            description: "Recursively includes the live role-mapping leaf.",
            capabilities: [],
            includedGroups: ["oidc-leaf"],
            expectedVersion: null,
            idempotencyKey: crypto.randomUUID(),
          }).orThrow();
          await putPolicy(["group", "same-authority"], true);
          oidc.setClaims({ roles: ["group"] });
          await (await connect()).authLoginPing({
            message: fixture.pingMessage,
          })
            .orThrow();
          const sqlite = new TrellisControlPlaneSqlite(
            join(runtime.workdir, "trellis", "trellis.sqlite.platform"),
          );
          const [groupBinding] = await sqlite.query(
            "SELECT roles_json, provider_id, authority_version FROM auth_portal_authority_bindings WHERE participant_id = ? ORDER BY updated_at DESC LIMIT 1",
            [fixture.clientContract.CONTRACT_ID],
          );
          assertEquals(groupBinding?.provider_id, "test-oidc");
          assertEquals(groupBinding?.roles_json, '["group"]');

          oidc.setClaims({ roles: ["same-authority"] });
          const linkedUser = await connect();
          await linkedUser.authLoginPing({
            message: fixture.pingMessage,
          })
            .orThrow();
          const [sameAuthorityBinding] = await sqlite.query(
            "SELECT roles_json, provider_id, authority_version FROM auth_portal_authority_bindings WHERE participant_id = ? ORDER BY updated_at DESC LIMIT 1",
            [fixture.clientContract.CONTRACT_ID],
          );
          assertEquals(sameAuthorityBinding?.provider_id, "test-oidc");
          assertEquals(sameAuthorityBinding?.roles_json, '["same-authority"]');
          assertEquals(
            sameAuthorityBinding?.authority_version,
            groupBinding?.authority_version,
          );

          await createLocalPasswordUser({
            admin,
            runtime,
            username: liveAccountLinkDuplicateExistingUsername,
            password: liveAccountLinkDuplicatePassword,
            name: "Browser Account Link Existing Local User",
          });
          const link = await linkedUser.authUsersIdentityLinkCreate({
            allowedProviders: ["local"],
            idempotencyKey: crypto.randomUUID(),
            returnTarget: null,
          }).orThrow();
          const flowId = accountFlowToken(link.flow.completionUrl);
          const linkResponse = await page.goto(
            accountFlowPortalUrl(
              portalOrigin,
              "/_trellis/portal/account/link",
              flowId,
            ),
            { waitUntil: "networkidle" },
          );
          assertEquals(linkResponse?.status(), 200);
          await waitForHeading(page, "Link local credentials");
          await page.getByLabel("Username").fill(
            liveAccountLinkDuplicateExistingUsername,
          );
          await page.getByLabel("Password").fill(liveAccountLinkPassword);
          await page.getByRole("button", { name: "Link credentials" }).click();
          try {
            await page.getByText(
              "That username is already in use. Choose a different username.",
            ).waitFor();
          } catch (error) {
            throw new Error(
              `Unexpected account-link conflict:\n${await page.locator("body")
                .innerText()}`,
              { cause: error },
            );
          }
          assertEquals(
            (await fetchJson(
              `${runtime.trellisUrl}/auth/account-flow/${
                encodeURIComponent(flowId)
              }`,
            )).status,
            "pending",
          );

          await page.getByLabel("Username").fill(liveAccountLinkUsername);
          await page.getByRole("button", { name: "Link credentials" }).click();
          await waitForHeading(page, "Account linked");
          const identities = await linkedUser.authUserIdentitiesList({
            limit: 100,
            providerId: "local",
          }).orThrow();
          assert(
            identities.entries.some((identity) =>
              identity.providerId === "local" &&
              identity.principalId === link.flow.targetPrincipalId &&
              identity.subject === liveAccountLinkUsername
            ),
            "expected local identity to be linked to the federated user",
          );

          await putPolicy("provider-scoped", false, "other-oidc");
          oidc.setClaims({ roles: ["provider-scoped"] });
          const providerScoped = await connect();
          await assertRejects(() =>
            providerScoped.authLoginPing({ message: fixture.pingMessage })
              .orThrow()
          );

          oidc.setClaims({ roles: { invalid: true } });
          await assertRejects(() => connect());
        } finally {
          await Promise.all(clients.map((client) => client.connection.close()));
          await admin.connection.close().catch(() => undefined);
          await service.stop();
        }
      });
    }, {
      webOrigins: [portalOrigin],
      oauthProviders: {
        "test-oidc": {
          type: "oidc",
          issuer: oidc.issuer,
          clientId: "trellis-test-client",
          displayName: "Test OIDC",
          roleClaims: ["/roles"],
        },
        "other-oidc": {
          type: "oidc",
          issuer: oidc.issuer,
          clientId: "trellis-test-client",
          displayName: "Other OIDC",
          roleClaims: ["/roles"],
        },
      },
    });
  } finally {
    await portalServer.shutdown();
    await oidc.shutdown();
  }
});

withLivePortalPage(
  "browser.login-portal live account password completes",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveAccountPasswordFixture;
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientAuth } = await fixture.setupClientRegistration(runtime);
    let client:
      | CallerRuntime<typeof fixture.clientContract>
      | undefined;

    try {
      const user = await createLocalPasswordUser({
        admin,
        runtime,
        username: liveAccountPasswordUsername,
        password: liveAccountPasswordInitialPassword,
        name: "Browser Account Password User",
        capabilities: [fixture.pingCapability],
      });
      await configureLocalLoginPortal({
        admin,
        fixture,
        portalOrigin,
        portalId: liveAccountPasswordPortalId,
      });
      const reset = await admin.authUsersPasswordResetCreate({
        idempotencyKey: crypto.randomUUID(),
        returnTarget: null,
        userId: user.user.userId,
      }).orThrow();
      const resetToken = accountFlowToken(reset.flow.completionUrl);

      const resetUrl = accountFlowPortalUrl(
        portalOrigin,
        "/_trellis/portal/account/password",
        resetToken,
      );
      const response = await page.goto(
        resetUrl,
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await waitForHeading(page, "Reset your password");
      await page.getByLabel("Password").fill(liveAccountPasswordNewPassword);
      await page.getByRole("button", { name: "Reset password" }).click();
      await page.getByRole("heading", { name: "Password saved" }).waitFor();

      assertEquals(
        (await fetchJson(
          `${runtime.trellisUrl}/auth/account-flow/${
            encodeURIComponent(resetToken)
          }`,
        )).status,
        "consumed",
      );
      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...clientAuth.auth,
          redirectTo: `${portalOrigin}/callback`,
        },
        onAuthRequired: async (ctx) => {
          const flowId = flowIdFromUrl(ctx.loginUrl);
          await completeLocalLoginByFetch({
            trellisUrl: runtime.trellisUrl,
            origin: portalOrigin,
            flowId,
            username: liveAccountPasswordUsername,
            password: liveAccountPasswordNewPassword,
          });
          return { status: "bound", flowId };
        },
      }).orThrow();
      const me = await client.authSessionsMe({}).orThrow();
      assertEquals(me.user?.userId, user.user.userId);
    } finally {
      await client?.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live account password too short keeps flow active",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveAccountPasswordTooShortFixture;
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientAuth } = await fixture.setupClientRegistration(runtime);
    let client:
      | CallerRuntime<typeof fixture.clientContract>
      | undefined;

    try {
      const user = await createLocalPasswordUser({
        admin,
        runtime,
        username: liveAccountPasswordTooShortUsername,
        password: liveAccountPasswordTooShortInitialPassword,
        name: "Browser Account Password Too Short User",
        capabilities: [fixture.pingCapability],
      });
      await configureLocalLoginPortal({
        admin,
        fixture,
        portalOrigin,
        portalId: liveAccountPasswordTooShortPortalId,
      });
      const reset = await admin.authUsersPasswordResetCreate({
        idempotencyKey: crypto.randomUUID(),
        returnTarget: null,
        userId: user.user.userId,
      }).orThrow();
      const resetToken = accountFlowToken(reset.flow.completionUrl);
      const stateUrl = `${runtime.trellisUrl}/auth/account-flow/${
        encodeURIComponent(resetToken)
      }`;
      const state = await fetchJson(stateUrl);
      const passwordPolicy = state.passwordPolicy;
      assert(
        isRecord(passwordPolicy),
        "expected password policy in flow state",
      );
      const minLength = passwordPolicy.minLength;
      assertEquals(typeof minLength, "number");

      const response = await page.goto(
        accountFlowPortalUrl(
          portalOrigin,
          "/_trellis/portal/account/password",
          resetToken,
        ),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Reset your password" })
        .waitFor();
      await page.getByLabel("Password").fill("short");
      await page.getByText(
        `Password must be at least ${minLength} characters.`,
      ).waitFor();

      assertEquals((await fetchJson(stateUrl)).status, "pending");
      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        participant: {
          id: fixture.clientContract.CONTRACT_ID,
          artifactDigest: fixture.clientContract.CONTRACT_DIGEST,
        },
        auth: {
          ...clientAuth.auth,
          redirectTo: `${portalOrigin}/callback`,
        },
        onAuthRequired: async (ctx) => {
          const flowId = flowIdFromUrl(ctx.loginUrl);
          await completeLocalLoginByFetch({
            trellisUrl: runtime.trellisUrl,
            origin: portalOrigin,
            flowId,
            username: liveAccountPasswordTooShortUsername,
            password: liveAccountPasswordTooShortInitialPassword,
          });
          return { status: "bound", flowId };
        },
      }).orThrow();
      const me = await client.authSessionsMe({}).orThrow();
      assertEquals(me.user?.userId, user.user.userId);
    } finally {
      await client?.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live missing account-flow token shows error without runtime changes",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveMissingAccountFlowFixture;
    const admin = await fixture.setupSessionAdmin(runtime);

    try {
      const user = await admin.authUsersCreate({
        name: "Browser Missing Account Flow User",
        email: `${liveMissingAccountFlowUsername}@example.test`,
        image: null,
        idempotencyKey: crypto.randomUUID(),
      }).orThrow();
      const before = await admin.authUserIdentitiesList({ limit: 100 })
        .orThrow();
      const beforeUserIdentities = before.entries.filter((identity) =>
        identity.principalId === user.user.principalId
      );

      const response = await page.goto(
        accountFlowPortalUrl(
          portalOrigin,
          "/_trellis/portal/account/password",
          `missing-flow-${integrationSlug(liveMissingAccountFlowCaseId)}`,
        ),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Password link expired" })
        .waitFor();
      await page.getByText(
        "This password request is missing or no longer active.",
      )
        .waitFor();

      const after = await admin.authUserIdentitiesList({ limit: 100 })
        .orThrow();
      assertEquals(
        after.entries.filter((identity) =>
          identity.principalId === user.user.principalId
        ),
        beforeUserIdentities,
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
    }
  },
);

withLivePortalPage(
  "browser.login-portal live reused terminal flow token preserves terminal state",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveReusedAccountFlowFixture;
    const admin = await fixture.setupSessionAdmin(runtime);

    try {
      const user = await createLocalPasswordUser({
        admin,
        runtime,
        username: liveReusedAccountFlowUsername,
        password: liveReusedAccountFlowInitialPassword,
        name: "Browser Reused Account Flow User",
      });
      const reset = await admin.authUsersPasswordResetCreate({
        idempotencyKey: crypto.randomUUID(),
        returnTarget: null,
        userId: user.user.userId,
      }).orThrow();
      const resetToken = accountFlowToken(reset.flow.completionUrl);
      const url = accountFlowPortalUrl(
        portalOrigin,
        "/_trellis/portal/account/password",
        resetToken,
      );

      const response = await page.goto(url, { waitUntil: "networkidle" });
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Reset your password" })
        .waitFor();
      await page.getByLabel("Password").fill(liveReusedAccountFlowNewPassword);
      await page.getByRole("button", { name: "Reset password" }).click();
      await page.getByRole("heading", { name: "Password saved" }).waitFor();

      const terminalState = await fetchJson(
        `${runtime.trellisUrl}/auth/account-flow/${
          encodeURIComponent(resetToken)
        }`,
      );
      assertEquals(terminalState.status, "consumed");
      const before = await admin.authUserIdentitiesList({ limit: 100 })
        .orThrow();

      const reusedResponse = await page.goto(url, { waitUntil: "networkidle" });
      assertEquals(reusedResponse?.status(), 200);
      await page.getByRole("heading", { name: "Password link already used" })
        .waitFor();
      await page.getByText(
        "This password reset request has already been completed.",
      )
        .waitFor();

      const after = await admin.authUserIdentitiesList({ limit: 100 })
        .orThrow();
      assertEquals(after.entries, before.entries);
      assertEquals(
        (await fetchJson(
          `${runtime.trellisUrl}/auth/account-flow/${
            encodeURIComponent(resetToken)
          }`,
        )).status,
        terminalState.status,
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
    }
  },
);

withLivePortalPage(
  "browser.login-portal live device activation completes",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveDeviceActivationFixture;
    const { admin, deploymentId } = await fixture.setupDeviceDeployment(
      runtime,
    );
    const loginAdmin = await liveDeviceActivationLoginFixture.setupSessionAdmin(
      runtime,
    );

    try {
      const { identity, rootSecret, provisioned } = await fixture
        .setupProvisionedDevice(admin, deploymentId);
      const { confirmationCode, flowId, participantNeedsDigest } = await fixture
        .setupActivationRequest(
          runtime,
          admin,
          deploymentId,
          identity,
          provisioned.device.instanceId,
        );
      await setupDeviceActivationPortalUser({
        admin: loginAdmin,
        runtime,
        portalOrigin,
        portalId: liveDeviceActivationPortalId,
        username: liveDeviceActivationUsername,
        password: liveDeviceActivationPassword,
        name: "Browser Device Activation User",
      });

      const response = await page.goto(
        deviceActivationPortalUrl(portalOrigin, flowId),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await waitForHeading(page, "Sign in to continue");
      await completeDeviceActivationPortalSignIn({
        page,
        username: liveDeviceActivationUsername,
        password: liveDeviceActivationPassword,
      });
      await waitForHeading(page, "Approve this device");
      await page.getByLabel("Confirmation code").fill(confirmationCode);
      await page.getByRole("button", { name: "Approve device" }).click();
      await page.getByRole("heading", { name: "Device approved" }).waitFor();
      await page.getByText("Approval complete.").waitFor();

      await waitForDeviceActivation({
        trellisUrl: runtime.trellisUrl,
        publicIdentityKey: identity.publicIdentityKey,
        identitySeed: identity.identitySeed,
        activationKey: identity.activationKey,
        deploymentId,
        instanceId: provisioned.device.instanceId,
        principalId: provisioned.device.instanceId,
        participantId: fixture.deviceContract.CONTRACT_ID,
        participantArtifactDigest: fixture.deviceContract.CONTRACT_DIGEST,
        participantNeedsDigest,
        pollIntervalMs: 25,
      });
      await runtime.waitFor(async () => {
        const activations = requireDeviceAuthorityList(
          await admin.authDeviceUserAuthoritiesList({
            deploymentId,
            limit: 20,
          }).orThrow(),
        );
        return activations.entries.find((entry) =>
          entry.instanceId === provisioned.device.instanceId &&
          entry.identityPublicKey === identity.publicIdentityKey &&
          entry.deploymentId === deploymentId &&
          entry.state === "active"
        );
      });

      const device = await TrellisDevice.connect({
        trellisUrl: runtime.trellisUrl,
        contract: fixture.deviceContract,
        rootSecret,
        identity: {
          deploymentId,
          instanceId: provisioned.device.instanceId,
          principalId: provisioned.device.instanceId,
          participantId: fixture.deviceContract.CONTRACT_ID,
          participantArtifactDigest: fixture.deviceContract.CONTRACT_DIGEST,
          participantNeedsDigest,
        },
        log: false,
        authorizationContextEphemeral: true,
      }).orThrow();
      try {
        const me = await device.authSessionsMe({}).orThrow();
        assertEquals(me.session.participantKind, "device");
        assertEquals(me.deploymentId, deploymentId);
        assertEquals(me.instanceId, provisioned.device.instanceId);
        assertEquals(me.session.principalId, provisioned.device.principalId);
      } finally {
        await device.connection.close().catch(() => undefined);
      }
    } finally {
      await loginAdmin.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
    }
  },
);

withLivePortalPage(
  "browser.login-portal live device activation stays pending before approval",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveDeviceActivationPendingFixture;
    const { admin, deploymentId } = await fixture.setupDeviceDeployment(
      runtime,
    );
    const loginAdmin = await liveDeviceActivationPendingLoginFixture
      .setupSessionAdmin(runtime);

    try {
      const { identity, rootSecret, provisioned } = await fixture
        .setupProvisionedDevice(admin, deploymentId);
      const { confirmationCode, flowId, participantNeedsDigest } = await fixture
        .setupActivationRequest(
          runtime,
          admin,
          deploymentId,
          identity,
          provisioned.device.instanceId,
        );
      await setupDeviceActivationPortalUser({
        admin: loginAdmin,
        runtime,
        portalOrigin,
        portalId: liveDeviceActivationPendingPortalId,
        username: liveDeviceActivationPendingUsername,
        password: liveDeviceActivationPendingPassword,
        name: "Browser Device Activation Pending User",
      });

      const response = await page.goto(
        deviceActivationPortalUrl(portalOrigin, flowId),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Sign in to continue" })
        .waitFor();
      await completeDeviceActivationPortalSignIn({
        page,
        username: liveDeviceActivationPendingUsername,
        password: liveDeviceActivationPendingPassword,
      });
      await waitForHeading(page, "Approve this device");
      await page.getByLabel("Confirmation code").fill(confirmationCode);
      await waitForDeviceActivationReview({
        admin,
        runtime,
        deploymentId,
        instanceId: provisioned.device.instanceId,
        publicIdentityKey: identity.publicIdentityKey,
      });
      await assertNoActivatedDeviceAuthority({
        admin,
        deploymentId,
        instanceId: provisioned.device.instanceId,
        publicIdentityKey: identity.publicIdentityKey,
      });
      await assertDeviceConnectRejected({
        runtime,
        fixture,
        identity,
        rootSecret,
        deploymentId,
        instanceId: provisioned.device.instanceId,
        participantNeedsDigest,
      });
    } finally {
      await loginAdmin.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
    }
  },
);

withLivePortalPage(
  "browser.login-portal live rejected device activation denies connect",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveDeviceActivationRejectedFixture;
    const { admin, deploymentId } = await fixture.setupDeviceDeployment(
      runtime,
      "required",
    );
    const loginAdmin = await liveDeviceActivationRejectedLoginFixture
      .setupSessionAdmin(runtime);
    const rejectionReason = "browser review rejected";

    try {
      const { identity, rootSecret, provisioned } = await fixture
        .setupProvisionedDevice(admin, deploymentId);
      const { confirmationCode, flowId, participantNeedsDigest } = await fixture
        .setupActivationRequest(
          runtime,
          admin,
          deploymentId,
          identity,
          provisioned.device.instanceId,
        );
      const review = await waitForDeviceActivationReview({
        admin,
        runtime,
        deploymentId,
        instanceId: provisioned.device.instanceId,
        publicIdentityKey: identity.publicIdentityKey,
      });
      const decided = await admin.authDeviceUserAuthoritiesReviewsDecide({
        reviewId: review.reviewId,
        decision: "reject",
        expectedVersion: review.version,
        idempotencyKey: crypto.randomUUID(),
        reason: rejectionReason,
      }).orThrow();
      assertEquals(decided.review.state, "rejected");
      await setupDeviceActivationPortalUser({
        admin: loginAdmin,
        runtime,
        portalOrigin,
        portalId: liveDeviceActivationRejectedPortalId,
        username: liveDeviceActivationRejectedUsername,
        password: liveDeviceActivationRejectedPassword,
        name: "Browser Device Activation Rejected User",
      });

      const response = await page.goto(
        deviceActivationPortalUrl(portalOrigin, flowId),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Sign in to continue" })
        .waitFor();
      await completeDeviceActivationPortalSignIn({
        page,
        username: liveDeviceActivationRejectedUsername,
        password: liveDeviceActivationRejectedPassword,
      });
      await waitForHeading(page, "Approve this device");
      await page.getByLabel("Confirmation code").fill(confirmationCode);
      await page.getByRole("button", { name: "Approve device" }).click();
      await waitForHeading(page, "Request denied");
      await page.getByText(rejectionReason).waitFor();

      await assertNoActivatedDeviceAuthority({
        admin,
        deploymentId,
        instanceId: provisioned.device.instanceId,
        publicIdentityKey: identity.publicIdentityKey,
      });
      await assertDeviceConnectRejected({
        runtime,
        fixture,
        identity,
        rootSecret,
        deploymentId,
        instanceId: provisioned.device.instanceId,
        participantNeedsDigest,
      });
    } finally {
      await loginAdmin.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
    }
  },
);

withLivePortalPage(
  "browser.login-portal live invalid device activation shows error",
  async ({ page, portalOrigin }) => {
    const response = await page.goto(
      new URL("/_trellis/portal/devices/activate", portalOrigin).toString(),
      { waitUntil: "networkidle" },
    );
    assertEquals(response?.status(), 200);
    await page.getByRole("heading", { name: "Invalid link" }).waitFor();
    await page.getByText("Missing flow id.").waitFor();
  },
);

function withLivePortalPage(
  name: string,
  fn: (args: {
    page: Page;
    portalOrigin: string;
    runtime: LiveTrellisRuntime;
  }) => Promise<void>,
): void {
  Deno.test(name, async () => {
    let trellisUrl: string | undefined;
    const server = Deno.serve(
      { hostname: "127.0.0.1", port: 0, onListen() {} },
      (request) =>
        trellisUrl === undefined
          ? new Response("Trellis is starting", { status: 503 })
          : serveStatic(request, buildDir, trellisUrl),
    );
    const portalOrigin = `http://127.0.0.1:${server.addr.port}`;

    try {
      await withTrellisRuntime(
        async (runtime) => {
          trellisUrl = runtime.trellisUrl;
          await withCoveredPage(name, async ({ page }) => {
            await fn({ page, portalOrigin, runtime });
          });
        },
        { webOrigins: [portalOrigin] },
      );
    } finally {
      await server.shutdown();
    }
  });
}

async function withCoveredPage(
  name: string,
  fn: (args: { page: Page }) => Promise<void>,
): Promise<void> {
  let browser: Browser | undefined;

  try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Profiler.enable");
    await cdp.send("Profiler.startPreciseCoverage", {
      callCount: true,
      detailed: true,
    });

    try {
      await fn({ page });
    } finally {
      const coverage = await cdp.send("Profiler.takePreciseCoverage");
      await cdp.send("Profiler.stopPreciseCoverage");
      await cdp.send("Profiler.disable");
      await Deno.mkdir(coverageDir, { recursive: true });
      await Deno.writeTextFile(
        join(coverageDir, `${coverageSlug(name)}-v8.json`),
        JSON.stringify(coverage, null, 2),
      );
    }
  } finally {
    await browser?.close();
  }
}

async function serveStatic(
  request: Request,
  root: string,
  runtimeUrl?: string,
): Promise<Response> {
  const url = new URL(request.url);
  if (runtimeUrl && shouldProxyToRuntime(url.pathname)) {
    const response = await fetch(
      new URL(url.pathname + url.search, runtimeUrl),
      {
        method: request.method,
        headers: request.headers,
        body: request.body === null ? undefined : await request.arrayBuffer(),
      },
    );
    const body = await response.arrayBuffer();
    return new Response(body, {
      status: response.status,
      headers: {
        "content-type": response.headers.get("content-type") ??
          "application/octet-stream",
      },
    });
  }

  const pathname = decodeURIComponent(url.pathname);
  const candidate = resolve(
    root,
    pathname === "/" ? "index.html" : `.${normalize(pathname)}`,
  );
  const path = isInside(root, candidate) && await exists(candidate)
    ? candidate
    : join(root, "200.html");
  const body = await Deno.readFile(path);
  return new Response(body, {
    headers: { "content-type": contentType(path) },
  });
}

function shouldProxyToRuntime(pathname: string): boolean {
  return pathname === "/auth/login/local" ||
    pathname === "/auth/requests" ||
    pathname.startsWith("/auth/account-flow/") ||
    pathname.startsWith("/auth/flow/") ||
    pathname.startsWith("/auth/login/");
}

async function createLocalPasswordUser(args: {
  admin: SessionAdminClient;
  runtime: LiveTrellisRuntime;
  username: string;
  password: string;
  name: string;
  capabilities?: string[];
}): Promise<
  Awaited<
    ReturnType<
      ReturnType<SessionAdminClient["authUsersCreate"]>["orThrow"]
    >
  >
> {
  const user = await args.admin.authUsersCreate({
    name: args.name,
    email: `${args.username}@example.test`,
    image: null,
    idempotencyKey: crypto.randomUUID(),
  }).orThrow();
  const reset = await args.admin.authUsersPasswordResetCreate({
    idempotencyKey: crypto.randomUUID(),
    returnTarget: null,
    userId: user.user.userId,
  }).orThrow();
  await completeLocalPasswordAccountFlow({
    completionUrl: reset.flow.completionUrl,
    username: args.username,
    password: args.password,
  });
  return user;
}

function accountFlowPortalUrl(
  portalOrigin: string,
  pathname: string,
  flowId: string,
): string {
  const url = new URL(pathname, portalOrigin);
  url.searchParams.set("flowId", flowId);
  return url.toString();
}

function accountFlowToken(completionUrl: string): string {
  const url = new URL(completionUrl);
  const token = url.searchParams.get("flowId") ??
    url.pathname.split("/").at(-1);
  assert(token, "account-flow completion URL must contain its bearer token");
  return decodeURIComponent(token);
}

function deviceActivationPortalUrl(
  portalOrigin: string,
  flowId: string,
): string {
  const url = new URL("/_trellis/portal/devices/activate", portalOrigin);
  url.searchParams.set("flowId", flowId);
  return url.toString();
}

async function completeLocalPasswordAccountFlow(args: {
  completionUrl: string;
  username: string;
  password: string;
}): Promise<void> {
  const response = await fetch(
    `${args.completionUrl}/local-password`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        origin: new URL(args.completionUrl).origin,
      },
      body: JSON.stringify({
        username: args.username,
        password: args.password,
      }),
    },
  );
  const body = await response.text();
  assertEquals(response.status, 200, body);
}

async function setupLocalLoginPortalUser(args: {
  admin: SessionAdminClient;
  runtime: LiveTrellisRuntime;
  fixture: AuthLocalLoginFixture;
  portalOrigin: string;
  portalId: string;
  username: string;
  password: string;
  name: string;
}): Promise<void> {
  const user = await args.admin.authUsersCreate({
    name: args.name,
    email: `${args.username}@example.test`,
    image: null,
    idempotencyKey: crypto.randomUUID(),
  }).orThrow();
  const reset = await args.admin.authUsersPasswordResetCreate({
    idempotencyKey: crypto.randomUUID(),
    returnTarget: null,
    userId: user.user.userId,
  }).orThrow();
  await completeLocalPasswordAccountFlow({
    completionUrl: reset.flow.completionUrl,
    username: args.username,
    password: args.password,
  });
  await configureLocalLoginPortal(args);
}

async function setupDeviceActivationPortalUser(args: {
  admin: SessionAdminClient;
  runtime: LiveTrellisRuntime;
  portalOrigin: string;
  portalId: string;
  username: string;
  password: string;
  name: string;
}): Promise<void> {
  const user = await args.admin.authUsersCreate({
    name: args.name,
    email: `${args.username}@example.test`,
    image: null,
    idempotencyKey: crypto.randomUUID(),
  }).orThrow();
  const reset = await args.admin.authUsersPasswordResetCreate({
    idempotencyKey: crypto.randomUUID(),
    returnTarget: null,
    userId: user.user.userId,
  }).orThrow();
  await completeLocalPasswordAccountFlow({
    completionUrl: reset.flow.completionUrl,
    username: args.username,
    password: args.password,
  });
  await args.admin.authPortalsPut({
    disabled: false,
    portalId: args.portalId,
    displayName: "Browser Device Activation Portal",
    entryUrl: `${args.portalOrigin}/_trellis/portal/users/login`,
    expectedVersion: null,
    idempotencyKey: crypto.randomUUID(),
    loginSettings: {
      federatedRegistration: false,
      localLogin: true,
      localRegistration: false,
      providers: ["local"],
    },
  }).orThrow();
  await args.admin.authPortalsRoutesPut({
    deploymentId: null,
    expectedVersion: null,
    idempotencyKey: crypto.randomUUID(),
    portalId: args.portalId,
    participantId: deviceActivationPortalContractId,
    origin: args.portalOrigin,
    priority: 0,
    routeId: null,
  }).orThrow();
}

async function configureLocalLoginPortal(args: {
  admin: SessionAdminClient;
  fixture: AuthLocalLoginFixture;
  localRegistration?: boolean;
  portalOrigin: string;
  portalId: string;
  routeOrigin?: string;
}) {
  const portal = await args.admin.authPortalsPut({
    disabled: false,
    portalId: args.portalId,
    displayName: "Browser Login Portal",
    entryUrl: `${args.portalOrigin}/_trellis/portal/users/login`,
    expectedVersion: null,
    idempotencyKey: crypto.randomUUID(),
    loginSettings: {
      federatedRegistration: false,
      localLogin: true,
      localRegistration: args.localRegistration ?? false,
      providers: ["local"],
    },
  }).orThrow();
  await args.admin.authPortalsRoutesPut({
    deploymentId: null,
    expectedVersion: null,
    idempotencyKey: crypto.randomUUID(),
    portalId: args.portalId,
    participantId: args.fixture.clientContract.CONTRACT_ID,
    origin: args.routeOrigin ?? args.portalOrigin,
    priority: 0,
    routeId: null,
  }).orThrow();
  return portal;
}

async function completeBrowserLocalLogin(args: {
  page: Page;
  loginUrl: string;
  portalOrigin: string;
  username: string;
  password: string;
}): Promise<string> {
  const flowId = flowIdFromUrl(args.loginUrl);
  const response = await args.page.goto(
    portalPageUrl(args.loginUrl, args.portalOrigin),
    { waitUntil: "networkidle" },
  );
  assertEquals(response?.status(), 200);

  await args.page.getByLabel("Username").fill(args.username);
  await args.page.getByLabel("Password").fill(args.password);
  await args.page.getByRole("button", { name: "Sign in" }).last().click();
  const approve = args.page.getByRole("button", { name: "Approve" });
  if (await approve.isVisible({ timeout: 10_000 }).catch(() => false)) {
    await Promise.all([
      args.page.waitForURL("**/_trellis/test/client-auth**"),
      approve.click(),
    ]);
  } else {
    const redirected = await args.page.waitForURL(
      "**/_trellis/test/client-auth**",
      { timeout: 2_000 },
    ).then(() => true, () => false);
    if (!redirected) {
      const state = await fetchJson(
        `${new URL(args.loginUrl).origin}/auth/flow/${
          encodeURIComponent(flowId)
        }`,
      );
      if (state.state === "approval_required") {
        const response = await args.page.goto(
          portalPageUrl(args.loginUrl, args.portalOrigin),
          { waitUntil: "networkidle" },
        );
        assertEquals(response?.status(), 200);
        await approve.waitFor({ state: "visible" });
        await approve.click();
        await args.page.waitForURL("**/_trellis/test/client-auth**", {
          timeout: 2_000,
        }).catch(() => undefined);
        const approved = await fetchJson(
          `${new URL(args.loginUrl).origin}/auth/flow/${
            encodeURIComponent(flowId)
          }`,
        );
        assertEquals(approved.state, "approved");
      } else {
        assertEquals(state.state, "approved");
      }
    }
  }
  return flowId;
}

async function completeDeviceActivationPortalSignIn(args: {
  page: Page;
  username: string;
  password: string;
}): Promise<void> {
  await args.page.getByRole("button", { name: "Continue to sign in" }).click();
  await args.page.getByLabel("Username").fill(args.username);
  await args.page.getByLabel("Password").fill(args.password);
  await args.page.getByRole("button", { name: "Sign in" }).last().click();
  const approve = args.page.getByRole("button", {
    name: "Approve",
    exact: true,
  });
  await approve.waitFor({ state: "visible" });
  await approve.click();
}

async function waitForDeviceActivationReview(args: {
  admin: DeviceActivationAdmin;
  runtime: LiveTrellisRuntime;
  deploymentId: string;
  instanceId: string;
  publicIdentityKey: string;
}): Promise<{ readonly reviewId: string; readonly version: number }> {
  return await args.runtime.waitFor(async () => {
    const reviews = await args.admin.authDeviceUserAuthoritiesReviewsList({
      deploymentId: args.deploymentId,
      limit: 20,
    }).orThrow();
    const review = reviews.entries.find((entry) =>
      entry.deploymentId === args.deploymentId &&
      entry.instanceId === args.instanceId
    );
    if (review && review.state !== "pending") {
      throw new Error(
        `expected pending activation review, got ${review.state}`,
      );
    }
    return review;
  }, { timeoutMs: 10_000, intervalMs: 25 });
}

async function assertNoActivatedDeviceAuthority(args: {
  admin: DeviceActivationAdmin;
  deploymentId: string;
  instanceId: string;
  publicIdentityKey: string;
}): Promise<void> {
  const activations = requireDeviceAuthorityList(
    await args.admin.authDeviceUserAuthoritiesList({
      deploymentId: args.deploymentId,
      limit: 20,
    }).orThrow(),
  );
  assertEquals(
    activations.entries.filter((entry) =>
      entry.instanceId === args.instanceId &&
      entry.identityPublicKey === args.publicIdentityKey &&
      entry.deploymentId === args.deploymentId &&
      entry.state === "active"
    ),
    [],
  );
}

async function assertDeviceConnectRejected(args: {
  runtime: LiveTrellisRuntime;
  fixture: ReturnType<typeof createDeviceActivationFixture>;
  identity: DeviceActivationIdentity;
  rootSecret: Uint8Array;
  deploymentId: string;
  instanceId: string;
  participantNeedsDigest: string;
}): Promise<void> {
  const connect = await TrellisDevice.connect({
    trellisUrl: args.runtime.trellisUrl,
    contract: args.fixture.deviceContract,
    rootSecret: args.rootSecret,
    identity: {
      deploymentId: args.deploymentId,
      instanceId: args.instanceId,
      principalId: args.instanceId,
      participantId: args.fixture.deviceContract.CONTRACT_ID,
      participantArtifactDigest: args.fixture.deviceContract.CONTRACT_DIGEST,
      participantNeedsDigest: args.participantNeedsDigest,
    },
    log: false,
    authorizationContextEphemeral: true,
  });
  if (!connect.isErr()) {
    await connect.orThrow().connection.close().catch(() => undefined);
  }
  assert(connect.isErr(), "device should not connect");
}

async function waitForHeading(page: Page, name: string): Promise<void> {
  try {
    await page.getByRole("heading", { name }).waitFor();
  } catch (error) {
    throw new Error(
      `Expected heading "${name}" at ${page.url()}. Browser body:\n${await page
        .locator("body")
        .innerText()}`,
      { cause: error },
    );
  }
}

async function completeLocalLoginByFetch(args: {
  trellisUrl: string;
  origin: string;
  flowId: string;
  username: string;
  password: string;
}): Promise<void> {
  const loginResponse = await fetch(`${args.trellisUrl}/auth/login/local`, {
    method: "POST",
    headers: { "content-type": "application/json", origin: args.origin },
    body: JSON.stringify({
      flowId: args.flowId,
      username: args.username,
      password: args.password,
    }),
  });
  const loginBody = await loginResponse.text();
  assertEquals(loginResponse.status, 200, loginBody);

  const state = await fetchJson(
    `${args.trellisUrl}/auth/flow/${encodeURIComponent(args.flowId)}`,
    { headers: { origin: args.origin } },
  );
  if (state.state === "approval_required") {
    assertEquals(typeof state.consentViewDigest, "string");
    const approved = await fetchJson(
      `${args.trellisUrl}/auth/flow/${
        encodeURIComponent(args.flowId)
      }/approval`,
      {
        method: "POST",
        headers: { "content-type": "application/json", origin: args.origin },
        body: JSON.stringify({
          approved: true,
          consentViewDigest: state.consentViewDigest,
          selectedOptionalBundles: [],
        }),
      },
    );
    assertEquals(approved.state, "approved");
  } else {
    assertEquals(state.state, "approved");
  }
}

async function appSessionsFor(
  admin: SessionAdminClient,
  sessionKey: string,
): Promise<AppSession[]> {
  const sessions = await admin.authSessionsList({ limit: 100 }).orThrow();
  return sessions.entries.filter((entry): entry is AppSession =>
    entry.participantKind === "app" && entry.sessionPublicKey === sessionKey
  );
}

async function assertNoAppSession(
  admin: SessionAdminClient,
  sessionKey: string,
): Promise<void> {
  assertEquals((await appSessionsFor(admin, sessionKey)).length, 0);
}

async function appSessionFor(
  admin: SessionAdminClient,
  sessionKey: string,
): Promise<AppSession> {
  const [session] = await appSessionsFor(admin, sessionKey);
  assert(session, "expected Auth.Sessions.List to include app session");
  return session;
}

async function singleConnectionFor(
  admin: SessionAdminClient,
  sessionKey: string,
): Promise<AppConnection> {
  const connections = await admin.authConnectionsList({
    sessionId: (await appSessionFor(admin, sessionKey)).sessionId,
    limit: 100,
  }).orThrow();
  assertEquals(connections.entries.length, 1);
  const [connection] = connections.entries;
  assert(connection, "expected exactly one app connection");
  return connection;
}

async function fetchJson(
  url: string,
  init?: RequestInit,
): Promise<Record<string, unknown>> {
  const response = await fetch(url, init);
  const body = await response.text();
  assert(
    response.ok,
    `HTTP request failed (${response.status}) for ${url}: ${body}`,
  );
  const value: unknown = JSON.parse(body);
  assert(isRecord(value), "expected JSON object response");
  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function flowIdFromUrl(url: string): string {
  const flowId = new URL(url).searchParams.get("flowId");
  if (!flowId) throw new Error(`Trellis auth URL is missing flowId: ${url}`);
  return flowId;
}

function portalPageUrl(loginUrl: string, portalOrigin: string): string {
  const url = new URL(loginUrl);
  const origin = new URL(portalOrigin);
  url.protocol = origin.protocol;
  url.host = origin.host;
  return url.toString();
}

function coverageSlug(name: string): string {
  return name.replace(/^browser\./, "").toLowerCase().replaceAll(
    /[^a-z0-9]+/g,
    "-",
  ).replace(/^-|-$/g, "");
}

async function exists(path: string): Promise<boolean> {
  try {
    return (await Deno.stat(path)).isFile;
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return false;
    throw error;
  }
}

function isInside(root: string, path: string): boolean {
  const rel = relative(resolve(root), resolve(path));
  return rel === "" || (!rel.startsWith("..") && !isAbsolute(rel));
}

function contentType(path: string): string {
  switch (extname(path)) {
    case ".css":
      return "text/css; charset=utf-8";
    case ".html":
      return "text/html; charset=utf-8";
    case ".js":
      return "text/javascript; charset=utf-8";
    case ".svg":
      return "image/svg+xml";
    default:
      return "application/octet-stream";
  }
}
