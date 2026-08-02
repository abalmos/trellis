---
title: Auth API
description: Current Rust-owned authentication and authorization boundaries.
---

# Design: Auth API

## Source Of Truth

The exact wire API is the source-owned `trellis.api.v1` artifact at
`rust/crates/runtime/trellis.api.json`. The Rust and TypeScript generated SDKs
are derived from that artifact and MUST NOT be edited by hand.

This document describes boundary and lifecycle semantics. It intentionally does
not duplicate every generated request or response field. For exact names,
nullability, error codes, and schemas, use the artifact and generated API docs.

The paired runtime identity is `rust/crates/runtime/trellis.participant.json`.
Administration uses the separate source-owned
`rust/crates/runtime/trellis.admin.participant.json` artifact.

## Ownership

The Rust platform runtime is the sole owner of:

- browser auth and account flows
- first-admin bootstrap
- service, device, and client bootstrap
- sessions and connection presence
- authority proposals, decisions, and reconciliation
- provisioning and device activation reviews
- NATS Auth Callout
- authorization trust distribution, context issuance, refresh, and revocation
- Auth RPC, operation, and event surfaces

TypeScript owns portal source, generated SDK consumption, session-proof
bindings, client/service/device integration, and tests. It does not register an
auth runtime, access auth tables, proxy mutations, or validate migrated auth
state.

## HTTP Surface

The Rust auth router is installed only after platform ownership, migrations,
repository construction, startup reconciliation, and required KV resources are
ready.

HTTP is limited to these boundary families:

- browser/account and portal flows, including auth-request, provider callback,
  flow, bind, and portal-asset routes
- initial service and device bootstrap
- proof-bound context recovery through `/auth/context/refresh`

Pending devices retry `/bootstrap/device` using the returned delay. Trust
material is returned in bootstrap/refresh responses; there is no public trust
lookup endpoint.

The built-in portal is served under `/_trellis/portal` and `/_trellis/assets/*`.
The retired device connect-info preflight route does not exist;
`POST /bootstrap/device` is the shared initial and reconnect boundary.

There is no HTTP client bootstrap or authorization-registry route, or logout
endpoint. Connected control uses generated Auth RPC, operation, and event
surfaces over NATS. Context refresh is the recovery exception, not a general
connected-control API; it is used after a restart or expired context or route
credential.

Router-wide controls include strict request body limits, configured-origin CORS,
security headers, optional HSTS, and Governor rate limiting keyed from the
actual peer address. Flow-specific origin and redirect checks remain domain
checks.

## Proof Boundary

Bootstrap requests use purpose-specific `trellis.session-proof.v1` envelopes.
Proofs include a format, signer identity, request id, safe-integer issue time,
payload digest, domain-separated purpose, and Ed25519 signature over the
protocol-owned length-prefixed transcript.

Browser bind consumes the proof-bound server flow with an idempotency key. It
does not accept a second concatenated-signature format.

Successful bootstrap or bind returns:

- current `serverNow`
- session metadata and server-owned reply inbox prefix
- exact participant artifact binding
- effective grants and structured resource evidence when applicable
- current NATS endpoints
- a deny-all Auth-account JWT bound to the session NKey
- the route JWT's exact expiry
- a signed authorization context plus pinned root, complete current manifest,
  internal trust/context bucket names, and bounded verification policy

No route returns a shared sentinel seed or reusable shared credential. Non-ready
proposal and activation states return no context. Refresh is session-key
proof-bound, requires nullable `currentContextDigest`, re-evaluates current
issuable state, enforces the client's root pin and manifest floor, and returns
the complete recovery bundle: `serverNow`, the signed context and trust
material, renewed route JWT material, session metadata, and current NATS
endpoints atomically. A null digest recovers a retained valid session after
context or route-JWT expiry; stale, revoked, or otherwise terminal session state
remains fail-closed.

## Browser And Account Flows

Browser flows are CAS-backed and carry the session key and exact participant
binding established by the initiating proof. Local login uses Argon2id. OIDC
uses discovery, PKCE, nonce verification, access-token-hash validation, and a
claim-before-exchange state transition. An exchange with an unknown outcome
becomes terminal `restart_required` rather than risking replay.

OIDC start sets a state-specific `HttpOnly`, `SameSite=Lax` cookie and stores
only its digest. Callback requires that browser binding, verifies the exact
portal and provider policy revision again, and only then claims and exchanges
the state. Unknown identities register only when current portal policy permits
it; identity-link flows remain target-bound and never self-register.

The flow response contains a server-owned consent view and digest. Approval
accepts only
`{ approved, consentViewDigest, selectedOptionalBundles,
idempotencyKey }`;
caller-authored grants, capabilities, expiry, resource atoms, and platform
authority are rejected. It re-resolves the exact participant before deciding the
same immutable authority proposal used by administrative flows. Bind creates the
session through the shared aggregate transaction.

