# Trellis 0.12 simplification

Short-lived tracker for the `rs` cleanup before the first stable release. This
file is the durable source of truth for the cleanup plan; keep it current as
work lands and delete it when the work is complete.

Repository: `abalmos/trellis` Branch: `rs` Status updated: 2026-08-20

## Working rules

- No backwards-compatibility work for unreleased Trellis behavior.
- Prefer deletion and direct code over new frameworks or registries.
- Live integration remains the primary test style for runtime behavior.
- Unit tests stay for small pure invariants or real adapter behavior that does
  not require the full control plane.
- Test-only code must not change normal production semantics.
- Do cleanup work as direct `rs` commits. Hidden `agent/*` branches may be used
  for Actions validation, but do not create cleanup PRs unless explicitly
  requested.
- Re-read current `rs` before every write; do not overwrite concurrent work.

## 1. Fast feedback / test, check, release

- [x] Keep shared fixed-subject cases serial within one live environment and
      delete the fake machine-adaptive worker setting.
- [x] Build the live executables once and run semantic subsystem slices as
      native parallel Actions jobs using only the prebuilt bundle.
- [x] Run external repository smoke independently from the subsystem matrix.
- [x] Cancel superseded everyday `Check` runs.
- [x] Delete stale nextest/test-governance artifacts and the stale release
      timing baseline.
- [x] Make the Rust live matrix track behavior instead of every compiled
      helper/adapter test.
- [x] Remove the custom release DAG/stage/wave planner and its graph/self-tests.
- [x] Remove the redundant standalone live-inventory pass and fixed `--jobs 20`
      override.
- [x] Add a normal `Check` workflow for everyday `rs` validation.
- [x] Fix clean-checkout SDK generation dependencies and the generated
      demo-contract compile dependency.
- [x] Make `Check` prepare generated SDKs and the embedded login portal once,
      then hand those ignored build inputs to Rust/TS/live jobs as an Actions
      artifact.
- [x] Keep hidden `agent/*` validation branches from triggering the full
      everyday `Check` suite.
- [ ] Finish reducing `release.yml` to release-only packaging/docs/platform
      work; normal correctness belongs in `Check`.
- [ ] Confirm the final `test` / `check` / `release` command split is simple and
      non-overlapping.
- [ ] Record a fresh cold/warm timing baseline after the final test architecture
      settles.

## 2. Real tests only / synthetic failure removal

- [x] Add a narrow real-SQLite Auth rollback test using an actual late
      constraint failure and assert aggregate state, idempotency state, and
      outbox actions all roll back.
- [x] Assert the rollback failure through the production
      `AuthorizationStateError::StorageConflict` mapping rather than
      SQLite-specific error text.
- [x] Remove the Rust demo contract's compile-time dependency on a generated
      TypeScript API artifact.
- [x] Remove the old live Auth transaction-failure case and trigger helper now
      replaced by the real SQLite rollback test.
- [x] Remove the synthetic Auth post-commit dispatch failure, SQLite hook,
      helper, live test, and matrix row.
- [x] Remove provider `fail_next_context_read` / `fail_next_readiness_check`
      switches and the production verifier branch that consumed them.
- [x] Reduce the event-authorization failure live case to real behavior: publish
      a legitimate signed event, corrupt the proof, require JetStream `TERM`,
      and verify the handler is not invoked for the corrupt copy.
- [ ] Audit the remaining `integration-test-hooks`; delete synthetic
      errors/faults and retain only genuine deterministic scheduling barriers
      where needed.
- [ ] Replace avoidable fixed sleeps with observable readiness/state boundaries
      where practical.

Validation:

- Actions run `32329172166` validated commit `e6615be3` against the real SQLite
  rollback test, affected-crate `cargo check`, Clippy with `-D warnings`, and
  Rust live-integration compilation. The packet deleted 375 lines across five
  files with no additions.
- Actions run `32331381130` validated commit `00c9f48f`: provider fault switches
  and verifier injection were deleted, `trellis-rs` check/Clippy/106 library
  tests and integration compilation passed, and the focused real JetStream
  invalid-proof case emitted `TERM` without invoking the handler for the corrupt
  event.

## 3. First public protocol version

- [ ] Treat the current request/event authorization proof wire format as the
      first public `v1`.
- [ ] Remove unreleased `AuthorizationRequestProofV2`,
      `AuthorizationEventProofV2`, V2 input/verified type names, and `*_v2`
      implementation function names.
