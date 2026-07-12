---
title: Trellis TypeScript Contract Authoring
description: TypeScript contract authoring architecture centered on kind-specific helpers, direct descriptors, and inferred runtime surfaces.
order: 20
---

# Design: Trellis TypeScript Contract Authoring

## Prerequisites

- [../core/trellis-patterns.md](./../core/trellis-patterns.md) - service and app
  boundaries
- [trellis-contracts-catalog.md](./trellis-contracts-catalog.md) - canonical
  manifest and `uses` semantics
- [../tooling/trellis-cli.md](./../tooling/trellis-cli.md) - source-first CLI
  boundary

## Context

The participant contract is both the canonical permission blueprint and the
source of the participant's TypeScript runtime surface. Dependency actions are
selected with generated descriptors; runtime facilities are selected with
`state(...)`, `kv(...)`, `store(...)`, and `jobs(...)` descriptors. Trellis
derives connected methods and resource handles from those selections.

This is especially awkward because Trellis participants are broader than
long-running services. Apps, CLIs, browser clients, and other callers also
connect to Trellis and need a typed declaration of what they own and what they
use.

This document records TypeScript contract-authoring architecture: package
ownership, generated projections, `uses` enforcement, and how TypeScript helpers
must emit canonical manifests. It is not the TypeScript tutorial or API
reference. Ordinary usage examples and exact signatures belong in
`/guides/libraries/typescript` and `/api`.

## Design

Trellis adopts a contract-first TypeScript model.

Every TypeScript participant that connects to Trellis defines one contract
through a single high-level API. That contract becomes the source of truth for
both:

- the emitted `trellis.contract.v1` release artifact
- the TypeScript `trellis` runtime API surface available to that participant

### 1) Primary authoring API

TypeScript authoring uses kind-specific public helpers:

- `defineServiceContract(...)`
- `defineAppContract(...)`
- `defineAgentContract(...)`
- `defineDeviceContract(...)`

These helpers are the public TypeScript authoring surface. Docs and normal
authored contract modules should use the kind-specific helper that matches the
participant.

This public surface covers contract authoring, emitted artifacts, and derived
runtime API views. Supporting internals should extend these helpers rather than
introducing alternate authoring entrypoints.

### 2) Package boundary

The kind-specific contract authoring helpers are available from
`@qlever-llc/trellis` and are the normal authoring entrypoint for everyday
contract source modules.

`@qlever-llc/trellis/contracts` remains the advanced contract-system surface for
broader contract-model helpers and codegen-facing types.

Rules:

- `@qlever-llc/trellis` is the normal package for kind-specific contract
  authoring helpers and runtime client connection helpers
- `@qlever-llc/trellis/contracts` is the advanced package for broader
  contract-model helpers, canonicalization, and SDK/codegen-facing types
- normal contract source modules and runtime client code should prefer
  `@qlever-llc/trellis`; advanced contract-model imports should come from
  `@qlever-llc/trellis/contracts`
- specialized helper return values remain usable anywhere a generated SDK
  contract module or runtime contract object is expected
- `@qlever-llc/trellis/service/node` and `@qlever-llc/trellis/service/deno`
  consume contract objects for service runtime helpers
- generated API reference owns the precise package export inventory and helper
  signatures

### 3) Descriptor-driven `uses`

TypeScript authors do not hand-write remote dependency contract ids in normal
use.

Generated owner SDKs export direct action descriptors, portable types, and
schemas. They do not export participant contracts, merged API maps, clients, or
dependency selectors. Authors place required descriptors directly in the
contract's `uses` array and wrap optional descriptors with `optional(...)`.

```ts
uses: [
  AccountsGet,
  InvoicePaid.subscribe,
  optional(HealthQuery, HealthWatch),
  state({ drafts: { kind: "map", schema: ref.schema("Draft") } }),
  kv({ cache: { schema: ref.schema("CacheEntry") } }),
  store({ uploads: { purpose: "Pending uploads" } }),
  jobs({ refresh: { payload: ref.schema("RefreshInput") } }),
];
```

Action identity comes from the descriptor's exact owner contract ID. Local
import names are not manifest aliases and do not affect digest identity.

