import {
  dirname,
  fromFileUrl,
  isAbsolute,
  resolve,
  toFileUrl,
} from "@std/path";
import { startTrellisIntegrationSharedRuntimeHost } from "./shared_runtime_host.ts";
import type { TrellisIntegrationSharedRuntimeHost } from "./shared_runtime_host.ts";
import {
  summarizeTrellisTestDurations,
  summarizeTrellisTestProcessStarts,
} from "./metrics.ts";
import { TRELLIS_TEST_EVENTS_ENV } from "./runtime.ts";
import { TRELLIS_TEST_SHARED_RUNTIME_ENV } from "./shared_runtime_protocol.ts";
import type {
  TrellisIntegrationCase,
  TrellisIntegrationRuntimeOptions,
} from "./types.ts";

/** Configuration loaded by the generic Trellis integration test runner. */
export type TrellisIntegrationRunnerConfig = {
  /** Runtime startup options used by direct tests and the shared runtime host. */
  readonly runtime: TrellisIntegrationRuntimeOptions;
  /**
   * Additional arguments passed to child `deno test` before runner-managed flags
   * and selected files, for example `-A`, `-c`, or lockfile options.
   */
  readonly denoTestArgs?: readonly string[];
  /** Integration cases known to this repo. */
  readonly cases: readonly TrellisIntegrationCase[];
  /** Optional conformance hook for repo-specific case validation. */
  readonly conformance?: () => Promise<void> | void;
};

/** Options for programmatic execution of the generic integration runner. */
export type TrellisIntegrationRunnerOptions = {
  /** CLI-style arguments to parse. Defaults to `Deno.args`. */
  readonly args?: readonly string[];
  /** Preloaded runner config, primarily for unit tests and custom wrappers. */
  readonly config?: TrellisIntegrationRunnerConfig;
  /** Directory used for relative config paths and `deno test`. Defaults to `Deno.cwd()`. */
  readonly cwd?: string;
  /** Additional child `deno test` arguments supplied by programmatic callers. */
  readonly denoTestArgs?: readonly string[];
  /**
   * Runs the constructed `deno test` command.
   *
   * Tests can inject this hook to assert command construction without spawning a
   * child Deno process. Production callers normally use the default runner.
   */
  readonly commandRunner?: (command: {
    /** Executable path for the Deno binary. */
    readonly executable: string;
    /** Arguments passed to the Deno executable. */
    readonly args: readonly string[];
    /** Working directory for the child test process. */
    readonly cwd: string;
    /** Environment overrides for the child test process. */
    readonly env?: Record<string, string>;
  }) => Promise<number>;
  /** Writes an LCOV report from Deno coverage output. Defaults to `deno coverage --lcov`. */
  readonly coverageReporter?: (coverage: {
    /** Directory containing raw Deno coverage profiles. */
    readonly rawDir: string;
    /** LCOV file to write. */
    readonly lcovPath: string;
  }) => Promise<void>;
  /**
   * Starts the shared runtime host used by `--parallel`.
   *
   * Tests can inject this hook to avoid starting Trellis while verifying the
   * command environment passed to worker tests.
   */
  readonly sharedRuntimeHostStarter?: (args: {
    /** Runtime options from the loaded runner config. */
    readonly runtime: TrellisIntegrationRuntimeOptions;
    /** Selected executable cases. */
    readonly assignments: readonly {
      id: string;
    }[];
  }) => Promise<TrellisIntegrationSharedRuntimeHost>;
  /** Output hook used for help text. Defaults to `console`. */
  readonly output?: {
    /** Writes ordinary runner output. */
    log(message: string): void;
  };
};

type ParsedRunnerArgs = {
  readonly configPath: string | undefined;
  readonly fixtureFilters: readonly string[];
  readonly caseFilters: readonly string[];
  readonly coverageFilters: readonly string[];
  readonly coverageDir: string | undefined;
  readonly skipConformance: boolean;
  readonly inventoryOnly: boolean;
  readonly parallel: boolean;
  readonly jobs: number | undefined;
  readonly denoTestArgs: readonly string[];
  readonly help: boolean;
};

