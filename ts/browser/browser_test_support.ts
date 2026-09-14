import { type BrowserContext, chromium, type Page } from "playwright";
import { join } from "@std/path";
import type { TrellisTestRuntime } from "@qlever-llc/trellis-test";
import type { TrellisTestRuntimeStartOptions } from "@qlever-llc/trellis-test";

/** Local administrator credentials created through the browser bootstrap page. */
export const BROWSER_ADMIN = {
  username: "browser-admin",
  password: "browser-admin-password",
};

/** Returns the prebuilt server binary required by browser acceptance tests. */
export function prebuiltServer(): string {
  const server = Deno.env.get("TRELLIS_TEST_SERVER_BIN");
  if (server === undefined) {
    throw new Error(
      "TRELLIS_TEST_SERVER_BIN must point to a prebuilt trellis-server",
    );
  }
  return server;
}

/** Runtime options that run the prebuilt control-plane binary. */
export function browserRuntimeOptions(
  options: Partial<TrellisTestRuntimeStartOptions> = {},
): Partial<TrellisTestRuntimeStartOptions> {
  return {
    ...options,
    trellis: {
      command: {
        cmd: prebuiltServer(),
        args: ["--config", "{config}", "all"],
      },
    },
  };
}

/** Launches a persistent Chromium profile owned by the test runtime workdir. */
export async function launchProfile(
  runtime: TrellisTestRuntime,
): Promise<BrowserContext> {
  return await chromium.launchPersistentContext(
    join(runtime.workdir, "browser-profile"),
    { headless: true },
  );
}

async function visibleWithin(
  locator: ReturnType<Page["getByLabel"]>,
  timeoutMs: number,
): Promise<boolean> {
  return await locator
    .first()
    .waitFor({ state: "visible", timeout: timeoutMs })
    .then(() => true)
    .catch(() => false);
}

/** Clicks the portal consent approval button when the flow asks for approval. */
export async function approveConsentIfRequired(
  page: Page,
  timeoutMs = 5_000,
): Promise<void> {
  const approve = page.getByRole("button", { name: "Approve" });
  if (await visibleWithin(approve, timeoutMs)) {
    await approve.click();
  }
}

/** Completes the local sign-in form when the portal displays it. */
export async function signInIfPrompted(
  page: Page,
  credentials: { username: string; password: string },
  timeoutMs = 20_000,
): Promise<void> {
  const username = page.getByLabel("Username", { exact: true });
  if (await visibleWithin(username, timeoutMs)) {
    await username.fill(credentials.username);
    await page.getByLabel("Password", { exact: true }).fill(
      credentials.password,
    );
    await page.getByRole("button", { name: "Sign in" }).click();
  }
  await approveConsentIfRequired(page);
}

/** Creates the first administrator through the real portal bootstrap page. */
export async function completeAdminBootstrapInBrowser(
  page: Page,
  runtime: TrellisTestRuntime,
  credentials: { username: string; password: string } = BROWSER_ADMIN,
): Promise<void> {
  await page.goto(await runtime.bootstrapUrl(), {
    waitUntil: "domcontentloaded",
  });
  await page.getByLabel("Username", { exact: true }).fill(credentials.username);
  await page.getByLabel("Password", { exact: true }).fill(credentials.password);
  await page.getByLabel("Confirm password").fill(credentials.password);
  await page.getByRole("button", { name: "Create administrator" }).click();
  await page.waitForURL(
    (url) => !url.pathname.startsWith("/login/admin/bootstrap"),
    { timeout: 60_000 },
  );
}

/** Completes console entry whether the portal prompts for sign-in, consent, or attaches directly. */
export async function completeConsoleEntry(
  page: Page,
  credentials: { username: string; password: string },
): Promise<void> {
  const ready = waitForConsoleReady(page).then(() => "ready" as const).catch(
    () => "waiting" as const,
  );
  const form = page
    .getByLabel("Username", { exact: true })
    .first()
    .waitFor({ state: "visible", timeout: 60_000 })
    .then(() => "form" as const);
  const approval = page
    .getByRole("button", { name: "Approve" })
    .waitFor({ state: "visible", timeout: 60_000 })
    .then(() => "approval" as const);

  const outcome = await Promise.race([ready, form, approval]);
  if (outcome === "form") {
    await page.getByLabel("Username", { exact: true }).fill(
      credentials.username,
    );
    await page.getByLabel("Password", { exact: true }).fill(
      credentials.password,
    );
    await page.getByRole("button", { name: "Sign in" }).click();
  }
  await approveConsentIfRequired(page);
  await waitForConsoleReady(page);
}

/** Opens the console and completes any sign-in or consent it prompts for. */
export async function openConsole(
  page: Page,
  runtime: TrellisTestRuntime,
  credentials: { username: string; password: string },
): Promise<void> {
  await page.goto(`${runtime.trellisUrl}/console`, {
    waitUntil: "domcontentloaded",
  });
  await completeConsoleEntry(page, credentials);
}

/** Waits for the authenticated console shell with its authorized navigation. */
export async function waitForConsoleReady(
  page: Page,
  timeoutMs = 60_000,
): Promise<void> {
  await page
    .getByText("Connected", { exact: true })
    .first()
    .waitFor({ state: "visible", timeout: timeoutMs });
  await page
    .getByRole("link", { name: "Overview" })
    .waitFor({ state: "visible", timeout: timeoutMs });
}
