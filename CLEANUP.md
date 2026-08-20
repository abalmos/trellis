# Trellis 0.12 simplification

Short-lived tracker for the `rs` cleanup before the first stable release. This file is the durable source of truth for the cleanup plan; keep it current as work lands and delete it when the work is complete.

Repository: `abalmos/trellis`
Branch: `rs`
Status updated: 2026-08-20

## Working rules

- No backwards-compatibility work for unreleased Trellis behavior.
- Prefer deletion and direct code over new frameworks or registries.
- Live integration remains the primary test style for runtime behavior.
- Unit tests stay for small pure invariants or real adapter behavior that does not require the full control plane.
- Test-only code must not change normal production semantics.
- Do cleanup work as direct `rs` commits. Hidden `agent/*` branches may be used for Actions validation, but do not create cleanup PRs unless explicitly requested.
- Re-read current `rs` before every write; do not overwrite concurrent work.

## 1. Fast feedback / test, check, release

- [x] Make live-test concurrency machine-adaptive instead of using a fixed worker count.
- [x] Share the live worker default between TS and Rust runners.
- [x] Delete stale nextest/test-governance artifacts and the stale release timing baseline.
- [x] Make the Rust live matrix track behavior instead of every compiled helper/adapter test.
- [x] Remove the custom release DAG/stage/wave planner and its graph/self-tests.
- [x] Remove the redundant standalone live-inventory pass and fixed `--jobs 20` override.
- [x] Add a normal `Check` workflow for everyday `rs` validation.
- [x] Fix clean-checkout SDK generation dependencies and the generated demo-contract compile dependency.
- [x] Make `Check` prepare generated SDKs once and hand the ignored generated packages to Rust/TS/live jobs as an Actions artifact.
- [x] Keep hidden `agent/*` validation branches from triggering the full everyday `Check` suite.
- [ ] Finish reducing `release.yml` to release-only packaging/docs/platform work; normal correctness belongs in `Check`.
- [ ] Confirm the final `test` / `check` / `release` command split is simple and non-overlapping.
- [ ] Record a fresh cold/warm timing baseline after the final test architecture settles.

## 2. Real tests only / synthetic failure removal

- [x] Add a narrow real-SQLite Auth rollback test using an actual late constraint failure and assert aggregate state, idempotency state, and outbox actions all roll back.
- [x] Remove the Rust demo contract's compile-time dependency on a generated TypeScript API artifact.
- [ ] Remove the old live Auth transaction-failure case and trigger helper now replaced by the real SQLite rollback test.
- [ ] Remove the synthetic Auth post-commit dispatch failure, SQLite hook, helper, live test, and matrix row.
- [ ] Remove provider `fail_next_context_read` / `fail_next_readiness_check` switches and the production verifier branch that consumes them.
- [ ] Reduce the event-authorization failure live case to real behavior: publish a legitimate signed event, corrupt the proof, require JetStream `TERM`, and verify the handler is not invoked for the corrupt copy.
- [ ] Audit the remaining `integration-test-hooks`; delete synthetic errors/faults and retain only genuine deterministic scheduling barriers where needed.
- [ ] Replace avoidable fixed sleeps with observable readiness/state boundaries where practical.

Current validation note: the first Auth-deletion validator did not reach the patch checks because a clean validation checkout was missing `rust/crates/runtime/generated/portal`; fix the validation/bootstrap order before treating that run as evidence about the Auth patch.

## 3. First public protocol version

- [ ] Treat the current request/event authorization proof wire format as the first public `v1`.
- [ ] Remove unreleased `AuthorizationRequestProofV2`, `AuthorizationEventProofV2`, V2 input/verified type names, and `*_v2` implementation function names.
- [ ] Change unreleased `.v2` proof domains to the first public `.v1` domains.
- [ ] Remove unpublished aliases, fallback subjects, dual read/write paths, obsolete parsers/schemas/vectors, and never-released migrations.
- [ ] Keep serialized format-version fields where they are actual wire-version identifiers.
- [ ] Prefer unversioned implementation names (`ApiArtifact`, `PermissionAtom`, `AuthorizationContext`, `AuthorizationRequestProof`, etc.) until a real second public implementation version exists.
- [ ] Regenerate final conformance vectors and generated artifacts after the clean break.

## 4. Ownership / locks

