---
title: Rust Authorization State
description: Rust-owned durable authorization records, materialization, and the internal issuable-state boundary.
order: 18
---

# Design: Rust Authorization State And Materialization

## Scope And Sequence

The Rust platform runtime is the authoritative owner of durable authorization
state. Authorization delivery is split into five ordered steps:

1. the authorization-context and request-proof protocol defines the signed
   objects and pure verification rules;
2. Rust owns principals, sessions, desired authority, runtime evidence, and
   materialized authority, as specified here;
3. Rust owns external auth, bootstrap, session, and auth-callout APIs;
4. Rust issues signed authorization contexts and distributes issuer trust;
5. runtimes validate ordinary context-bound requests locally.

This document specifies the durable authorization-state step. Public auth
routes, context signing, trust distribution, and local request/event runtime
verification are composed from this state by the surrounding Auth design.

## Ownership

Authorization state is part of the platform subsystem. A process in `platform`
or `all` mode must acquire `platform.owner` before it opens the platform SQLite
store, runs migrations, validates authorization records, or starts authority
reconciliation. Loss of that lease immediately aborts the platform task and
terminates the owning process; the process does not reacquire ownership.

The Rust platform database is independent of the TypeScript control-plane
database. There are no shared tables, dual writes, TypeScript repository calls,
or transport-ACL imports. A future deployment may perform a one-time import, but
the durable Rust model does not encode the TypeScript schema.

## TypeScript Ownership Inventory

The complete external-auth cutover inventory is maintained in
`rust-auth-service-ownership.md`. It records every current TypeScript owner,
storage boundary, public or internal surface, required product behavior, Rust
target, and final disposition. TypeScript is a behavioral reference after the
cutover, never a second owner of Rust state.

## Durable Records

All timestamps are Unix milliseconds represented by signed 64-bit integers, but
durable timestamps and counters that can enter a signed authorization object are
restricted to the interoperable JSON integer range `0..=2^53-1`. Versions are
restricted to `1..=2^53-1`. Every unset lifecycle timestamp is stored as SQL
`NULL` and represented by `Option<i64>`. IDs and digests are nonempty canonical
strings.

### Principal And Provider Identity

`auth_principals` stores `principal_id`, kind (`user`, `service`, or `device`),
state (`active`, `disabled`, or `revoked`), creation/update timestamps,
authorization version, and nullable disabled/revoked timestamps. Authorization
state changes use compare-and-swap on the version. Human metadata does not live
in this authority row.

`auth_provider_identities` is uniquely keyed by `(provider, provider_subject)`
and links that identity to a user principal. A provider subject never replaces
the principal ID.

### Session

`auth_sessions` stores one stable session ID and one unique session-key ID. It
binds a principal and principal kind to an exact participant ID, participant
kind, participant artifact digest, accepted-needs digest, canonical Ed25519
public key, derived key ID, and authoritative inbox prefix. It also stores
state, creation/last-seen/expiry/revocation timestamps, and a positive version.
Private keys are never accepted by the repository or persisted.

Touch updates only `last_seen_at` and is conditional on the session remaining
active. Revoke, expire, and explicit participant rebind use compare-and-swap.
Rebinding changes the participant meaning and increments the session version;
ordinary touch does not.

### Exact Participant Binding

`auth_participant_bindings` stores the canonical participant artifact plus the
exact API artifacts used to resolve it, keyed by participant ID and artifact
digest. It also records the resolved needs digest, resolution state, nullable
safe error, and resolution time. Loading reruns protocol parsing and resolution
and verifies both digests. An exact lookup never falls back to the latest row.

### Desired Authority

Identity and deployment authority use separate tables and typed repositories.
Each row stores its stable authority ID, authority subject (principal or
deployment), exact participant identity and artifact digest, accepted-needs
digest, normalized `GrantSetV1`, sorted unique platform capabilities, state,
positive desired version, timestamps, nullable expiry, and nullable decision
fields.

States are `pending`, `accepted`, `rejected`, `revoked`, and `stale`. Any change
to enforceability increments the desired version transactionally: accepted
grants, capabilities, participant or needs digest, state, expiry, or a decision
that changes enforceability. Descriptive text alone does not increment it.

Deployment authority additionally binds one deployment and participant kind. It
never authorizes a different deployment or derives desired authority from an
instance offer.

### Runtime Evidence

Service and device authorization projections store deployment, runtime instance,
device, and device-delegation state as independently owned records. A session
runtime binding only selects a deployment and applicable instance; replacing or
removing that binding does not replace or delete the selected records. Multiple
sessions can select one shared instance, and multiple instances can coexist
under one deployment. A stable instance ID cannot move to another deployment or
principal.

Service issuance requires an active service principal, deployment, and selected
instance. Device issuance additionally requires an active deployment-scoped
device record and any activation/delegation evidence required by that device's
lifecycle. Device delegation is keyed by device principal and deployment so one
deployment cannot replace another deployment's evidence. The in-memory and
SQLite repositories persist and project these same entities and constraints.
Instance lifecycle and delegation expiry apply only to issuance, never to shared
deployment materialization.