type LoadedRunnerConfig = {
  readonly config: TrellisIntegrationRunnerConfig;
  readonly baseDir: string;
};

type SelectedCases = {
  readonly caseIds: readonly string[];
  readonly cases: readonly TypeScriptIntegrationTestIdentity[];
  readonly registrations: readonly TypeScriptIntegrationTestIdentity[];
  readonly files: readonly string[];
  readonly testNames: readonly string[];
};

export type TypeScriptIntegrationTestIdentity = {
  readonly caseId: string;
  readonly testName: string;
};

export type TypeScriptIntegrationTestEvent = {
  readonly event: "integration-case";
  readonly language: "typescript";
  readonly caseId: string;
  readonly testName: string;
  readonly status: "registered" | "started" | "passed" | "failed" | "ignored";
  readonly timestamp: string;
  readonly durationMs?: number;
};

type TypeScriptIntegrationResults = {
  readonly event: "typescript-integration-results";
  readonly registered: number;
  readonly selected: number;
  readonly passed: number;
  readonly failed: number;
  readonly ignored: number;
  readonly tests: readonly {
    readonly name: string;
    readonly status: "passed" | "failed";
  }[];
};

/**
 * Runs Trellis integration tests from a loaded config or `--config` module.
 *
 * The runner is intentionally generic: it does not add repo-local Deno config,
 * lockfile, permission, or Trellis command defaults. Case file paths are
 * resolved relative to the config module when a config path is supplied.
 */
export async function runTrellisIntegrationTests(
  options: TrellisIntegrationRunnerOptions = {},
): Promise<number> {
  const args = parseRunnerArgs(options.args ?? Deno.args);
  const output = options.output ?? console;
  const cwd = options.cwd ?? Deno.cwd();

  if (args.help) {
    output.log(helpText());
    return 0;
  }

  const loaded = options.config === undefined
    ? await loadRunnerConfig(args.configPath, cwd)
    : {
      config: options.config,
      baseDir: args.configPath === undefined
        ? cwd
        : resolveConfigLocation(args.configPath, cwd).baseDir,
    };

  validateFilters(args, loaded.config.cases);
  const selected = selectCases(args, loaded.config.cases, loaded.baseDir);
  if (selected.files.length === 0) {
    throw new Error("no Trellis integration test cases selected");
  }
  assertUniqueCaseIds(loaded.config.cases);

  if (!args.skipConformance || args.inventoryOnly) {
    await loaded.config.conformance?.();
  }

  const commandRunner = options.commandRunner ?? runDenoTestCommand;
  const childDenoTestArgs = [
    ...(loaded.config.denoTestArgs ?? []),
    ...(options.denoTestArgs ?? []),
    ...args.denoTestArgs,
  ];
  if (args.inventoryOnly) {
    const run = await runDenoTestsWithEvents(commandRunner, {
      executable: Deno.execPath(),
      args: denoTestArgs({
        parallel: false,
        extraArgs: childDenoTestArgs,
        files: selected.files,
        filter: "/a^/",
      }),
      cwd,
    });
    const registrations = reconcileTypeScriptIntegrationInventory(
      configuredIdentities(loaded.config.cases),
      selected.registrations,
      run.events,
    );
    output.log(JSON.stringify({
      event: "typescript-integration-inventory",
      registered: registrations.length,
      tests: registrations,
    }));
    return run.code;
  }
  const coverage = args.coverageDir === undefined
    ? undefined
    : coveragePaths(args.coverageDir, cwd);
  if (coverage !== undefined) {
    await removeIfExists(coverage.rawDir, { recursive: true });
    await removeIfExists(coverage.lcovPath);
    await Deno.mkdir(coverage.dir, { recursive: true });
    childDenoTestArgs.push(`--coverage=${coverage.rawDir}`);
  }
  if (args.parallel) {
    const inheritedManifest = Deno.env.get(TRELLIS_TEST_SHARED_RUNTIME_ENV);
    if (inheritedManifest !== undefined) {
      const env: Record<string, string> = {
        [TRELLIS_TEST_SHARED_RUNTIME_ENV]: inheritedManifest,
        DENO_JOBS: "1",
      };
      const run = await runDenoTestsWithEvents(commandRunner, {
        executable: Deno.execPath(),
        args: denoTestArgs({
          parallel: true,
          extraArgs: childDenoTestArgs,
          files: selected.files,
          filter: testNameFilter(selected.testNames),
        }),
        cwd,
        env,
      });
      await writeCoverageReport(coverage, options.coverageReporter);
      return reportTypeScriptResults(
        output,
        loaded.config.cases,
        selected,
        run,
      );
    }
    const startHost = options.sharedRuntimeHostStarter ??
      startTrellisIntegrationSharedRuntimeHost;
    const host = await startHost({
      runtime: loaded.config.runtime,
      assignments: selected.caseIds.map((id) => {
        const integrationCase = loaded.config.cases.find((entry) =>
          entry.id === id
        );
        if (integrationCase === undefined) {
          throw new Error(`selected integration case ${id} is not registered`);
        }
        return {
          id,
          namespacePrefix: "ts",
        };
      }),
    });
    const env = { ...host.env, DENO_JOBS: "1" };

    try {
      const run = await runDenoTestsWithEvents(commandRunner, {
        executable: Deno.execPath(),
        args: denoTestArgs({
          parallel: true,
          extraArgs: childDenoTestArgs,
          files: selected.files,
          filter: testNameFilter(selected.testNames),
        }),
        cwd,
        env,
      });
      await writeCoverageReport(coverage, options.coverageReporter);
      const code = reportTypeScriptResults(
        output,
        loaded.config.cases,
        selected,
        run,
      );
      if (code !== 0 && host.output !== undefined) output.log(host.output());
      return code;
    } finally {
      try {
        const metrics = host.metrics === undefined ? [] : await host.metrics();
        output.log(JSON.stringify({
          event: "integration-process-summary",
          starts: summarizeTrellisTestProcessStarts(metrics),
          slowest: summarizeTrellisTestDurations(metrics),
        }));
      } finally {
        await host.stop();
      }
    }
  }

  const run = await runDenoTestsWithEvents(commandRunner, {
    executable: Deno.execPath(),
    args: denoTestArgs({
      parallel: false,
      extraArgs: childDenoTestArgs,
      files: selected.files,
      filter: testNameFilter(selected.testNames),
    }),
    cwd,
  });
  await writeCoverageReport(coverage, options.coverageReporter);
  return reportTypeScriptResults(output, loaded.config.cases, selected, run);
}

