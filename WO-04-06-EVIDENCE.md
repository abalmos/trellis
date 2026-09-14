# WO-04 through WO-06 candidate evidence

This is the implementation and acceptance handoff for WO-04 through WO-06. It
records the exact pre-commit candidate tree and does not claim release
acceptance.

## Candidate

- Accepted WO-03 base: `63f5ebd1ec83b6a858ea9d3f11d9fd39d14cdd1d`.
- Final implementation binary-diff SHA-256, excluding this evidence file:
  `7c92ca540988a8b4285876395592010243bc043bc5754765c5cb7bb0929ad9ed`.
- `cargo xtask install` produced that identical diff on two consecutive runs.
- Publishable Trellis, Result, and Svelte npm trees were rebuilt before package
  consumer validation.
- `workorders/` is protected review input and is not part of the candidate.
- The exact candidate commit and matching `origin/rs` SHA are reported in the
  external handoff because a commit cannot contain its own SHA.

## Coverage

- **WO-04:** authority-bound State/KV/Store resources, resource lifecycle and
  physical validation, companion consent/delegation/session fencing, shared
  companion State, and refresh/restart behavior.
- **WO-05:** generated Jobs and Operation surfaces, durable operation lifecycle,
  cancellation/signals/uploads/replica recovery, keyed Jobs coordination, Events
  projection/query/watch/Consumer/DLQ/replay, exact signed event descriptor
  authorization, and broker-evidence validation.
- **WO-06:** native source-package cutover, generated Rust and TypeScript
  consumers, cursor pagination, CLI and Console surfaces, browser hosting,
  documentation, demos, and removal of superseded Event Log/runtime and
  handwritten contract paths.

## Verification

The following commands passed against the final candidate on 2026-09-13:

| Boundary                         | Command and result                                                                                                                                                             |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Rust lint                        | `cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets --all-features -- -D warnings` passed.                                                                 |
| Rust workspace                   | `cargo test --manifest-path rust/Cargo.toml --workspace --all-targets --all-features` passed every target.                                                                     |
| Rust formatting                  | `cargo fmt --manifest-path rust/Cargo.toml --all --check` passed.                                                                                                              |
| TypeScript packages and UI tools | `deno task test` in `ts/` passed, including 196 Trellis tests with 70 nested steps.                                                                                            |
| Public TypeScript surfaces       | Public entrypoint and integration `deno check` commands passed.                                                                                                                |
| TypeScript live integration      | `deno test -A --no-check integration/*_test.ts` passed 20 tests with 15 nested steps in 14m47s.                                                                                |
| Rust live integration            | `cargo test --manifest-path rust/Cargo.toml -p trellis-rs --features live-integration --test integration -- --nocapture` passed 2/2.                                           |
| Console                          | Svelte check reported zero errors and warnings; unit tests and the production build passed.                                                                                    |
| Documentation                    | Svelte check, generated TypeScript API docs, and the production build passed.                                                                                                  |
| Protocol/browser                 | Protocol WASM and embedded web assets built; embedded, configured-directory, and reverse-proxy hosting passed 3/3.                                                             |
| Package consumers                | Isolated staged Node, Deno, and Svelte consumers passed; Rust service and device demos compiled with a command-line crates.io patch to the unpublished candidate `trellis-rs`. |
| Orders example                   | `deno run -A ts/tools/package_build/orders_example.ts` built and passed its real service/caller test without repository imports.                                               |
| Repository integrity             | `git diff --check` passed and no merge conflicts were present.                                                                                                                 |

Focused production-bound suites also passed for companion authority and expiry,
resource transactions, operation cancellation/lease/revision behavior, Jobs
horizons, Events projection and recovery, Consumer replay, stable cursor
windows, outbox conflicts, retained authorization, State CAS, and Store/KV
physical validation.

## Review Status

The candidate incorporates eight correction gates from independent WO-04, WO-05,
WO-06, and adversarial reviews. Per the final handoff direction, further
independent review cycling stops here so the primary designer can perform the
whole-release review.

**READY FOR WHOLE-RELEASE REVIEW - NOT ACCEPTED**

## Whole-release correction candidate

The follow-up to candidate `4997f94a5ffa9f80d271421ddc7eb2b90c8ff5d5` implements
the bounded C1-C7 correction round requested by the primary whole-release
review. It does not replace or reinterpret the verification record above.