- [x] Give each runtime lease one owner task; remove the shared `Arc<Vec<Mutex<HeldLease>>>` renewal model.
- [x] Fix the durable listener concurrent first-registration race by making check/create/spawn/insert one registry critical section (`09125056`).
- [ ] Make durable-listener teardown/drop ownership deterministic and simple; do not depend on ad-hoc runtime availability if it can be avoided.
- [ ] Consolidate coherent authorization provider trust state instead of independent root/policy/manifest/health locks where those values must move together.
- [ ] Keep per-digest context singleflight; do not actorize normal cache reads.
- [ ] Simplify `JobRef` to immutable identity/seed plus concrete Jobs client/waiter/manager state; remove boxed callback backends.

## 5. Runtime composition

- [ ] Split oversized runtime startup / `platform::start` responsibilities into a few explicit phases with clear ownership.
- [ ] Split the oversized service runtime facade along real responsibility boundaries (bootstrap/lifecycle, routing/operations, resources, event consumers/jobs) where doing so reduces coupling.
- [ ] Do not introduce a DI container, generic actor framework, or abstract lifecycle framework.

## 6. Production / test boundary

- [ ] Remove impossible disconnected/test-only `ConnectedServiceRuntime` states; make connected runtime values valid by construction.
- [ ] Remove raw integration-test methods from production facades where tests can use normal product behavior instead.
- [ ] Remove `runtime-internals` exposure that exists only to let tests reach implementation details.
- [ ] Remove `integration-test-scoping` behavior that changes normal production semantics; retain only genuine product concepts.
- [ ] Stop building production runtime binaries with test-fault features once the remaining hooks are gone.

## 7. Jobs and constructors

- [x] Remove the typed `ActiveJob` heartbeat/progress/log callback adapter, duplicated active-job state, and its eight-argument constructor (`cb0955fe`).
- [ ] Remove `JobRef`'s `Arc<dyn Fn>` get/wait/cancel backend and use concrete Jobs objects directly.
- [ ] Collapse the remaining Jobs function/wrapper ladder to one implementation plus a small cohesive execution context/hooks where needed.
- [ ] Remove duplicate contract metadata from `ServiceConnectOptions` / generated service contract evidence; generated contract evidence must have one source of truth.
- [ ] Apply the same constructor simplification to device connection paths where equivalent duplication exists.
- [ ] Collapse combinatorial operation-registration callback variants to one provider interface; generated code adapts to it.

## 8. Names and lint causes

- [x] Delete the custom release lint-exception registry.
- [x] Remove one concrete `too_many_arguments` cause with the `ActiveJob` callback-adapter deletion.
- [ ] Fix remaining `too_many_arguments` causes through cohesive inputs/ownership instead of adding suppressions.
- [ ] Fix `result_large_err`, `large_enum_variant`, `module_inception`, and test-only `dead_code` causes where the design can be made simpler.
- [ ] Shorten long names by using module context and splitting oversized modules, not by inventing abbreviations.

## 9. Final gate

- [ ] One understandable production-semantics live suite using real Trellis/NATS/SQLite behavior.
- [ ] Small pure/unit/real-adapter checks only where they are the right level.
- [ ] Normal `check`: format + lint/type + tests + generated-files-up-to-date.
- [ ] Release-only package/docs/container/platform validation and publication metadata.
- [ ] Full final format/Clippy/type/live run on the finished tree.
- [ ] Record the new timing baseline and compare it with the deleted historical baseline only as historical context.
- [ ] Update user/developer docs for the final public API/protocol shape.
- [ ] Delete this tracker.

## Landed cleanup commits worth preserving

- `dbb2444b` — give each runtime lease one owner task.
- `581fb43e` — test Auth rollback with a real SQLite failure.
- `cb0955fe` — remove Jobs active-job callback adapter.
- `09125056` — make durable listener registration atomic.
- CI/release simplification also includes adaptive concurrency, stale-governance deletion, release-DAG deletion, generated-artifact fixes, and one-generation `Check` preparation; use `git log rs` for the exact intermediate commit chain.

## Immediate next work

1. Fix the hidden Auth validator bootstrap so the generated embedded portal exists before compiling `trellis-runtime`.
2. Land the validated synthetic Auth transaction/post-commit deletion.
3. Delete provider readiness/context-read fault switches and keep only the real corrupt-proof `TERM` behavior test.
4. Simplify `JobRef` callbacks.
5. Consolidate authorization provider trust state.
6. Do the first-public `v1` proof cleanup.
7. Continue production/test-boundary and runtime-composition cleanup.
