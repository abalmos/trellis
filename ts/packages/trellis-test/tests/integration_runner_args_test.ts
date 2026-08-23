import {
  assertEquals,
  assertRejects,
  assertStringIncludes,
  assertThrows,
} from "@std/assert";
import { join } from "@std/path";
import {
  parseTypeScriptIntegrationEvents,
  reconcileTypeScriptIntegrationEvents,
  reconcileTypeScriptIntegrationInventory,
  runTrellisIntegrationTests,
  type TrellisIntegrationRunnerConfig,
  type TrellisIntegrationRunnerOptions,
} from "../src/integration/runner.ts";
import { TRELLIS_TEST_EVENTS_ENV } from "../src/integration/runtime.ts";
import { TRELLIS_TEST_SHARED_RUNTIME_ENV } from "../src/integration/shared_runtime_protocol.ts";

type RunnerCommand = Parameters<
  NonNullable<TrellisIntegrationRunnerOptions["commandRunner"]>
>[0];

const runtime = {
  trellis: {
    command: { cmd: "deno", args: ["run", "./trellis.ts"] },
  },
} satisfies TrellisIntegrationRunnerConfig["runtime"];

const config = {
  runtime,
  cases: [
    {
      id: "billing.invoice-created",
      fixture: "billing",
      file: "integration/billing/invoice_created.integration_test.ts",
      testName: "billing.invoice-created publishes an invoice event",
      coverage: ["rpc", "events"],
    },
    {
      id: "billing.invoice-refunded",
      fixture: "billing",
      file: "integration/billing/invoice_refunded.integration_test.ts",
      testName: "billing.invoice-refunded publishes refund (v2)",
      coverage: ["events"],
    },
    {
      id: "orders.created",
      fixture: "orders",
      file: "integration/orders/orders.integration_test.ts",
      testName: "orders.created creates state [smoke]",
      coverage: ["state"],
    },
  ],
} satisfies TrellisIntegrationRunnerConfig;

function mockRun(
  command: RunnerCommand,
  cases: readonly { caseId: string; testName: string }[],
  commands?: RunnerCommand[],
  code = 0,
): Promise<number> {
  const path = command.env?.[TRELLIS_TEST_EVENTS_ENV];
  if (path === undefined) throw new Error("runner did not supply event path");
  const terminal = code === 0 ? "passed" : "failed";
  Deno.writeTextFileSync(
    path,
    cases.flatMap(({ caseId, testName }) =>
      ["registered", "started", terminal].map((status) =>
        JSON.stringify({
          event: "integration-case",
          language: "typescript",
          caseId,
          testName,
          status,
          timestamp: "2026-07-30T00:00:00.000Z",
          ...(status === "passed" || status === "failed"
            ? { durationMs: 5 }
            : {}),
        })
      )
    ).join("\n") + "\n",
  );
  if (commands !== undefined) {
    const { [TRELLIS_TEST_EVENTS_ENV]: _, ...env } = command.env ?? {};
    const { env: _originalEnv, ...withoutEnv } = command;
    commands.push(
      Object.keys(env).length === 0 ? withoutEnv : {
        ...withoutEnv,
        env,
      },
    );
  }
  return Promise.resolve(code);
}

function mockInventory(
  command: RunnerCommand,
  cases: readonly { caseId: string; testName: string }[],
  commands?: RunnerCommand[],
  code = 0,
): Promise<number> {
  const path = command.env?.[TRELLIS_TEST_EVENTS_ENV];
  if (path === undefined) throw new Error("runner did not supply event path");
  Deno.writeTextFileSync(
    path,
    cases.map(({ caseId, testName }) =>
      JSON.stringify({
        event: "integration-case",
        language: "typescript",
        caseId,
        testName,
        status: "registered",
        timestamp: "2026-07-30T00:00:00.000Z",
      })
    ).join("\n") + "\n",
  );
  if (commands !== undefined) {
    const { [TRELLIS_TEST_EVENTS_ENV]: _, ...env } = command.env ?? {};
    const { env: _originalEnv, ...withoutEnv } = command;
    commands.push(
      Object.keys(env).length === 0 ? withoutEnv : {
        ...withoutEnv,
        env,
      },
    );
  }
  return Promise.resolve(code);
}