function reportTypeScriptResults(
  output: { log(message: string): void },
  cases: readonly TrellisIntegrationCase[],
  selected: SelectedCases,
  run: {
    readonly code: number;
    readonly events: readonly TypeScriptIntegrationTestEvent[];
  },
): number {
  const results = reconcileTypeScriptIntegrationEvents(
    configuredIdentities(cases),
    selected.cases,
    run.events,
    selected.registrations,
  );
  output.log(JSON.stringify(results));
  return run.code === 0 && results.failed > 0 ? 1 : run.code;
}

async function runDenoTestsWithEvents(
  commandRunner: NonNullable<TrellisIntegrationRunnerOptions["commandRunner"]>,
  command: Parameters<
    NonNullable<TrellisIntegrationRunnerOptions["commandRunner"]>
  >[0],
): Promise<{
  readonly code: number;
  readonly events: readonly TypeScriptIntegrationTestEvent[];
}> {
  const eventPath = await Deno.makeTempFile({
    prefix: "trellis-integration-",
    suffix: ".jsonl",
  });
  try {
    const code = await commandRunner({
      ...command,
      env: { ...command.env, [TRELLIS_TEST_EVENTS_ENV]: eventPath },
    });
    return {
      code,
      events: parseTypeScriptIntegrationEvents(
        await Deno.readTextFile(eventPath),
      ),
    };
  } finally {
    await removeIfExists(eventPath);
  }
}