The required user-facing contract metadata is:

- `displayName`
- `description`

Contracts that own capability-gated surfaces SHOULD also declare top-level
capability metadata. TypeScript authors write local capability keys in the
contract source; emission projects declared local keys to global capability keys
using `<contract id without @vN>::<local capability>`.

The emitted manifest contains `trellis.jobs::admin.read` in both the top-level
`capabilities` map and the RPC capability list. Undeclared platform capabilities
such as `service` remain raw strings.

Operations that accept post-start caller input declare named signals in the
operation descriptor. Signal input schemas live in the local schema registry and
are referenced with the same `ref.schema(...)` pattern as operation input,
progress, and output schemas.

Rules:

- `signals` is an operation-local map of named post-start inputs.
- each signal requires an `input` schema reference from the local schema
  registry.
- signal schemas are reachable contract schemas and therefore participate in
  manifest emission, digest projection, validation, docs, and generated SDK
  aliases.
- `capabilities.control` is the coarse capability gate for signal submission;
  `capabilities.cancel` remains the coarse gate for cancellation only.
- TypeScript operation references expose universal `cancel()` and `signal(...)`
  helpers for ergonomic wrappers. Unsupported cancel or signal attempts are
  expected runtime failures returned through `Result` / `AsyncResult`, not
  omitted protocol semantics.

For locally authored TypeScript contract source files, whether a top-level
`contract.ts` or `contract.js` for a single contract or `contracts/*.ts` for a
multi-contract layout:

- the file MUST `default export` the contract helper return value
- Trellis source loading resolves the default export only for TypeScript
  contract files
- authors should use the kind-specific helper that matches the participant kind;
  exact overloads and setup examples belong in `/guides/libraries/typescript`
  and `/api`
- `schemas` and local `errors` act as local registries supplied to the contract
  builder for service contracts, while the callback body defines the emitted
  contract body including owned surfaces, resources, `uses`, and `exports`
- emitted manifest fields such as `exports` are authored in the callback body,
  not in the local registry argument
- app-, agent-, and device-style contracts may also take a `schemas` registry
  when they declare schema-backed runtime features such as `state(...)`
- schema and error references should use the public reference helpers so
  manifest emission can validate local declarations and built-in Trellis RPC
  errors
- `TransportError` is built into Trellis runtime call surfaces, but it is not a
  contract-authored RPC `errors: [...]` entry; it represents Trellis
  transport/runtime boundary failures rather than a handler-declared remote
  error
- authors should not construct generated-style contract or API wrapper objects
- generated SDK package roots export `descriptors`, `types`, and `schemas`;
  `manifest` is a tooling-only subpath
- local `operations`, `rpc`, `events`, and `errors` remain the source for owned
  actions; state, KV, store, and jobs are declared as runtime feature entries in
  `uses`
- local top-level `capabilities` metadata remains the source for emitted global
  capability metadata and approval copy
- a participant may omit owned `operations`, `rpc`, or `events`, and may omit
  `uses`
- the defined contract computes and exposes the manifest digest from the emitted
  canonical manifest

### 3a) Service-local RPC errors

TypeScript contract authoring also owns service-local transportable RPC errors.

Authors should normally create them through the public error helper and register
the generated error classes directly in the builder registry `errors` map. Full
syntax belongs in `/guides/libraries/typescript` and `/api`.

Rules:

- the `errors` map stays local to the contract rather than using a central
  global registry
- new local transportable errors should normally use `defineError(...)`
- each local transportable error still becomes a real runtime class, not a plain
  manifest object
- the generated class `type` is the wire `type`
- `defineServiceContract(...)` derives manifest-emitted local error schema refs
  from local error runtime metadata when the schema is not already present in
  the local `schemas` map
- authors may still include the error schema explicitly in `schemas` when they
  want a stable local schema key or to reference that schema elsewhere
- RPC `errors: [...]` entries should usually be authored through
  `ref.error(...)` so local declaration keys and built-in Trellis errors share
  one pattern