- [ ] Change unreleased `.v2` proof domains to the first public `.v1` domains.
- [ ] Remove unpublished aliases, fallback subjects, dual read/write paths,
      obsolete parsers/schemas/vectors, and never-released migrations.
- [ ] Keep serialized format-version fields where they are actual wire-version
      identifiers.
- [ ] Prefer unversioned implementation names (`ApiArtifact`, `PermissionAtom`,
      `AuthorizationContext`, `AuthorizationRequestProof`, etc.) until a real
      second public implementation version exists.
- [ ] Regenerate final conformance vectors and generated artifacts after the
      clean break.

### TS protocol WASM boundary

- [ ] Keep the Rust protocol crate as the single authoritative implementation
      for substantial authorization verification; do not duplicate the
      trust/context verifier in TypeScript.
- [ ] Stop committing protocol-WASM binary/bindgen/base64 output as normal
      source state. Build it only for TS tests/package assembly/release and
      transfer it as a CI/package artifact where needed.
- [ ] Make Rust-only `check`/test paths independent of protocol-WASM generation.
- [ ] Make the canonical preparation/build order work from a clean checkout:
      generate SDK dependencies before any WASM build that loads the Rust
      workspace.
- [ ] Replace per-message root + manifest + context JSON round-trips with an
      opaque WASM verified-context handle. Verify each context/trust chain once;
      request/event hot paths verify only their proof and authorization
      requirements against the already-verified context.
- [ ] Preserve manifest/revocation invalidation in the TS provider cache and
      discard/rebuild verified-context handles when their trust epoch changes.
- [ ] Keep simple request/event proof-input construction/signing native in TS;
      WASM is for the substantial verifier, not every cryptographic helper.
- [ ] Validate the boundary with shared Rust/TS conformance vectors and a
      focused repeated-message test that proves one context verification can
      serve many request/event verifications without re-verifying the trust
      chain.

## 4. Ownership / locks

- [x] Give each runtime lease one owner task; remove the shared
      `Arc<Vec<Mutex<HeldLease>>>` renewal model.
- [x] Fix the durable listener concurrent first-registration race by making
      check/create/spawn/insert one registry critical section (`09125056`).
- [x] Make durable-listener teardown/drop ownership deterministic and simple;
      cleanup no longer depends on a Tokio runtime being available (`ed1a40c`).
- [x] Consolidate authorization provider state behind one coherent synchronous
      state lock so trust root, policy floor, manifest, verified-context
      retention, revocations, and provider health transition together
      (`3859764b`).
- [x] Keep per-digest context singleflight separate; normal cache reads remain
      direct and are not actorized.
- [x] Simplify `JobRef` to immutable seed plus concrete waiter/manager state and
      remove boxed callback backends. `NatsJobWaiter::get` now projects the
      complete durable per-job lifecycle so accumulated progress/logs and legal
      transitions no longer depend on a mutable cached snapshot (`73ad6df2`).

Validation:

- Actions run `32333290228` validated the provider-state consolidation: clean
  generation/portal baseline, format/check/Clippy/library and integration
  compilation passed, and the focused real invalid-proof `TERM` live case
  remained green.

## 5. Runtime composition

- [x] Split oversized runtime startup / `platform::start` responsibilities into
      a few explicit phases with clear ownership.
- [x] Split the oversized service runtime facade along real responsibility
      boundaries (bootstrap/lifecycle, routing/operations, resources, event
      consumers/jobs) where doing so reduces coupling.
- [x] Do not introduce a DI container, generic actor framework, or abstract
      lifecycle framework.

## 6. Production / test boundary

- [x] Remove impossible disconnected/test-only `ConnectedServiceRuntime` states;
      make connected runtime values valid by construction.
- [x] Remove raw integration-test methods from production facades where tests
      can use normal product behavior instead.
- [x] Remove `runtime-internals` exposure that exists only to let tests reach
      implementation details.
- [x] Remove `integration-test-scoping` behavior that changes normal production
      semantics; retain only genuine product concepts.
- [x] Stop building production runtime binaries with test-fault features once
      the remaining hooks are gone.

## 7. Jobs and constructors

- [x] Remove the typed `ActiveJob` heartbeat/progress/log callback adapter,
      duplicated active-job state, and its eight-argument constructor
      (`cb0955fe`).