export function parseTypeScriptIntegrationEvents(
  jsonl: string,
): TypeScriptIntegrationTestEvent[] {
  return jsonl.split("\n").filter((line) => line !== "").map((line) => {
    let value: unknown;
    try {
      value = JSON.parse(line);
    } catch {
      throw new Error(`invalid TypeScript integration event JSON: ${line}`);
    }
    if (!isTypeScriptIntegrationTestEvent(value)) {
      throw new Error(`invalid TypeScript integration event: ${line}`);
    }
    return value;
  });
}

export function reconcileTypeScriptIntegrationEvents(
  configuredCases: readonly TypeScriptIntegrationTestIdentity[],
  selectedCases: readonly TypeScriptIntegrationTestIdentity[],
  events: readonly TypeScriptIntegrationTestEvent[],
  expectedRegistrations: readonly TypeScriptIntegrationTestIdentity[] =
    selectedCases,
): TypeScriptIntegrationResults {
  const configured = identityMap(
    "registered TypeScript integration cases",
    configuredCases,
  );
  const selected = identityMap(
    "selected TypeScript integration cases",
    selectedCases,
  );
  const expected = identityMap(
    "expected TypeScript integration registrations",
    expectedRegistrations,
  );
  const seen = new Map<string, TypeScriptIntegrationTestEvent["status"][]>();

  for (const event of events) {
    assertConfiguredEvent(configured, event);
    const statuses = seen.get(event.caseId) ?? [];
    if (statuses.includes(event.status)) {
      throw new Error(
        `duplicate TypeScript integration ${event.status} event: ${event.caseId}`,
      );
    }
    if (event.status === "ignored") {
      throw new Error(`ignored TypeScript integration case: ${event.caseId}`);
    }
    statuses.push(event.status);
    seen.set(event.caseId, statuses);
  }

  const observedRegistrations = new Set(
    events.filter((event) => event.status === "registered").map((event) =>
      event.caseId
    ),
  );
  assertSameIds(
    "registered TypeScript integration cases",
    new Set(expected.keys()),
    observedRegistrations,
  );

  const tests: { name: string; status: "passed" | "failed" }[] = [];
  for (const { caseId } of selectedCases) {
    const statuses = seen.get(caseId) ?? [];
    const terminal = statuses.filter((status) =>
      status === "passed" || status === "failed"
    );
    if (
      statuses[0] !== "registered" || statuses[1] !== "started" ||
      terminal.length !== 1 || statuses[2] !== terminal[0] ||
      statuses.length !== 3
    ) {
      throw new Error(
        `incomplete TypeScript integration case ${caseId}: ${
          statuses.join(", ") || "no events"
        }`,
      );
    }
    tests.push({ name: caseId, status: terminal[0] });
  }

  for (const [caseId, statuses] of seen) {
    if (
      !selected.has(caseId) &&
      statuses.some((status) => status !== "registered")
    ) {
      throw new Error(`unselected TypeScript integration case ran: ${caseId}`);
    }
  }

  return {
    event: "typescript-integration-results",
    registered: configuredCases.length,
    selected: selectedCases.length,
    passed: tests.filter((test) => test.status === "passed").length,
    failed: tests.filter((test) => test.status === "failed").length,
    ignored: 0,
    tests,
  };
}

export function reconcileTypeScriptIntegrationInventory(
  configuredCases: readonly TypeScriptIntegrationTestIdentity[],
  expectedRegistrations: readonly TypeScriptIntegrationTestIdentity[],
  events: readonly TypeScriptIntegrationTestEvent[],
): TypeScriptIntegrationTestIdentity[] {
  const configured = identityMap(
    "registered TypeScript integration cases",
    configuredCases,
  );
  const expected = identityMap(
    "expected TypeScript integration registrations",
    expectedRegistrations,
  );
  const observed = new Set<string>();
  for (const event of events) {
    assertConfiguredEvent(configured, event);
    if (event.status !== "registered") {
      throw new Error(
        `inventory emitted TypeScript integration ${event.status} event: ${event.caseId}`,
      );
    }
    if (observed.has(event.caseId)) {
      throw new Error(
        `duplicate TypeScript integration registered event: ${event.caseId}`,
      );
    }
    observed.add(event.caseId);
  }
  assertSameIds(
    "registered TypeScript integration cases",
    new Set(expected.keys()),
    observed,
  );
  return expectedRegistrations.map(({ caseId, testName }) => ({
    caseId,
    testName,
  }));
}

