---
title: Observability Patterns
description: Health, stats, documentation, telemetry, and request-correlation patterns for Trellis services.
order: 60
---

# Design: Observability Patterns

## Prerequisites

- [trellis-patterns.md](./trellis-patterns.md) - Trellis architecture and
  communication model
- [type-system-patterns.md](./type-system-patterns.md) - Result and error
  conventions

## Scope

This document defines Trellis observability, documentation, telemetry, and
request-correlation patterns.

## Service Observability

Every service exposes:

- `<Service>.Health` RPC
- baseline heartbeat sample publishing through the private Trellis health
  transport
- optional `<Service>.Stats` RPC
- OpenTelemetry tracing and metrics
- structured logging

Activated devices publish through the same private health transport. Heartbeat
publishing is a runtime protocol grant, not a contract dependency or event
surface, so contract authors do not declare it.

Health example:

```ts
const service = await TrellisService.connect({
  trellisUrl: config.trellisUrl,
  contract: graph,
  name: "graph",
  sessionKeySeed: config.sessionKeySeed,
  server: {
    log,
    healthChecks: {
      db: () => db.ping(),
    },
  },
});

service.health.setInfo({
  version: build.version,
  info: { region: config.region },
});

service.health.add("db", async () => ({
  status: (await db.ping()) ? "ok" : "failed",
  info: { engine: "postgres" },
}));
```

Heartbeat behavior:

- `TrellisService.connect(...)`, Rust `TrellisClient::connect_service(...)`,
  `TrellisDevice.connect(...)`, and Rust `TrellisClient::connect_device(...)`
  publish baseline samples automatically after authenticated bootstrap
- baseline heartbeats include runtime metadata, instance identity, publish
  interval, and a built-in NATS connectivity check
- `service.health.setInfo(...)` and `service.health.add(...)` extend service
  heartbeat payloads at publish time using callback-based state snapshots; the
  same helper surface is also available on device connections
- heartbeat samples are not Trellis events and are not exposed as a public live
  feed; Console reads the Rust-owned health projection through `Health.Query`,
  `Health.Inspect`, and `Health.Metrics`, then uses `Health.Watch` as a
  post-commit invalidation feed

### Runtime Health And Eventlog Views

The Rust runtime has first-class `health` and `eventlog` subsystems. In
all-in-one mode both run with the platform and jobs subsystems. In split mode,
operators run `trellis-server health` for health projection and may omit
`trellis-server eventlog` when projected event capture is not wanted.

Health subsystem rules:

- publishers send samples to
  `health.v1.heartbeat.<kind>.<contract>.<digest>.<deployment>.<instance>.<session>`;
  identity components other than kind and session are unpadded base64url UTF-8
  tokens
- Auth grants each authenticated service or device exactly one matching publish
  subject. The projector treats this subject identity as authoritative and
  rejects payload identity mismatches.
- `TRELLIS_HEALTH` captures `health.v1.heartbeat.>` with file storage, limits
  retention, a default 24-hour maximum age, a default 1 GiB maximum size, and no
  inactive threshold on projector durables
- JetStream ingress time is canonical for freshness. Publisher sample time is
  retained only as diagnostic data. A participant becomes offline at
  `observedAt + 2 * publishIntervalMs`.
- the health store retains only latest instance state, status intervals,
  five-minute metric buckets, bounded rejection diagnostics, and a transition
  outbox; it does not retain one SQL row per raw sample
- health projection is independent from eventlog storage; it must not depend on
  an eventlog store to answer latest or freshness queries
- health stores bounded history according to runtime config, with a default of
  30 days when not overridden
- health projector and retention loops are singleton runtime loops coordinated
  with NATS KV leases
- every committed projection change increments a monotonic revision and
  publishes cross-process invalidation; RPC responses include that revision and
  projection completeness diagnostics
- only meaningful effective-status transitions publish the durable
  `Health.StatusChanged` event on the normal event stream

Eventlog subsystem rules:

- eventlog captures Trellis-owned event subjects under `events.v1.>` and stores
  queryable metadata plus raw payloads for those events
