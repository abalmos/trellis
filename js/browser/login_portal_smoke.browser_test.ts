import { assert, assertArrayIncludes, assertEquals } from "@std/assert";
import {
  type ConnectedTrellisClient,
  TrellisClient,
} from "@qlever-llc/trellis";
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

const buildDir = resolve("portals/login/build");
const coverageDir = resolve("coverage/browser");
const liveLocalLoginCaseId = "browser.login-portal-live-local-login";
const liveLocalLoginFixture = createAuthLocalLoginFixture(
  liveLocalLoginCaseId,
);
const liveLocalLoginUsername = caseScopedName(
  "browser-login-portal-user",
  liveLocalLoginCaseId,
);
const liveLocalLoginPassword =
  `trellis-integration-${liveLocalLoginCaseId}-password-2026`;

Deno.test("browser.login-portal smoke renders in Chromium", async () => {
  let browser: Browser | undefined;
  let server: ReturnType<typeof Deno.serve> | undefined;

  try {
    browser = await chromium.launch();
    server = Deno.serve(
      { hostname: "127.0.0.1", port: 0, onListen() {} },
      (request) => serveStatic(request, buildDir),
    );
    const { port } = server.addr;
    const page = await browser.newPage();
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Profiler.enable");
    await cdp.send("Profiler.startPreciseCoverage", {
      callCount: true,
      detailed: true,
    });

    const response = await page.goto(
      `http://127.0.0.1:${port}/_trellis/portal/users/login`,
      { waitUntil: "networkidle" },
    );

    assertEquals(response?.status(), 200);
    await page.locator("body").waitFor({ state: "visible" });
    await page.locator("script").first().waitFor({ state: "attached" });

    const coverage = await cdp.send("Profiler.takePreciseCoverage");
    await Deno.mkdir(coverageDir, { recursive: true });
    await Deno.writeTextFile(
      join(coverageDir, "login-portal-smoke-v8.json"),
      JSON.stringify(coverage, null, 2),
    );
  } finally {
    await browser?.close();
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
      | ConnectedTrellisClient<typeof liveLocalLoginFixture.clientContract>
      | undefined;

    try {
      const user = await admin.rpc.auth.usersCreate({
        username: liveLocalLoginUsername,
        name: "Browser Login Portal User",
        email: `${liveLocalLoginUsername}@example.test`,
        active: true,
        capabilities: [liveLocalLoginFixture.pingCapability],
        capabilityGroups: ["admin"],
      }).orThrow();
      const reset = await admin.rpc.auth.usersPasswordResetCreate({
        userId: user.user.userId,
      }).orThrow();
      await completeLocalPasswordAccountFlow({
        trellisUrl: runtime.trellisUrl,
        flowId: reset.flowId,
        username: liveLocalLoginUsername,
        password: liveLocalLoginPassword,
      });

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
              page.waitForURL(`${portalOrigin}/_trellis/test/client-auth`),
              approve.click(),
            ]);
          } else {
            await page.waitForURL(`${portalOrigin}/_trellis/test/client-auth`);
          }

          return { status: "bound", flowId };
        },
      }).orThrow();

      assert(authRequired, "expected local-login flow to require auth");
      const me = await client.rpc.auth.sessionsMe({}).orThrow();
      assertEquals(me.participantKind, "app");
      assert(me.user !== null, "expected Auth.Sessions.Me to return a user");
      assertEquals(me.user.active, true);
      assertArrayIncludes(me.user.capabilities, ["admin"]);

      const ping = await client.rpc.authLogin.ping({
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
      let browser: Browser | undefined;
      let server: ReturnType<typeof Deno.serve> | undefined;

      try {
        server = Deno.serve(
          { hostname: "127.0.0.1", port: 0, onListen() {} },
          (request) => serveStatic(request, buildDir, runtime.trellisUrl),
        );
        const portalOrigin = `http://127.0.0.1:${server.addr.port}`;
        browser = await chromium.launch();
        const page = await browser.newPage();
        const cdp = await page.context().newCDPSession(page);
        await cdp.send("Profiler.enable");
        await cdp.send("Profiler.startPreciseCoverage", {
          callCount: true,
          detailed: true,
        });

        try {
          await fn({ page, portalOrigin, runtime });
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
        await server?.shutdown();
      }
    });
  });
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
    pathname.startsWith("/auth/flow/") ||
    pathname.startsWith("/auth/login/");
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
