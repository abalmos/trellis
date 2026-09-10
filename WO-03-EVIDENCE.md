# WO-03 implementation evidence - revision 3

This is the targeted implementation handoff for revision-3 WO-03 only. It is
ready for independent implementation review and is **not release-validated**.

## Accepted base and scope

- Accepted base SHA: `c7080f164b4ba75537ee90d5d3ba801a0f8907fc`.
- The current checkout remains at that SHA with the WO-03 implementation in the
  worktree; this evidence does not claim a commit or remote SHA.
- Scope completed here: native semantic IDL/package compilation, canonical
  semantics and compatibility, source-package locking/cache/OCI handling, Rust
  and TypeScript generation and codec support, generated builtin semantics,
  source-evidence verification/storage, and consolidation of the unreleased Auth
  SQL chain into one fresh-create schema.
- Scope not claimed here: WO-04 authority/resource/companion behavior, WO-05
  execution/Events/delivery behavior, or WO-06 repository integration,
  pagination consumers, documentation, and release acceptance.

## Key invariants established

- `compile_project` consumes explicit source units and an exact supplied
  dependency closure; it does not acquire dependencies or publish a partial
  graph after diagnostics.
- Canonical semantic identity excludes source filenames, local aliases, spans,
  titles, and the root package version while including exact dependency
  identities/digests and consent-bearing capability text.
- Selected API compatibility is directional and recursion-safe; implementation
  and retained-resource comparison use semantic surfaces rather than SemVer,
  first-API selection, or legacy JSON artifact round trips.
- Source packages use `trellis.package.source.v1`, normalized bounded archive
  entries, exact frozen resolution, and content revalidation on cache reads.
- Generated Rust and TypeScript surfaces carry every implemented API, exact
  action identity, optional availability, resource migration registration,
  recursive/open model behavior, open enums, declared errors, decimal-string
  64-bit integers, base64 bytes, and non-finite-number rejection.
- Grant derivation now expands an Operation invocation into exact Invoke,
  Observe, and Cancel atoms plus Control for each declared signal. Required and
  optional capability/resource grants remain separate, and the needs digest
  covers participant kind/companion, implemented and selected API surfaces,
  resources, complete required grants, required capabilities, and each optional
  grant bundle.
- A participant projection digest is derived from that participant's identity
  and needs digest. Unrelated declarations in the same package do not change it,
  while changes to that participant's semantic projection do.
- Installed availability is runtime state, not generated static authority. Rust
  replaces an immutable grant/resource snapshot whenever it installs an
  authorization context; TypeScript initializes client, device, and service
  connections from bootstrap API/resource bindings and replaces the snapshot on
  refresh. Generated callers expose current/watch access and reject unavailable
  optional actions before transport.
- Runtime package evidence is recompiled from canonical sources, checked against
  the package and participant digests, stored immutably by package digest, and
  projected into read-only runtime records. Reserved builtin trust is tied to
  generated evidence rather than matching API shape.
- The fresh Auth schema directly creates the current evidence and installed
  participant records; the development-only V1001-V1006 migration chain and old
  Auth evidence columns/readers are not retained. One removed-field reference in
  the WO-04-owned State consumer is recorded below.

## Targeted verification in this worktree

The following commands were run against the current worktree on 2026-09-10:

| Command                                                                                                                                                                                                                                                                          | Result                                                                                                              |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `cargo test --manifest-path rust/Cargo.toml -p trellis-idl`                                                                                                                                                                                                                      | Passed: 17 integration tests; no failures or ignored tests.                                                         |
| `cargo test --manifest-path rust/Cargo.toml -p trellis-codegen-rust -p trellis-codegen-ts`                                                                                                                                                                                       | Passed: Rust generator 3 unit + 1 integration tests; TypeScript generator 3 unit tests; no failures.                |
| `cargo test --manifest-path rust/Cargo.toml -p trellis-cli --no-default-features --lib`                                                                                                                                                                                          | Passed: 34 targeted CLI/source-package unit tests; no failures or ignored tests.                                    |
| `cargo test --manifest-path rust/Cargo.toml -p trellis-cli --no-default-features --test generation`                                                                                                                                                                              | Passed: 1 staged and repeatable install/generation integration test.                                                |
| `cargo test --manifest-path rust/Cargo.toml -p xtask --test wo03_generated_package`                                                                                                                                                                                              | Passed: 1 generated cross-language package fixture, including generated Rust tests plus TypeScript check/tests.     |
| `deno test -A -c ts/deno.json ts/packages/trellis/generated_support_test.ts ts/packages/trellis/participant_runtime/participant_test.ts ts/packages/trellis/auth/device_activation_test.ts ts/packages/trellis/service/runtime/health_test.ts ts/packages/trellis/store_test.ts` | Passed: 16 targeted TypeScript tests with required WASM file access; no failures.                                   |
| `cargo test --manifest-path rust/Cargo.toml -p trellis-runtime platform::auth::evidence::tests`                                                                                                                                                                                  | Blocked before test execution by six compile errors in WO-04-owned State/resource consumers; not counted as a pass. |

