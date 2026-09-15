# Metric catalog disposition

Every instrument in `metric-catalog.json` with its language owner and evidence
category. A catalog declaration without an emitting call is **not implemented**,
even if another language emits the same family.

Evidence categories, from weakest to strongest:

- **source inspected** — the emitting owner exists and was reviewed, but no
  sample was collected from it in this candidate.
- **executed boundary** — a focused test exercises the real owner and asserts
  its observation (for example the R1 dispatch outcome tests or the exporter
  coalescing test), without a live Collector.
- **collected metric** — a real sample was captured through a Collector into
  Prometheus by the workflows in `evidence.md`.
- **collected trace** — a real span with its parent relationship was captured
  through a Collector by those workflows.
- **not implemented** — the catalog declares an owner but no production emitting
  call exists at that language's boundary. No sample from another owner closes
  it.

The exact commands, emitting source SHA, provider settings, and sanitized
samples behind collected categories are in `docs/observability/evidence.md` and
the source-pinned `collected-*.json` manifests. The historical
`collected-2bfabca3.json` remains pinned to its original source.

| Metric                                      | Emitting owner                                                                                  | Evidence                                                      |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `trellis.auth.flow.duration`                | `runtime/src/platform/auth/{compiled_evidence,api_bindings,http/bootstrap}` (`record_duration`) | collected metric                                              |
| `trellis.auth.callout.duration`             | `runtime/src/platform/auth_callout.rs`                                                          | collected metric                                              |
| `trellis.contract.analysis.duration`        | `runtime/src/platform/auth/compiled_evidence.rs`                                                | collected metric                                              |
| `trellis.contract.cache.requests`           | `runtime/src/platform/auth/compiled_evidence.rs` (`record_cache_request`)                       | collected metric                                              |
| `trellis.connect.duration`                  | Rust `trellis/src/client/connection.rs` connect wrappers                                        | collected metric                                              |
| `trellis.connect.duration`                  | TS `client_connect.ts`, `service/runtime/service.ts`                                            | collected metric                                              |
| `trellis.auth.approval_resolution.duration` | TS production owner: no emitting call found (declaration in `telemetry/metrics.ts` only)        | not implemented                                               |
| `trellis.admin.workflow.duration`           | TS `trellis-test/src/admin/{deployment,admin_client}.ts` (test automation only)                 | source inspected; production owner not identified             |
| `trellis.errors`                            | Rust `trellis/src/client/connection.rs`                                                         | source inspected                                              |
| `trellis.errors`                            | TS `recordTrellisError`/`recordRuntimeError`                                                    | source inspected                                              |
| `trellis.rpc.client.duration`               | Rust `trellis/src/client/connection.rs`                                                         | collected metric                                              |
| `trellis.rpc.client.duration`               | TS `session.ts`                                                                                 | collected metric                                              |
| `trellis.rpc.client.attempts`               | Rust `trellis/src/client/connection.rs`                                                         | collected metric                                              |
| `trellis.rpc.client.attempts`               | TS `session.ts`                                                                                 | collected metric                                              |
| `trellis.rpc.server.duration`               | Rust `trellis/src/service/request_loop.rs`                                                      | collected metric and trace                                    |
| `trellis.rpc.server.duration`               | TS `session.ts`                                                                                 | collected metric and trace                                    |
| `trellis.rpc.server.inflight`               | Rust `trellis/src/service/request_loop.rs`                                                      | collected metric                                              |
| `trellis.rpc.server.inflight`               | TS `session.ts`                                                                                 | collected metric                                              |
| `trellis.http.server.duration`              | `runtime/src/platform/auth/http/telemetry.rs`                                                   | collected metric                                              |
| `trellis.connection.count`                  | Rust `trellis/src/telemetry/lifecycle.rs`                                                       | collected metric; lifetime test                               |
| `trellis.connection.count`                  | TS connection registry: no emitting call found                                                  | not implemented                                               |
| `trellis.connection.transitions`            | Rust `trellis/src/telemetry/lifecycle.rs`                                                       | collected metric                                              |
| `trellis.connection.transitions`            | TS `connection.ts`                                                                              | collected metric                                              |
| `trellis.auth.refresh.attempts`             | Rust `trellis/src/client/connection.rs` refresh owner                                           | source inspected                                              |
| `trellis.auth.refresh.attempts`             | TS refresh owner: no emitting call found                                                        | not implemented                                               |
| `trellis.auth.verification.duration`        | Rust `runtime/src/platform/auth/verifier.rs`                                                    | collected metric                                              |
| `trellis.auth.verification.duration`        | TS authorization verifier: no emitting call found                                               | not implemented                                               |
| `trellis.auth.coverage.count`               | Rust `trellis/src/client/authorization/provider_cache.rs`                                       | collected metric                                              |
| `trellis.auth.coverage.count`               | TS provider cache: no emitting call found                                                       | not implemented                                               |
| `trellis.auth.post_commit.duration`         | `runtime/src/platform/auth_post_commit.rs::dispatch_action`                                     | collected metric                                              |
| `trellis.auth.post_commit.pending`          | `runtime/src/telemetry/snapshots.rs` Auth sampler                                               | collected metric                                              |
| `trellis.auth.post_commit.oldest.age`       | `runtime/src/telemetry/snapshots.rs` Auth sampler                                               | collected metric                                              |
| `trellis.job.submission.duration`           | Rust `trellis/src/jobs/manager.rs`                                                              | collected metric                                              |
| `trellis.job.submission.duration`           | TS `service/runtime/internal_jobs/job-manager.ts`, `jobs.ts`                                    | collected metric                                              |
| `trellis.job.attempt.duration`              | Rust `trellis/src/jobs/runtime_worker.rs` attempt                                               | collected metric and linked span                              |
| `trellis.job.attempt.duration`              | TS `service/runtime/internal_jobs/job-manager.ts::processWithHeartbeat`                         | collected metric and linked retry/completion spans            |
| `trellis.job.lease.events`                  | Rust `trellis/src/jobs/runtime_worker.rs` acquire/renew/release                                 | collected metric                                              |
| `trellis.job.lease.events`                  | TS `service/runtime/internal_jobs/key-coordinator.ts`                                           | collected acquire/release; renew/takeover source inspected    |
| `trellis.jobs.ready`                        | `runtime/src/telemetry/snapshots.rs` Jobs sampler                                               | source owner                                                  |
| `trellis.jobs.oldest_ready.age`             | `runtime/src/telemetry/snapshots.rs` Jobs sampler                                               | source owner                                                  |
| `trellis.jobs.dead`                         | `runtime/src/telemetry/snapshots.rs` Jobs sampler                                               | source owner                                                  |
| `trellis.jobs.worker.registrations`         | `runtime/src/telemetry/snapshots.rs` Jobs sampler                                               | source owner                                                  |
| `trellis.jobs.waiting_retry`                | `runtime/src/telemetry/snapshots.rs` Jobs sampler                                               | source owner                                                  |
| `trellis.operation.execution.duration`      | Rust `trellis/src/service/operations.rs` acquired execution                                     | collected metric and linked span                              |
| `trellis.operation.execution.duration`      | TS `service/runtime/core.ts::executeHandler` acquired execution                                 | collected terminal/interrupted metric and linked spans        |
| `trellis.operation.active`                  | Rust `trellis/src/service/operations.rs` acquired execution                                     | collected metric                                              |
| `trellis.operation.active`                  | TS `service/runtime/core.ts::executeHandler` acquired execution                                 | collected balanced zero after completion/failover             |
| `trellis.operation.ownership.events`        | Rust `trellis/src/service/operations.rs` claim/renew                                            | collected metric                                              |
| `trellis.operation.ownership.events`        | TS `service/runtime/core.ts` fenced claim/recover/renew/control                                 | collected claim/recover/control; renew source inspected       |
| `trellis.event.publish.duration`            | Rust `trellis/src/client/connection.rs`                                                         | collected metric                                              |
| `trellis.event.publish.duration`            | TS `session.ts`                                                                                 | collected metric                                              |
| `trellis.event.process.duration`            | Rust `trellis/src/service/runtime_facade.rs` Consumer handler (no `EventProcess` emission)      | not implemented                                               |
| `trellis.event.process.duration`            | TS `session.ts::#invokeEventHandler` (durable/ephemeral Consumer attempts)                      | collected metric                                              |
| `trellis.delivery.dispositions`             | Rust `trellis/src/client/connection.rs` ack/nak/term                                            | source inspected                                              |
| `trellis.delivery.dispositions`             | TS `service/runtime/internal_jobs/runtime-worker.ts` job ACK/NAK                                | collected metric                                              |
| `trellis.delivery.dispositions`             | TS `session.ts` Event Consumer ACK/NAK: no emitting call found                                  | not implemented                                               |
| `trellis.consumer.pending`                  | `runtime/src/telemetry/snapshots.rs` Consumer sampler                                           | collected metric                                              |
| `trellis.consumer.ack_pending`              | `runtime/src/telemetry/snapshots.rs` Consumer sampler                                           | collected metric                                              |
| `trellis.consumer.missing`                  | `runtime/src/telemetry/snapshots.rs` Consumer sampler                                           | collected metric                                              |
| `trellis.consumer.progress.age`             | `runtime/src/telemetry/snapshots.rs` Consumer sampler                                           | collected metric                                              |
| `trellis.dlq.entries`                       | `runtime/src/telemetry/snapshots.rs` Events DLQ sampler                                         | collected metric                                              |
| `trellis.dlq.transitions`                   | `events-runtime/src/dead_letters/journal.rs::append`                                            | collected metric                                              |
| `trellis.projection.duration`               | `events-runtime/src/projector.rs::persist_message` (component `events`)                         | collected metric                                              |
| `trellis.projection.pending`                | `runtime/src/telemetry/snapshots.rs` projection samplers                                        | collected metric                                              |
| `trellis.projection.progress.age`           | `runtime/src/telemetry/snapshots.rs` projection samplers                                        | collected metric                                              |
| `trellis.storage.duration`                  | Rust `runtime/src/platform/auth/sqlite/common.rs` wait/execute/total                            | collected metric                                              |
| `trellis.storage.duration`                  | TS `kv.ts::TypedKV` caller-visible total                                                        | collected KV read/CAS/conflict; disabled-mode boundary        |
| `trellis.transfer.duration`                 | Rust `trellis/src/client/connection.rs`                                                         | source inspected                                              |
| `trellis.transfer.duration`                 | TS `transfer.ts`                                                                                | source inspected                                              |
| `trellis.transfer.wire.bytes`               | Rust `trellis/src/client/connection.rs`                                                         | source inspected                                              |
| `trellis.transfer.wire.bytes`               | TS `transfer.ts`                                                                                | source inspected                                              |
| `trellis.feed.active`                       | Rust `trellis/src/client/connection.rs` feed guard                                              | source inspected                                              |
| `trellis.feed.active`                       | TS feed owner: no emitting call found                                                           | not implemented                                               |
| `trellis.feed.ends`                         | Rust `trellis/src/client/connection.rs` feed guard                                              | source inspected                                              |
| `trellis.feed.ends`                         | TS feed owner: no emitting call found                                                           | not implemented                                               |
| `trellis.resource.bindings`                 | `runtime/src/telemetry/snapshots.rs` Auth sampler                                               | collected metric                                              |
| `trellis.runtime.component.ready`           | `runtime/src/supervisor.rs` lifecycle transitions                                               | collected metric                                              |
| `trellis.runtime.component.observed.time`   | `runtime/src/supervisor.rs` lifecycle transitions                                               | collected metric                                              |
| `trellis.runtime.lease.events`              | `runtime/src/ownership.rs` acquire/release                                                      | collected metric                                              |
| `trellis.snapshot.observed.time`            | every `runtime/src/telemetry/snapshots.rs` sampler                                              | collected metric                                              |
| `trellis.snapshot.errors`                   | every `runtime/src/telemetry/snapshots.rs` sampler                                              | source owner                                                  |
| `trellis.cli.duration`                      | `rust/crates/cli/src/app.rs` dispatch                                                           | source inspected; earlier CLI sample not repeated on this SHA |
| `trellis.browser.navigation.duration`       | `web/src/lib/browser_telemetry.ts`, TS `browser.ts`                                             | collected metric in Chromium                                  |
| `trellis.browser.errors`                    | `web/src/lib/browser_telemetry.ts`                                                              | browser/npm checks                                            |
| `trellis.telemetry.route_overflow`          | Rust `trellis/src/telemetry/instruments.rs` route catalog                                       | source inspected; overflow tests                              |
| `trellis.telemetry.route_overflow`          | TS `telemetry/metrics.ts::routeToken`                                                           | source inspected; overflow tests                              |

## Coverage

The corrected Rust and TypeScript Operation, Job, and KV owners above emit
independently. A sampled family from one language does not establish the other's
coverage. Source inspection still finds these catalog-owner gaps, outside the
bounded M1a–M1c implementations:

- Rust Consumer execution has no `EventProcess` emission at
  `trellis/src/service/runtime_facade.rs`'s handler boundary.
- TypeScript has no emitting call for refresh attempts, verification duration,
  coverage count, connection count, Feed active/ends, or Event Consumer
  dispositions (Job ACK/NAK is implemented separately).
- `trellis.auth.approval_resolution.duration` has no production emitting call;
  `trellis.admin.workflow.duration` currently emits only from `trellis-test`
  administration automation, not a production SDK or application owner.

These are **not implemented** for the stated language/owner, not uncollected
implementations. No substitute owner or new instrumentation design is assumed.
