import { fromFileUrl } from "@std/path";
import { startTrellisIntegrationSharedRuntimeHost } from "../js/packages/trellis-test/src/integration/shared_runtime_host.ts";
import {
  summarizeTrellisTestDurations,
  summarizeTrellisTestProcessStarts,
} from "../js/packages/trellis-test/src/integration/metrics.ts";
import {
  buildIntegrationLiveArtifacts,
  buildIntegrationTest,
  buildRuntimeBinaries,
  expectedRustTests,
  INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
  loadIntegrationLiveArtifacts,
  rustTestClassifications,
  verifyCompiledRustInventory,
} from "../rust/crates/trellis-test/integration_runner.ts";
import clientMatrix from "./client-test-matrix.json" with { type: "json" };

const repoRoot = fromFileUrl(new URL("../", import.meta.url));
const PREBUILT_ONLY_ENV = "TRELLIS_TEST_PREBUILT_ONLY";

if (import.meta.main) {
  if (
    Deno.args.includes("--prebuilt-only") ||
    Deno.args.includes("--inventory-only")
  ) {
    Deno.env.set(PREBUILT_ONLY_ENV, "1");
  }
  if (Deno.args.includes("--build-only")) {
    await buildIntegrationLiveArtifacts(
      optionValue(Deno.args, "--artifacts-manifest") ??
        INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
    );
  } else {
    Deno.exit(await main(Deno.args));
  }
}

async function main(args: readonly string[]): Promise<number> {
  const typescriptCase = optionValue(args, "--typescript-case");
  const typescriptPrefix = optionValue(args, "--typescript-prefix");
  const rustFilter = optionValue(args, "--rust-filter");
  const keepWorkdir = args.includes("--keep-workdir") ||
    Deno.env.get("TRELLIS_TEST_KEEP_WORKDIR") === "1";
  const typescriptOnly = args.includes("--typescript-only");
  const inventoryOnly = args.includes("--inventory-only");
  const jobs = positiveInteger(optionValue(args, "--jobs") ?? "8", "--jobs");
  const typescriptCases = selectTypeScriptCases(
    clientMatrix.cases,
    typescriptCase,
    typescriptPrefix,
  );
  const artifactsManifest = optionValue(args, "--artifacts-manifest");
  const prebuiltOnly = args.includes("--prebuilt-only") || inventoryOnly ||
    Deno.env.get(PREBUILT_ONLY_ENV) === "1";
  const artifacts = prebuiltOnly
    ? await loadIntegrationLiveArtifacts(
      artifactsManifest ?? INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
    )
    : {
      integrationBinary: await buildIntegrationTest(),
      runtimeBinaries: await buildRuntimeBinaries(),
    };
  const { integrationBinary, runtimeBinaries } = artifacts;
  if (inventoryOnly) {
    await Promise.all([
      verifyTypeScriptInventory({
        ...runtimeBinaries,
        [PREBUILT_ONLY_ENV]: "1",
      }),
      verifyCompiledRustInventory(integrationBinary),
    ]);
    return 0;
  }
  const rustClassifications = rustTestClassifications();
  const typescriptCaseIds = typescriptCases.map((testCase) => {
    const implementation = testCase.implementations.typescript;
    if (implementation === undefined) {
      throw new Error(
        `implemented TypeScript case ${testCase.id} has no implementation`,
      );
    }
    return implementation.id;
  });
  const rustTests = typescriptOnly
    ? []
    : expectedRustTests().filter((id) =>
      rustFilter === undefined || id.includes(rustFilter)
    );
  const assignments = [
    ...typescriptCases
      .map((testCase, index) => ({
        id: typescriptCaseIds[index],
        namespacePrefix: "ts",
        classification: testCase.classification === "isolated-process"
          ? "isolated-process" as const
          : "shared" as const,
      })),
    ...rustTests.map((id) => ({
      id,
      namespacePrefix: "rs",
      classification: rustClassifications.get(id) === "isolated-process"
        ? "isolated-process" as const
        : "shared" as const,
    })),
  ];
  const host = await startTrellisIntegrationSharedRuntimeHost({
    runtime: {
      keepWorkdir,
      trellis: {
        command: {
          cmd: runtimeBinaries.TRELLIS_TEST_SERVER_BIN,
          args: ["--config", "{config}", "all"],
        },
      },
    },
    assignments,
  });
  const env = {
    ...host.env,
    ...runtimeBinaries,
    TRELLIS_TEST_INTEGRATION_BIN: integrationBinary,
    ...(prebuiltOnly ? { [PREBUILT_ONLY_ENV]: "1" } : {}),
  };

  const lanes: WorkerLane[] = [];
  if (typescriptCases.length > 0) {
    lanes.push({
      name: "typescript",
      run: async (workers) => {
        const status = await new Deno.Command(Deno.execPath(), {
          args: [
            "run",
            "-A",
            "-c",
            "js/integration/deno.json",
            "js/integration/runner.ts",
            "--parallel",
            "--jobs",
            String(workers),
            ...typescriptCaseIds.flatMap((id) => ["--case", id]),
          ],
          cwd: repoRoot,
          env,
          stdin: "inherit",
          stdout: "inherit",
          stderr: "inherit",
        }).spawn().status;
        return status.code;
      },
    });
  }
  if (rustTests.length > 0) {
    lanes.push({
      name: "rust",
      run: async (workers) => {
        const status = await new Deno.Command(Deno.execPath(), {
          args: [
            "run",
            "-A",
            "-c",
            "js/deno.json",
            "rust/crates/trellis-test/integration_runner.ts",
            "--jobs",
            String(workers),
            ...(rustFilter === undefined ? [] : ["--", rustFilter]),
          ],
          cwd: repoRoot,
          env,
          stdin: "inherit",
          stdout: "inherit",
          stderr: "inherit",
        }).spawn().status;
        return status.code;
      },
    });
  }

  return await orchestrateWorkerLanes(lanes, jobs, async (code) => {
    try {
      if (code !== 0) {
        console.error(
          `shared Trellis output:\n${host.output?.() ?? "<unavailable>"}`,
        );
      }
      const metrics = host.metrics === undefined ? [] : await host.metrics();
      console.log(JSON.stringify({
        event: "integration-process-summary",
        starts: summarizeTrellisTestProcessStarts(metrics),
        slowest: summarizeTrellisTestDurations(metrics),
      }));
    } finally {
      await host.stop();
    }
  });
}