The blocked runtime command found three errors in
`rust/crates/runtime/src/platform/state.rs`: removed `StateKind`, removed
`StateDefinition`, and removed `ParticipantBindingRecord::participant_json`. It
found three more in `rust/crates/runtime/src/platform/auth/resources.rs`: an
uninferred retry-backoff vector, missing `duration_default`, and an `i32` value
supplied to the `i64` `max_ack_pending` field. These are concrete WO-04 cutover
dependencies. The independently runnable evidence compiler and generated-package
boundaries above passed; the runtime builtin-evidence tests did not execute.

The generated cross-language fixture exercises recursive/open models, open
enums, bigint/base64 codecs, typed known and unknown errors, multiple
implemented APIs, optional resources and migration registration, immutable
availability snapshots, watched replacement, and the no-transport
unavailable-action guard in generated Rust and TypeScript consumers.

No whole-workspace build/test, live NATS or SQL acceptance, browser/demo matrix,
CI workflow, package-manager/platform build, release packaging, or release
validation was run or inferred.

There is exactly one platform migration file,
`rust/crates/runtime/src/storage/sqlite/platform/V1000__platform_init.sql`, and
its focused storage assertions require the complete fresh Auth shape and only
migration version 1000. That runtime test target is not claimed as executed
because the same crate compilation stops at the WO-04 blockers above.

`web/trellis.toml` remains `format = 1` with old `[apis.*]` path dependencies
and cannot enter the new package-manifest pipeline. This is the exact WO-06 web
manifest blocker; the web caller was not used as WO-03 validation evidence.

## Deleted migration and compatibility paths

- Deleted the old `trellis.api.v1` and `trellis.participant.v1` schemas and the
  protocol artifact parser/resolution modules under
  `rust/crates/protocol/src/{api,resolution,schema_profile}.rs`.
- Replaced the old IDL artifact/project lowering and generator projection paths;
  old project tests and both code generators' JSON projection modules were
  removed rather than bridged.
- Deleted checked-in builtin API/participant JSON, runtime canonical JSON, and
  the Rust and TypeScript handwritten internal generated descriptor islands.
- Removed old TypeScript artifact/resolution modules and handwritten Trellis
  model files superseded by the generated support ABI.
- Deleted V1001-V1006 and folded their final fresh shape into
  `V1000__platform_init.sql`; there is no SQL ALTER/backfill or old installed
  artifact salvage path.
- Source-package cache acquisition uses only the new format namespace and does
  not probe or convert old API-JSON entries.

## Retained apparent compatibility for new-system evolution

- Semantic selected-surface, implementation, and resource compatibility remain
  for revisions produced by the new package format.
- Immutable package evidence and installed participant revisions/CAS remain for
  records created by the new system.
- Generated State/KV migration adapters retain explicit direct
  historical-to-current handlers for declared representation versions written by
  the new envelope format; they are not pre-redesign decoders.
- Open models, open enums, qualified known errors with unknown-error fallback,
  optional availability snapshots, and exact dependency locks remain deliberate
  forward-evolution behavior.

## Remaining old-format code and concrete reasons

- **WO-04:** `rust/crates/runtime/src/platform/state.rs` still contains the old
  keyed/map State implementation and its final legacy binding read. WO-04 Part B
  owns replacement with exact bound single-value State and versioned State/KV
  envelopes. `platform/auth/resources.rs` likewise awaits WO-04 resource
  provisioning and approval/readiness completion. Device/app/agent companion
  authority is also WO-04-owned.
- **WO-05:** `rust/crates/eventlog-runtime/**` and
  `rust/crates/trellis/src/service/eventlog_runtime.rs` retain EventLog names
  and old storage/consumer cleanup behavior until the assigned Events rename,
  fresh `events_*` schema, Consumer DLQ, and delivery cutover. The old-grammar
  `rust/crates/jobs-runtime/contract.trellis` remains with the Jobs execution
  body that WO-05 migrates; its obsolete-watch cleanup remains in
  `rust/crates/trellis/src/jobs/runtime.rs`. The rejected legacy EOF branch in
  `ts/packages/trellis/service/runtime/transfer.ts` remains until WO-05's
  assigned transfer/delivery cleanup. Durable Operations, route `apiBindings`,
  Jobs corrections, and transfer/delivery behavior are not claimed here.
- **WO-06:** old-grammar application contracts remain under `demos/**`,
  `integration/fixtures/runtime/contract.trellis`,
  `docs/examples/orders/contract.trellis`, and `web/contract.trellis`.
  `web/trellis.toml` also remains the old format-1 manifest. WO-06 owns final
  caller manifest/source conversion and regeneration, demos/browser/out-of-tree
  integration, pagination consumers, and obsolete-path audit. Their presence is
  not a compatibility promise and they were not used as WO-03 validation
  evidence.

**READY FOR INDEPENDENT IMPLEMENTATION REVIEW - NOT RELEASE-VALIDATED**
