import { assert } from "@std/assert";
import { fromFileUrl, join } from "@std/path";

import { participants } from "../../integration/fixtures/runtime/packages/runtime-trellis/index.js";
import { withTrellisRuntime } from "./_support/runtime.ts";

function rustFixtureCommand(bin: string): string[] {
  return [
    "cargo",
    "run",
    "--config",
    `patch.crates-io.trellis-rs.path=${
      JSON.stringify(
        fromFileUrl(new URL("../../rust/crates/trellis", import.meta.url)),
      )
    }`,
    "--bin",
    bin,
    "--manifest-path",
    fromFileUrl(
      new URL("../../integration/fixtures/runtime/Cargo.toml", import.meta.url),
    ),
  ];
}

async function completeRustLogin(
  runtime: {
    trellisUrl: string;
    workdir: string;
    completeClientAuth: (opts: {
      loginUrl: string;
      sessionKey: string;
      mode: "session_key";
    }) => Promise<unknown>;
  },
  args: string[],
  configDir: string,
  loginMarker: string,
  doneMarker: string,
): Promise<void> {
  const child = new Deno.Command("cargo", {
    args,
    env: {
      TRELLIS_URL: runtime.trellisUrl,
      XDG_CONFIG_HOME: join(runtime.workdir, configDir),
      CARGO_TARGET_DIR: fromFileUrl(
        new URL("../../rust/target", import.meta.url),
      ),
    },
    stdout: "piped",
    stderr: "inherit",
  }).spawn();
  const reader = child.stdout.pipeThrough(new TextDecoderStream()).getReader();
  let output = "";
  while (!output.includes(loginMarker)) {
    const chunk = await reader.read();
    assert(!chunk.done, output);
    output += chunk.value;
  }
  const loginUrl = output.split(loginMarker)[1]?.trim().split(/\s/)[0];
  assert(loginUrl, output);
  await runtime.completeClientAuth({
    loginUrl,
    sessionKey: "completed-by-rust",
    mode: "session_key",
  });
  while (!output.includes(doneMarker)) {
    const chunk = await reader.read();
    assert(!chunk.done, output);
    output += chunk.value;
  }
  assert((await child.status).success, output);
}

Deno.test("NX01 rust caller receives Watch frames from rust provider", async () => {
  await withTrellisRuntime(async (runtime) => {
    const identity = await runtime.registerService({
      name: "rust",
      contract: participants.OperationProvider.participant,
    });
    const process = new Deno.Command("setsid", {
      args: rustFixtureCommand("trellis-runtime-acceptance"),
      env: {
        TRELLIS_URL: runtime.trellisUrl,
        TRELLIS_IDENTITY_SEED: identity.seed,
        CARGO_TARGET_DIR: fromFileUrl(
          new URL("../../rust/target", import.meta.url),
        ),
      },
      stdout: "inherit",
      stderr: "inherit",
    }).spawn();
    let exited = false;
    const status = process.status.then((value) => {
      exited = true;
      return value;
    });
    try {
      const client = await runtime.connectClient({
        name: "nx01-echo",
        contract: participants.Caller.participant,
      });
      await runtime.waitFor(async () => {
        if (exited) {
          throw new Error(
            `Rust provider exited: ${JSON.stringify(await status)}`,
          );
        }
        const result = await client.echo({ value: "from TypeScript" }, {
          timeout: 1000,
        });
        return result.isOk();
      }, { timeoutMs: 120_000 });

      await completeRustLogin(
        runtime,
        rustFixtureCommand("caller").slice(1),
        "rust-caller-config",
        "rust login ",
        "rust caller complete",
      );
    } finally {
      if (!exited) Deno.kill(-process.pid, "SIGTERM");
      await status;
    }
  });
});

Deno.test("NX03 empty finite Watch completes with no frames", async () => {
  await withTrellisRuntime(async (runtime) => {
    const identity = await runtime.registerService({
      name: "rust",
      contract: participants.OperationProvider.participant,
    });
    const process = new Deno.Command("setsid", {
      args: rustFixtureCommand("trellis-runtime-acceptance"),
      env: {
        TRELLIS_URL: runtime.trellisUrl,
        TRELLIS_IDENTITY_SEED: identity.seed,
        TRELLIS_FEED_EMPTY: "1",
        CARGO_TARGET_DIR: fromFileUrl(
          new URL("../../rust/target", import.meta.url),
        ),
      },
      stdout: "inherit",
      stderr: "inherit",
    }).spawn();
    let exited = false;
    const status = process.status.then((value) => {
      exited = true;
      return value;
    });
    try {
      const client = await runtime.connectClient({
        name: "nx03-echo",
        contract: participants.Caller.participant,
      });
      await runtime.waitFor(async () => {
        if (exited) {
          throw new Error(
            `Rust provider exited: ${JSON.stringify(await status)}`,
          );
        }
        const result = await client.echo({ value: "from TypeScript" }, {
          timeout: 1000,
        });
        return result.isOk();
      }, { timeoutMs: 120_000 });
      await completeRustLogin(
        runtime,
        rustFixtureCommand("empty_watch").slice(1),
        "empty-watch-config",
        "empty login ",
        "empty watch complete",
      );
    } finally {
      if (!exited) Deno.kill(-process.pid, "SIGTERM");
      await status;
    }
  });
});

Deno.test("NX02 rust console client receives Health Watch", async () => {
  await withTrellisRuntime(async (runtime) => {
    await completeRustLogin(
      runtime,
      [
        "run",
        "--manifest-path",
        fromFileUrl(new URL("../../rust/Cargo.toml", import.meta.url)),
        "-p",
        "trellis-runtime",
        "--example",
        "health_watch",
      ],
      "health-caller-config",
      "health login ",
      "health watch complete",
    );
  });
});