- jobs lifecycle and worker-presence subjects are jobs subsystem stream traffic,
  not initial eventlog input
- eventlog stores full NATS-valid payloads unless a later explicit storage or
  retention policy defines a different bound
- eventlog stores bounded history according to runtime config, with a default of
  7 days when not overridden
- eventlog projector and retention loops are singleton runtime loops coordinated
  with NATS KV leases

Stats example:

```ts
await service.handle.rpc.graph.stats(async () => {
  return Result.ok({
    users: { count: await db.countUsers() },
    partners: { count: await db.countPartners() },
  });
});
```

## Documentation

Exported functions, classes, and methods require JSDoc.

Required fields:

- brief purpose description
- `@param` for each parameter
- `@returns` description
- `@throws` or `@errors` for error conditions
- `@example` for complex usage

Skip JSDoc for private helpers when the code is self-evident and for tests.

## Telemetry

`@qlever-llc/trellis/telemetry` is the public TypeScript telemetry entrypoint.
The former public `@qlever-llc/trellis/tracing` subpath is not part of the
current public surface.

`TrellisService.connect()` initializes OpenTelemetry automatically using the
service name unless automatic telemetry is disabled by the caller. Runtime
helpers must keep browser-safe imports separate from Node/Deno telemetry SDK
setup; package entrypoints may use `@opentelemetry/api`, but exporter and SDK
packages should be loaded only from server/runtime initialization paths.

Tracing rules:

Span naming:

- RPC client: `rpc.client.<MethodName>`
- RPC server: `rpc.server.<MethodName>`
- Event publish: `event.publish.<Domain>.<Action>`
- Event handle: `event.handle.<Domain>.<Action>`
- Job handle: `job.handle.<service>.<queue>`

Required attributes:

- `rpc.system`
- `rpc.method`
- `messaging.destination`

Library support rule:

- libraries performing I/O must accept trace context, create child spans, and
  propagate context
- `TrellisError` subclasses should include `traceId` when tracing is active
- if a runtime has not installed an OpenTelemetry tracer provider, RPC error
  responses should still attach `traceId` from a valid inbound `traceparent`
  header before the error leaves the server span boundary

Error metrics:

- Trellis runtime libraries record caller-visible and runtime-observed failures
  with the `trellis.errors` OpenTelemetry counter
- metrics are no-op unless an OpenTelemetry meter provider/exporter is installed
  or configured by telemetry runtime initialization
- metric attributes must be low cardinality and must not include user IDs,
  session keys, raw subjects, payload data, trace IDs, request IDs, or raw error
  messages
- allowed Trellis error attributes are stable labels such as `trellis.surface`,
  `trellis.direction`, `trellis.operation`, `trellis.phase`,
  `trellis.error.type`, `trellis.remote_error.type`, and bounded
  `trellis.auth.reason`
- `trellis.operation` should come from contract metadata or runtime operation
  kind, not from NATS subjects, URLs, payloads, or remote peer-provided values
- expected public failures remain `Result`-modeled behavior; metrics are
  observability side effects and must not change error semantics

### Latency metrics

Trellis records OpenTelemetry histogram metrics for key lifecycle durations.

Metric naming convention: `trellis.<surface>.<operation>.duration`

Recorded histograms:

- `trellis.connect.duration` — connection lifecycle timing. Attributes:
  `trellis.participant.kind` (`admin`, `client`, `service`, `device`),
  `trellis.phase` (`bootstrap`, `auth_resolution`, `nats_connect`, `total`),
  `trellis.outcome`.

- `trellis.auth.flow.duration` — auth flow timing (login, approval, bind).
  Attributes: `trellis.auth.flow` (`local`, `oauth`, `device`, `bootstrap`),
  `trellis.phase` (`start`, `local_login`, `approval_fetch`, `approval_submit`,
  `bind`, `final_bootstrap`, `total`), `trellis.outcome`.