Deno.test("runner prints help without requiring config", async () => {
  const output: string[] = [];
  const code = await runTrellisIntegrationTests({
    args: ["--help"],
    output: {
      log(message) {
        output.push(message);
      },
    },
  });

  assertEquals(code, 0);
  assertEquals(output.length, 1);
  assertStringIncludes(output[0], "--config <path>");
  assertStringIncludes(output[0], "--inventory-only");
});

Deno.test("runner validates required and malformed arguments", async () => {
  await assertRejects(
    () => runTrellisIntegrationTests({ args: [] }),
    Error,
    "--config is required",
  );
  await assertRejects(
    () => runTrellisIntegrationTests({ args: ["--config"] }),
    Error,
    "--config requires a value",
  );
  await assertRejects(
    () =>
      runTrellisIntegrationTests({
        args: ["--jobs", "0"],
        config,
        commandRunner: () => Promise.resolve(0),
      }),
    Error,
    "--jobs requires a positive integer",
  );
  await assertRejects(
    () =>
      runTrellisIntegrationTests({
        args: ["--jobs=1.5"],
        config,
        commandRunner: () => Promise.resolve(0),
      }),
    Error,
    "--jobs requires a positive integer",
  );
});

Deno.test("runner validates unknown case filters clearly", async () => {
  await assertRejects(
    () =>
      runTrellisIntegrationTests({
        args: ["--fixture", "missing"],
        config,
        commandRunner: () => Promise.resolve(0),
      }),
    Error,
    "unknown Trellis integration fixture filter(s): missing",
  );
  await assertRejects(
    () =>
      runTrellisIntegrationTests({
        args: ["--case=missing.case"],
        config,
        commandRunner: () => Promise.resolve(0),
      }),
    Error,
    "unknown Trellis integration case filter(s): missing.case",
  );
  await assertRejects(
    () =>
      runTrellisIntegrationTests({
        args: ["--coverage", "missing"],
        config,
        commandRunner: () => Promise.resolve(0),
      }),
    Error,
    "unknown Trellis integration coverage filter(s): missing",
  );
});

Deno.test("runner loads config modules and resolves case files relative to config", async () => {
  const tempDir = await Deno.makeTempDir();
  const configDir = join(tempDir, "service-repo", "test-config");
  await Deno.mkdir(configDir, { recursive: true });
  const configPath = join(configDir, "trellis.integration.ts");
  await Deno.writeTextFile(
    configPath,
    `export default {
      runtime: {
        trellis: { command: { cmd: "deno", args: ["run", "./trellis.ts"] } },
      },
      cases: [{
        id: "billing.invoice-created",
        fixture: "billing",
        file: "../integration/billing_test.ts",
        testName: "billing.invoice-created publishes (v1)",
        coverage: ["events"],
      }],
    };`,
  );
  const commands: RunnerCommand[] = [];

  const code = await runTrellisIntegrationTests({
    args: ["--config", configPath, "--fixture", "billing"],
    cwd: tempDir,
    commandRunner: (command) =>
      mockRun(command, [{
        caseId: "billing.invoice-created",
        testName: "billing.invoice-created publishes (v1)",
      }], commands),
  });

  assertEquals(code, 0);
  assertEquals(commands.length, 1);
  assertEquals(commands[0].cwd, tempDir);
  assertEquals(commands[0].args, [
    "test",
    "--filter",
    "/^(?:billing\\.invoice-created publishes \\(v1\\))$/",
    join(configDir, "..", "integration", "billing_test.ts"),
  ]);
});