- C1-C6 and the C7 CI protocol-WASM dependency correction have focused
  production-bound evidence.
- The generated Runtime workflow passed all eight steps, including exact
  Operation identity and epoch fencing, same-deployment replica updates,
  authored Job policy, and the bounded native-admin revocation scenario.
- Historical Check run #310 remains failed in Install tooling and Live
  integration; it is not evidence for this follow-up tree.
- Complete live acceptance is not green: the focused
  `generated Rust resources use live NATS` case reproducibly leaves stale NATS
  ACLs and resource handles usable after the server has durably revoked the
  service context and completed its post-commit actions.
- A speculative client reconnect-generation gate was removed after it delayed
  unrelated authority and deployment workflows. The unresolved C7b boundary
  requires root-cause review rather than another mitigation.
- `workorders/` remains protected review input and is excluded from the
  follow-up commit.

**REVIEW-BLOCKED CORRECTION CANDIDATE - NOT ACCEPTED**

## Whole-release correction follow-up

This follow-up to correction candidate
`ce88e73259ae471547444e421fa9d64da6ad0cd9` addresses the independent review's
remaining R1-R4 findings and the C7b live failure. It does not replace the
historical evidence above.

- Historical Check #310 remains failed in Install tooling and Live integration.
- Exact-candidate Check #311 (`34759048838`) remains failed because
  `trellis-test-generated` did not match its semantic lock. Dependent lanes were
  skipped and are not treated as green evidence.
- The affected semantic lock was refreshed through normal `trellis update` and
  generated artifacts were rebuilt. Two consecutive `cargo xtask install` passes
  produced the identical tracked binary-diff SHA-256
  `4d3f66faa9a98226fa079c05e6ca641d71b899136ece246c0857874ac3cf4172` before this
  evidence section was appended.
- R1 now derives presentation and companion approval from one current,
  server-owned portal-policy ceiling and carries its expiry and provenance into
  the atomic decision. Repeated approval submits only the current delta and
  retains only server-confirmed eligible authority.
- R2 now fences admission after acknowledged physical presence and makes SDK
  authorization installation candidate-then-promote. Revocation suspends
  resource usability before refresh. A correlated live trace is retained at
  `target/device-companion-auth-flow-trace.log` for follow-up flow analysis.
- R3 requires the immutable fence acquired by a successful local Operation claim
  through executor-held controls; durable rereads cannot mint ownership.
- R4 uses one signed cross-language transient-update envelope with operation,
  API, action, deployment, process executor, signed connection, owner epoch,
  sequence, timestamp, and generated update identity.

The following local verification passed on 2026-09-14:

| Boundary                    | Result                                                                                                                                                                                                    |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust static/workspace       | Formatting passed; warning-denied workspace/all-target Clippy passed; the complete Rust workspace passed.                                                                                                 |
| TypeScript static/packages  | Repository formatting, public package checks, integration checks, and all package/UI-tool tests passed.                                                                                                   |
| TypeScript live integration | Complete matrix passed 20 tests with 14 nested steps in 15m23s. Device.Companion separately passed its two-device policy, State, restart, logout, and sibling-survival case with retained trace evidence. |
| Rust live integration       | Passed 2/2 with prebuilt repository server and CLI binaries.                                                                                                                                              |
| Runtime and Events          | Complete Runtime passed 7/7 with 12 nested steps; complete Events passed 6/6.                                                                                                                             |
| Protocol and browser        | Protocol WASM and embedded applications built; embedded, configured-directory, and reverse-proxy hosting passed 3/3.                                                                                      |
| Package consumers           | Staged Node, Deno, and Svelte consumers passed; Rust service and device demos compiled against the unpublished local `trellis-rs`; packaged Orders passed its real service/caller test.                   |
| Console and docs            | Both Svelte checks reported zero errors and warnings; embedded Console and documentation production builds passed, including generated TypeScript API docs.                                               |
| Repository integrity        | Consecutive generation digests matched; `git diff --check` passed; no merge-conflict markers or temporary print diagnostics remain.                                                                       |

One initial local Rust-live attempt overlapped an embedded-asset replacement and
compiled a stale generated `include_bytes!` list; one browser-hosting attempt
also omitted its required prebuilt-server environment and waited behind that
Cargo lock. Neither is counted as product evidence. After the asset build
completed, the server and CLI were rebuilt once and both exact lanes passed as
recorded above.

