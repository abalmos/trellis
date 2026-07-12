import { assert, assertEquals } from "@std/assert";
import { fromFileUrl, join } from "@std/path";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { createHealthFixture } from "./_fixture.ts";

const CASE_ID = "health.projection-lifecycle-and-recovery" as const;
const fixture = createHealthFixture(CASE_ID);
const rustRoot = fromFileUrl(new URL("../../../rust", import.meta.url));

liveTrellisTest({
  name:
    "health.projection-lifecycle-and-recovery projects lifecycle and replays downtime samples",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const configPath = await writeHealthConfig(runtime);
    await buildHealthRuntime();
    let healthProcess = startHealthRuntime(configPath);
    const observer = await runtime.connectClient({
      name: "health-projection-observer",
      contract: fixture.observerContract,
    });
    const service = await fixture.setupService(runtime);
    const waitForHealth = async (status: string) =>
      await runtime.waitFor(async () => {
        const response = await observer.healthQuery({
          participantKinds: ["service"],
          contractIds: [fixture.serviceContract.CONTRACT_ID],
          limit: 20,
          offset: 0,
        }).orThrow().catch(() => undefined);
        return response?.entries[0]?.effectiveStatus === status
          ? response
          : undefined;
      }, { timeoutMs: 30_000, intervalMs: 100 });

    try {
      const healthy = await waitForHealth("healthy");
      const initialRevision = healthy.projection.revision;

      await stopHealthRuntime(healthProcess);
      await new Promise((resolve) => setTimeout(resolve, 1_500));
      healthProcess = startHealthRuntime(configPath);
      const recovered = await waitForHealth("healthy");
      assert(recovered.projection.revision > initialRevision);
      assertEquals(recovered.projection.gapDetected, false);

      await service.stop();
      const offline = await waitForHealth("offline");
      assertEquals(offline.entries[0].offlineInstances, 1);
      await runtime.waitFor(async () => {
        const inspect = await observer.healthInspect({
          participantKind: "service",
          contractId: fixture.serviceContract.CONTRACT_ID,
          historyLimit: 20,
        }).orThrow();
        return inspect.history.some((interval) =>
            interval.effectiveStatus === "offline" &&
            interval.reason === "deadline-expired"
          )
          ? inspect
          : undefined;
      }, { timeoutMs: 5_000, intervalMs: 100 });
      const now = Date.now();
      const metrics = await observer.healthMetrics({
        participantKind: "service",
        contractId: fixture.serviceContract.CONTRACT_ID,
        start: new Date(now - 5 * 60_000).toISOString(),
        end: new Date(now + 1_000).toISOString(),
        stepMs: 300_000,
      }).orThrow();
      assert(metrics.summary.sampleCount >= 2);
      assert(metrics.summary.transitions >= 1);
    } finally {
      await service.stop().catch(() => undefined);
      await stopHealthRuntime(healthProcess);
    }
  },
});

async function buildHealthRuntime() {
  const result = await new Deno.Command("cargo", {
    args: [
      "build",
      "--quiet",
      "--manifest-path",
      join(rustRoot, "Cargo.toml"),
      "-p",
      "trellis-runtime",
      "--bin",
      "trellis-server",
    ],
    cwd: rustRoot,
    stdout: "null",
    stderr: "piped",
  }).output();
  assert(result.success, new TextDecoder().decode(result.stderr));
}

function startHealthRuntime(configPath: string): Deno.ChildProcess {
  return new Deno.Command(join(rustRoot, "target", "debug", "trellis-server"), {
    args: ["health", "--config", configPath],
    cwd: rustRoot,
    stdin: "null",
    stdout: "null",
    stderr: "inherit",
  }).spawn();
}

async function stopHealthRuntime(process: Deno.ChildProcess) {
  try {
    process.kill("SIGTERM");
  } catch (error) {
    if (!(error instanceof TypeError)) throw error;
  }
  await process.status;
}

async function writeHealthConfig(runtime: LiveTrellisRuntime) {
  const configPath = join(runtime.workdir, "health-runtime.toml");
  const controlPlaneConfig: unknown = JSON.parse(
    await Deno.readTextFile(join(runtime.workdir, "trellis", "config.jsonc")),
  );
  if (
    typeof controlPlaneConfig !== "object" || controlPlaneConfig === null ||
    !("sessionKeySeed" in controlPlaneConfig) ||
    typeof controlPlaneConfig.sessionKeySeed !== "string"
  ) {
    throw new Error("test control-plane config is missing sessionKeySeed");
  }
  const eventSeedPath = join(runtime.workdir, "health-session.seed");
  await Deno.writeTextFile(eventSeedPath, controlPlaneConfig.sessionKeySeed);
  const path = (value: string) => value.replaceAll("\\", "\\\\");
  await Deno.writeTextFile(
    configPath,
    `instance_name = "health-integration"
event_session_seed_file = "${path(eventSeedPath)}"

[http]
port = 0

[nats]
servers = "${runtime.natsUrl}"

[nats.runtime]
auth_creds_path = "${
      path(join(runtime.workdir, "nats", "creds", "auth-auth.creds"))
    }"
trellis_creds_path = "${
      path(join(runtime.workdir, "nats", "creds", "trellis-auth.creds"))
    }"
system_creds_path = "${
      path(join(runtime.workdir, "nats", "creds", "system.creds"))
    }"
sentinel_creds_path = "${
      path(join(runtime.workdir, "nats", "creds", "sentinel.creds"))
    }"

[health]
history_retention_days = 30
transport_retention_hours = 24
transport_max_bytes = 16777216

[health.storage]
kind = "sqlite"
path = "${path(join(runtime.workdir, "health.sqlite"))}"
journal_mode = "wal"
busy_timeout_ms = 5000
single_writer = true

[leases]
bucket = "trellis_runtime_leases"
replicas = 1
ttl_ms = 15000
renew_ms = 5000
`,
  );
  return configPath;
}