- `TransportError` should not be used as a service-local domain error; it is
  reserved for Trellis-native transport/runtime boundary failures, while
  `UnexpectedError` remains for true internal or otherwise unexpected conditions
- the emitted manifest remains plain JSON; Trellis attaches JS-only
  reconstruction metadata to the local contract object rather than serializing
  class constructors
- generated TypeScript SDKs follow the same class shape so external TS consumers
  also receive real error instances
- callers receive declared remote errors as reconstructed runtime instances of
  the declared class where the SDK or local contract has runtime metadata
- undeclared or unknown remote error payloads remain forward-compatible and fall
  back to `RemoteError`

Required descriptors fail closed when their owner contract or surface is
unavailable. Descriptors inside `optional(...)` remain part of contract identity
but grant no authority when unavailable. Participant contracts do not receive
implicit public Auth selections; caller-visible Auth actions must be explicit.
State feature descriptors privately select the State transport actions needed by
their typed state facade.

### 3b) Event consumer groups

TypeScript service contracts declare durable event processing with the top-level
`eventConsumers` map. Each group selects dependency events through
`eventConsumers.<group>.uses` and owned events through
`eventConsumers.<group>.self`.

Dependency selections use exact owner contract IDs. A subscribe descriptor in
the top-level `uses` array grants live subscribe authority, while
`eventConsumers` asks Trellis to materialize a durable cursor over selected
authorized dependency events and/or events owned by the same contract.

Example:

```ts
const contract = defineServiceContract({ schemas }, () => ({
  id: "billing-projection@v1",
  displayName: "Billing Projection",
  description: "Projects billing events into workspace state.",
  uses: [BillingSubscriptionConfirmed.subscribe],
  eventConsumers: {
    workspaceBilling: {
      uses: {
        "billing@v1": ["Billing.SubscriptionConfirmed"],
      },
      replay: "new",
      ordering: "strict",
      concurrency: 1,
      ackWaitMs: 300_000,
      maxDeliver: 6,
      backoffMs: [5_000, 30_000, 120_000, 600_000, 1_800_000],
    },
  },
}));
```

For durable self-consumption, list owned events in `self`:

```ts
const contract = defineServiceContract({ schemas }, (ref) => ({
  id: "entity@v1",
  displayName: "Entity",
  description: "Owns and durably processes entity ingest events.",
  events: {
    "Entity.Observation": {
      version: "v1",
      event: ref.schema("Observation"),
    },
  },
  eventConsumers: {
    ingest: {
      self: ["Entity.Observation"],
    },
  },
}));
```

Rules:

- `replay` defaults to `"new"`; use `"all"` only when a new deployment should
  project all retained historical events
- `ordering` defaults to `"strict"`, and strict ordering requires
  `concurrency: 1`
- each group must declare at least one selected dependency event in `uses` or
  one owned event in `self`
- group names are logical aliases; service code passes the alias as
  `opts.group`, while Trellis provisions the physical durable consumer name
- `eventConsumers.<group>.uses.<contractId>` must point at the exact owner of a
  subscribe descriptor selected in `uses`, and each listed event must be
  selected there
- `eventConsumers.<group>.self` names events from the same contract's `events`
  map
- callers must not pass `durableName` for service event processing
- runtime durable consumers are Trellis-provisioned only; service code consumes
  reconciled bindings and must not create arbitrary JetStream durable consumers
- one event may appear in multiple groups when the service intentionally wants
  independent durable cursors and duplicate delivery
- docs metadata may describe the group for review UIs, but nested docs do not
  affect the digest projection

### 3c) Named contract state stores

TypeScript contract authoring declares public Trellis-managed state with a
`state(...)` runtime feature descriptor in `uses`.

Rules:

- state stores are declared inside one `state({...})` selection
- each state store requires `kind: "value" | "map"`
- each state store requires `schema: ref.schema("...")`
- the referenced schema must exist in the local `schemas` registry
- each state store may declare `stateVersion`; omit it only when the default
  `"v1"` is sufficient
- keep `stateVersion` stable for additive compatible schema changes and bump it
  only when stored values require migration
- `acceptedVersions` declares older state versions and schemas that the runtime
  can surface for app/device-side migration