`workorders/` remains protected review input and is excluded from the follow-up
commit. This evidence records local execution only; it does not claim an
exact-candidate CI result or release acceptance.

**READY FOR INDEPENDENT WHOLE-RELEASE REVIEW - NOT ACCEPTED**

## W1-W5 correction candidate on `0e94770`

This section records a bounded correction on top of candidate
`660619ff7263e61827217b8b0d943b184af6ae01`, which is now contained in
`0e947700fafe87ea3745f7ea0f841ba5d2da8118` (the designer's auth-cache and Rust
telemetry commit). It preserves the accepted C2, C3, C6, C7a, and R5 corrections
and does not restart WO-03-06. The initial local verification targeted
`660619ff`; it was rebased onto `0e94770` with no conflicts and the complete
campaign was re-run on the rebased base.

- Historical Check #310 and exact-candidate Check #311 remain failed and are not
  evidence for any tree here.
- Exact-candidate Check #313 (run 34797945995) remains valid evidence for its
  own predecessor tree only. No CI run is claimed for this correction.
- The implementation-only tracked diff (22 files, 1,590 insertions, 487
  deletions, excluding this appended evidence section) has SHA-256
  `6574dd9b1ef6bec9d7285e094736ac84077fc4449e9c9c3db707f8c31ef0a0a5` against
  `0e94770`. Two consecutive `cargo xtask install` passes produced no tracked
  drift beyond this evidence file, and no generated artifact, `trellis.lock`, or
  demo/web generated tree appears in the change surface.

### W1 - one consent entitlement source

Browser and Device.Companion consent now share one private
`policy::consent_authority` resolver with three sources: an explicit
deployment-configured target binding, current portal policy, and the
public-capability baseline. It returns the effective ceiling, the minimum
applicable finite expiry, provenance, and `ConsentAuthorityPreconditions`
(policy snapshot plus exact owner/participant/revision/expiry/ceiling/provenance
per source binding). `set_consent_grant_binding` verifies those preconditions,
including source expiry, inside the same SQLite transaction that replaces the
binding, so a stale or expired source can never extend or widen a binding.
Companion activation compares the retained or replacement child delegation
ceiling instead of re-deriving `participant_delegation_ceiling`, and companion
consent evaluates current portal policy without remembered-role expansion,
intersected with the retained child ceiling.

Evidence: `explicit_target_entitlement_keeps_its_ceiling_and_finite_expiry`,
`portal_authority_never_widens_a_narrower_retained_child` (unit, real records),
and `consent_authority_preconditions_fence_expiry_and_revision` (real SQLite:
finite expiry preserved; stale revision and expired source rejected with
`StorageConflict` and no revision movement). Complete Device.Companion live case
passed 1/1 (2 substeps, 1m06s) covering policy-backed approval, direct-bootstrap
rejection, shared State, restart, and logout. Rust runtime library suite passed
190/190.

### W2 - one connection-owned authorization lifecycle

Both SDKs now keep a planned refresh as a private verified candidate while the
active installation stays usable; the transport reauthorizes with the candidate,
exact own-context revocation coverage initializes on that admitted transport
epoch, and only then does a digest-checked promotion swap routes, resources, and
the held lease. Revocation or coverage loss suspends application usability and
retires the stale own lease before refresh; terminal outcomes use typed
`authority_revoked`, `authority_expired`, and `authority_not_found` codes with
bounded retry. Ordinary reconnects reconcile coverage without fetching another
context. The Rust side previously suspended application use for the whole HTTP
refresh window; that was corrected to the private-candidate model, which is what
the live acceptance exposed. TypeScript mirrors the model through
`prepare`/`promote`, `retainOwnCandidate`/`promoteOwnCandidate`, and connection
phase ownership.

Evidence: full Rust live-integration library suite 122/122, including
`initialized_watch_put_revokes_cached_context`; complete Runtime live module 7/7
(12 nested steps, 8m03s) including the Rust resource-replacement case whose
invalidation now returns typed unavailable after KICK; complete Events live
module 6/6 (4m04s); full TypeScript live matrix 20/20 (14 nested steps, 14m55s).
A redacted control-plane excerpt of the resource-replacement lifecycle:

