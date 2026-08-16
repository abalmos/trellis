import { assertEquals } from "@std/assert";
import { resolve } from "node:path";
import { type Browser, chromium } from "playwright";

Deno.test("a real browser imports and defines a native contract synchronously", async () => {
  const build = new Deno.Command(Deno.execPath(), {
    args: [
      "run",
      "-A",
      "vite",
      "build",
      "browser/define_contract_fixture",
    ],
    stdout: "piped",
    stderr: "piped",
  });
  const built = await build.output();
  if (!built.success) throw new Error(new TextDecoder().decode(built.stderr));

  const root = resolve("browser/define_contract_fixture/dist");
  const server = Deno.serve({ port: 0 }, async (request) => {
    const pathname = new URL(request.url).pathname;
    const path = resolve(
      root,
      pathname === "/" ? "index.html" : `.${pathname}`,
    );
    if (!path.startsWith(root)) return new Response(null, { status: 404 });
    try {
      return new Response(await Deno.readFile(path), {
        headers: {
          "content-type": path.endsWith(".js")
            ? "text/javascript"
            : "text/html",
        },
      });
    } catch {
      return new Response(null, { status: 404 });
    }
  });
  let browser: Browser | undefined;
  try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.addr.port}`);
    const identity = await page.waitForFunction(() =>
      Reflect.get(globalThis, "contractIdentity")
    )
      .then((handle) => handle.jsonValue() as Promise<Record<string, string>>);
    assertEquals(identity.id, identity.participantId);
    assertEquals(identity.digest.length > 0, true);
    assertEquals(identity.needsDigest.length > 0, true);
  } finally {
    await browser?.close();
    await server.shutdown();
    await Deno.remove(root, { recursive: true });
  }
});
