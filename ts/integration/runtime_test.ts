import { Result } from "@qlever-llc/trellis";
import { TransportError } from "@qlever-llc/trellis/errors";
import { RetryJobError } from "@qlever-llc/trellis/jobs";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { assert, assertEquals } from "@std/assert";
import { fromFileUrl } from "@std/path";

import { participant as caller } from "../../integration/fixtures/runtime/.trellis/ts/participants/test-caller/mod.ts";
import { participant as denied } from "../../integration/fixtures/runtime/.trellis/ts/participants/test-denied/mod.ts";
import { participant as provider } from "../../integration/fixtures/runtime/.trellis/ts/participants/test-provider/mod.ts";
import { participant as adminParticipant } from "../../web/.trellis/ts/participants/app-console/mod.ts";
import { withTrellisRuntime } from "./_support/runtime.ts";

Deno.test("generated TypeScript caller reaches Rust provider", async () => {
  await withTrellisRuntime(async (runtime) => {
    const identity = await runtime.registerService({
      name: "rust",
      contract: provider,
    });
    const process = new Deno.Command("cargo", {
      args: [
        "run",
        "--manifest-path",
        fromFileUrl(
          new URL(
            "../../integration/fixtures/runtime/Cargo.toml",
            import.meta.url,
          ),
        ),
      ],
      env: {
        TRELLIS_URL: runtime.trellisUrl,
        TRELLIS_DEPLOYMENT: identity.deploymentId,
        TRELLIS_IDENTITY_SEED: identity.seed,
        TRELLIS_SESSION_SEED: identity.sessionSeed,
        TRELLIS_INSTANCE: identity.instanceId,
        CARGO_TARGET_DIR: fromFileUrl(
          new URL("../../rust/target", import.meta.url),
        ),
      },
      stdout: "inherit",
      stderr: "inherit",
    }).spawn();
    let exited = false;
    const status = process.status.then((status) => {
      exited = true;
      return status;
    });
    try {
      const client = await runtime.connectClient({
        name: "cross-language",
        contract: caller,
      });
      const response = await runtime.waitFor(async () => {
        if (exited) {
          throw new Error(
            `Rust provider exited: ${JSON.stringify(await status)}`,
          );
        }
        const result = await client.echo({ value: "from TypeScript" }, {
          timeout: 1000,
        });
        return result.isOk() ? result.orThrow() : false;
      }, { timeoutMs: 120_000 });
      assertEquals(response.value, "Rust received from TypeScript");
    } finally {
      if (!exited) process.kill("SIGTERM");
      await status;
    }
  });
});