```
{"message":"recorded physical connection presence before final admission validation","context_digest":"abc123…","runtime_connection_id":"abc123…","physical_connection_id":"abc123…","server_id":"abc123…","client_id":"NN","presence_revision":N,"participant_id":"runtime-trellis.Provider"}
{"message":"final admission validation succeeded","context_digest":"abc123…","physical_connection_id":"abc123…","presence_revision":N}
{"message":"enumerated authoritative physical connections for revoked context","context_digest":"abc123…","physical_connection_count":1}
{"message":"processed authorization connection kick","context_digest":"abc123…","runtime_connection_id":"abc123…","server_id":"abc123…","client_id":"NN","presence_revision":N,"outcome":"Disconnected"}
```

The client-side lifecycle events (`prepared verified authorization candidate`,
`retained own authorization coverage` with `transport_epoch`,
`promoted
authorization installation`, `suspended authorization installation`)
are emitted by the production client and remain enabled for flow study.

### W3 - authoritative presence reads and KICK responses

Connection presence enumeration and conditional deletion already used
leader-authoritative `get_last_raw_message_by_subject` with exact subject,
marker, identity, and captured-revision validation. KICK reply handling now also
validates the responding server identity against the presence record and accepts
only a well-formed success or the exact `no such client or leafnode id`
already-absent error; any other response fails the action for retry. The exact
outcome is logged and conditional deletion still targets only the captured
revision.

Evidence: `connection_kick_response_rejects_system_errors` (unit) rejects a
wrong-server response; complete Events live module 6/6 covers real KICK.

### W4 - immutable Operation execution fences

Rust `resume` requires the locally acquired `OwnerFence` (fresh process executor
ID, signed-context connection ID, owner epoch) and rechecks the repository
immediately before invoking user code for exact executor, connection, epoch,
api/operation/deployment identity, unexpired lease, nonterminal state, and
creator identity. Upload staging callbacks and recovery continuations recheck
the same fence before mutation, and a superseding epoch makes stale
continuations exit without invoking a handler. TypeScript carries the same
immutable fence through handler admission, heartbeats, transfer callbacks,
cancellation watching, and cleanup; same-process reacquisition aborts the
previous execution and replaces its record so E1 continuations cannot receive E2
controls.

Evidence: `durable_operation_recovery_and_controls_use_real_kv` (real KV/NATS)
passed; complete Runtime live module 7/7 includes the two-replica owner-epoch
workflow step.

### W5 - transient update eligibility and codec boundary

Cross-replica transient updates are only forwarded after receiver-side proof
verification over the received raw bytes and a current-record check for
unexpired lease, nonterminal state, no cancellation request, and exact
owner/executor/connection/epoch, with canonical positive epoch and sequence
decimals. Both languages encode and decode the generated wire update through the
generated action codec at the private envelope boundary, so nested 64-bit values
and bytes round-trip; the transmitted signed bytes are the exact published
bytes.

Evidence: complete Runtime live module 7/7 (including forged-proof rejection and
two-replica transient fan-out); `update_tests` codec coverage.

### C1-C7 disposition

C1, C4, C5, and C7b are re-implemented by W1, W4, W5, and W2 respectively and
carry the evidence above. C2, C3, C6, and C7a remain as accepted in `660619ff`
and are unchanged by this correction. No compatibility shim, migration
machinery, test-only hook, or generated-file edit is included.

### Local Check-equivalent campaign

| Lane                  | Result                                                                                                                    |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Install / generated   | Consecutive installs identical (`8b94bf62…`); no generated or lock drift; formatting idempotent.                          |
| Protocol WASM         | Built by install and protocol checks.                                                                                     |
| Embedded browser apps | `browser:embedded` built.                                                                                                 |
| Rust                  | `fmt --all --check`, warning-denied workspace/all-target Clippy, and full workspace tests passed.                         |
| Rust tooling          | Both tooling workspaces passed.                                                                                           |
| TypeScript            | Repository formatting, public package and integration checks, and all package/UI-tool tests passed.                       |
| Demos                 | Staged Node/Deno/Svelte consumers passed; isolated Rust service and device demos compiled against the local `trellis-rs`. |
| Live                  | Rust live integration 2/2; TypeScript live matrix 20/20 (14 nested steps); packaged Orders passed.                        |
| UI hosting            | Embedded, configured-directory, and reverse-proxy modes passed 3/3.                                                       |
| Repository integrity  | `git diff --check` clean; no conflict markers, temporary print diagnostics, or fixture diagnostics.                       |