Deno.test("runner selects cases by filters and deduplicates files in case order", async () => {
  const cwd = Deno.cwd();
  const commands: RunnerCommand[] = [];

  const code = await runTrellisIntegrationTests({
    args: ["--coverage", "events"],
    config: {
      runtime,
      cases: [
        {
          id: "first",
          fixture: "shared",
          file: "integration/shared.integration_test.ts",
          testName: "shared first [case]",
          coverage: ["events"],
        },
        {
          id: "second",
          fixture: "shared",
          file: "integration/shared.integration_test.ts",
          testName: "shared second +case",
          coverage: ["events"],
        },
        {
          id: "third",
          fixture: "other",
          file: "integration/other.integration_test.ts",
          testName: "other third",
          coverage: ["state"],
        },
      ],
    },
    commandRunner: (command) =>
      mockRun(command, [
        { caseId: "first", testName: "shared first [case]" },
        { caseId: "second", testName: "shared second +case" },
      ], commands),
  });

  assertEquals(code, 0);
  assertEquals(commands[0].args, [
    "test",
    "--filter",
    "/^(?:shared first \\[case\\]|shared second \\+case)$/",
    join(cwd, "integration", "shared.integration_test.ts"),
  ]);
});

Deno.test("runner filters serial runs and passes child deno test arguments", async () => {
  const cwd = Deno.cwd();
  const commands: RunnerCommand[] = [];

  const code = await runTrellisIntegrationTests({
    args: ["--deno-test-arg", "-A", "--", "-c", "deno.json"],
    denoTestArgs: ["--allow-read"],
    config: {
      runtime,
      denoTestArgs: ["--quiet"],
      cases: [
        {
          id: "billing.invoice-created",
          fixture: "billing",
          file: "integration/billing/invoice_created.integration_test.ts",
          testName: "billing.invoice-created publishes an invoice event",
        },
      ],
    },
    commandRunner: (command) =>
      mockRun(command, [{
        caseId: "billing.invoice-created",
        testName: "billing.invoice-created publishes an invoice event",
      }], commands),
  });

  assertEquals(code, 0);
  assertEquals(commands[0].args, [
    "test",
    "--quiet",
    "--allow-read",
    "-A",
    "-c",
    "deno.json",
    "--filter",
    "/^(?:billing\\.invoice-created publishes an invoice event)$/",
    join(cwd, "integration", "billing", "invoice_created.integration_test.ts"),
  ]);
});

Deno.test("runner collects Deno coverage only when requested", async () => {
  const cwd = Deno.cwd();
  const commands: RunnerCommand[] = [];
  const reports: { rawDir: string; lcovPath: string }[] = [];

  const code = await runTrellisIntegrationTests({
    args: ["--coverage-dir", "coverage/live-integration"],
    config: {
      runtime,
      cases: [
        {
          id: "billing.invoice-created",
          fixture: "billing",
          file: "integration/billing/invoice_created.integration_test.ts",
          testName: "billing.invoice-created publishes an invoice event",
        },
      ],
    },
    commandRunner: (command) =>
      mockRun(
        command,
        [{
          caseId: "billing.invoice-created",
          testName: "billing.invoice-created publishes an invoice event",
        }],
        commands,
        7,
      ),
    coverageReporter(coverage) {
      reports.push(coverage);
      return Promise.resolve();
    },
  });

  assertEquals(code, 7);
  assertEquals(commands[0].args, [
    "test",
    `--coverage=${join(cwd, "coverage", "live-integration", "raw")}`,
    "--filter",
    "/^(?:billing\\.invoice-created publishes an invoice event)$/",
    join(cwd, "integration", "billing", "invoice_created.integration_test.ts"),
  ]);
  assertEquals(reports, [{
    rawDir: join(cwd, "coverage", "live-integration", "raw"),
    lcovPath: join(cwd, "coverage", "live-integration", "lcov.info"),
  }]);
});

