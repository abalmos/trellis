---
title: Testing Patterns
description: Trellis testing policy for smallest real boundaries and live integration coverage.
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

### Smallest Real Boundary

Test an invariant at the smallest real boundary that proves it. Use complete
live Trellis integration when transport, authorization, cross-language behavior,
process lifecycle, restart, NATS, or distributed coordination is part of the
invariant. Use real component or adapter integration tests for deterministic
transaction, repository, projection, reducer, and state-machine invariants that
do not require the complete runtime. Such tests use the production
implementation and a real backing adapter such as temporary or in-memory SQLite,
not a fake implementation of the boundary under test.

Do not prove Trellis behavior with fake NATS, fake Hono, fake storage, fake
runtime, fake auth, fake generated clients, or fake control-plane responders.
Those tests create a second implementation of Trellis and drift from the
runtime.

When behavior is not reachable through a public generated API, test the
authoritative production component directly at its real storage or transport
adapter boundary. Do not expose runtime internals, add generic chaos frameworks,
or add synthetic fault hooks solely for tests.

### Test Ownership

Executable Rust and Deno tests are the catalog. Cross-language behavior is
proved by the smallest live test that crosses the real transport boundary;
Rust-only runtime behavior remains in ordinary Rust integration tests.

### Case-Owned Runtime

A live subsystem run owns one managed NATS server. Every selected case receives
its own NATS account and starts its own `trellis-server` process and SQLite set.
Built-in Jobs and Event Log run as subsystems of that case-owned process. Cases
keep authored contract ids, participant ids, action names, versions, and
protocol subjects fixed; account isolation prevents those fixed protocol
subjects and participant identities from colliding across cases. Run and case
slugs still isolate physical deployment ids, resource keys, state keys, durable
names, and domain records.

Rust and TypeScript live tests use ordinary Rust and Deno discovery. The normal
Check runs both discovered suites against real Trellis infrastructure. Focused
local runs select tests through the native Rust and Deno test runners rather
than a second registry or scheduler.

### Smaller Test Boundary

Real component and adapter integration tests may cover transaction, repository,
projection, reducer, and state-machine invariants without starting the complete
runtime. Pure unit tests remain appropriate for:

- pure parser, codec, canonicalization, crypto vector, schema pointer, or error
  serialization checks
- package export/import, publishing, generated artifact, or type-surface smoke
  checks
- CLI argument parsing and release-tool planning checks
- tiny UI copy or page-state helpers

If a smaller test needs fake Trellis runtime pieces to pass, move it to live
integration or replace it with a real component boundary. Delete it after live
TS/Rust coverage proves the same behavior if no independent smaller invariant
remains.

Retained smaller tests should make the proving boundary clear. Comments, when
needed, should name the deterministic invariant, not say that behavior is merely
"private" or "not public". Current Trellis behavior is the behavior to protect;
the question is which smallest real boundary proves it honestly.

### Verification Practice

Verification has three explicit tiers:

- **Tier 1, inner loop:** format changed files and run the smallest affected
  package tests, type checks, and live fixture or case that prove the change.
- **Tier 2, phase gate:** run preparation when generated artifacts may change,
  workspace formatting, lint and documentation checks, affected package suites,
  and the matching TypeScript and Rust live tests.
- **Tier 3, integrated gate:** the normal `Check` workflow owns formatting,
  lint/type checks, generated freshness, package tests, and the complete
  unfiltered live suites with no hidden skips. The release workflow does not
  repeat those checks; it verifies release metadata and the exact packages,
  archives, images, and publication inputs assembled from a green `rs` base.
  Trellis supports current stable Rust only; no older compiler compatibility is
  promised.

Tier 1 and Tier 2 results are scoped evidence, not a full verification claim. Do
not rerun Tier 3 after every implementation track; run it once from the final
integrated source state before preparing a release.