Deno.test("generated runtime workflows", async (t) => {
  await withTrellisRuntime(async (runtime) => {
    const identity = await runtime.registerService({
      name: "provider",
      contract: provider,
    });
    const service = await TrellisService.connect({
      authorizationContextEphemeral: true,
      trellisUrl: runtime.trellisUrl,
      participant: provider,
      name: "provider",
      identity,
      telemetry: false,
      runtime: {},
    }).orThrow();
    let cancelled = false;
    let serviceExit: Promise<unknown> | undefined;
    try {
      await service.handleEcho(({ input }) => Result.ok(input));
      await service.handleWork(async ({ input, op }) => {
        await op.started().orThrow();
        return await op.complete({ value: `completed ${input.value}` })
          .orThrow();
      });
      let attempts = 0;
      service.jobs.work.handle(async ({ job }) =>
        ++attempts === 1
          ? Result.err(new RetryJobError())
          : Result.ok(job.payload)
      );
      await service.handleWatch(async ({ emit, signal }) => {
        await emit({ value: "first frame" }).orThrow();
        await new Promise<void>((resolve) =>
          signal.addEventListener("abort", () => resolve(), { once: true })
        );
        cancelled = true;
      });
      await service.handleUpload(async ({ input, op, transfer }) => {
        await transfer.completed().orThrow();
        return await op.complete(input).orThrow();
      });
      let received: string | undefined;
      await service.onChanged(({ event }) => {
        received = event.value;
        return Result.ok(undefined);
      }).orThrow();
      serviceExit = service.wait().catch((error: unknown) => error);
      const client = await runtime.connectClient({
        name: "caller",
        contract: caller,
      });
      await t.step("operation executes and completes", async () => {
        const operation = await client.work({ value: "work" }).start()
          .orThrow();
        const terminal = await operation.wait().orThrow();
        assertEquals(terminal.state, "completed");
        assertEquals(terminal.output?.value, "completed work");
      });
      await t.step("event reaches generated subscriber", async () => {
        await service.publishChanged({ value: "published" }).orThrow();
        assertEquals(await runtime.waitFor(() => received), "published");
      });
      await t.step("job retries then completes", async () => {
        const job = await service.jobs.work.create({ value: "retried" })
          .orThrow();
        const terminal = await job.wait().orThrow();
        assertEquals(terminal.state, "completed");
        assertEquals(terminal.result, { value: "retried" });
        assertEquals(attempts, 2);
      });
      await t.step("transfer preserves exact bytes", async () => {
        const bytes = Uint8Array.from(
          { length: 131073 },
          (_, index) => index % 251,
        );
        const operation = await client.upload({ value: "bytes" }).transfer(
          bytes,
        ).start().orThrow();
        const terminal = await operation.wait().orThrow();
        assertEquals(terminal.terminal.state, "completed");
        assertEquals(terminal.transferred.size, bytes.length);
        const store = await service.store.files.open().orThrow();
        const entry = await store.get("bytes").orThrow();
        assertEquals(await entry.bytes().orThrow(), bytes);
      });
      await t.step("feed abort cancels provider", async () => {
        const abort = new AbortController();
        try {
          const feed = await client.watch({}, {
            signal: AbortSignal.any([
              abort.signal,
              AbortSignal.timeout(10_000),
            ]),
          })
            .orThrow();
          assertEquals(
            (await feed[Symbol.asyncIterator]().next()).value?.value,
            "first frame",
          );
        } finally {
          abort.abort();
        }
        await runtime.waitFor(() => cancelled);
      });
      await t.step("state survives control-plane restart", async () => {
        await client.state.saved.put({ value: "durable" }).orThrow();
        await runtime.restartControlPlane();
        const stored = await client.state.saved.get().orThrow();
        assert("found" in stored && stored.found);
        assertEquals(stored.entry.value.value, "durable");
      });
      await t.step(
        "authority denial, success, and session revocation",
        async () => {
          const unauthorized = await runtime.connectClient({
            name: "denied",
            contract: denied,
          });
          const rejected = await unauthorized.echo({ value: "denied" });
          assert(rejected.isErr());
          assert(rejected.error instanceof TransportError);
          assertEquals(rejected.error.code, "trellis.request.denied");
          assertEquals(
            (await client.echo({ value: "allowed" }).orThrow()).value,
            "allowed",
          );
          const admin = await runtime.connectClient({
            name: "admin",
            contract: adminParticipant,
          });
          const sessions = await admin.authSessionsList({
            participantId: caller.id,
            state: "active",
          }).orThrow();
          assertEquals(sessions.entries.length, 1);
          const session = sessions.entries[0];
          await admin.authSessionsRevoke({
            sessionId: session.sessionId,
            expectedVersion: session.version,
            idempotencyKey: crypto.randomUUID(),
            reason: "acceptance",
          }).orThrow();
          await runtime.waitFor(
            () => client.connection.status.phase === "closed",
          );
          const revoked = await client.echo({ value: "revoked" }, {
            timeout: 1000,
          });
          assert(revoked.isErr());
          assert(revoked.error instanceof TransportError);
          assertEquals(revoked.error.code, "trellis.request.closed");
        },
      );
    } finally {
      await service.stop();
      const error = await serviceExit;
      if (error instanceof Error) throw error;
    }
  });
});
