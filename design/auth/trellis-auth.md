---
title: Trellis Authentication And Authorization
description: Rust-owned identity, authority, session, and transport architecture.
---

# Design: Trellis Authentication And Authorization

## Principles

1. The Rust platform runtime is the only auth runtime owner.
2. Authentication, desired authority, materialized authority, and transport
   permissions are separate boundaries.
3. Principals and participants are separate identities.
4. Authority is structured data; NATS subjects never create authority.
5. Every principal uses the same session creation and NATS admission path.
6. Expected failures are typed and fail closed.
7. State-changing workflows are aggregate, idempotent transactions.

## Ownership

Rust owns browser/account flows, OAuth discovery, first-admin bootstrap,
provisioning, sessions, authority proposals and reconciliation, HTTP, Auth RPC,
Auth Callout, connection presence, revocation, and post-commit delivery.

TypeScript remains first class for portal source, generated SDKs, WASM proof
bindings, browser/service/device clients, and integration tests. It does not
own, read, write, proxy, or validate migrated auth state at runtime.

The auth owner starts only after `platform.owner` is held, SQLite migrations and
repository construction complete, startup reconciliation succeeds, and required
KV resources exist. Ownership loss stops admission and terminates the platform
owner; there is no in-process reacquisition by a stale owner.

## Canonical Artifacts

`trellis.api.v1` and `trellis.participant.v1` are protocol-owned runtime
artifacts. The source-owned Auth pair lives in `rust/crates/runtime/`. Normal
TypeScript contracts are compiled to those formats before bootstrap. The Rust
runtime does not maintain a legacy contract-manifest authority parser.

Every authority and session binding pins:

- participant id and kind
- exact participant artifact digest
- exact accepted needs digest
- effective `GrantSetV1`
- effective expiry

## Principals

### Users

The Trellis user account is the stable principal. Local password and OIDC
identities are authentication methods linked to that account. Local credentials
use Argon2id PHC strings and uniform unknown-user verification. OIDC flows use
discovery, PKCE, nonce and `at_hash` validation, and CAS claim-before-exchange.
Each state is bound to a state-specific `HttpOnly`, `SameSite=Lax` browser
cookie and exact portal-policy revision. Callback verifies browser possession
and current provider/registration policy before claim or exchange. Identity-link
flows never register a new account.

### Services

A provisioned service identity is an immutable Ed25519 key assignment to one
deployment and principal. The private key is returned once and never stored by
Trellis. Service bootstrap proves that identity, exact deployment/instance, and
participant binding before creating a session.

### Devices

Devices use either an operator-provisioned identity or one-time-secret
enrollment followed by activation. Identity installation is part of the
provisioning transaction. Pending devices receive no usable session credential.
Device delegation is scoped by device principal and deployment.

## Authority Model

Deployment authority and identity authority use the same split model. Desired
authority records accepted intent. Materialized authority records the currently
resolvable permission-bearing result. They have separate versions.

Authority proposals are immutable and move from pending to accepted, rejected,
superseded, or expired. Acceptance writes the decision, updates the desired
projection, supersedes competing pending proposals, and inserts an outbox record
in one transaction.

Deployment authority has one stable lineage ID derived from the deployment ID
and participant ID. Proposal IDs identify immutable review/decision records, not
authority lineages. The same unexpired pending semantic proposal is reused;
changed semantics supersede older pending records, while rejected, expired, and
superseded history never blocks a later proposal with the same semantics.
Proposal expiry is record metadata and is not part of the desired-authority
semantic digest. Initial proposals are listable by deployment before a desired
authority row exists.

Service/device bootstrap and administrative planning use one protocol-driven
presentation path. It normalizes the participant and exact referenced APIs,
derives required and optional authority, classifies API changes with the
compatibility engine, and creates a semantic proposal against the current
authority version. Equivalent retries reuse one proposal; changed semantics
supersede an older pending proposal.

Reconciliation:

1. reads and validates the current desired record
2. resolves the exact participant and API artifacts
3. loads authority-scoped dependency and resource evidence
4. intersects desired grants, participant needs, and available evidence
5. atomically replaces materialized authority or invalidates stale output

Missing required evidence fails closed. Missing optional evidence omits only the
affected optional grants. A physical resource binding is evidence for an exact
participant-resource atom; it never creates authority. Runtime instance, device,
delegation, and expiry eligibility are evaluated separately at issuance so one
unhealthy instance does not rewrite shared authority.

Mutable capability groups, deployment grant overrides, contract-era authority
shapes, persisted subject ACLs, and session-owned authority are retired.

## Sessions

One aggregate session constructor serves browser bind, service bootstrap, and
device bootstrap. It owns session ids, inbox prefixes, TTL bounds, principal and
participant bindings, runtime bindings, idempotency, reconciliation intent, and
post-commit actions.