- the declared stores project to the runtime surface at `trellis.state.<store>`
- normal runtime callers do not declare or pass a public `scope`
- conditional writes use runtime `put(..., { expectedRevision })`, not a
  separate compare-and-set helper

State-specific runtime, migration, validation, and corruption-handling rules are
canonicalized in [../core/state-patterns.md](./../core/state-patterns.md). Exact
state helper signatures belong in the generated TypeScript API reference under
`/api`.

### 3d) Exported schemas and SDK type reuse

Service-owned data model types that cross a contract boundary should be declared
as named schemas and exported through `exports.schemas`.

Rules:

- browser apps, devices, and peer services should import server-owned model
  types from the generated SDK instead of redefining those shapes locally
- generated TypeScript SDKs export aliases for schemas listed in
  `exports.schemas`
- generated RPC, operation, event, and job types should reuse exported schema
  aliases when nested wire shapes match those exported schemas
- exact alias names and declaration forms belong in the generated TypeScript API
  reference under `/api`

### 4) TypeScript enforcement of declared permissions

The TypeScript type system must enforce both of these rules:

- a referenced remote operation, RPC, event, or feed must exist on the imported
  SDK module
- a participant may only invoke, call, publish, or subscribe to remote
  operations, events, and feeds explicitly selected in its local contract

This makes two important guarantees in normal authoring: nonexistent actions
cannot be imported as descriptors, and actions omitted from `uses` do not appear
on the connected runtime.

No separate linting or external analysis tool is required for this workflow. The
contract object itself defines the allowed TypeScript runtime surface.

### 5) Derived runtime surfaces

The contract definition retains private transport metadata and exposes direct
owned descriptors. Connection helpers project that metadata into two public
roles:

- callers receive flat methods for selected actions
- providers receive flat `handle<Name>` registrations for owned RPCs,
  operations, and feeds; `on<Name>` for subscriptions; and `publish<Name>` for
  owned publication

Rules:

- caller methods derive only from descriptors selected in `uses`
- provider registrations and owned event publication derive only from locally
  owned actions
- state, KV, store, and jobs remain structured runtime handles
- the private session may merge transport metadata internally, but no public API
  map or merged facade is exported

This preserves the distinction between what a participant owns and what it is
merely allowed to use.

### 6) Runtime connection helpers are contract-driven

TypeScript runtime helpers consume contract objects directly. The design
requirement is that connection helpers receive the local participant contract
and return contract-derived active or provider facades; exact connection option
shapes and examples belong in `/guides/libraries/typescript` and `/api`.

Rules:

- connected clients and services expose contract-inferred flat methods
- server registration uses `service.handle<Name>(handler)`
- handlers should use the payload type Trellis derives from the registration;
  docs and examples should not re-parse mounted payloads
- mounted RPC handlers may return either `Result` or `Promise<Result>`
- returned runtimes must not expose raw request, publish, listen, NATS, or
  JetStream escape hatches
- service-side helpers must not expose used remote APIs as mountable local
  handlers
- request and operation helpers may fail with `TransportError` for Trellis
  transport/runtime boundary failures even when that error is not a
  contract-authored remote error; `UnexpectedError` remains for true internal or
  otherwise unexpected runtime conditions
- returned runtimes expose operation-native send transfer through the transfer
  builder flow and grant consumption through runtime transfer helpers
- contract descriptors declare transfer direction explicitly for operations that
  ingest caller bytes and RPCs that issue service-owned byte grants
- inline handlers infer from `service.handle<Name>(...)`; extracted handlers use
  `Parameters<ConnectedTrellisService<typeof contract>["handle<Name>"]>[0]`
- extracted handler factories close over application dependencies; the handler
  type remains a plain function signature without a dependency slot
- callers do not manually assemble runtime API arrays for normal usage
- locally authored contracts should normally export the helper return value
  directly; do not wrap it in a generated-style compatibility object
- for TypeScript contract source files, that direct export should be the file's
  default export so prepare/generation can resolve it consistently
- single-contract examples should normally use a top-level `contract.ts`
- for contracts that own schemas or local errors, keep the local registries
  separate from the emitted contract body so generation can validate references
