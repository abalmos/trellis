import { jetstreamManager } from "@nats-io/jetstream";
import { credsAuthenticator } from "@nats-io/nats-core";
import { connect } from "@nats-io/transport-node";
import { Result } from "@qlever-llc/trellis";
import { TransportError } from "@qlever-llc/trellis/errors";
import { RetryJobError, TrellisService } from "@qlever-llc/trellis/service";
import { assert, assertEquals } from "@std/assert";
import { fromFileUrl, join } from "@std/path";
import { participants as webParticipants } from "trellis-web-generated";

import { participants } from "../../integration/fixtures/runtime/packages/runtime-trellis/index.js";
import { withTrellisRuntime } from "./_support/runtime.ts";

Deno.test("generated TypeScript caller reaches Rust provider", async () => {
  await withTrellisRuntime(async (runtime) => {
    const identity = await runtime.registerService({
      name: "rust",
      contract: participants.testProvider.participant,
    });
    const process = new Deno.Command("cargo", {
      args: [
        "run",
        "--config",
        `patch.crates-io.trellis-rs.path=${
          JSON.stringify(
            fromFileUrl(new URL("../../rust/crates/trellis", import.meta.url)),
          )
        }`,
        "--bin",
        "trellis-runtime-acceptance",
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
        TRELLIS_IDENTITY_SEED: identity.seed,
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
        contract: participants.testCaller.participant,
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
      contract: participants.testProvider.participant,
    });
    const service = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      participant: participants.testProvider.participant,
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
      let coverageRecoveryEffects = 0;
      await service.onChanged(({ event }) => {
        if (event.value === "coverage-recovery") coverageRecoveryEffects += 1;
        received = event.value;
        return Result.ok(undefined);
      }).orThrow();
      serviceExit = service.wait().catch((error: unknown) => error);
      const client = await runtime.connectClient({
        name: "caller",
        contract: participants.testCaller.participant,
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
      await t.step(
        "durable event waits for exact revocation-watch coverage",
        async () => {
          const nats = await connect({
            servers: runtime.natsUrl,
            authenticator: credsAuthenticator(
              await Deno.readFile(
                join(runtime.workdir, "nats/creds/trellis-auth.creds"),
              ),
            ),
          });
          try {
            const manager = await jetstreamManager(nats);
            const streams = await manager.streams.list().next();
            const eventStream = streams.find((stream) =>
              stream.config.subjects?.some((subject) =>
                subject.startsWith("events.")
              )
            );
            if (!eventStream) throw new Error("event stream missing");
            const eventConsumer = (await manager.consumers.list(
              eventStream.config.name,
            ).next()).find((consumer) =>
              consumer.config.filter_subjects?.includes("events.v1.Changed")
            );
            if (!eventConsumer) throw new Error("event consumer missing");
            const retainedEvent = await manager.streams.getMessage(
              eventStream.config.name,
              { last_by_subj: "events.v1.Changed" },
            );
            if (!retainedEvent) throw new Error("retained event missing");
            const eventContextDigest = retainedEvent.header?.get(
              "authorization-context",
            );
            if (!eventContextDigest) {
              throw new Error("retained event context digest missing");
            }
            received = undefined;
            let contextStream: string | undefined;
            let contextSubjectPrefix: string | undefined;
            for (const stream of await manager.streams.list().next()) {
              for (const subject of stream.config.subjects ?? []) {
                if (!subject.startsWith("$KV.") || !subject.endsWith(">")) {
                  continue;
                }
                const prefix = subject.slice(0, -1);
                try {
                  const stored = await manager.streams.getMessage(
                    stream.config.name,
                    { last_by_subj: `${prefix}${eventContextDigest}` },
                  );
                  if (!stored) continue;
                  contextStream = stream.config.name;
                  contextSubjectPrefix = prefix;
                  break;
                } catch {
                  // This KV stream does not own authorization contexts.
                }
              }
              if (contextStream) break;
            }
            if (!contextStream || !contextSubjectPrefix) {
              throw new Error("authorization context stream missing");
            }
            const coldIdentity = await runtime.services.createInstance({
              name: "cold-event-publisher",
              contract: participants.testProvider.participant,
            });
            const coldPublisher = await TrellisService.connect({
              trellisUrl: runtime.trellisUrl,
              participant: participants.testProvider.participant,
              name: "cold-event-publisher",
              identity: coldIdentity,
              telemetry: false,
              runtime: {},
            }).orThrow();
            const coldPublisherExit = coldPublisher.wait().catch((error) =>
              error
            );
            let coldEventContextDigest: string | undefined;
            try {
              await manager.consumers.pause(
                eventStream.config.name,
                eventConsumer.name,
                new Date(Date.now() + 60_000),
              );
              await coldPublisher.publishChanged({ value: "coverage-recovery" })
                .orThrow();
              const coldEvent = await manager.streams.getMessage(
                eventStream.config.name,
                { last_by_subj: "events.v1.Changed" },
              );
              assert(coldEvent);
              coldEventContextDigest = coldEvent.header?.get(
                "authorization-context",
              );
              assert(coldEventContextDigest);
              assert(coldEventContextDigest !== eventContextDigest);
              await manager.streams.delete(contextStream);
              await manager.consumers.resume(
                eventStream.config.name,
                eventConsumer.name,
              );
              await runtime.waitFor(async () => {
                const consumers = await manager.consumers.list(
                  eventStream.config.name,
                ).next();
                return consumers.some((consumer) =>
                  (consumer.config.filter_subjects?.includes(
                    "events.v1.Changed",
                  ) ?? false) &&
                  consumer.delivered.consumer_seq >
                    consumer.ack_floor.consumer_seq
                );
              }, { timeoutMs: 15_000 });
              assertEquals(coverageRecoveryEffects, 0);
            } finally {
              await coldPublisher.stop();
              const error = await coldPublisherExit;
              assert(
                !(error instanceof Error),
                error instanceof Error ? error.message : undefined,
              );
              await manager.consumers.resume(
                eventStream.config.name,
                eventConsumer.name,
              ).catch(() => undefined);
              await runtime.restartControlPlane();
              assert(coldEventContextDigest);
              await runtime.waitFor(async () => {
                try {
                  return Boolean(
                    await manager.streams.getMessage(
                      contextStream,
                      {
                        last_by_subj: contextSubjectPrefix +
                          coldEventContextDigest,
                      },
                    ),
                  );
                } catch {
                  return false;
                }
              });
              await manager.consumers.resume(
                eventStream.config.name,
                eventConsumer.name,
              );
            }
            assertEquals(
              await runtime.waitFor(
                () =>
                  received === "coverage-recovery" && coverageRecoveryEffects,
                { timeoutMs: 30_000 },
              ),
              1,
            );
          } finally {
            await nats.close();
          }
        },
      );
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
            contract: participants.testDenied.participant,
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
            contract: webParticipants.appConsole.participant,
          });
          await runtime.deployments.create({
            id: "native-admin",
            kind: "service",
          });
          const nativeKey = await runtime.registerService({
            name: "native-admin",
            contract: participants.testAdminService.participant,
            deployment: "native-admin",
          });
          const nativeSiblingKey = await runtime.services.createInstance({
            name: "native-admin-sibling",
            contract: participants.testAdminService.participant,
            deployment: "native-admin",
          });
          let nativeAdmin = await TrellisService.connect({
            trellisUrl: runtime.trellisUrl,
            participant: participants.testAdminService.participant,
            name: "native-admin",
            identity: nativeKey,
            telemetry: false,
            runtime: {},
          }).orThrow();
          let nativeAdminExit = nativeAdmin.wait().catch((error: unknown) =>
            error
          );
          const deniedBinding = (await admin.authGrantsList({
            participantId: participants.testDenied.participant.id,
            state: "active",
          }).orThrow()).entries[0];
          assert(deniedBinding);
          const deniedMutation = await nativeAdmin.authGrantsRevoke({
            ownerKind: deniedBinding.ownerKind,
            ownerId: deniedBinding.ownerId,
            participantId: deniedBinding.participantId,
            expectedRevision: deniedBinding.revision,
            idempotencyKey: crypto.randomUUID(),
            reason: "native administrator acceptance",
          });
          assert(deniedMutation.isErr());

          const nativeBinding = (await admin.authGrantsGet({
            ownerKind: "deployment",
            ownerId: nativeKey.deploymentId,
            participantId: participants.testAdminService.participant.id,
          }).orThrow()).binding;
          assert(nativeBinding);
          await admin.authGrantsSet({
            ownerKind: nativeBinding.ownerKind,
            ownerId: nativeBinding.ownerId,
            participantId: nativeBinding.participantId,
            installedRevision: nativeBinding.installedRevision,
            grants: nativeBinding.grants,
            platformPrivileges: ["trellis.auth::admin"],
            expiresAt: nativeBinding.expiresAt,
            expectedRevision: nativeBinding.revision,
            idempotencyKey: crypto.randomUUID(),
          }).orThrow();
          await nativeAdmin.stop();
          const firstNativeExit = await nativeAdminExit;
          assert(
            !(firstNativeExit instanceof Error),
            firstNativeExit instanceof Error
              ? firstNativeExit.message
              : undefined,
          );
          nativeAdmin = await TrellisService.connect({
            trellisUrl: runtime.trellisUrl,
            participant: participants.testAdminService.participant,
            name: "native-admin",
            identity: nativeKey,
            telemetry: false,
            runtime: {},
          }).orThrow();
          nativeAdminExit = nativeAdmin.wait().catch((error: unknown) => error);
          const nativeSibling = await TrellisService.connect({
            trellisUrl: runtime.trellisUrl,
            participant: participants.testAdminService.participant,
            name: "native-admin-sibling",
            identity: nativeSiblingKey,
            telemetry: false,
            runtime: {},
          }).orThrow();
          const nativeSiblingExit = nativeSibling.wait().catch(
            (error: unknown) => error,
          );
          try {
            await nativeAdmin.authGrantsRevoke({
              ownerKind: deniedBinding.ownerKind,
              ownerId: deniedBinding.ownerId,
              participantId: deniedBinding.participantId,
              expectedRevision: deniedBinding.revision,
              idempotencyKey: crypto.randomUUID(),
              reason: "native administrator acceptance",
            }).orThrow();
            await nativeSibling.authGrantsGet({
              ownerKind: nativeBinding.ownerKind,
              ownerId: nativeBinding.ownerId,
              participantId: nativeBinding.participantId,
            }).orThrow();
            const disabled = await admin.authServiceInstancesDisable({
              instanceId: nativeKey.instanceId,
              expectedVersion: 1,
              idempotencyKey: crypto.randomUUID(),
              reason: "instance isolation acceptance",
            }).orThrow();
            assertEquals(disabled.instance.state, "disabled");
            await nativeAdminExit;
            const disabledReconnect = await TrellisService.connect({
              trellisUrl: runtime.trellisUrl,
              participant: participants.testAdminService.participant,
              name: "disabled-native-admin",
              identity: nativeKey,
              telemetry: false,
              runtime: {},
            });
            assert(disabledReconnect.isErr());
            await nativeSibling.authGrantsGet({
              ownerKind: nativeBinding.ownerKind,
              ownerId: nativeBinding.ownerId,
              participantId: nativeBinding.participantId,
            }).orThrow();
          } finally {
            await nativeAdmin.stop();
            await nativeSibling.stop();
            const siblingError = await nativeSiblingExit;
            assert(
              !(siblingError instanceof Error),
              siblingError instanceof Error ? siblingError.message : undefined,
            );
          }
          const sessions = await admin.authSessionsList({
            participantId: participants.testCaller.participant.id,
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
      assertEquals(error, undefined);
    }
  });
});