`workorders/` remains protected untracked review input and is excluded from the
commit. This evidence records local execution only and does not claim an
exact-candidate CI result.

**READY FOR INDEPENDENT WHOLE-RELEASE REVIEW - NOT ACCEPTED**

## R1-R4 correction candidate on `08b310db`

This section records the bounded R1-R4 correction ordered by
`TRELLIS-WHOLE-RELEASE-REVIEW-08b310d.md`, implemented on top of candidate
`08b310db02e6b118df59be274ada5c5f6675a4cb` (parent `0e94770`). It preserves the
integrated immutable-evidence optimization, ordinary correctness tests, the
manual performance tool, OpenTelemetry instrumentation, and every accepted
security correction, including W3 and C2/C3/C6/C7a. No compatibility shim,
second authority record, weakened validation, or new trust service is included.

- Check #315 remains evidence only for `08b310d`. No CI run is claimed for this
  correction; the review requires an independent exact-candidate Check.
- The implementation-only tracked diff (21 files, 702 insertions, 133 deletions,
  excluding this appended evidence section) has SHA-256
  `68bfe62933d8521f175a2d44eb61f0d68ddf79399728cbf9f75c01a20bdf72ed`. Two
  consecutive `cargo xtask install` passes produced that identical hash; the
  only generated outputs in the surface are the regenerated
  `integration/fixtures/runtime/trellis.lock` and gitignored fixture trees.

### R1 - own-authorization recovery and final publication

- **Resource identity vs transient usability.** A disconnect or watch
  interruption now suspends use without retiring unchanged resource identities;
  generations are bumped only by `replace`/`remove` or terminal state. The
  TypeScript promotion path restores the exact retained `active` object when the
  binding signature is unchanged and retires the previous object when it
  changed, covering both the availability snapshot and the retained-handle gate.
- **Explicit same-current resumption.** Rust's coverage-only branch now calls
  `AuthorizationProviderCache::finalize_own_installation(digest, false)`, which
  republishes the retained installation's availability without an HTTP request
  or generation change. TypeScript's `#restoreOwnContext` success drives a new
  `onOwnResumed` connection callback that reinstalls the last installed
  availability; service, client, and device connections register it.
- **Guarded final publication.** `finalize_own_installation` runs on one short
  local synchronization boundary with no network await: expected digest,
  retained lease identity, current admitted transport epoch, initialized live
  coverage, no stored revocation, not-before/expiry, and nonterminal state.
  Promotion is digest-checked against the prepared candidate; resumption checks
  the retained digest.
- **Both digests invalidate correctly.** Observed revocation of the active
  digest suspends it and requests a fresh credential; observed revocation of the
  current private candidate discards that candidate and requests a new one
  without touching the still-valid active predecessor. A late watch from a
  retired lease/epoch is ignored unless its coverage identity is still the
  current cache entry for that digest.
- **Transport authentication is independent of application-use eligibility.**
  TypeScript presents a verified private candidate or a retained, unexpired,
  not-definitively-revoked installation even while application use is suspended;
  terminal state still rejects. Rust's authenticator already selected the
  candidate or retained credential without consulting availability and now also
  refuses a digest with locally observed revocation evidence.
- **Typed refusal table and durable login.** `login_not_found` and
  `context_owner_mismatch` are definitive in both SDK classifiers; pending and
  transient codes are not. The browser client clears its durable login only for
  login/session-invalid codes, so a revoked or expired authority context closes
  the connection and recovers without erasing the user login.
- **Predecessor identity.** `stored_context_digest`/`storedContextDigest`
  preserve an expired-but-retained predecessor for refresh ownership and
  reconciliation; time/usability checks remain on use and authentication.

Runtime evidence (server and SDK from this tree): complete Rust resources live
case passed in 38s, proving a replacement revokes retained KV/Store/Consumer
handles (typed `Unavailable`) and that a same-digest coverage reinitialization
restores usability without new issuance; complete Device.Companion live case
passed 1/1 (two device approvals, companion bootstrap, cross-device State,
restart, logout isolation); Rust live SDK suite 122/122; complete TypeScript
live matrix 20/20 with 14 nested steps in 8m07s.

### R2 - re-encode at the final TypeScript watch boundary