- `trellis.auth.approval_resolution.duration` — approval-resolution planning
  timing. Attributes: `trellis.phase` (`plan_contract`, `analyze_contract`,
  `load_authorities`, `availability_boundaries`, `load_user`, `load_grants`,
  `resolve_capabilities`, `grant_overrides`, `total`), `trellis.auth.approval`
  (`approved`, `denied`, `required`, `none`), `trellis.authority.present`
  (`true`, `false`).

- `trellis.auth.callout.duration` — NATS auth callout timing. Attributes:
  `trellis.session.kind` (`user`, `service`, `device`), `trellis.phase`
  (`decode`, `validate_token`, `resolve_session`, `build_permissions`,
  `encode_user`, `encode_response`, `respond`, `total`), `trellis.outcome`.

- `trellis.admin.workflow.duration` — admin workflow timing (service
  registration, contract approval, capability grants). Attributes:
  `trellis.operation` (`register_service`, `provision_service`,
  `approve_contract`, `grant_client_capabilities`), `trellis.phase` (`connect`,
  `plan`, `accept`, `reconcile`, `wait_ready`, `total`),
  `trellis.plan.classification` (`install`, `update`, `migration`),
  `trellis.outcome`.

Duration values are recorded in seconds (unit `s`). All metric attributes follow
the same low-cardinality rules as error metrics: do not include user IDs,
session keys, flow IDs, request IDs, trace IDs, contract IDs, NATS subjects, or
URLs. Attributes that fail the low-cardinality check are silently dropped.

Gauge instruments for point-in-time health state (such as active sessions,
pending auth flows, connection counts, queue depth) may be added in a future
pass.

## Request Correlation

RPCs and jobs include a `requestId` for correlation and audit. Domain event
contract payloads are bodies only; Trellis assigns event identity and timestamp
as runtime metadata (`Nats-Msg-Id` and `Trellis-Event-Time` headers) alongside
trace context. Domain events do not currently emit a separate `request-id` NATS
header unless they are job lifecycle events.

Rules:

- the client supplies a unique `request-id` for signed RPCs; auth includes it in
  the RPC proof and replay-cache key
- after auth validation, the server may use the request id as correlation
  context but must still treat logs/traces as observability data, not as a
  source of authorization policy
- request IDs propagate across downstream RPC and job flows
- logs and traces include `requestId`

Propagation:

| Context                        | `request-id` value                           |
| ------------------------------ | -------------------------------------------- |
| RPC handler                    | generated on receipt                         |
| RPC response                   | echoed from handler                          |
| Domain event                   | not set; use `Nats-Msg-Id` and trace         |
| Job created from RPC/event/job | inherited when available; otherwise new ULID |
| Job lifecycle event            | copied from `job.context.requestId`          |
| Scheduled or cron job/event    | new ULID for jobs; event `Nats-Msg-Id` only  |

Job correlation:

- job creation records `job.context.requestId`, `job.context.traceId`,
  `job.context.traceparent`, and optional `job.context.tracestate`
- if no active trace exists when a job is created, the runtime creates a fresh
  W3C trace context rather than leaving the job untraced
- every job lifecycle publish includes matching `request-id`, `traceparent`, and
  `tracestate` NATS headers when present
- workers expose immutable job context to handlers and create job handling spans
  from that context where the language runtime supports tracing

Auth/admin control-plane correlation:

- built-in auth/admin RPCs follow the same inbound `traceparent` extraction as
  application RPCs
- traced admin errors include the request trace ID in serialized Trellis error
  data so operators can correlate failed control-plane calls with logs and spans
- language-owned client integration suites cover traced auth/admin behavior
  through live NATS/auth-callout according to the shared `kind: "client"` cases
  in `integration/test-matrix.json`; successful and failing auth/admin flows
  should be represented as separate matrix cases when both are required coverage

Event deduplication:

- domain events include `Nats-Msg-Id: <event id>` as transport metadata
- domain events include `Trellis-Event-Time: <event timestamp>` as transport
  metadata
- JetStream deduplicates within its configured window
- this protects against duplicate publication on retries and reconnects
