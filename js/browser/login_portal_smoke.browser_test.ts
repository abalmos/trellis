import {
  type CallerRuntime,
  TrellisClient,
  TrellisDevice,
} from "@qlever-llc/trellis";
import {
  base64urlEncode,
  sha256,
  utf8,
  waitForDeviceActivation,
} from "@qlever-llc/trellis/auth";
import type {
  AuthConnectionsListOutput,
  AuthSessionsListOutput,
} from "@qlever-llc/trellis/sdk/auth";
import {
  assert,
  assertArrayIncludes,
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

import { caseScopedName } from "../integration/_support/names.ts";
import {
  type LiveTrellisRuntime,
  withTrellisRuntime,
} from "../integration/_support/runtime.ts";
import { createAuthLocalLoginFixture } from "../integration/auth/_fixture.ts";
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
const liveLocalLoginPortalId = caseScopedName(
  "browser-login-portal",
  liveLocalLoginCaseId,
);
const liveLocalLoginUsername = caseScopedName(
  "browser-login-portal-user",
  liveLocalLoginCaseId,
);
const liveLocalLoginPassword =
  `trellis-integration-${liveLocalLoginCaseId}-password-2026`;
const liveSessionRefreshCaseId =
  "browser.login-portal-live-existing-session-refreshes-authority";
const liveSessionRefreshFixture = createAuthLocalLoginFixture(
  liveSessionRefreshCaseId,
);
const liveSessionRefreshPortalId = caseScopedName(
  "browser-login-portal",
  liveSessionRefreshCaseId,
);
const liveSessionRefreshUsername = caseScopedName(
  "browser-login-portal-user",
  liveSessionRefreshCaseId,
);
const liveSessionRefreshPassword =
  `trellis-integration-${liveSessionRefreshCaseId}-password-2026`;
const liveInvalidLocalLoginCaseId =
  "browser.login-portal-live-invalid-local-credentials";
const liveInvalidLocalLoginFixture = createAuthLocalLoginFixture(
  liveInvalidLocalLoginCaseId,
);
const liveInvalidLocalLoginPortalId = caseScopedName(
  "browser-login-portal",
  liveInvalidLocalLoginCaseId,
);
const liveInvalidLocalLoginUsername = caseScopedName(
  "browser-login-portal-user",
  liveInvalidLocalLoginCaseId,
);
const liveInvalidLocalLoginPassword =
  `trellis-integration-${liveInvalidLocalLoginCaseId}-password-2026`;
const liveInactiveLocalLoginCaseId =
  "browser.login-portal-live-inactive-local-user";
const liveInactiveLocalLoginFixture = createAuthLocalLoginFixture(
  liveInactiveLocalLoginCaseId,
);
const liveInactiveLocalLoginPortalId = caseScopedName(
  "browser-login-portal",
  liveInactiveLocalLoginCaseId,
);
const liveInactiveLocalLoginUsername = caseScopedName(
  "browser-login-portal-user",
  liveInactiveLocalLoginCaseId,
);
const liveInactiveLocalLoginPassword =
  `trellis-integration-${liveInactiveLocalLoginCaseId}-password-2026`;
const liveDeniedConsentCaseId = "browser.login-portal-live-denied-consent";
const liveDeniedConsentFixture = createAuthLocalLoginFixture(
  liveDeniedConsentCaseId,
);
const liveDeniedConsentPortalId = caseScopedName(
  "browser-login-portal",
  liveDeniedConsentCaseId,
);
const liveDeniedConsentUsername = caseScopedName(
  "browser-login-portal-user",
  liveDeniedConsentCaseId,
);
const liveDeniedConsentPassword =
  `trellis-integration-${liveDeniedConsentCaseId}-password-2026`;
const liveInsufficientCapabilitiesCaseId =
  "browser.login-portal-live-insufficient-capabilities";
const liveInsufficientCapabilitiesFixture = createAuthLocalLoginFixture(
  liveInsufficientCapabilitiesCaseId,
);
const liveInsufficientCapabilitiesPortalId = caseScopedName(
  "browser-login-portal",
  liveInsufficientCapabilitiesCaseId,
);
const liveInsufficientCapabilitiesUsername = caseScopedName(
  "browser-login-portal-user",
  liveInsufficientCapabilitiesCaseId,
);
const liveInsufficientCapabilitiesPassword =
  `trellis-integration-${liveInsufficientCapabilitiesCaseId}-password-2026`;
const liveAccountLinkCaseId = "browser.login-portal-live-account-link";
const liveAccountLinkFixture = createAuthLocalLoginFixture(
  liveAccountLinkCaseId,
);
const liveAccountLinkUsername = caseScopedName(
  "browser-account-link-user",
  liveAccountLinkCaseId,
);
const liveAccountLinkPassword =
  `trellis-integration-${liveAccountLinkCaseId}-password-2026`;
const liveAccountLinkDuplicateCaseId =
  "browser.login-portal-live-account-link-duplicate-local-username";
const liveAccountLinkDuplicateFixture = createAuthLocalLoginFixture(
  liveAccountLinkDuplicateCaseId,
);
const liveAccountLinkDuplicateExistingUsername = caseScopedName(
  "browser-account-link-existing-user",
  liveAccountLinkDuplicateCaseId,
);
const liveAccountLinkDuplicatePassword =
  `trellis-integration-${liveAccountLinkDuplicateCaseId}-password-2026`;
const liveAccountPasswordCaseId = "browser.login-portal-live-account-password";
const liveAccountPasswordFixture = createAuthLocalLoginFixture(
  liveAccountPasswordCaseId,
);
const liveAccountPasswordPortalId = caseScopedName(
  "browser-login-portal",
  liveAccountPasswordCaseId,
);
const liveAccountPasswordUsername = caseScopedName(
  "browser-account-password-user",
  liveAccountPasswordCaseId,
);
const liveAccountPasswordInitialPassword =
  `trellis-integration-${liveAccountPasswordCaseId}-initial-password-2026`;
const liveAccountPasswordNewPassword =
  `trellis-integration-${liveAccountPasswordCaseId}-new-password-2026`;
const liveAccountPasswordTooShortCaseId =
  "browser.login-portal-live-account-password-too-short";
const liveAccountPasswordTooShortFixture = createAuthLocalLoginFixture(
  liveAccountPasswordTooShortCaseId,
);
const liveAccountPasswordTooShortPortalId = caseScopedName(
  "browser-login-portal",
  liveAccountPasswordTooShortCaseId,
);
const liveAccountPasswordTooShortUsername = caseScopedName(
  "browser-account-password-short-user",
  liveAccountPasswordTooShortCaseId,
);
const liveAccountPasswordTooShortInitialPassword =
  `trellis-integration-${liveAccountPasswordTooShortCaseId}-initial-password-2026`;
const liveMissingAccountFlowCaseId =
  "browser.login-portal-live-missing-account-flow";
const liveMissingAccountFlowFixture = createAuthLocalLoginFixture(
  liveMissingAccountFlowCaseId,
);
const liveMissingAccountFlowUsername = caseScopedName(
  "browser-missing-account-flow-user",
  liveMissingAccountFlowCaseId,
);
const liveReusedAccountFlowCaseId =
  "browser.login-portal-live-reused-account-flow";
const liveReusedAccountFlowFixture = createAuthLocalLoginFixture(
  liveReusedAccountFlowCaseId,
);
const liveReusedAccountFlowUsername = caseScopedName(
  "browser-reused-account-flow-user",
  liveReusedAccountFlowCaseId,
);
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
const liveDeviceActivationPortalId = caseScopedName(
  "browser-device-activation-portal",
  liveDeviceActivationCaseId,
);
const liveDeviceActivationUsername = caseScopedName(
  "browser-device-activation-user",
  liveDeviceActivationCaseId,
);
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
const liveDeviceActivationPendingPortalId = caseScopedName(
  "browser-device-activation-pending-portal",
  liveDeviceActivationPendingCaseId,
);
const liveDeviceActivationPendingUsername = caseScopedName(
  "browser-device-activation-pending-user",
  liveDeviceActivationPendingCaseId,
);
const liveDeviceActivationPendingPassword =
  `trellis-integration-${liveDeviceActivationPendingCaseId}-password-2026`;
const liveInvalidDeviceActivationCaseId =
  "browser.login-portal-live-invalid-device-activation";
const liveInvalidDeviceActivationFixture = createDeviceActivationFixture(
  liveInvalidDeviceActivationCaseId,
);
const liveInvalidDeviceActivationLoginFixture = createAuthLocalLoginFixture(
  `${liveInvalidDeviceActivationCaseId}.login`,
);
const liveInvalidDeviceActivationPortalId = caseScopedName(
  "browser-invalid-device-activation-portal",
  liveInvalidDeviceActivationCaseId,
);
const liveInvalidDeviceActivationUsername = caseScopedName(
  "browser-invalid-device-activation-user",
  liveInvalidDeviceActivationCaseId,
);
const liveInvalidDeviceActivationPassword =
  `trellis-integration-${liveInvalidDeviceActivationCaseId}-password-2026`;
const liveDeviceActivationRejectedCaseId =
  "browser.login-portal-live-device-activation-rejected";
const liveDeviceActivationRejectedFixture = createDeviceActivationFixture(
  liveDeviceActivationRejectedCaseId,
);
const liveDeviceActivationRejectedLoginFixture = createAuthLocalLoginFixture(
  `${liveDeviceActivationRejectedCaseId}.login`,
);
const liveDeviceActivationRejectedPortalId = caseScopedName(
  "browser-device-activation-rejected-portal",
  liveDeviceActivationRejectedCaseId,
);
const liveDeviceActivationRejectedUsername = caseScopedName(
  "browser-device-activation-rejected-user",
  liveDeviceActivationRejectedCaseId,
);
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
type ControlPlaneSqlite = NonNullable<
  LiveTrellisRuntime["controlPlane"]
>["sqlite"];
type AppSession = Extract<
  AuthSessionsListOutput["entries"][number],
  { participantKind: "app" }
>;
type AppConnection = Extract<
  AuthConnectionsListOutput["entries"][number],
  { participantKind: "app" }
>;

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
        username: liveLocalLoginUsername,
        name: "Browser Login Portal User",
        email: `${liveLocalLoginUsername}@example.test`,
        active: true,
        capabilities: [liveLocalLoginFixture.pingCapability],
        capabilityGroups: ["admin"],
      }).orThrow();
      const reset = await admin.authUsersPasswordResetCreate({
        userId: user.user.userId,
      }).orThrow();
      await completeLocalPasswordAccountFlow({
        trellisUrl: runtime.trellisUrl,
        flowId: reset.flowId,
        username: liveLocalLoginUsername,
        password: liveLocalLoginPassword,
      });
      await admin.authPortalsPut({
        portalId: liveLocalLoginPortalId,
        displayName: "Browser Login Portal",
        entryUrl: `${portalOrigin}/_trellis/portal/users/login`,
      }).orThrow();
      await admin.authPortalsRoutesPut({
        portalId: liveLocalLoginPortalId,
        contractId: liveLocalLoginFixture.clientContract.CONTRACT.id,
        origin: portalOrigin,
      }).orThrow();
      await admin.authDeploymentAuthorityGrantOverridesPut({
        deploymentId: liveLocalLoginFixture.deploymentId,
        overrides: [{
          deploymentId: liveLocalLoginFixture.deploymentId,
          identityKind: "web",
          grantKind: "capability",
          contractId: liveLocalLoginFixture.clientContract.CONTRACT.id,
          origin: portalOrigin,
          sessionPublicKey: null,
          capability: liveLocalLoginFixture.pingCapability,
          capabilityGroupKey: null,
        }],
      }).orThrow();

      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: liveLocalLoginFixture.clientName,
        contract: liveLocalLoginFixture.clientContract,
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
          if (await approve.isVisible({ timeout: 10_000 }).catch(() => false)) {
            await Promise.all([
              page.waitForURL(`${portalOrigin}/_trellis/test/client-auth**`),
              approve.click(),
            ]);
          } else {
            await page.waitForURL(
              `${portalOrigin}/_trellis/test/client-auth**`,
            );
          }

          return { status: "bound", flowId };
        },
      }).orThrow();

      assert(authRequired, "expected local-login flow to require auth");
      const me = await client.authSessionsMe({}).orThrow();
      assertEquals(me.participantKind, "app");
      assert(me.user !== null, "expected Auth.Sessions.Me to return a user");
      assertEquals(me.user.active, true);
      assertArrayIncludes(me.user.capabilities, ["admin"]);

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
  "browser.login-portal live existing session refreshes authority",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveSessionRefreshFixture;
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientKey, clientAuth } = await fixture.setupClientRegistration(
      runtime,
    );
    let originalClient:
      | CallerRuntime<typeof fixture.clientContract>
      | undefined;
    let reboundClient:
      | CallerRuntime<typeof fixture.updatedClientContract>
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
        auth: clientAuth.auth,
        onAuthRequired: async (ctx) => {
          const flowId = flowIdFromUrl(ctx.loginUrl);
          await completeLocalLoginByFetch({
            trellisUrl: runtime.trellisUrl,
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
        contract: fixture.updatedClientContract,
        auth: {
          ...clientAuth.auth,
          redirectTo: `${portalOrigin}/_trellis/test/client-auth`,
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

      assert(authRequired, "expected updated authority to require local login");
      const afterSession = await appSessionFor(admin, clientKey.sessionKey);
      assertEquals(afterSession.createdAt, beforeSession.createdAt);
      assertEquals(
        afterSession.principal.userId,
        beforeSession.principal.userId,
      );
      assertEquals(
        afterSession.contractDisplayName,
        fixture.updatedClientDisplayName,
      );

      const allowedByUpdatedAuthority = await reboundClient.authConnectionsList(
        { sessionKey: clientKey.sessionKey, limit: 500 },
      )
        .orThrow();
      assert(
        allowedByUpdatedAuthority.entries.length >= 1,
        "expected updated authority to list a live app connection",
      );

      const afterConnection = await runtime.waitFor(async () => {
        const connections = await admin.authConnectionsList({
          sessionKey: clientKey.sessionKey,
          limit: 500,
        }).orThrow();
        return connections.entries.find((entry): entry is AppConnection =>
          entry.participantKind === "app" &&
          entry.userNkey !== beforeConnection.userNkey
        );
      });
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
        sessionKey: clientKey.sessionKey,
        limit: 500,
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
        userId: user.user.userId,
        active: false,
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
      await assertNoAppSessionOrConnection(admin, clientKey.sessionKey);
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
        useGrantOverride: false,
      });

      let authRequired = false;
      const connectResult = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
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
      await assertNoAppSessionOrConnection(admin, clientKey.sessionKey);
      await assertNoApprovedGrantForContract({
        sqlite: requireControlPlaneSqlite(runtime),
        contractId: fixture.clientContract.CONTRACT.id,
      });
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live insufficient capabilities fails closed",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveInsufficientCapabilitiesFixture;
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientKey, clientAuth } = await fixture.setupClientRegistration(
      runtime,
    );

    try {
      await createLocalPasswordUser({
        admin,
        runtime,
        username: liveInsufficientCapabilitiesUsername,
        password: liveInsufficientCapabilitiesPassword,
        name: "Browser Missing Capability User",
      });
      await configureLocalLoginPortal({
        admin,
        fixture,
        portalOrigin,
        portalId: liveInsufficientCapabilitiesPortalId,
        useGrantOverride: false,
      });

      let authRequired = false;
      const connectResult = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
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
            liveInsufficientCapabilitiesUsername,
          );
          await page.getByLabel("Password").fill(
            liveInsufficientCapabilitiesPassword,
          );
          await page.getByRole("button", { name: "Sign in" }).last().click();
          await page.getByRole("heading", { name: "Access denied" }).waitFor();
          await page.getByText("Missing capabilities").waitFor();
          await page.getByText("Call local-login ping").waitFor();
          return { status: "handled" };
        },
      });

      assert(authRequired, "expected missing capability to require auth");
      assert(
        connectResult.isErr(),
        "expected insufficient capabilities to stop connect",
      );
      await assertNoAppSessionOrConnection(admin, clientKey.sessionKey);
      await assertNoApprovedGrantForContract({
        sqlite: requireControlPlaneSqlite(runtime),
        contractId: fixture.clientContract.CONTRACT.id,
      });
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
);

