import { Result } from "@qlever-llc/trellis";
import { RetryJobError, TrellisService } from "@qlever-llc/trellis/service";
import { assert, assertEquals } from "@std/assert";
import { fromFileUrl, join } from "@std/path";

import { participants } from "../../integration/fixtures/runtime/packages/runtime-trellis/index.js";
import { withTrellisRuntime } from "./_support/runtime.ts";

/** Built server binary used so each scenario can set its own environment. */
function serverBinary(): string {
  return Deno.env.get("TRELLIS_TEST_SERVER_BIN") ??
    fromFileUrl(
      new URL("../../rust/target/debug/trellis-server", import.meta.url),
    );
}

const captureEndpoint = Deno.env.get("TRELLIS_OBS_CAPTURE_ENDPOINT");
const prometheusUrl = Deno.env.get("TRELLIS_OBS_PROMETHEUS_URL");

/**
 * Telemetry is optional: a runtime started with no OTLP endpoint, with the SDK
 * disabled, with traces only, or with an unreachable Collector must keep
 * serving ordinary generated Jobs and Events work rather than stopping its
 * Jobs or Events subsystems.
 *
 * Each scenario waits for a generated Job result and an Event consumer delivery.
 */
for (
  const scenario of [
    {
      name: "no endpoint",
      env: {
        OTEL_TRACES_EXPORTER: "otlp",
        OTEL_METRICS_EXPORTER: "otlp",
      } as Record<string, string>,
    },
    {
      name: "SDK disabled",
      env: {
        OTEL_SDK_DISABLED: "true",
        OTEL_TRACES_EXPORTER: "otlp",
        OTEL_METRICS_EXPORTER: "otlp",
      } as Record<string, string>,
    },
    {
      name: "traces only",
      env: {
        OTEL_TRACES_EXPORTER: "otlp",
        OTEL_METRICS_EXPORTER: "otlp",
      } as Record<string, string>,
    },
    {
      name: "collector unavailable",
      env: {
        OTEL_EXPORTER_OTLP_ENDPOINT: "http://127.0.0.1:49999",
      } as Record<string, string>,
    },
    ...(captureEndpoint
      ? [{
        name: "collector enabled",
        env: {
          OTEL_EXPORTER_OTLP_ENDPOINT: captureEndpoint,
          OTEL_TRACES_SAMPLER: "always_on",
          OTEL_METRIC_EXPORT_INTERVAL: "1000",
          OTEL_BSP_SCHEDULE_DELAY: "100",
        },
      }]
      : []),
  ]
) {
  Deno.test(
    `generated jobs and events work with ${scenario.name} telemetry`,
    async () => {
      const requests: string[] = [];
      let endpoint = "";
      const collector = scenario.name === "traces only" ||
          scenario.name === "SDK disabled"
        ? Deno.serve({
          hostname: "127.0.0.1",
          port: 0,
          onListen: ({ port }) => {
            endpoint = `http://127.0.0.1:${port}`;
          },
        }, async (request) => {
          requests.push(new URL(request.url).pathname);
          await request.arrayBuffer();
          return new Response(null, { status: 200 });
        })
        : undefined;
      try {
        const env = {
          ...scenario.env,
          ...(scenario.name === "traces only" && {
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: `${endpoint}/v1/traces`,
            OTEL_TRACES_SAMPLER: "always_on",
            OTEL_BSP_SCHEDULE_DELAY: "100",
          }),
          ...(scenario.name === "SDK disabled" && {
            OTEL_EXPORTER_OTLP_ENDPOINT: endpoint,
          }),
        };
        await withTrellisRuntime(async (runtime) => {
          const identity = await runtime.registerService({
            name: `obs-${scenario.name.replaceAll(" ", "-")}`,
            contract: participants.Provider.participant,
          });
          // A real authenticated service connection exercises the running
          // runtime's platform, Jobs, and Events wiring end to end.
          const service = await TrellisService.connect({
            trellisUrl: runtime.trellisUrl,
            participant: participants.Provider.participant,
            name: "provider",
            seed: identity.seed,
          }).orThrow();
          let serviceExit: Promise<unknown> | undefined;
          try {
            if (scenario.name === "collector enabled") {
              await service.handleEcho(async ({ input }) => {
                if (input.value === "linked-job") {
                  const job = await service.jobs.work.create({
                    value: input.value,
                  }).orThrow();
                  const terminal = await job.wait().orThrow();
                  assertEquals(terminal.state, "completed");
                  assertEquals(terminal.result, { value: input.value });
                }
                return Result.ok({ value: `Rust received ${input.value}` });
              });
              await service.handleWork(async ({ input, op }) => {
                await op.started().orThrow();
                if (input.value === "routing-check") {
                  const signal = await op.nextSignal("Continue").orThrow();
                  await op.acknowledgeSignal(signal.sequence).orThrow();
                  return await op.complete({ value: "completed" }).orThrow();
                }
                return await op.complete({ value: `completed ${input.value}` })
                  .orThrow();
              });
              await service.handleWatch(async ({ emit }) => {
                await emit({ value: "rust-feed-from-ts" }).orThrow();
              });
              service.jobs.keyedWork.handle(({ job }) =>
                Promise.resolve(Result.ok(job.payload))
              );
            }
            let linkedAttempts = 0;
            service.jobs.work.handle(({ job }) =>
              Promise.resolve(
                job.payload.value === "linked-job" && ++linkedAttempts === 1
                  ? Result.err(new RetryJobError())
                  : Result.ok(job.payload),
              )
            );
            let received: string | undefined;
            await service.onChanged(({ event }) => {
              received = event.value;
              return Result.ok(undefined);
            }).orThrow();
            serviceExit = service.wait().catch((error: unknown) => error);
            const job = await service.jobs.work.create({ value: "observed" })
              .orThrow();
            const terminal = await job.wait().orThrow();
            assertEquals(terminal.state, "completed");
            assertEquals(terminal.result, { value: "observed" });
            if (scenario.name === "collector enabled") {
              const keyed = await service.jobs.keyedWork.create({
                key: "observed",
                value: "observed",
              }).orThrow();
              assertEquals((await keyed.wait().orThrow()).state, "completed");
            }
            await service.publishChanged({ value: "observed" }).orThrow();
            assertEquals(await runtime.waitFor(() => received), "observed");
            if (scenario.name === "collector enabled") {
              const client = await runtime.connectClient({
                name: "observability-caller",
                contract: participants.Caller.participant,
              });
              assertEquals(
                (await client.echo({ value: "observed" }).orThrow()).value,
                "Rust received observed",
              );
              assertEquals(
                (await client.echo({ value: "linked-job" }).orThrow()).value,
                "Rust received linked-job",
              );
              assertEquals(linkedAttempts, 2);
              const operation = await client.work({ value: "observed" })
                .start().orThrow();
              const finished = await operation.wait().orThrow();
              assertEquals(finished.state, "completed");
              assertEquals(finished.output?.value, "completed observed");
              const child = new Deno.Command("cargo", {
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
                  "caller",
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
                  XDG_CONFIG_HOME: join(
                    runtime.workdir,
                    "rust-observability-caller",
                  ),
                  CARGO_TARGET_DIR: fromFileUrl(
                    new URL("../../rust/target", import.meta.url),
                  ),
                },
                stdout: "piped",
                stderr: "inherit",
              }).spawn();
              const reader = child.stdout.pipeThrough(new TextDecoderStream())
                .getReader();
              let output = "";
              while (!output.includes("rust login ")) {
                const chunk = await reader.read();
                assert(!chunk.done, output);
                output += chunk.value;
              }
              const loginUrl = output.match(/rust login (\S+)/)?.[1];
              assert(loginUrl);
              await runtime.completeClientAuth({
                loginUrl,
                sessionKey: "observability-rust-caller",
                mode: "session_key",
              });
              while (!output.includes("rust caller complete")) {
                const chunk = await reader.read();
                assert(!chunk.done, output);
                output += chunk.value;
              }
              assert((await child.status).success, output);
            }
          } finally {
            await service.stop();
            if (serviceExit) assertEquals(await serviceExit, undefined);
          }
        }, {
          trellis: {
            command: {
              cmd: "env",
              args: [
                ...Object.keys(Deno.env.toObject()).filter((key) =>
                  key.startsWith("OTEL_")
                ).flatMap((key) => ["-u", key]),
                ...Object.entries(env).map(([key, value]) => `${key}=${value}`),
                serverBinary(),
                "--config",
                "{config}",
                "all",
              ],
            },
          },
        });
        if (scenario.name === "traces only") {
          assertEquals(requests.includes("/v1/traces"), true);
          assertEquals(requests.includes("/v1/metrics"), false);
        }
        if (scenario.name === "SDK disabled") assertEquals(requests, []);
      } finally {
        await collector?.shutdown();
      }
    },
  );
}