Deno.test("runner constructs parallel commands with shared host env and DENO_JOBS", async () => {
  const commands: RunnerCommand[] = [];
  const startedRuntime: TrellisIntegrationRunnerConfig["runtime"][] = [];
  const assignments: {
    id: string;
    namespacePrefix?: string;
    classification?: string;
  }[][] = [];
  let stopCalls = 0;

  const code = await runTrellisIntegrationTests({
    args: ["--parallel", "--jobs", "1", "--case", "orders.created"],
    config,
    commandRunner: (command) =>
      mockRun(
        command,
        [{
          caseId: "orders.created",
          testName: "orders.created creates state [smoke]",
        }],
        commands,
        7,
      ),
    sharedRuntimeHostStarter(args) {
      startedRuntime.push(args.runtime);
      assignments.push([...args.assignments]);
      return Promise.resolve({
        manifestPath: "/tmp/manifest.json",
        env: { [TRELLIS_TEST_SHARED_RUNTIME_ENV]: "/tmp/manifest.json" },
        stop() {
          stopCalls += 1;
          return Promise.resolve();
        },
      });
    },
  });

  assertEquals(code, 7);
  assertEquals(startedRuntime, [runtime]);
  assertEquals(assignments, [[{
    id: "orders.created",
    namespacePrefix: "ts",
    classification: undefined,
  }]]);
  assertEquals(stopCalls, 1);
  assertEquals(commands.length, 1);
  assertEquals(commands[0].args, [
    "test",
    "--parallel",
    "--filter",
    "/^(?:orders\\.created creates state \\[smoke\\])$/",
    join(Deno.cwd(), "integration", "orders", "orders.integration_test.ts"),
  ]);
  assertEquals(commands[0].env, {
    [TRELLIS_TEST_SHARED_RUNTIME_ENV]: "/tmp/manifest.json",
    DENO_JOBS: "1",
  });
});

Deno.test("runner skips optional conformance hook when requested", async () => {
  let conformanceCalls = 0;

  await runTrellisIntegrationTests({
    args: ["--skip-conformance"],
    config: {
      ...config,
      conformance() {
        conformanceCalls += 1;
      },
    },
    commandRunner: (command) =>
      mockRun(
        command,
        config.cases.map((entry) => ({
          caseId: entry.id,
          testName: entry.testName,
        })),
      ),
  });

  assertEquals(conformanceCalls, 0);
});

Deno.test("runner reconciles focused lifecycle events into Rust-shaped results", () => {
  assertEquals(
    reconcileTypeScriptIntegrationEvents(
      [
        { caseId: "first", testName: "first test" },
        { caseId: "second", testName: "second test" },
      ],
      [{ caseId: "second", testName: "second test" }],
      [
        {
          event: "integration-case",
          language: "typescript",
          caseId: "first",
          testName: "first test",
          status: "registered",
          timestamp: "2026-07-30T00:00:00.000Z",
        },
        ...["registered", "started", "passed"].map((status) => ({
          event: "integration-case" as const,
          language: "typescript" as const,
          caseId: "second",
          testName: "second test",
          status: status as "registered" | "started" | "passed",
          timestamp: "2026-07-30T00:00:00.000Z",
          ...(status === "passed" ? { durationMs: 8 } : {}),
        })),
      ],
      [
        { caseId: "first", testName: "first test" },
        { caseId: "second", testName: "second test" },
      ],
    ),
    {
      event: "typescript-integration-results",
      registered: 2,
      selected: 1,
      passed: 1,
      failed: 0,
      ignored: 0,
      tests: [{ name: "second", status: "passed" }],
    },
  );
});

Deno.test("runner rejects mismatched and swapped names in full and focused runs", () => {
  const lifecycle = (caseId: string, testName: string) =>
    ["registered", "started", "passed"].map((status) => ({
      event: "integration-case" as const,
      language: "typescript" as const,
      caseId,
      testName,
      status: status as "registered" | "started" | "passed",
      timestamp: "2026-07-30T00:00:00.000Z",
      ...(status === "passed" ? { durationMs: 1 } : {}),
    }));
  const configured = [
    { caseId: "first", testName: "first test" },
    { caseId: "second", testName: "second test" },
  ];

  assertThrows(
    () =>
      reconcileTypeScriptIntegrationEvents(
        configured,
        configured,
        [
          ...lifecycle("first", "second test"),
          ...lifecycle("second", "first test"),
        ],
      ),
    Error,
    "TypeScript integration case first reported test name second test; expected first test",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationEvents(
        configured,
        [configured[1]],
        [
          lifecycle("first", "first test")[0],
          ...lifecycle("second", "wrong focused test"),
        ],
        configured,
      ),
    Error,
    "TypeScript integration case second reported test name wrong focused test; expected second test",
  );
});