Account-management flow tokens are returned once and stored only as hashes.
First-admin bootstrap creates one unexpired single-use flow when no usable
administrator authority exists and emits its URL only when newly created.
Restart reuses the pending flow without rotating or reprinting the secret.
`trellis-server --rotate-first-admin` is the explicit operator replacement path.
Local and configured OIDC completion atomically create the first account and
exact administration authority; only one concurrent completion can succeed. It
never logs a default password.

Service and device bootstrap may present one exact participant artifact plus all
referenced API artifacts. The server parses, normalizes, resolves, and stores
the binding through the same domain method used by administrative planning.
Stable non-ready states are `manifest_required`, `authority_pending`,
`authority_rejected`, `migration_required`, `dependency_pending`,
`resource_pending`, and `disabled`. Equivalent retries reuse one semantic
proposal; additive compatible replacement is an update and incompatible API
replacement is a migration.

## RPC Surface

The API artifact defines the complete public RPC inventory. Its families cover:

- sessions and connection presence
- users, credentials, and linked identities
- portals, portal settings, and portal routes
- deployments and deployment authority
- service instances and device instances
- identity authority and device-user authority
- activation reviews and connect metadata
- read-only capability review metadata

Representative exact surfaces are `Auth.Deployments.Create`,
`Auth.Devices.List`, `Auth.DeploymentAuthority.Plan`,
`Auth.DeploymentAuthority.AcceptUpdate`,
`Auth.DeploymentAuthority.AcceptMigration`,
`Auth.DeploymentAuthority.Reconcile`, `Auth.IdentityAuthority.List`, and
`Auth.IdentityAuthority.Revoke`. The source artifact remains authoritative for
the complete inventory.

Every aggregate mutation carries an idempotency key and commits its domain
state, idempotency result, and post-commit intents in one repository
transaction. A same-digest replay returns the committed result; a conflicting
digest fails.

Expected caller-visible failures use the typed error codes declared in the API
artifact. Ordinary request and event verification is provider-local and is not
an Auth API surface. Internal storage, crypto, provider, SQL, and topology
causes are logged only; HTTP, RPC, operation, and callout payloads expose fixed
codes and safe context.

Connected Auth control is generated API control over NATS. Session logout,
session revocation, authority changes, activation review, and other
caller-visible control do not gain HTTP counterparts. Service authors use the
generated participant surfaces and returned runtime handles; authorization
registry KV handles, bucket names, and watch subjects are internal runtime
material and are not exposed as service APIs.

Provider runtimes resolve authorization evidence from the connected internal
NATS KV registry. They watch the current manifest generation and revocations,
load both initial snapshots before becoming ready, fail closed while unready,
and stop after their bounded staleness limit. Revocations apply immediately in
memory and are not undone by a deleted record. Unknown context digests resolve
exact evidence once per digest; ordinary request and event cache hits perform
zero HTTP, SQLite, Auth RPC, or registry I/O.

## Authority

Authority proposals are immutable historical records with terminal accepted,
rejected, superseded, or expired decisions. Acceptance atomically updates the
current desired-authority projection and inserts its transition outbox record.
Reconciliation resolves the exact participant artifact and needs against
dependency/resource evidence, then atomically replaces materialized authority.

Mutable capability groups, grant overrides, contract-era identity-grant objects,
authored NATS ACLs, and session-owned authority are not API concepts.

Physical resource bindings never imply access. Exact participant-resource atoms
must be present in `GrantSetV1`; optional evidence contributes only currently
available optional atoms. Bootstrap replay stores only stable admission identity
and reruns current issuance before generating a new JWT.

## Device Activation Operation

`Auth.DeviceUserAuthorities.Resolve` is the caller-visible activation operation.
Its input identifies the durable review only; deployment and instance identity
are resolved from the locked review record. Start, get, wait, and cancel use the
standard authenticated operation router. Approval commits activation and
delegation evidence atomically before the operation reports completion.

The online device wait remains a deliberate bounded pre-auth setup product
requirement. It is identity/proof-bound to the device activation flow and exact
contract evidence, is capped by the server, and is not a general pre-auth RPC or
connected control path. Once approval is observed, the device uses the shared
`/bootstrap/device` boundary for its session, context, route JWT, and NATS
metadata.

## Events And Post-Commit Work

Auth events are declared in `trellis.api.json`; durable actions may publish only
those authored event identities. The runtime signs deliveries with its
runtime-owned auth event session. Revocation and lifecycle mutations enqueue
kick/event intents in the same transaction as their state change. Delivery
failure retries after commit and never rolls the authoritative mutation back.

## Non-Goals

- PostgreSQL and federation
- compatibility adapters for retired TypeScript auth, sentinel credentials,
  capability groups, grant overrides, or legacy binding RPCs
- HTTP context-registry, revocation, logout, and client-bootstrap APIs