The session key is a local Ed25519 credential. Browser clients may keep it for a
single tab or store bounded session seed material for synchronous NATS
reconnect. Service and device clients keep their provisioned identity separately
from each session key.

Revocation commits session state, authored event intent, and connection-kick
intent together. Connection presence is short-lived KV state, not authority.

## Browser And First-Admin Flows

Browser requests are proof-bound to a session key and exact participant
artifact. Flow state is CAS-backed. Portal routing selects UX only; it does not
grant authority. The server stores one exact consent proposal derived from the
resolved participant. Approval accepts only its consent-view digest, selected
server-issued optional bundle ids, the decision, and an idempotency key; callers
cannot submit grants, capabilities, resource atoms, or platform authority.
Approval decides the standard authority proposal, and bind calls the shared
session constructor. The server-owned consent proposal is part of the immutable
browser-flow transcript and cannot change during a state transition.

The built-in portal is compiled reproducibly from `js/portals/login` and
embedded in the Rust binary. A development-only `TRELLIS_BUILTIN_PORTAL_DIR`
override may serve local assets.

When no accepted unexpired administrator authority exists, startup creates one
single-use account flow. Only its hash is stored. Ordinary restarts reuse the
same unexpired pending flow without rotating or reprinting its secret. Operators
may explicitly replace it with `trellis-server --rotate-first-admin`; rotation
atomically revokes the old pending flow. Local-password and configured OIDC
completion both create the account, identity, and exact built-in administration
participant authority in one transaction, with a durable active-admin check that
prevents concurrent double completion. Mandatory admin grants are derived from
that artifact rather than assumed or stored as a mutable group.

## NATS Auth Callout

Bootstrap or browser bind returns a deny-all Auth-account JWT whose subject is
the session key encoded as a NATS User NKey. There is no shared sentinel user or
seed.

Every connect and reconnect uses the same session private key to produce:

- the standard NATS signature over the server challenge nonce
- a separate nonce-bound `trellis.session-proof.v1` proof

The callout pipeline is:

1. XKey decrypt the AuthRequest
2. validate server, audience, account, time, XKey, nonce, and generated user
   NKey
3. validate the deny-all bootstrap JWT
4. verify the standard NATS nonce signature
5. verify the domain-separated Trellis proof
6. admit the proof through CAS replay protection
7. resolve current issuable authorization state
8. verify the exact participant binding and compile transport permissions
9. issue the short-lived target-account user JWT
10. record connection presence

The outer AuthResponse is signed by the Auth-account signing key. The inner user
JWT is signed by the target-account signing key. Its expiry is bounded by the
configured maximum, session expiry, effective authority expiry, and device
delegation expiry.

Transport permissions are deterministic projections of exact grants, API
descriptors, typed resource bindings, the session inbox, and narrow built-in
subjects. Request and event validation resolve the exact API action and require
its atom; capability metadata authorizes only when every mapped atom is present.
Transport subjects and bindings are never read back as authority.

## HTTP And RPC

One auth router is merged into the Rust Axum listener. It owns CORS, security
headers, body limits, peer-address rate limiting, redirect allowlists, and
flow-specific origin checks. See [auth-api.md](./auth-api.md) for route and
surface boundaries; use `rust/crates/runtime/trellis.api.json` for exact current
schemas.

Auth RPC uses the generated dispatcher and implements the complete authored
surface. Transitional `Auth.Requests.Validate` and `Auth.Events.Validate` remain
internal until ordinary local proof validation replaces them in Milestone 10.

## Storage And Atomicity

The V1001 authority core stores principals, participants, sessions, authority,
materialization, runtime evidence, and transition outbox records. V1002
companion state stores accounts, credentials, portals, flows, workflows,
provisioning, activation reviews, replay records, idempotency results, and
post-commit actions.

Accepted migration files are immutable. V1001 remains byte-identical to the
accepted Milestone 7 migration; V1002 performs M8 evolution and is verified
against a populated accepted-M7 database, including repeated upgrade and
foreign-key checks.

Repository methods follow domain transactions, not table CRUD. Every accepted
mutation records its idempotency result in the same transaction as state and
post-commit intents. Replaying the same request digest returns the committed
result; reusing the id with different content fails.

Bootstrap idempotency stores stable admission identity, not serialized JWTs or
other dynamic credentials. Every replay revalidates current session, principal,
authority, deployment, instance, device/delegation, expiry, and participant
binding before minting a fresh credential. HTTP, RPC, operation, and callout
boundaries log internal causes but return only stable public codes and safe
context.

## Trust Evolution

Milestone 8 retains internal request/event validators while Rust owns all auth
state and admission. Milestone 9 adds issuer trust distribution and signed
authorization-context issuance. Milestone 10 moves ordinary request-proof v2
validation local to services and removes `Auth.Requests.Validate` from that
path.

Those later trust layers must consume the same exact principals, participant
bindings, grant sets, authority versions, session identity, and inbox prefix;
they must not introduce a parallel authority model.