/**
 * CLI entrypoint for the generic Trellis integration test runner.
 *
 * This function returns the desired process exit code so tests and wrappers can
 * call it directly. The module only calls `Deno.exit` from the `import.meta.main`
 * branch below.
 */
export async function main(
  args: readonly string[] = Deno.args,
): Promise<number> {
  try {
    return await runTrellisIntegrationTests({ args });
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    return 1;
  }
}

if (import.meta.main) {
  Deno.exit(await main());
}

function parseRunnerArgs(args: readonly string[]): ParsedRunnerArgs {
  const fixtureFilters: string[] = [];
  const caseFilters: string[] = [];
  const coverageFilters: string[] = [];
  let coverageDir: string | undefined;
  let configPath: string | undefined;
  let skipConformance = false;
  let inventoryOnly = false;
  let parallel = false;
  let jobs: number | undefined;
  const denoTestArgs: string[] = [];
  let help = false;

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--") {
      denoTestArgs.push(...args.slice(index + 1));
      break;
    }
    if (arg === "--help" || arg === "-h") {
      help = true;
    } else if (arg === "--config") {
      configPath = setSingleValue(
        configPath,
        readFlagValue(args, index, arg),
        arg,
      );
      index += 1;
    } else if (arg.startsWith("--config=")) {
      configPath = setSingleValue(
        configPath,
        readInlineFlagValue(arg, "--config"),
        "--config",
      );
    } else if (arg === "--fixture") {
      fixtureFilters.push(readFlagValue(args, index, arg));
      index += 1;
    } else if (arg.startsWith("--fixture=")) {
      fixtureFilters.push(readInlineFlagValue(arg, "--fixture"));
    } else if (arg === "--case") {
      caseFilters.push(readFlagValue(args, index, arg));
      index += 1;
    } else if (arg.startsWith("--case=")) {
      caseFilters.push(readInlineFlagValue(arg, "--case"));
    } else if (arg === "--coverage") {
      coverageFilters.push(readFlagValue(args, index, arg));
      index += 1;
    } else if (arg.startsWith("--coverage=")) {
      coverageFilters.push(readInlineFlagValue(arg, "--coverage"));
    } else if (arg === "--coverage-dir") {
      coverageDir = setSingleValue(
        coverageDir,
        readFlagValue(args, index, arg),
        arg,
      );
      index += 1;
    } else if (arg.startsWith("--coverage-dir=")) {
      coverageDir = setSingleValue(
        coverageDir,
        readInlineFlagValue(arg, "--coverage-dir"),
        "--coverage-dir",
      );
    } else if (arg === "--parallel") {
      parallel = true;
    } else if (arg === "--jobs") {
      jobs = parsePositiveInteger(readFlagValue(args, index, arg), arg);
      index += 1;
    } else if (arg.startsWith("--jobs=")) {
      jobs = parsePositiveInteger(readInlineFlagValue(arg, "--jobs"), "--jobs");
    } else if (arg === "--deno-test-arg") {
      denoTestArgs.push(readFlagValue(args, index, arg));
      index += 1;
    } else if (arg.startsWith("--deno-test-arg=")) {
      denoTestArgs.push(readInlineFlagValue(arg, "--deno-test-arg"));
    } else if (arg === "--skip-conformance") {
      skipConformance = true;
    } else if (arg === "--inventory-only") {
      inventoryOnly = true;
    } else {
      throw new Error(`unknown Trellis integration runner argument: ${arg}`);
    }
  }

  if (jobs !== undefined && jobs !== 1) {
    throw new Error("--jobs must be 1 for fixed shared protocol subjects");
  }
  return {
    configPath,
    fixtureFilters,
    caseFilters,
    coverageFilters,
    coverageDir,
    skipConformance,
    inventoryOnly,
    parallel,
    jobs,
    denoTestArgs,
    help,
  };
}

