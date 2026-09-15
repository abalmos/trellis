import { assert, assertEquals } from "@std/assert";

import { withTrellisRuntime } from "../integration/_support/runtime.ts";
import {
  approveConsentIfRequired,
  browserRuntimeOptions,
  launchProfile,
  signInIfPrompted,
} from "./browser_test_support.ts";

function cliBinary(): string {
  const binary = Deno.env.get("TRELLIS_TEST_CLI_BIN") ??
    "rust/target/debug/trellis";
  try {
    Deno.statSync(binary);
  } catch {
    throw new Error(
      `the trellis CLI binary is missing at ${binary}; build it or set TRELLIS_TEST_CLI_BIN`,
    );
  }
  return binary;
}

type RuntimeLike = Parameters<typeof launchProfile>[0];

async function loginOnce(
  runtime: RuntimeLike,
  configHome: string,
): Promise<{ sessionKey: string; userId: string }> {
  const child = new Deno.Command(cliBinary(), {
    args: ["--format", "json", "login", runtime.trellisUrl],
    env: { XDG_CONFIG_HOME: configHome },
    stdout: "piped",
    stderr: "piped",
  }).spawn();

  let progress = "";
  const progressReader = child.stderr.pipeThrough(new TextDecoderStream())
    .getReader();
  while (!progress.includes("loginUrl")) {
    const chunk = await progressReader.read();
    assert(
      !chunk.done,
      `CLI exited before printing a login URL: ${progress}`,
    );
    progress += chunk.value;
  }
  const loginUrl = JSON.parse(
    progress.trim().split("\n").find((line) => line.includes("loginUrl"))!,
  ).loginUrl as string;
  assert(loginUrl.startsWith(runtime.trellisUrl));

  const context = await launchProfile(runtime);
  try {
    const page = await context.newPage();
    const pageErrors: string[] = [];
    page.on("pageerror", (error) => pageErrors.push(String(error)));
    await page.goto(loginUrl, { waitUntil: "domcontentloaded" });
    await signInIfPrompted(page, {
      username: runtime.adminUsername,
      password: runtime.adminPassword,
    });
    await approveConsentIfRequired(page, 15_000);
    assertEquals(pageErrors, []);
  } finally {
    await context.close();
  }

  while (true) {
    const chunk = await progressReader.read();
    if (chunk.done) break;
    progress += chunk.value;
  }
  const status = await child.status;
  const stdout = new TextDecoder().decode(
    await new Response(child.stdout).arrayBuffer(),
  );
  assertEquals(status.code, 0, progress);
  const result = JSON.parse(
    stdout.slice(stdout.indexOf("{"), stdout.lastIndexOf("}") + 1),
  );
  assert(result.sessionKey, stdout);
  assert(result.userId, stdout);
  return {
    sessionKey: result.sessionKey as string,
    userId: result.userId as string,
  };
}

Deno.test("CLI agent logins reuse one durable login with distinct runtime keys", async () => {
  await withTrellisRuntime(async (runtime) => {
    await runtime.ensureAdmin();
    const configHome = await Deno.makeTempDir({ prefix: "trellis-cli-login-" });
    const first = await loginOnce(runtime, configHome);
    const second = await loginOnce(runtime, configHome);
    assertEquals(second.userId, first.userId);
    assert(
      second.sessionKey !== first.sessionKey,
      "each login must issue its own runtime session key",
    );
  }, browserRuntimeOptions());
});
