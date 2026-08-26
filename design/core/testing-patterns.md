---
title: Testing Patterns
description: Trellis testing policy for live integration coverage, matrix parity, and rare retained unit tests.
order: 90
---

# Design: Testing Patterns

## Prerequisites

- [trellis-patterns.md](./trellis-patterns.md) - Trellis architecture and
  runtime boundaries
- [../contracts/trellis-api-participants.md](./../contracts/trellis-api-participants.md) -
  contract-owned surfaces and permission derivation

## Design

Trellis behavior is distributed runtime behavior. Tests that prove Trellis
runtime behavior must use a live Trellis control plane and real TypeScript and
Rust client/service paths.

### Live-First Rule

Use live integration tests for Trellis behavior. This includes behavior observed
through HTTP routes, generated RPCs, operations, jobs, feeds, events, transfer,
state, resources, auth, bootstrap, device activation, service deployment,
catalog, permissions, NATS subjects, storage-backed runtime state, retry,
rollback, restart, and failure handling.

Do not prove Trellis behavior with fake NATS, fake Hono, fake storage, fake
runtime, fake auth, fake generated clients, or fake control-plane responders.
Those tests create a second implementation of Trellis and drift from the
runtime.

When a behavior is not reachable through current public test helpers, extend the
test library with the smallest named live-test surface needed to produce or
observe the behavior. Prefer case-scoped helpers in `trellis-test` over generic
chaos frameworks. Examples include a one-shot failure hook, a control-plane
SQLite inspection helper, or a JetStream inspection helper.

### Test Ownership

`integration/client-test-matrix.json` owns externally observable client
interoperability. Every supported client language implements those rows against
the Rust runtime. `integration/rust-runtime-test-matrix.json` owns Rust runtime
correctness and does not require TypeScript twins.

Rules:

- derive runner registration from the matrices; do not maintain shadow skip or
  registration lists
- compare implemented Rust mappings to the compiled executable list and the
  executed result stream
- treat any implemented, uncompiled, or unexecuted case as a release failure
- keep pending rows explicit with a reason and owner
- do not use source-text annotations as proof that a test compiled or ran

### Case-Owned Runtime

A live subsystem run owns one managed NATS server. Every selected case receives
its own NATS account and starts its own `trellis-server` process and SQLite set.
Built-in Jobs and Event Log run as subsystems of that case-owned process. Cases
keep authored contract ids, participant ids, action names, versions, and
protocol subjects fixed; account isolation prevents those fixed protocol
subjects and participant identities from colliding across cases. Run and case
slugs still isolate physical deployment ids, resource keys, state keys, durable
names, and domain records.

Rust executes each case as an exact-test process. The Rust and TypeScript
runners use bounded, machine-relative worker pools; `TRELLIS_INTEGRATION_JOBS`
may override each runner's pool for local diagnosis. Do not restore parallelism
by rewriting semantic identities or by adding test-only protocol behavior.

The normal Check runs semantic subsystem slices as independent GitHub Actions
jobs. Live executables are built once as normal locked release binaries and
distributed to those jobs; live execution does not compile Rust. Subsystem jobs
may run concurrently without a fixed matrix-topology cap. Each selected case
acquires one host-wide slot controlled by `TRELLIS_TEST_HOST_JOBS`, which Check
sets to the measured aggregate bound of eight on the current self-hosted runner.
Custom fixtures borrow that slot when replacing the case runtime and acquire an
additional slot for each concurrently running child process.

`isolated-process` remains matrix documentation for genuinely process- or
deployment-global behavior such as restart, ownership loss, startup migration,
destructive trust rotation, or malformed global configuration. Every such row
records its reason, but it uses the same case-owned process pool as other cases;
feature area alone is never an isolation reason.

Each live run emits machine-readable inventory, result, process-start, and
duration records. Case-owned duration summaries aggregate count, average, and
maximum time by operation and phase so gate-performance claims use retained
measurements rather than terminal-log estimates.

### Unit Test Boundary

Unit tests are the exception. Keep them only when the behavior is not Trellis
runtime behavior:

- pure parser, codec, canonicalization, crypto vector, schema pointer, or error
  serialization checks
- package export/import, publishing, generated artifact, or type-surface smoke
  checks
- CLI argument parsing and release-tool planning checks
- tiny UI copy or page-state helpers

If a unit test needs fake Trellis runtime pieces to pass, it probably belongs in
live integration. Delete it after live TS/Rust coverage proves the same
behavior, or replace it with a smaller pure-function test if a real pure
invariant remains.

Retained unit tests should document why they are not live integration tests. The
comment should name the pure invariant, not say that the behavior is merely
"private" or "not public". Current Trellis behavior is the behavior to protect;
the question is whether it is runtime-observable and therefore belongs in live
integration.

### Verification Practice

Verification has three explicit tiers:

- **Tier 1, inner loop:** format changed files and run the smallest affected
  package tests, type checks, and live fixture or case that prove the change.
- **Tier 2, phase gate:** run preparation when generated artifacts may change,
  workspace formatting, lint and documentation checks, affected package suites,
  and the matching TypeScript and Rust live cases plus matrix conformance.
- **Tier 3, integrated gate:** the normal `Check` workflow owns formatting,
  lint/type checks, generated freshness, package tests, and the complete
  unfiltered live matrices with no hidden skips. The release workflow does not
  repeat those checks; it verifies release metadata and the exact packages,
  archives, images, and publication inputs assembled from a green `rs` base.
  Trellis supports current stable Rust only; no older compiler compatibility is
  promised.

Tier 1 and Tier 2 results are scoped evidence, not a full verification claim. Do
not rerun Tier 3 after every implementation track; run it once from the final
integrated source state before preparing a release.