export type WorkerLane = {
  readonly name: string;
  readonly run: (jobs: number) => Promise<number>;
};

export function selectTypeScriptCases<
  T extends {
    readonly id: string;
    readonly completion: { readonly typescript: string };
  },
>(
  cases: readonly T[],
  exact?: string,
  prefix?: string,
): T[] {
  const implemented = cases.filter((entry) =>
    entry.completion.typescript === "implemented"
  );
  if (exact !== undefined && !implemented.some((entry) => entry.id === exact)) {
    throw new Error(
      `--typescript-case ${exact} does not name an implemented TypeScript case`,
    );
  }
  const selected = implemented
    .filter((entry) => exact === undefined || entry.id === exact)
    .filter((entry) => prefix === undefined || entry.id.startsWith(prefix));
  if (prefix !== undefined && selected.length === 0) {
    throw new Error(
      `--typescript-prefix ${prefix} selects no implemented TypeScript cases`,
    );
  }
  return selected;
}

export function allocateWorkers(
  totalJobs: number,
  laneCount: number,
): number[] {
  if (!Number.isInteger(totalJobs) || totalJobs <= 0 || laneCount <= 0) {
    throw new Error("worker allocation requires positive integers");
  }
  const base = Math.floor(totalJobs / laneCount);
  const remainder = totalJobs % laneCount;
  return Array.from(
    { length: laneCount },
    (_, index) => base + (index >= laneCount - remainder ? 1 : 0),
  );
}

export async function orchestrateWorkerLanes(
  lanes: readonly WorkerLane[],
  totalJobs: number,
  afterSettled: (code: number) => Promise<void> = () => Promise.resolve(),
): Promise<number> {
  positiveInteger(String(totalJobs), "--jobs");
  let code = 0;
  try {
    for (let offset = 0; offset < lanes.length; offset += totalJobs) {
      const batch = lanes.slice(offset, offset + totalJobs);
      const allocations = allocateWorkers(totalJobs, batch.length);
      const results = await Promise.allSettled(
        batch.map((lane, index) => lane.run(allocations[index])),
      );
      for (const result of results) {
        const laneCode = result.status === "fulfilled" ? result.value : 1;
        if (code === 0 && laneCode !== 0) code = laneCode;
      }
    }
    return code;
  } finally {
    await afterSettled(code);
  }
}

async function verifyTypeScriptInventory(
  env: Readonly<Record<string, string>>,
): Promise<void> {
  const status = await new Deno.Command(Deno.execPath(), {
    args: [
      "run",
      "-A",
      "-c",
      "js/integration/deno.json",
      "js/integration/runner.ts",
      "--inventory-only",
    ],
    cwd: repoRoot,
    env: { ...env },
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  }).spawn().status;
  if (!status.success) {
    throw new Error(
      `TypeScript inventory check failed with status ${status.code}`,
    );
  }
}

function optionValue(
  args: readonly string[],
  name: string,
): string | undefined {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  const value = args[index + 1];
  if (value === undefined || value.startsWith("--")) {
    throw new Error(`${name} requires a value`);
  }
  return value;
}

function positiveInteger(value: string, flag: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${flag} requires a positive integer`);
  }
  return parsed;
}