if (captureEndpoint && prometheusUrl) {
  Deno.test("enabled Rust provider exports durable attempt links", async () => {
    const previousMetrics = new Set(
      (await (await fetch(prometheusUrl)).text()).split("\n"),
    );
    await withTrellisRuntime(async (runtime) => {
      const identity = await runtime.registerService({
        name: "rust-observability-provider",
        contract: participants.OperationProvider.participant,
      });
      const provider = new Deno.Command("setsid", {
        args: [
          "cargo",
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
        stdout: "null",
        stderr: "inherit",
      }).spawn();
      const status = provider.status;
      try {
        await runtime.deployments.create({
          id: "rust-observability-caller",
          kind: "service",
        });
        const callerIdentity = await runtime.registerService({
          name: "rust-operation-caller",
          contract: participants.OperationCaller.participant,
          deployment: "rust-observability-caller",
        });
        const caller = await TrellisService.connect({
          trellisUrl: runtime.trellisUrl,
          participant: participants.OperationCaller.participant,
          seed: callerIdentity.seed,
        }).orThrow();
        try {
          const operation = await runtime.waitFor(async () => {
            const started = await caller.work({ value: "routing-check" })
              .start();
            return started.isOk() ? started.orThrow() : false;
          }, { timeoutMs: 120_000 });
          await operation.signal("Continue", { value: "continue" }).orThrow();
          const terminal = await operation.wait().orThrow();
          assertEquals(terminal.state, "completed");
          assertEquals(terminal.output?.value, "completed");
          await runtime.waitFor(
            async () =>
              (await (await fetch(prometheusUrl)).text()).split("\n").some(
                (line) =>
                  line.startsWith(
                    "trellis_operation_execution_duration_count{",
                  ) &&
                  !previousMetrics.has(line),
              ),
            { timeoutMs: 30_000 },
          );
        } finally {
          await caller.stop();
        }
      } finally {
        Deno.kill(-provider.pid, "SIGTERM");
        await status;
      }
    }, {
      trellis: {
        command: {
          cmd: serverBinary(),
          args: ["--config", "{config}", "all"],
        },
      },
    });
  });
}
