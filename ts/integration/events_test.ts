import { credsAuthenticator } from "@nats-io/nats-core";
import { jetstreamManager } from "@nats-io/jetstream";
import { connect } from "@nats-io/transport-node";
import { assertEquals } from "@std/assert";
import { fromFileUrl, join } from "@std/path";

import { participants } from "../../integration/fixtures/runtime/packages/runtime-trellis/index.js";

import { withTrellisRuntime } from "./_support/runtime.ts";

Deno.test("Rust durable events match registrations and retain unhandled messages", async () => {
  for (const reverse of [false, true]) {
    await withTrellisRuntime(async (runtime) => {
      const identity = await runtime.registerService({
        name: "events",
        contract: participants.testEvents.participant,
      });
      const process = new Deno.Command("cargo", {
        args: [
          "run",
          "--config",
          `patch.crates-io.trellis-rs.path=${
            JSON.stringify(
              fromFileUrl(
                new URL("../../rust/crates/trellis", import.meta.url),
              ),
            )
          }`,
          "--bin",
          "events",
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
          REVERSE: String(reverse),
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
      const nats = await connect({
        servers: runtime.natsUrl,
        authenticator: credsAuthenticator(
          await Deno.readFile(
            join(runtime.workdir, "nats/creds/trellis-auth.creds"),
          ),
        ),
      });
      try {
        const alpha = await runtime.connectClient({
          name: "alpha",
          contract: participants.testAlpha.participant,
        });
        const beta = await runtime.connectClient({
          name: "beta",
          contract: participants.testBeta.participant,
        });
        await runtime.waitFor(async () => {
          if (exited) {
            throw new Error(
              `event service exited: ${JSON.stringify(await status)}`,
            );
          }
          return (await alpha.observed({}, { timeout: 1000 })).isOk();
        }, { timeoutMs: 120_000 });
        await alpha.publishAlpha({ site: "one", value: "alpha" }).orThrow();
        await beta.publishBeta({ site: "one", value: "beta-one" }).orThrow();
        await beta.publishBeta({ site: "two", value: "beta-two" }).orThrow();
        await runtime.waitFor(async () => {
          const observed = await alpha.observed({}).orThrow();
          return observed.values.length === 3 && observed;
        }).then((observed) =>
          assertEquals(observed.values, ["alpha", "beta-one", "beta-two"])
        );

        const manager = await jetstreamManager(nats);
        const streams = await manager.streams.list().next();
        const eventStream = streams.find((stream) =>
          stream.config.subjects?.some((subject) =>
            subject.startsWith("events.")
          )
        );
        if (!eventStream) throw new Error("event stream missing");
        const consumers = (await manager.consumers.list(eventStream.config.name)
          .next()).filter((consumer) =>
            consumer.config.filter_subjects?.includes("events.v1.Alpha")
          );
        assertEquals(consumers.length, 1);
        await alpha.dropAlpha({}).orThrow();
        await alpha.publishAlpha({ site: "one", value: "unhandled" }).orThrow();
        await beta.publishBeta({ site: "three", value: "beta-after-drop" })
          .orThrow();
        await runtime.waitFor(async () =>
          (await alpha.observed({}).orThrow()).values.includes(
            "beta-after-drop",
          )
        );
        await runtime.waitFor(async () => {
          const info = await manager.consumers.info(
            eventStream.config.name,
            consumers[0].name,
          );
          return info.num_ack_pending === 1 && info.num_redelivered > 0;
        });
        assertEquals((await alpha.observed({}).orThrow()).values, [
          "alpha",
          "beta-after-drop",
          "beta-one",
          "beta-two",
        ]);

        // A real consumer failure must terminate the owning service, not a dummy registration task.
        await manager.consumers.delete(
          eventStream.config.name,
          consumers[0].name,
        );
        await runtime.waitFor(
          () => exited,
          { timeoutMs: 15_000 },
        );
        assertEquals((await status).success, false);
      } finally {
        if (!exited) process.kill("SIGTERM");
        await status;
        await nats.close();
      }
    });
  }
});