The TypeScript watch-serving provider now re-encodes the decoded update through
the generated action codec immediately before `publishFrame`, preserving the
envelope, sequence handling, raw-byte proof verification, and every exact
owner/epoch/lease/route check. To make the receiver boundary complete, the
caller path now decodes operation `progress`/`update`/`output` payloads through
the generated descriptor codecs and encodes typed operation and signal inputs
before serialization, so nested 64-bit and byte values round-trip instead of
relying on the previous wire-shaped leak.

Evidence: the live generated workflow contract now carries
`Update { value, nested: UpdateDetail { count: int64, payload: bytes } }`; the
Rust fixture persists and emits `count: 9007199254740993` and bytes, and the
separate live "generated TypeScript caller reaches Rust provider" case passed in
1m21s. The complete generated workflow passed with two live same-deployment
replicas, both watches receiving the typed update, forged unsigned update
rejection, cancellation, and `bigint`/`Uint8Array` identity assertions. Only
`Update` was added to the fixture contract; payloads stay transient and no
update journal or public envelope changed.

### R3 - explicit target entitlement after consent

The explicit target source no longer requires `ApprovalMode::Exact`; the current
target binding's stored server-owned delegation ceiling is used whenever it has
no portal provenance, is active, and is unexpired, so a first approval persisted
as `Capabilities` keeps its ceiling, restrictions, and finite expiry on the next
consent or second-device activation. Owner/participant identity, active state,
expiry, and transactional revision preconditions are unchanged; public-only
targets keep their empty named-capability ceiling; revoked or expired bindings
remain unusable; policy-backed authority is still resolved through portal policy
with retained-target narrowing.

Evidence:
`capabilities_mode_target_keeps_explicit_entitlement_after_first_consent` and
the existing explicit-ceiling/expiry/precondition and SQLite precondition tests;
complete Device.Companion live case.

### R4 - final ownership checks and cleanup ownership

- Rust `resume` now performs its execution-entry admission
  (`admit_operation_execution`) inside the spawned execution after
  `repository.watch(...)` and immediately before the user handler. The check is
  the exact fence/API/action/deployment/creator/lease/nonterminal predicate,
  built from the freshly read record; a stale or cancelled acquisition returns
  without invoking user code or renewing the old lease. The mutation gate fences
  only the check, not the handler.
- TypeScript's lease heartbeat now stops when its captured execution is aborted
  as well as terminal, and asynchronous cleanup deletes the execution cache
  entry and active fence only when the map still holds that captured runtime.

Evidence: `durable_operation_recovery_and_controls_use_real_kv` (real KV/NATS)
now asserts that an E1 fence is refused by the final entry boundary after E2
acquired the operation while the current fence is admitted, on top of the
existing expired-owner controls, snapshot revisions, and real replica recovery.
The complete TypeScript live matrix covers same-process and cross-replica
execution, cancellation, and forged-proof rejection.

### Local Check-equivalent campaign on the rebased tree

| Lane                | Result                                                                                                                                        |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Install / generated | Two installs identical (`68bfe629…`); only the regenerated fixture lock is tracked; `git diff --check` clean.                                 |
| Rust                | Formatting, warning-denied workspace/all-target Clippy, and the full workspace test suite passed.                                             |
| Rust tooling        | Both tooling workspaces passed.                                                                                                               |
| Rust live           | Live-integration library suite 122/122 and live integration target 2/2 with prebuilt server and CLI.                                          |
| TypeScript          | Repository formatting, public and integration checks, and all package/UI-tool tests passed.                                                   |
| TypeScript live     | Complete matrix 20/20 with 14 nested steps in 8m07s; Device.Companion 1/1.                                                                    |
| Demos / consumers   | Staged Node/Deno/Svelte consumers passed; isolated Rust service and device demos compiled against local `trellis-rs`; packaged Orders passed. |
| UI hosting          | Embedded, configured-directory, and reverse-proxy hosting passed 3/3.                                                                         |

One implementation root cause was found and fixed during focused live work: the
new Rust resumption path originally held a watch `borrow()` guard across
`send_replace`, self-deadlocking the first coverage resumption; the guard is now
released before publication, and the resources live case passed afterward.

`workorders/` remains protected untracked review input and is excluded from the
commit. This evidence records local execution only and does not claim an
exact-candidate CI result.

**READY FOR INDEPENDENT WHOLE-RELEASE REVIEW - NOT ACCEPTED**
