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