- [x] Remove `JobRef`'s `Arc<dyn Fn>` get/wait/cancel backend and mutable
      snapshot cache; use immutable seed + concrete `NatsJobWaiter` + concrete
      `JobManager`, with complete durable lifecycle projection (`73ad6df2`).
- [x] Collapse the remaining Jobs function/wrapper ladder to one implementation
      plus a small cohesive execution context/hooks where needed.
- [x] Remove duplicate contract metadata from `ServiceConnectOptions` /
      generated service contract evidence; generated contract evidence must have
      one source of truth.
- [x] Apply the same constructor simplification to device connection paths where
      equivalent duplication exists.
- [x] Collapse combinatorial operation-registration callback variants to one
      provider interface; generated code adapts to it.

Validation:

- Actions run `32331902316` validated the JobRef packet:
  format/check/Clippy/library/integration compilation passed, and the focused
  live `jobs_terminal_local_job_edges_and_admin_rpcs` case verified `wait()`,
  stateless durable `get()` including progress/log reconstruction, and terminal
  `cancel()`.

## 8. Names and lint causes

- [x] Delete the custom release lint-exception registry.
- [x] Remove one concrete `too_many_arguments` cause with the `ActiveJob`
      callback-adapter deletion.
- [x] Remove the `client::client` module inception and its Clippy suppression by
      naming the private implementation module `connection` (`88da68fc`).
- [x] Shrink `CallError`, `ServiceRuntimeError`, and
      `OperationTransferStartError` by boxing genuinely large cold-path
      payloads; remove the corresponding `result_large_err` /
      `large_enum_variant` suppressions (`65dae41f`).
- [ ] Fix remaining `too_many_arguments` causes through cohesive
      inputs/ownership instead of adding suppressions.
- [ ] Fix remaining `result_large_err`, `large_enum_variant`, and test-only
      `dead_code` causes where the design can be made simpler.
- [ ] Shorten long names by using module context and splitting oversized
      modules, not by inventing abbreviations.

## 9. Final gate

- [ ] One understandable production-semantics live suite using real
      Trellis/NATS/SQLite behavior.
- [ ] Small pure/unit/real-adapter checks only where they are the right level.
- [ ] Normal `check`: format + lint/type + tests + generated-files-up-to-date.
- [ ] Release-only package/docs/container/platform validation and publication
      metadata.
- [ ] Full final format/Clippy/type/live run on the finished tree.
- [ ] Record the new timing baseline and compare it with the deleted historical
      baseline only as historical context.
- [ ] Update user/developer docs for the final public API/protocol shape.
- [ ] Delete this tracker.

## Landed cleanup commits worth preserving

- `dbb2444b` — give each runtime lease one owner task.
- `581fb43e` — test Auth rollback with a real SQLite failure.
- `cb0955fe` — remove Jobs active-job callback adapter.
- `09125056` — make durable listener registration atomic.
- `ed1a40c` — make durable listener teardown/drop deterministic without
  runtime-dependent cleanup tasks.
- `6e8fa721` — prepare the embedded portal once in `Check` with generated SDK
  artifacts.
- `86e6c95b` — assert the real rollback through the typed Auth storage-conflict
  boundary.
- `e6615be3` — remove synthetic Auth transaction/post-commit failure injection
  and its live cases.
- `00c9f48f` — remove provider Auth fault injection and retain a real
  invalid-proof `TERM` live boundary.
- `73ad6df2` — remove JobRef callback/cache state and reconstruct snapshots from
  the durable lifecycle.
- `3859764b` — consolidate authorization provider state while retaining
  per-digest singleflight.
- `88da68fc` — remove the `client::client` module-inception suppression with a
  behavior-neutral private module rename.
- `65dae41f` — shrink typed runtime error payloads so Rust 1.98 no longer needs
  repeated `result_large_err` / `large_enum_variant` suppressions for those
  envelopes.
- CI/release simplification also includes native semantic live-job parallelism,
  stale-governance deletion, release-DAG deletion, generated-artifact fixes, and
  one-generation `Check` preparation; use `git log rs` for the exact
  intermediate commit chain.

## Immediate next work

1. Finish the protocol-WASM build/boundary cleanup and first-public `v1` proof
   clean break together so the TS/WASM API is only renamed once.
2. Continue production/test-boundary and runtime-composition cleanup.
3. Finish release-only gate cleanup and record final timing baseline.