Deno.test("inventory evaluates selected modules without bodies or a shared host", async () => {
  const cwd = Deno.cwd();
  const commands: RunnerCommand[] = [];
  const output: string[] = [];
  let conformanceCalls = 0;
  let hostStarts = 0;
  const inventoryConfig = {
    runtime,
    cases: [
      {
        id: "first",
        fixture: "shared",
        file: "integration/shared.integration_test.ts",
        testName: "first test",
      },
      {
        id: "second",
        fixture: "shared",
        file: "integration/shared.integration_test.ts",
        testName: "second test",
      },
      {
        id: "third",
        fixture: "other",
        file: "integration/other.integration_test.ts",
        testName: "third test",
      },
    ],
    conformance() {
      conformanceCalls += 1;
    },
  } satisfies TrellisIntegrationRunnerConfig;

  const code = await runTrellisIntegrationTests({
    args: [
      "--inventory-only",
      "--parallel",
      "--skip-conformance",
      "--case",
      "first",
    ],
    config: inventoryConfig,
    commandRunner: (command) =>
      mockInventory(command, [
        { caseId: "first", testName: "first test" },
        { caseId: "second", testName: "second test" },
      ], commands),
    sharedRuntimeHostStarter() {
      hostStarts += 1;
      return Promise.reject(new Error("inventory started shared host"));
    },
    output: { log: (message) => output.push(message) },
  });

  assertEquals(code, 0);
  assertEquals(conformanceCalls, 1);
  assertEquals(hostStarts, 0);
  assertEquals(commands[0].args, [
    "test",
    "--filter",
    "/a^/",
    join(cwd, "integration", "shared.integration_test.ts"),
  ]);
  assertEquals(JSON.parse(output[0]), {
    event: "typescript-integration-inventory",
    registered: 2,
    tests: [
      { caseId: "first", testName: "first test" },
      { caseId: "second", testName: "second test" },
    ],
  });
});

Deno.test("inventory rejects invalid registrations and propagates child failure", async () => {
  const configured = [
    { caseId: "first", testName: "first test" },
    { caseId: "second", testName: "second test" },
  ];
  const registration = (caseId: string, testName: string) => ({
    event: "integration-case" as const,
    language: "typescript" as const,
    caseId,
    testName,
    status: "registered" as const,
    timestamp: "2026-07-30T00:00:00.000Z",
  });

  assertThrows(
    () =>
      reconcileTypeScriptIntegrationInventory(
        configured,
        configured,
        [registration("first", "second test")],
      ),
    Error,
    "TypeScript integration case first reported test name second test; expected first test",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationInventory(
        configured,
        configured,
        [registration("wrong", "first test")],
      ),
    Error,
    "unexpected TypeScript integration case: wrong",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationInventory(
        configured,
        configured,
        [registration("first", "first test")],
      ),
    Error,
    "missing [second]",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationInventory(
        configured,
        configured,
        [
          registration("first", "first test"),
          registration("first", "first test"),
        ],
      ),
    Error,
    "duplicate TypeScript integration registered event: first",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationInventory(
        configured,
        [configured[0]],
        [{ ...registration("first", "first test"), status: "started" }],
      ),
    Error,
    "inventory emitted TypeScript integration started event: first",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationInventory(
        configured,
        [configured[0]],
        [{
          ...registration("first", "first test"),
          status: "failed",
          durationMs: 1,
        }],
      ),
    Error,
    "inventory emitted TypeScript integration failed event: first",
  );

  const code = await runTrellisIntegrationTests({
    args: ["--inventory-only", "--case", "billing.invoice-created"],
    config,
    commandRunner: (command) =>
      mockInventory(
        command,
        [{
          caseId: "billing.invoice-created",
          testName: "billing.invoice-created publishes an invoice event",
        }],
        undefined,
        9,
      ),
  });
  assertEquals(code, 9);
});