withLivePortalPage(
  "browser.login-portal live account link completes",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveAccountLinkFixture;
    const admin = await fixture.setupSessionAdmin(runtime);

    try {
      const user = await admin.authUsersCreate({
        name: "Browser Account Link User",
        email: `${liveAccountLinkUsername}@example.test`,
        active: true,
      }).orThrow();
      const flowId = await putIdentityLinkFlow({
        sqlite: requireControlPlaneSqlite(runtime),
        caseId: liveAccountLinkCaseId,
        targetUserId: user.user.userId,
      });

      const response = await page.goto(
        accountFlowPortalUrl(
          portalOrigin,
          "/_trellis/portal/account/link",
          flowId,
        ),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Link local credentials" })
        .waitFor();
      await page.getByLabel("Username").fill(liveAccountLinkUsername);
      await page.getByLabel("Password").fill(liveAccountLinkPassword);
      await page.getByLabel(/Name/).fill("Browser Account Link Local");
      await page.getByLabel(/Email/).fill(
        `${liveAccountLinkUsername}-local@example.test`,
      );
      await page.getByRole("button", { name: "Link credentials" }).click();
      await page.getByRole("heading", { name: "Account linked" }).waitFor();

      const identities = await admin.authUserIdentitiesList({
        userId: user.user.userId,
        limit: 500,
      }).orThrow();
      assert(
        identities.entries.some((identity) =>
          identity.provider === "local" &&
          identity.subject === liveAccountLinkUsername
        ),
        "expected local identity to be linked to target user",
      );
      assertEquals(
        (await fetchJson(
          `${runtime.trellisUrl}/auth/account-flow/${
            encodeURIComponent(flowId)
          }`,
        )).status,
        "consumed",
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
    }
  },
);

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
        userId: user.user.userId,
      }).orThrow();

      const response = await page.goto(
        accountFlowPortalUrl(
          portalOrigin,
          "/_trellis/portal/account/password",
          reset.flowId,
        ),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Reset your password" })
        .waitFor();
      await page.getByLabel("Password").fill(liveAccountPasswordNewPassword);
      await page.getByRole("button", { name: "Reset password" }).click();
      await page.getByRole("heading", { name: "Password saved" }).waitFor();

      assertEquals(
        (await fetchJson(
          `${runtime.trellisUrl}/auth/account-flow/${
            encodeURIComponent(reset.flowId)
          }`,
        )).status,
        "consumed",
      );
      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        auth: clientAuth.auth,
        onAuthRequired: async (ctx) => {
          const flowId = flowIdFromUrl(ctx.loginUrl);
          await completeLocalLoginByFetch({
            trellisUrl: runtime.trellisUrl,
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
        userId: user.user.userId,
      }).orThrow();
      const stateUrl = `${runtime.trellisUrl}/auth/account-flow/${
        encodeURIComponent(reset.flowId)
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
          reset.flowId,
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

      assertEquals((await fetchJson(stateUrl)).status, "active");
      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        auth: clientAuth.auth,
        onAuthRequired: async (ctx) => {
          const flowId = flowIdFromUrl(ctx.loginUrl);
          await completeLocalLoginByFetch({
            trellisUrl: runtime.trellisUrl,
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
  "browser.login-portal live account link duplicate local username keeps flow active",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveAccountLinkDuplicateFixture;
    const admin = await fixture.setupSessionAdmin(runtime);

    try {
      await createLocalPasswordUser({
        admin,
        runtime,
        username: liveAccountLinkDuplicateExistingUsername,
        password: liveAccountLinkDuplicatePassword,
        name: "Browser Account Link Existing Local User",
      });
      const target = await admin.authUsersCreate({
        name: "Browser Account Link Duplicate Target User",
        email:
          `${liveAccountLinkDuplicateExistingUsername}-target@example.test`,
        active: true,
      }).orThrow();
      const flowId = await putIdentityLinkFlow({
        sqlite: requireControlPlaneSqlite(runtime),
        caseId: liveAccountLinkDuplicateCaseId,
        targetUserId: target.user.userId,
      });
      const before = await admin.authUserIdentitiesList({
        userId: target.user.userId,
        limit: 500,
      }).orThrow();

      const response = await page.goto(
        accountFlowPortalUrl(
          portalOrigin,
          "/_trellis/portal/account/link",
          flowId,
        ),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Link local credentials" })
        .waitFor();
      await page.getByLabel("Username").fill(
        liveAccountLinkDuplicateExistingUsername,
      );
      await page.getByLabel("Password").fill(liveAccountLinkDuplicatePassword);
      await page.getByLabel(/Name/).fill("Browser Account Link Duplicate");
      await page.getByLabel(/Email/).fill(
        `${liveAccountLinkDuplicateExistingUsername}-duplicate@example.test`,
      );
      await page.getByRole("button", { name: "Link credentials" }).click();
      await page.getByText(
        "That username is already in use. Choose a different username.",
      ).waitFor();

      const after = await admin.authUserIdentitiesList({
        userId: target.user.userId,
        limit: 500,
      }).orThrow();
      assertEquals(after.entries, before.entries);
      assertEquals(
        (await fetchJson(
          `${runtime.trellisUrl}/auth/account-flow/${
            encodeURIComponent(flowId)
          }`,
        )).status,
        "active",
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
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
        username: liveMissingAccountFlowUsername,
        name: "Browser Missing Account Flow User",
        email: `${liveMissingAccountFlowUsername}@example.test`,
        active: true,
      }).orThrow();
      const before = await admin.authUserIdentitiesList({
        userId: user.user.userId,
        limit: 500,
      }).orThrow();

      const response = await page.goto(
        accountFlowPortalUrl(
          portalOrigin,
          "/_trellis/portal/account/password",
          caseScopedName("missing-flow", liveMissingAccountFlowCaseId),
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

      const after = await admin.authUserIdentitiesList({
        userId: user.user.userId,
        limit: 500,
      }).orThrow();
      assertEquals(after.entries, before.entries);
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
        userId: user.user.userId,
      }).orThrow();
      const url = accountFlowPortalUrl(
        portalOrigin,
        "/_trellis/portal/account/password",
        reset.flowId,
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
          encodeURIComponent(reset.flowId)
        }`,
      );
      assertEquals(terminalState.status, "consumed");
      const before = await admin.authUserIdentitiesList({
        userId: user.user.userId,
        limit: 500,
      }).orThrow();

      const reusedResponse = await page.goto(url, { waitUntil: "networkidle" });
      assertEquals(reusedResponse?.status(), 200);
      await page.getByRole("heading", { name: "Password link already used" })
        .waitFor();
      await page.getByText(
        "This password reset request has already been completed.",
      )
        .waitFor();

      const after = await admin.authUserIdentitiesList({
        userId: user.user.userId,
        limit: 500,
      }).orThrow();
      assertEquals(after.entries, before.entries);
      assertEquals(
        (await fetchJson(
          `${runtime.trellisUrl}/auth/account-flow/${
            encodeURIComponent(reset.flowId)
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
      const { flowId } = await fixture.setupActivationRequest(
        runtime,
        identity,
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
      await page.getByRole("heading", { name: "Sign in to continue" })
        .waitFor();
      await completeDeviceActivationPortalSignIn({
        page,
        username: liveDeviceActivationUsername,
        password: liveDeviceActivationPassword,
      });
      await waitForHeading(page, "Approve this device");
      await page.getByRole("button", { name: "Approve device" }).click();
      await page.getByRole("heading", { name: "Device approved" }).waitFor();
      await page.getByText("Approval complete.").waitFor();

      await waitForDeviceActivation({
        trellisUrl: runtime.trellisUrl,
        publicIdentityKey: identity.publicIdentityKey,
        identitySeed: identity.identitySeed,
        deploymentId,
        instanceId: provisioned.instance.instanceId,
        principalId: provisioned.instance.principalId,
        participantId: fixture.deviceContract.CONTRACT.id,
        participantArtifactDigest: fixture.deviceContract.CONTRACT_DIGEST,
        participantNeedsDigest: fixture.deviceContract.CONTRACT_DIGEST,
        pollIntervalMs: 25,
      });
      await runtime.waitFor(async () => {
        const activations = requireDeviceAuthorityList(
          await admin.authDeviceUserAuthoritiesList({
            deploymentId,
            instanceId: provisioned.instance.instanceId,
            state: "activated",
            limit: 20,
          }).orThrow(),
        );
        return activations.entries.find((entry) =>
          entry.instanceId === provisioned.instance.instanceId &&
          entry.publicIdentityKey === identity.publicIdentityKey &&
          entry.deploymentId === deploymentId &&
          entry.state === "activated"
        );
      });

      const device = await TrellisDevice.connect({
        trellisUrl: runtime.trellisUrl,
        contract: fixture.deviceContract,
        rootSecret,
        log: false,
      }).orThrow();
      try {
        const me = await device.authSessionsMe({}).orThrow();
        assertEquals(me.participantKind, "device");
        assertEquals(me.device?.deploymentId, deploymentId);
        assertEquals(me.device?.runtimePublicKey, identity.publicIdentityKey);
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
  "browser.login-portal live device activation review required stays pending",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveDeviceActivationPendingFixture;
    const { admin, deploymentId } = await fixture.setupDeviceDeployment(
      runtime,
      { reviewMode: "required" },
    );
    const loginAdmin = await liveDeviceActivationPendingLoginFixture
      .setupSessionAdmin(runtime);

    try {
      const { identity, rootSecret, provisioned } = await fixture
        .setupProvisionedDevice(admin, deploymentId);
      const { flowId } = await fixture.setupActivationRequest(
        runtime,
        identity,
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
      await page.getByRole("button", { name: "Approve device" }).click();
      await waitForDeviceActivationReview({
        admin,
        runtime,
        deploymentId,
        instanceId: provisioned.instance.instanceId,
        publicIdentityKey: identity.publicIdentityKey,
      });
      await waitForHeading(page, "Approval pending");
      await page.getByText(
        "Approval has been requested and is waiting for review.",
      )
        .waitFor();
      await assertNoActivatedDeviceAuthority({
        admin,
        deploymentId,
        instanceId: provisioned.instance.instanceId,
        publicIdentityKey: identity.publicIdentityKey,
      });
      await assertDeviceConnectRejected({
        runtime,
        identity,
        rootSecret,
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
      { reviewMode: "required" },
    );
    const loginAdmin = await liveDeviceActivationRejectedLoginFixture
      .setupSessionAdmin(runtime);
    const rejectionReason = "browser review rejected";

    try {
      const { identity, rootSecret, provisioned } = await fixture
        .setupProvisionedDevice(admin, deploymentId);
      const { flowId } = await fixture.setupActivationRequest(
        runtime,
        identity,
      );
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
      await page.getByRole("button", { name: "Approve device" }).click();

      const review = await waitForDeviceActivationReview({
        admin,
        runtime,
        deploymentId,
        instanceId: provisioned.instance.instanceId,
        publicIdentityKey: identity.publicIdentityKey,
      });
      const decided = await admin.authDeviceUserAuthoritiesReviewsDecide({
        reviewId: review.reviewId,
        decision: "reject",
        reason: rejectionReason,
      }).orThrow();
      assertEquals(decided.review.state, "rejected");
      await page.getByRole("heading", { name: "Request denied" }).waitFor();
      await page.getByText(rejectionReason).waitFor();

      await assertNoActivatedDeviceAuthority({
        admin,
        deploymentId,
        instanceId: provisioned.instance.instanceId,
        publicIdentityKey: identity.publicIdentityKey,
      });
      await assertRejects(
        () =>
          waitForDeviceActivation({
            trellisUrl: runtime.trellisUrl,
            flowId,
            publicIdentityKey: identity.publicIdentityKey,
            identitySeed: identity.identitySeed,
            deploymentId,
            instanceId: provisioned.instance.instanceId,
            principalId: provisioned.instance.principalId,
            participantId: fixture.deviceContract.CONTRACT.id,
            participantArtifactDigest: fixture.deviceContract.CONTRACT_DIGEST,
            participantNeedsDigest: fixture.deviceContract.CONTRACT_DIGEST,
            pollIntervalMs: 25,
          }),
        Error,
        `device activation rejected: ${rejectionReason}`,
      );
      await assertDeviceConnectRejected({
        runtime,
        identity,
        rootSecret,
      });
    } finally {
      await loginAdmin.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
    }
  },
);

withLivePortalPage(
  "browser.login-portal live invalid device activation shows error",
  async ({ page, portalOrigin, runtime }) => {
    const fixture = liveInvalidDeviceActivationFixture;
    const { admin, deploymentId } = await fixture.setupDeviceDeployment(
      runtime,
    );
    const loginAdmin = await liveInvalidDeviceActivationLoginFixture
      .setupSessionAdmin(runtime);

    try {
      const { identity, provisioned } = await fixture.setupProvisionedDevice(
        admin,
        deploymentId,
      );
      await setupDeviceActivationPortalUser({
        admin: loginAdmin,
        runtime,
        portalOrigin,
        portalId: liveInvalidDeviceActivationPortalId,
        username: liveInvalidDeviceActivationUsername,
        password: liveInvalidDeviceActivationPassword,
        name: "Browser Invalid Device Activation User",
      });

      const response = await page.goto(
        deviceActivationPortalUrl(
          portalOrigin,
          caseScopedName(
            "missing-device-flow",
            liveInvalidDeviceActivationCaseId,
          ),
        ),
        { waitUntil: "networkidle" },
      );
      assertEquals(response?.status(), 200);
      await page.getByRole("heading", { name: "Sign in to continue" })
        .waitFor();
      await completeDeviceActivationPortalSignIn({
        page,
        username: liveInvalidDeviceActivationUsername,
        password: liveInvalidDeviceActivationPassword,
      });
      await waitForHeading(page, "Approve this device");
      await page.getByRole("button", { name: "Approve device" }).click();
      await page.getByRole("heading", { name: "Invalid link" }).waitFor();
      await page.getByText("This activation link is no longer valid.")
        .waitFor();

      const activations = requireDeviceAuthorityList(
        await admin.authDeviceUserAuthoritiesList({
          deploymentId,
          instanceId: provisioned.instance.instanceId,
          state: "activated",
          limit: 20,
        }).orThrow(),
      );
      assertEquals(
        activations.entries.filter((entry) =>
          entry.instanceId === provisioned.instance.instanceId &&
          entry.publicIdentityKey === identity.publicIdentityKey &&
          entry.deploymentId === deploymentId
        ),
        [],
      );
    } finally {
      await loginAdmin.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
    }
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
    await withTrellisRuntime(async (runtime) => {
      let server: ReturnType<typeof Deno.serve> | undefined;

      try {
        server = Deno.serve(
          { hostname: "127.0.0.1", port: 0, onListen() {} },
          (request) => serveStatic(request, buildDir, runtime.trellisUrl),
        );
        const portalOrigin = `http://127.0.0.1:${server.addr.port}`;
        await withCoveredPage(name, async ({ page }) => {
          await fn({ page, portalOrigin, runtime });
        });
      } finally {
        await server?.shutdown();
      }
    });
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
    return await fetch(
      new Request(new URL(url.pathname + url.search, runtimeUrl), request),
    );
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
    username: args.username,
    name: args.name,
    email: `${args.username}@example.test`,
    active: true,
    capabilities: args.capabilities ?? [],
    capabilityGroups: [],
  }).orThrow();
  const reset = await args.admin.authUsersPasswordResetCreate({
    userId: user.user.userId,
  }).orThrow();
  await completeLocalPasswordAccountFlow({
    trellisUrl: args.runtime.trellisUrl,
    flowId: reset.flowId,
    username: args.username,
    password: args.password,
  });
  return user;
}