function coveragePaths(
  dir: string,
  cwd: string,
): {
  readonly dir: string;
  readonly rawDir: string;
  readonly lcovPath: string;
} {
  const resolvedDir = isAbsolute(dir) ? dir : resolve(cwd, dir);
  return {
    dir: resolvedDir,
    rawDir: resolve(resolvedDir, "raw"),
    lcovPath: resolve(resolvedDir, "lcov.info"),
  };
}

async function writeCoverageReport(
  coverage: ReturnType<typeof coveragePaths> | undefined,
  reporter: TrellisIntegrationRunnerOptions["coverageReporter"],
): Promise<void> {
  if (coverage === undefined) return;
  const writeReport = reporter ?? runDenoCoverageCommand;
  await writeReport({ rawDir: coverage.rawDir, lcovPath: coverage.lcovPath });
}

async function runDenoCoverageCommand(coverage: {
  readonly rawDir: string;
  readonly lcovPath: string;
}): Promise<void> {
  const process = new Deno.Command(Deno.execPath(), {
    args: [
      "coverage",
      "--lcov",
      `--output=${coverage.lcovPath}`,
      coverage.rawDir,
    ],
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  const status = await process.spawn().status;
  if (status.code !== 0) {
    throw new Error(`deno coverage failed with exit code ${status.code}`);
  }
}

async function removeIfExists(
  path: string,
  options?: Deno.RemoveOptions,
): Promise<void> {
  try {
    await Deno.remove(path, options);
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) throw error;
  }
}

function readFlagValue(
  args: readonly string[],
  index: number,
  flag: string,
): string {
  const value = args[index + 1];
  if (value === undefined || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function readInlineFlagValue(arg: string, flag: string): string {
  const value = arg.slice(`${flag}=`.length);
  if (value === "") {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function setSingleValue(
  previous: string | undefined,
  value: string,
  flag: string,
): string {
  if (previous !== undefined) {
    throw new Error(`${flag} may only be provided once`);
  }
  return value;
}

function parsePositiveInteger(value: string, flag: string): number {
  if (!/^[1-9]\d*$/.test(value)) {
    throw new Error(`${flag} requires a positive integer`);
  }
  return Number(value);
}

async function loadRunnerConfig(
  configPath: string | undefined,
  cwd: string,
): Promise<LoadedRunnerConfig> {
  if (configPath === undefined) {
    throw new Error("--config is required");
  }

  const location = resolveConfigLocation(configPath, cwd);
  const moduleValue: unknown = await import(location.specifier);
  if (!isRecord(moduleValue) || !isRunnerConfig(moduleValue.default)) {
    throw new Error(
      `Trellis integration runner config ${configPath} must export a default config`,
    );
  }

  return { config: moduleValue.default, baseDir: location.baseDir };
}

function resolveConfigLocation(
  configPath: string,
  cwd: string,
): { readonly specifier: string; readonly baseDir: string } {
  if (configPath.startsWith("file:")) {
    const url = new URL(configPath);
    return { specifier: url.href, baseDir: dirname(fromFileUrl(url)) };
  }

  if (/^[A-Za-z][A-Za-z\d+.-]*:\/\//.test(configPath)) {
    throw new Error("--config must be a local path or file URL");
  }

  const absolutePath = isAbsolute(configPath)
    ? configPath
    : resolve(cwd, configPath);
  return {
    specifier: toFileUrl(absolutePath).href,
    baseDir: dirname(absolutePath),
  };
}

function isRunnerConfig(
  value: unknown,
): value is TrellisIntegrationRunnerConfig {
  if (
    !isRecord(value) || !isRecord(value.runtime) || !Array.isArray(value.cases)
  ) {
    return false;
  }

  if (
    value.conformance !== undefined && typeof value.conformance !== "function"
  ) {
    return false;
  }

  return value.cases.every(isIntegrationCase);
}

function isIntegrationCase(value: unknown): value is TrellisIntegrationCase {
  if (!isRecord(value)) return false;
  if (
    typeof value.id !== "string" || typeof value.fixture !== "string" ||
    typeof value.file !== "string" || typeof value.testName !== "string"
  ) {
    return false;
  }

  return value.coverage === undefined ||
    (Array.isArray(value.coverage) &&
      value.coverage.every((tag) => typeof tag === "string"));
}

function isTypeScriptIntegrationTestEvent(
  value: unknown,
): value is TypeScriptIntegrationTestEvent {
  if (
    !isRecord(value) || value.event !== "integration-case" ||
    value.language !== "typescript" ||
    typeof value.caseId !== "string" || typeof value.testName !== "string" ||
    typeof value.timestamp !== "string" ||
    !Number.isFinite(Date.parse(value.timestamp)) ||
    !["registered", "started", "passed", "failed", "ignored"].includes(
      String(value.status),
    )
  ) {
    return false;
  }
  const terminal = value.status === "passed" || value.status === "failed";
  return terminal
    ? typeof value.durationMs === "number" &&
      Number.isFinite(value.durationMs) && value.durationMs >= 0
    : value.durationMs === undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function assertUniqueCaseIds(cases: readonly TrellisIntegrationCase[]): void {
  assertUniqueIds(
    "registered TypeScript integration cases",
    cases.map((entry) => entry.id),
  );
}

function configuredIdentities(
  cases: readonly TrellisIntegrationCase[],
): TypeScriptIntegrationTestIdentity[] {
  return cases.map((entry) => ({
    caseId: entry.id,
    testName: entry.testName,
  }));
}

function identityMap(
  label: string,
  identities: readonly TypeScriptIntegrationTestIdentity[],
): Map<string, string> {
  assertUniqueIds(label, identities.map((identity) => identity.caseId));
  return new Map(
    identities.map(({ caseId, testName }) => [caseId, testName]),
  );
}

function assertConfiguredEvent(
  configured: ReadonlyMap<string, string>,
  event: TypeScriptIntegrationTestEvent,
): void {
  const testName = configured.get(event.caseId);
  if (testName === undefined) {
    throw new Error(`unexpected TypeScript integration case: ${event.caseId}`);
  }
  if (event.testName !== testName) {
    throw new Error(
      `TypeScript integration case ${event.caseId} reported test name ${event.testName}; expected ${testName}`,
    );
  }
}

function assertUniqueIds(label: string, ids: readonly string[]): void {
  const duplicates = ids.filter((id, index) => ids.indexOf(id) !== index)
    .toSorted();
  if (duplicates.length > 0) {
    throw new Error(`${label} contain duplicates: ${duplicates.join(", ")}`);
  }
}

function assertSameIds(
  label: string,
  expected: ReadonlySet<string>,
  actual: ReadonlySet<string>,
): void {
  const missing = [...expected].filter((id) => !actual.has(id)).toSorted();
  const unexpected = [...actual].filter((id) => !expected.has(id)).toSorted();
  if (missing.length > 0 || unexpected.length > 0) {
    throw new Error(
      `${label} differ from config: missing [${
        missing.join(", ")
      }], unexpected [${unexpected.join(", ")}]`,
    );
  }
}

function validateFilters(
  options: ParsedRunnerArgs,
  cases: readonly TrellisIntegrationCase[],
): void {
  const fixtures = new Set(cases.map((caseEntry) => caseEntry.fixture));
  const caseIds = new Set(cases.map((caseEntry) => caseEntry.id));
  const coverageTags = new Set(
    cases.flatMap((caseEntry) => [...(caseEntry.coverage ?? [])]),
  );

  rejectUnknownFilters("fixture", options.fixtureFilters, fixtures);
  rejectUnknownFilters("case", options.caseFilters, caseIds);
  rejectUnknownFilters("coverage", options.coverageFilters, coverageTags);
}

function rejectUnknownFilters(
  kind: string,
  filters: readonly string[],
  validValues: ReadonlySet<string>,
): void {
  const unknown = filters.filter((filter) => !validValues.has(filter))
    .toSorted();
  if (unknown.length > 0) {
    throw new Error(
      `unknown Trellis integration ${kind} filter(s): ${unknown.join(", ")}`,
    );
  }
}

function selectCases(
  options: ParsedRunnerArgs,
  cases: readonly TrellisIntegrationCase[],
  baseDir: string,
): SelectedCases {
  const fixtureFilters = new Set(options.fixtureFilters);
  const caseFilters = new Set(options.caseFilters);
  const coverageFilters = new Set(options.coverageFilters);
  const files: string[] = [];
  const caseIds: string[] = [];
  const seenFiles = new Set<string>();
  const testNames: string[] = [];

  for (const caseEntry of cases) {
    if (
      hasFilters(options) && !fixtureFilters.has(caseEntry.fixture) &&
      !caseFilters.has(caseEntry.id) &&
      !(caseEntry.coverage ?? []).some((tag) => coverageFilters.has(tag))
    ) {
      continue;
    }

    const file = resolveCaseFile(baseDir, caseEntry.file);
    if (!seenFiles.has(file)) {
      files.push(file);
      seenFiles.add(file);
    }
    caseIds.push(caseEntry.id);
    testNames.push(caseEntry.testName);
  }

  const selectedFiles = new Set(files);
  const registrations = cases.filter((entry) =>
    selectedFiles.has(resolveCaseFile(baseDir, entry.file))
  ).map((entry) => ({ caseId: entry.id, testName: entry.testName }));
  const selectedCases = caseIds.map((caseId, index) => ({
    caseId,
    testName: testNames[index],
  }));
  return {
    caseIds,
    cases: selectedCases,
    registrations,
    files,
    testNames,
  };
}

function resolveCaseFile(baseDir: string, file: string): string {
  return isAbsolute(file) ? file : resolve(baseDir, file);
}

function hasFilters(options: ParsedRunnerArgs): boolean {
  return options.fixtureFilters.length > 0 || options.caseFilters.length > 0 ||
    options.coverageFilters.length > 0;
}

function denoTestArgs(args: {
  readonly parallel: boolean;
  readonly extraArgs: readonly string[];
  readonly files: readonly string[];
  readonly filter: string | undefined;
}): readonly string[] {
  const denoArgs = ["test"];
  denoArgs.push(...args.extraArgs);
  if (args.parallel) {
    denoArgs.push("--parallel");
  }
  if (args.filter !== undefined) {
    denoArgs.push("--filter", args.filter);
  }
  denoArgs.push(...args.files);
  return denoArgs;
}

function testNameFilter(testNames: readonly string[]): string | undefined {
  if (testNames.length === 0) return undefined;
  return `/^(?:${testNames.map(escapeRegExp).join("|")})$/`;
}

function escapeRegExp(value: string): string {
  return value.replaceAll(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

async function runDenoTestCommand(command: {
  readonly executable: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly env?: Record<string, string>;
}): Promise<number> {
  const process = new Deno.Command(command.executable, {
    args: [...command.args],
    cwd: command.cwd,
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
    env: command.env === undefined
      ? undefined
      : { ...Deno.env.toObject(), ...command.env },
  });
  const status = await process.spawn().status;
  return status.code;
}

function helpText(): string {
  return `Run Trellis integration tests.

Usage:
  deno run -A jsr:@qlever-llc/trellis-test/integration/runner --config trellis.integration.ts [options]

Options:
  --config <path>       Required. Module exporting default runner config.
  --fixture <fixture>   Select cases by fixture. May be repeated.
  --case <case-id>      Select a case id. May be repeated.
  --coverage <tag>      Select cases by coverage tag. May be repeated.
  --coverage-dir <dir>  Collect Deno coverage under <dir>/raw and write <dir>/lcov.info.
  --inventory-only      Evaluate selected modules and validate registrations without running tests.
  --parallel            Run selected tests with one shared Trellis runtime.
  --jobs <n>            Shared mode requires 1 for fixed protocol subjects.
  --deno-test-arg <arg> Pass one argument through to child deno test. May be repeated.
  --                    Pass all remaining arguments through to child deno test.
  --skip-conformance    Skip the optional config conformance hook.
  --help, -h            Print this help text.`;
}