- keep the first `define*Contract(...)` argument limited to local authoring
  registries such as `schemas` and service-local `errors`; put emitted contract
  body fields such as `exports` inside the callback return object
- Trellis-specific bootstrap exceptions should stay in Trellis platform code and
  use lower-level runtime APIs directly rather than becoming general public
  service helpers

### 7) Scope of contracts beyond connect

Contracts matter beyond the initial connect phase.

In TypeScript they remain the source for:

- emitted manifest generation
- runtime operation, call, and subscribe typing
- owned handler and publisher typing
- `CONTRACT_ID` and digest metadata used for discovery and binding lookup

This document therefore treats the contract object as the primary participant
definition, not as a one-time connection option.

## Normative Surface Ownership

This document constrains the architectural direction behind the TypeScript
contract API. Exact public signatures, contract-module types, runtime helper
members, overloads, and generated inventories belong in the generated TypeScript
API reference under `/api`.

The architectural rules are:

- kind-specific helpers are the supported public authoring entrypoints for
  normal local contract modules
- `@qlever-llc/trellis` exposes the preferred contract authoring helpers used by
  apps and services and returns contract objects with manifest and private
  runtime metadata
- `@qlever-llc/trellis` also remains the runtime package for
  `TrellisClient.connect(...)`, auth helpers, and `Result`
- runtime connection helpers live in `@qlever-llc/trellis` and
  `@qlever-llc/trellis/service*`
- generated owner SDKs are vocabulary packages, not participant contracts
- `uses` declarations are direct descriptor lists rather than handwritten
  dependency objects
- caller and provider surfaces are inferred at connection time from the local
  participant contract
- public documentation should lead with `TrellisClient.connect(...)`,
  `TrellisService.connect(...)`, and `TrellisDevice.connect(...)`; public
  service author guidance should not point at Trellis-internal bootstrap paths
- emitted manifests remain canonical `trellis.contract.v1` artifacts; this
  design does not create a parallel manifest format
- TypeScript compile-time typing enforces declared remote usage shape, while
  runtime validation still enforces canonical manifest, auth, subject ownership,
  and dependency-resolution rules
- TypeScript authoring is an implementation of the canonical manifest
  architecture, not a parallel manifest format
- generated SDK outputs contain `mod.ts`, `descriptors.ts`, `types.ts`, and
  `schemas.ts`; tooling may additionally consume the non-root `manifest` export

The replacement rule also remains the same: normal TypeScript user code should
not need to use `defineContractSource(...)`, `buildContractArtifacts(...)`, or
`mergeApis(...)` directly once this model is complete.

### User approval semantics

Contracts are also the user-facing identity and approval surface for user-facing
clients.

Rules:

- `displayName` and `description` are what approval and session-management UIs
  show to the user
- top-level `capabilities` metadata is what approval UIs show for requested
  capability-level authority; raw global capability keys are technical detail
- browser apps send their contract manifest during login so auth can plan
  routing and approval; they are approved per-user and are not installed like
  services
- user approval is granted to a specific contract digest, not merely to a
  contract `id`
- if a client changes its contract and therefore changes its digest, it must be
  approved again
- `id` remains useful for lineage and code generation, but approval is bound to
  the exact concrete contract artifact identified by `CONTRACT_DIGEST`
- the canonical manifest and digest still belong to the release boundary, but
  normal app and service repos should generate or verify them inside `dev`,
  `build`, or CI tasks rather than teaching users a separate manual manifest
  step for routine usage

Expected type behavior:

- `service.trellisCatalog({})` is valid when `TrellisCatalog` is selected
- `service.handleTrellisCatalog(...)` is a type error because that RPC is used,
  not owned
- omitted descriptors do not produce callable methods

### Implementation notes

- TS SDK generation emits owner-only descriptors, types, and schemas
- runtime helpers should consume contract objects directly for client and
  service creation
- the emitted manifest format and agent contract workflow stay stable

## References

- `design/contracts/trellis-contracts-catalog.md`
- `design/tooling/trellis-cli.md`