function requireControlPlaneSqlite(
  runtime: LiveTrellisRuntime,
): ControlPlaneSqlite {
  const sqlite = runtime.controlPlane?.sqlite;
  assert(sqlite, "live runtime must expose control-plane SQLite");
  return sqlite;
}

async function putIdentityLinkFlow(args: {
  sqlite: ControlPlaneSqlite;
  caseId: string;
  targetUserId: string;
}): Promise<string> {
  const flowId = caseScopedName("identity-link-flow", args.caseId);
  const now = new Date().toISOString();
  await args.sqlite.execute(
    `INSERT INTO account_flows
      (id, flow_id_hash, kind, target_user_id, target_identity_id, target_local_username, created_by_user_id, allowed_providers, capabilities, profile_hint, return_to, created_at, expires_at, consumed_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    [
      caseScopedName("account-flow", args.caseId),
      await hashKey(flowId),
      "identity_link",
      args.targetUserId,
      null,
      null,
      args.targetUserId,
      JSON.stringify(["local"]),
      null,
      null,
      null,
      now,
      new Date(Date.now() + 300_000).toISOString(),
      null,
    ],
  );
  return flowId;
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

function deviceActivationPortalUrl(
  portalOrigin: string,
  flowId: string,
): string {
  const url = new URL("/_trellis/portal/devices/activate", portalOrigin);
  url.searchParams.set("flowId", flowId);
  return url.toString();
}

async function hashKey(value: string): Promise<string> {
  return base64urlEncode(await sha256(utf8(value)));
}

async function completeLocalPasswordAccountFlow(args: {
  trellisUrl: string;
  flowId: string;
  username: string;
  password: string;
}): Promise<void> {
  const response = await fetch(
    `${args.trellisUrl}/auth/account-flow/${
      encodeURIComponent(args.flowId)
    }/local-password`,
    {
      method: "POST",
      headers: { "content-type": "application/json" },
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
  useGrantOverride?: boolean;
}): Promise<void> {
  const user = await args.admin.authUsersCreate({
    username: args.username,
    name: args.name,
    email: `${args.username}@example.test`,
    active: true,
    capabilities: [args.fixture.pingCapability],
    capabilityGroups: ["admin"],
  }).orThrow();
  const reset = await args.admin.authUsersPasswordResetCreate({
    userId: user.user.userId,
  }).orThrow();
  await completeLocalPasswordAccountFlow({
    trellisUrl: args.runtime.trellisUrl,
    flowId: reset.flowId,
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
    username: args.username,
    name: args.name,
    email: `${args.username}@example.test`,
    active: true,
    capabilities: [],
    capabilityGroups: ["admin"],
  }).orThrow();
  const reset = await args.admin.authUsersPasswordResetCreate({
    userId: user.user.userId,
  }).orThrow();
  await completeLocalPasswordAccountFlow({
    trellisUrl: args.runtime.trellisUrl,
    flowId: reset.flowId,
    username: args.username,
    password: args.password,
  });
  await args.admin.authPortalsPut({
    portalId: args.portalId,
    displayName: "Browser Device Activation Portal",
    entryUrl: `${args.portalOrigin}/_trellis/portal/users/login`,
  }).orThrow();
  await args.admin.authPortalsRoutesPut({
    portalId: args.portalId,
    contractId: deviceActivationPortalContractId,
    origin: args.portalOrigin,
  }).orThrow();
}

async function configureLocalLoginPortal(args: {
  admin: SessionAdminClient;
  fixture: AuthLocalLoginFixture;
  portalOrigin: string;
  portalId: string;
  useGrantOverride?: boolean;
}): Promise<void> {
  await args.admin.authPortalsPut({
    portalId: args.portalId,
    displayName: "Browser Login Portal",
    entryUrl: `${args.portalOrigin}/_trellis/portal/users/login`,
  }).orThrow();
  await args.admin.authPortalsRoutesPut({
    portalId: args.portalId,
    contractId: args.fixture.clientContract.CONTRACT.id,
    origin: args.portalOrigin,
  }).orThrow();
  if (args.useGrantOverride === false) return;
  await args.admin.authDeploymentAuthorityGrantOverridesPut({
    deploymentId: args.fixture.deploymentId,
    overrides: [{
      deploymentId: args.fixture.deploymentId,
      identityKind: "web",
      grantKind: "capability",
      contractId: args.fixture.clientContract.CONTRACT.id,
      origin: args.portalOrigin,
      sessionPublicKey: null,
      capability: args.fixture.pingCapability,
      capabilityGroupKey: null,
    }],
  }).orThrow();
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
      if (state.status === "approval_required") {
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
        assertEquals(approved.status, "redirect");
      } else {
        assertEquals(state.status, "redirect");
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
  if (await approve.isVisible({ timeout: 10_000 }).catch(() => false)) {
    await approve.click();
  }
}

async function waitForDeviceActivationReview(args: {
  admin: DeviceActivationAdmin;
  runtime: LiveTrellisRuntime;
  deploymentId: string;
  instanceId: string;
  publicIdentityKey: string;
}): Promise<{ readonly reviewId: string }> {
  return await args.runtime.waitFor(async () => {
    const reviews = await args.admin.authDeviceUserAuthoritiesReviewsList({
      deploymentId: args.deploymentId,
      instanceId: args.instanceId,
      state: "pending",
      limit: 20,
    }).orThrow();
    return reviews.entries.find((entry) =>
      entry.deploymentId === args.deploymentId &&
      entry.instanceId === args.instanceId &&
      entry.publicIdentityKey === args.publicIdentityKey
    );
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
      instanceId: args.instanceId,
      state: "activated",
      limit: 20,
    }).orThrow(),
  );
  assertEquals(
    activations.entries.filter((entry) =>
      entry.instanceId === args.instanceId &&
      entry.publicIdentityKey === args.publicIdentityKey &&
      entry.deploymentId === args.deploymentId
    ),
    [],
  );
}

async function assertDeviceConnectRejected(args: {
  runtime: LiveTrellisRuntime;
  identity: DeviceActivationIdentity;
  rootSecret: Uint8Array;
}): Promise<void> {
  const connect = await TrellisDevice.connect({
    trellisUrl: args.runtime.trellisUrl,
    contract: args.fixture.deviceContract,
    rootSecret: args.rootSecret,
    log: false,
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
      `Expected heading "${name}". Browser body:\n${await page.locator("body")
        .innerText()}`,
      { cause: error },
    );
  }
}

async function completeLocalLoginByFetch(args: {
  trellisUrl: string;
  flowId: string;
  username: string;
  password: string;
}): Promise<void> {
  const loginResponse = await fetch(`${args.trellisUrl}/auth/login/local`, {
    method: "POST",
    headers: { "content-type": "application/json" },
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
  );
  if (state.status === "approval_required") {
    const approved = await fetchJson(
      `${args.trellisUrl}/auth/flow/${
        encodeURIComponent(args.flowId)
      }/approval`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ approved: true }),
      },
    );
    assertEquals(approved.status, "redirect");
  } else {
    assertEquals(state.status, "redirect");
  }
}

async function appSessionsFor(
  admin: SessionAdminClient,
  sessionKey: string,
): Promise<AppSession[]> {
  const sessions = await admin.authSessionsList({ limit: 500 }).orThrow();
  return sessions.entries.filter((entry): entry is AppSession =>
    entry.participantKind === "app" && entry.sessionKey === sessionKey
  );
}

async function assertNoAppSessionOrConnection(
  admin: SessionAdminClient,
  sessionKey: string,
): Promise<void> {
  assertEquals((await appSessionsFor(admin, sessionKey)).length, 0);
  const connections = await admin.authConnectionsList({
    sessionKey,
    limit: 500,
  }).orThrow();
  assertEquals(connections.entries.length, 0);
}

async function assertNoApprovedGrantForContract(args: {
  sqlite: ControlPlaneSqlite;
  contractId: string;
}): Promise<void> {
  const rows = await args.sqlite.query(
    "SELECT COUNT(*) AS approvedGrantCount FROM identity_grants WHERE contract_id = ? AND answer = 'approved'",
    [args.contractId],
  );
  const approvedGrantCount = rows[0]?.approvedGrantCount;
  assertEquals(approvedGrantCount, 0);
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
    sessionKey,
    limit: 500,
  }).orThrow();
  const appConnections = connections.entries.filter((
    entry,
  ): entry is AppConnection => entry.participantKind === "app");
  assertEquals(appConnections.length, 1);
  const [connection] = appConnections;
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
