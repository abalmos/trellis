import { assert, assertEquals, assertStringIncludes } from "@std/assert";
import { base64urlEncode, sha256, utf8 } from "@qlever-llc/trellis/auth";
import { caseScopedName } from "../_support/names.ts";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID = "auth.account-flow-oauth-callback-runtime" as const;
const providerId = "fake_oidc";
const otherProviderId = "other_fake_oidc";
const fixture = createAuthLocalLoginFixture(CASE_ID);
const oidcPort = reserveLocalPort();
const oidcIssuer = `http://127.0.0.1:${oidcPort}`;

liveTrellisTest({
  name:
    "auth.account-flow-oauth-callback-runtime handles provider errors, provider mismatch, and link success",
  scope: runtimeScopeForCase(CASE_ID),
  runtime: {
    oauthProviders: {
      [providerId]: {
        type: "oidc",
        issuer: oidcIssuer,
        clientId: "client-id",
        clientSecret: "client-secret",
        displayName: "Fake OIDC",
      },
      [otherProviderId]: {
        type: "oidc",
        issuer: oidcIssuer,
        clientId: "other-client-id",
        clientSecret: "other-client-secret",
        displayName: "Other Fake OIDC",
      },
    },
  },
  async fn(runtime) {
    const oidcProvider = startOidcProvider(oidcPort, oidcIssuer);
    const sqlite = requireControlPlaneSqlite(runtime);
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);

    try {
      const target = await admin.rpc.auth.usersCreate({
        username: caseScopedName("oauth-target", CASE_ID),
        name: "OAuth Target",
        email: "oauth-target@example.test",
        active: true,
      }).orThrow();

      const errorFlowId = await putIdentityLinkFlow(
        sqlite,
        target.user.userId,
        "error",
      );
      const errorStart = await startAccountFlowProviderLogin(
        runtime.trellisUrl,
        errorFlowId,
        providerId,
      );
      const errorCallback = await fetch(
        errorStart.callbackUrl({
          error: "invalid_request",
          error_description: "account selection failed",
        }),
        { headers: { cookie: errorStart.cookie } },
      );
      assertEquals(errorCallback.status, 400);
      assertStringIncludes(
        await errorCallback.text(),
        "Identity provider rejected sign-in: account selection failed",
      );

      const mismatchFlowId = await putIdentityLinkFlow(
        sqlite,
        target.user.userId,
        "mismatch",
      );
      const mismatchStart = await startAccountFlowProviderLogin(
        runtime.trellisUrl,
        mismatchFlowId,
        providerId,
      );
      const mismatchUrl = new URL(mismatchStart.callbackUrl({ code: "code" }));
      mismatchUrl.pathname = `/auth/callback/${otherProviderId}`;
      const mismatch = await fetch(mismatchUrl, {
        headers: { cookie: mismatchStart.cookie },
      });
      assertEquals(mismatch.status, 400);
      assertStringIncludes(await mismatch.text(), "OAuth provider mismatch");

      const successFlowId = await putIdentityLinkFlow(
        sqlite,
        target.user.userId,
        "success",
      );
      const successStart = await startAccountFlowProviderLogin(
        runtime.trellisUrl,
        successFlowId,
        providerId,
      );
      const success = await fetch(successStart.callbackUrl({ code: "code" }), {
        headers: { cookie: successStart.cookie },
        redirect: "manual",
      });
      assertEquals(success.status, 302);
      const location = success.headers.get("location") ?? "";
      assertStringIncludes(location, "status=completed");
      assertStringIncludes(location, `userId=${target.user.userId}`);

      const identities = await admin.rpc.auth.userIdentitiesList({
        userId: target.user.userId,
        limit: 500,
      }).orThrow();
      assert(
        identities.entries.some((identity) =>
          identity.provider === providerId && identity.subject === "oidc-user"
        ),
        "expected OAuth identity to be linked to target user",
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
      await oidcProvider.shutdown();
    }
  },
});

async function putIdentityLinkFlow(
  sqlite: NonNullable<LiveTrellisRuntime["controlPlane"]>["sqlite"],
  targetUserId: string,
  suffix: string,
): Promise<string> {
  const flowId = caseScopedName(`oauth-flow-${suffix}`, CASE_ID);
  const now = new Date().toISOString();
  await sqlite.execute(
    `INSERT INTO account_flows
      (id, flow_id_hash, kind, target_user_id, target_identity_id, target_local_username, created_by_user_id, allowed_providers, capabilities, profile_hint, return_to, created_at, expires_at, consumed_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    [
      `acf_${suffix}`,
      await hashKey(flowId),
      "identity_link",
      targetUserId,
      null,
      null,
      targetUserId,
      JSON.stringify([providerId]),
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

async function startAccountFlowProviderLogin(
  trellisUrl: string,
  flowId: string,
  provider: string,
): Promise<{
  cookie: string;
  callbackUrl: (
    args: { code?: string; error?: string; error_description?: string },
  ) => string;
}> {
  const response = await fetch(
    new URL(`/auth/account-flow/${flowId}/login/${provider}`, trellisUrl),
    { redirect: "manual" },
  );
  assertEquals(response.status, 302, await response.text());
  const setCookie = response.headers.get("set-cookie") ?? "";
  const cookie = setCookie.split(";")[0] ?? "";
  const state = new URL(response.headers.get("location") ?? "").searchParams
    .get("state");
  assert(state, "provider redirect should include oauth state");
  return {
    cookie,
    callbackUrl: (args) => {
      const url = new URL(`/auth/callback/${provider}`, trellisUrl);
      url.searchParams.set("state", state);
      if (args.code) url.searchParams.set("code", args.code);
      if (args.error) url.searchParams.set("error", args.error);
      if (args.error_description) {
        url.searchParams.set("error_description", args.error_description);
      }
      return url.toString();
    },
  };
}

function startOidcProvider(port: number, issuer: string): Deno.HttpServer {
  return Deno.serve(
    { hostname: "127.0.0.1", port, onListen: () => {} },
    (request: Request) => {
      const url = new URL(request.url);
      if (url.pathname === "/.well-known/openid-configuration") {
        return json({
          issuer,
          authorization_endpoint: `${issuer}/authorize`,
          token_endpoint: `${issuer}/token`,
          userinfo_endpoint: `${issuer}/userinfo`,
        });
      }
      if (url.pathname === "/token") {
        return json({
          access_token: "access-token",
          token_type: "Bearer",
          expires_in: 300,
        });
      }
      if (url.pathname === "/userinfo") {
        return json({
          sub: "oidc-user",
          name: "OIDC User",
          email: "oidc-user@example.test",
          email_verified: true,
        });
      }
      return new Response(null, { status: 404 });
    },
  );
}

function json(value: unknown): Response {
  return new Response(JSON.stringify(value), {
    headers: { "content-type": "application/json" },
  });
}

async function hashKey(value: string): Promise<string> {
  return base64urlEncode(await sha256(utf8(value)));
}

function requireControlPlaneSqlite(
  runtime: LiveTrellisRuntime,
): NonNullable<LiveTrellisRuntime["controlPlane"]>["sqlite"] {
  const sqlite = runtime.controlPlane?.sqlite;
  assert(sqlite, "live runtime must expose control-plane SQLite");
  return sqlite;
}

function reserveLocalPort(): number {
  const listener = Deno.listen({ hostname: "127.0.0.1", port: 0 });
  const port = listener.addr.port;
  listener.close();
  return port;
}