Deno.test("runner rejects incomplete, duplicate, unexpected, and ignored events", () => {
  const event = (
    caseId: string,
    status: "registered" | "started" | "passed" | "failed" | "ignored",
  ) => ({
    event: "integration-case" as const,
    language: "typescript" as const,
    caseId,
    testName: `${caseId} test`,
    status,
    timestamp: "2026-07-30T00:00:00.000Z",
    ...(status === "passed" || status === "failed" ? { durationMs: 5 } : {}),
  });

  assertThrows(
    () =>
      reconcileTypeScriptIntegrationEvents(
        [{ caseId: "case", testName: "case test" }],
        [{ caseId: "case", testName: "case test" }],
        [],
      ),
    Error,
    "registered TypeScript integration cases differ from config: missing [case]",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationEvents(
        [{ caseId: "case", testName: "case test" }],
        [{ caseId: "case", testName: "case test" }],
        [event("case", "registered"), event("case", "registered")],
      ),
    Error,
    "duplicate TypeScript integration registered event",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationEvents(
        [{ caseId: "case", testName: "case test" }],
        [{ caseId: "case", testName: "case test" }],
        [event("other", "registered")],
      ),
    Error,
    "unexpected TypeScript integration case",
  );
  assertThrows(
    () =>
      reconcileTypeScriptIntegrationEvents(
        [{ caseId: "case", testName: "case test" }],
        [{ caseId: "case", testName: "case test" }],
        [event("case", "registered"), event("case", "ignored")],
      ),
    Error,
    "ignored TypeScript integration case",
  );
});

Deno.test("runner validates shared event schema and terminal timing", () => {
  const registered = {
    event: "integration-case",
    language: "typescript",
    caseId: "case",
    testName: "case test",
    status: "registered",
    timestamp: "2026-07-30T00:00:00.000Z",
  };
  const terminal = {
    ...registered,
    status: "failed",
    timestamp: "2026-07-30T00:00:01.000Z",
    durationMs: 12.5,
  };
  const started = {
    ...registered,
    status: "started",
    timestamp: "2026-07-30T00:00:00.500Z",
  };
  assertEquals(
    parseTypeScriptIntegrationEvents(
      `${JSON.stringify(registered)}\n${JSON.stringify(started)}\n${
        JSON.stringify(terminal)
      }\n`,
    ),
    [registered, started, terminal],
  );
  assertThrows(
    () =>
      parseTypeScriptIntegrationEvents(JSON.stringify({
        event: "integration-case",
        language: "typescript",
        caseId: "case",
        testName: "case test",
        status: "passed",
        timestamp: "2026-07-30T00:00:01.000Z",
      })),
    Error,
    "invalid TypeScript integration event",
  );
});

Deno.test("runner preserves exact sibling IDs when one selected case fails", () => {
  const event = (
    caseId: string,
    status: "registered" | "started" | "passed" | "failed",
  ) => ({
    event: "integration-case" as const,
    language: "typescript" as const,
    caseId,
    testName: `${caseId} test`,
    status,
    timestamp: "2026-07-30T00:00:00.000Z",
    ...(status === "passed" || status === "failed" ? { durationMs: 4 } : {}),
  });
  assertEquals(
    reconcileTypeScriptIntegrationEvents(
      [
        { caseId: "sibling.pass", testName: "sibling.pass test" },
        { caseId: "sibling.fail", testName: "sibling.fail test" },
      ],
      [
        { caseId: "sibling.pass", testName: "sibling.pass test" },
        { caseId: "sibling.fail", testName: "sibling.fail test" },
      ],
      [
        ...["registered", "started", "passed"].map((status) =>
          event(
            "sibling.pass",
            status as "registered" | "started" | "passed",
          )
        ),
        ...["registered", "started", "failed"].map((status) =>
          event(
            "sibling.fail",
            status as "registered" | "started" | "failed",
          )
        ),
      ],
    ).tests,
    [
      { name: "sibling.pass", status: "passed" },
      { name: "sibling.fail", status: "failed" },
    ],
  );
});

Deno.test("runner rejects a zero-exit child with incomplete lifecycle events", async () => {
  await assertRejects(
    () =>
      runTrellisIntegrationTests({
        args: ["--case", "billing.invoice-created"],
        config,
        commandRunner: () => Promise.resolve(0),
      }),
    Error,
    "registered TypeScript integration cases differ from config: missing [billing.invoice-created]",
  );
});