Dependency evidence is keyed by typed authority identity and repeats the exact
consumer participant ID, artifact digest, and accepted-needs digest. It records
requiredness, alias, exact API ID and digest, provider participant/deployment/
instance IDs, availability state, and observed time. A stale implementation
offer is unavailable evidence. Two deployments of one participant cannot read or
replace each other's selections.

Resource evidence has the same typed-authority and exact-participant scope. It
records resource kind, local name, stable binding ID, owner participant,
provider/storage identity, state, materialization time, and a nullable safe
error. Authority is never reconstructed from a bucket or stream name.

## Materialized Authority

One current `auth_materialized_authorities` row exists per typed desired
authority identity `(authorityKind, authorityId)`. Identity materialization is
scoped to its user principal, exact participant, and desired authority.
Deployment materialization is scoped to its deployment, exact participant, and
desired authority. It never stores a session, session key, runtime instance,
device instance, delegation, or instance expiry.

The row stores desired authority identity and version, authority-level subject
ID, its own positive materialization version, exact participant evidence,
normalized effective `GrantSetV1`, canonical capabilities, state (`available`,
`unavailable`, or `error`), reconciliation time, nullable error category, and
the authority/deployment expiry bound. Dependency and resource child rows are
replaced in the same transaction.

The materialization version increments only when enforceable output or its
supporting selection changes: grants, capabilities, dependency selection,
resource bindings, authority-level eligibility, state, or authority/deployment
expiry. Session, instance, and delegation changes do not rewrite shared
materialization. A semantic no-op keeps the version and creates no transition.

Materialization is fail closed:

1. load the typed desired authority and its identity principal or deployment;
2. load and verify the exact participant artifact and accepted-needs digest;
3. require accepted, unexpired desired authority and active authority-level
   subject state;
4. load dependency and resource evidence only at the authority's exact scope;
5. begin with the participant's required and optional exact grant sets;
6. require every required API and resource evidence row;
7. remove each optional API grant set or participant-resource selection whose
   evidence is unavailable;
8. intersect the remaining atoms with the accepted desired `GrantSetV1`;
9. intersect requested and accepted canonical capabilities independently;
10. atomically replace materialized authority, supporting evidence, and its
    deterministic transition-outbox record.

No capability, NATS subject, transport ACL, responder, or bucket name creates a
permission atom.

## Versions And Issuable State

The desired authority version is the decision-history version used by the future
signed `authorityRef.version`. The materialization version is an internal
effective-state generation for reconciliation and cache invalidation. They are
never overloaded.

`reconcile_authority(authorityTarget, now)` and
`resolve_issuable_state(sessionId, now)` are separate operations. Reconciliation
converges or invalidates shared authority state. Issuable-state resolution reads
one coherent session snapshot, verifies current session, principal, deployment,
instance, device, and delegation eligibility, and combines that eligibility with
an already-current authority materialization. Issuance never rewrites shared
materialization.

The internal authorization service returns a fully resolved unsigned value with
the protocol principal, exact participant evidence, desired authority reference,
session ID/public key/key ID/inbox prefix, optional deployment and instance IDs,
exact effective grant set, canonical capabilities, session/effective authority/
delegation expiry bounds, and materialization version. The effective authority
expiry comes from current materialization and therefore includes the tighter of
desired-authority and deployment expiry. It signs and encodes nothing.

Deployment and instance IDs follow the authorization-context protocol as a pair.
Service contexts always carry both. Under the current deployment-bound device
policy, device runtime evidence without an instance is rejected before issuance,
so successful device issuable state also always carries both IDs.

Expected unavailable states are typed domain failures, including missing,
expired, revoked, inactive, stale, digest-mismatched, evidence-missing, and
optimistic-conflict cases. Storage corruption and unexpected database errors
remain separate internal failures.

## Reconciliation And Events

Desired-authority, identity-principal, participant, deployment, dependency,
resource, and authority/deployment expiry transitions schedule a typed authority
target. Session, instance, device, and delegation transitions affect issuance
eligibility but do not rewrite deployment materialization. Startup enumerates
the union of desired and materialized targets, creates missing projections,
recomputes every target from complete inputs, invalidates unavailable targets,
and removes orphaned projections before readiness.

Each backend implements reconciliation as one coherent materialization unit of
work. SQLite holds an immediate transaction on its owner-scoped writer
connection across all reads, protocol resolution, semantic computation,
replacement, child-row replacement, and outbox insertion. The in-memory backend
holds one state lock across the equivalent operation. The port returns an opaque
snapshot token that covers all authority-level inputs so a future optimistic
backend can compare the complete revision. Expected optimistic conflicts are
retried at most three times; stale input is never committed.

Meaningful changes insert a deterministic transition into
`auth_transition_outbox` in the same transaction as materialization. Its logical
event ID is derived from authority kind, authority ID, materialization version,
and transition kind. A later Event Log publisher lists pending rows and
acknowledges successful delivery; failed publication or restart leaves the row
pending. The outbox is transient delivery state, not an activity or audit log.
Session touches and semantic no-ops create no outbox row.

## Local Authorization Boundary

Rust owns auth/bootstrap HTTP and RPC, session creation and revocation, login,
NATS Auth Callout, issuer configuration, trust publication, context signing,
bootstrap bundles, and refresh. Providers consume signed contexts locally with
generated route or receiver-owned event metadata; they do not reconstruct
authority or call Auth for an ordinary authorization decision.
