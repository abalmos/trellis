---
title: Rust Auth Service Ownership
description: TypeScript-to-Rust auth responsibility inventory and permanent cutover boundaries.
order: 19
---

# Design: Rust Auth Service Ownership

## Purpose

The Rust `platform` subsystem is the sole runtime owner of external auth,
bootstrap, session, connection, and NATS Auth Callout behavior. The TypeScript
implementation remains only where this inventory identifies client or portal
source, generated vocabulary, or behavioral reference tests. It does not read,
write, proxy, validate, or fall back for Rust-owned auth state.

This inventory records the legacy implementation because product behavior is
preserved without preserving its storage or authority model. The Rust service
extends the accepted records and materialization in
`rust-authorization-state.md`; it does not introduce a parallel auth subsystem.

Disposition meanings are:

- **migrate**: preserve the product behavior on Rust-owned state;
- **replace with clean Rust model**: preserve behavior while retiring the legacy
  storage or authority representation;
- **keep only as test reference**: no supported runtime registration remains;
- **intentionally retire**: the behavior or surface is not part of the Rust API;
- **defer**: the responsibility remains outside this cutover and cannot affect
  current authority.

## Ownership Inventory

Paths are relative to `js/services/trellis` unless stated otherwise.

| Responsibility                   | Current TypeScript module/file                                                                                                                                          | Current storage, KV, or subject                                                                                                 | Current route, RPC, operation, or event                                                       | Required product behavior                                                                                                                                                                                                      | Rust target                                                                                        | Disposition                                                  |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| Browser auth request             | `auth/http/start_request.ts`, `auth/http/browser_routes.ts`, `auth/http/route_context.ts`                                                                               | `contracts`; `trellis_browser_flows`; `trellis_pending_auth`                                                                    | `POST /auth/requests`                                                                         | Validate proof, origin, and exact artifacts; bind directly only when current authority covers exact needs; otherwise create a portal-selected flow                                                                             | `platform::auth::browser::{requests,flow_state}` over participant, session, and authority services | replace with clean Rust model                                |
| OAuth state and callback         | `auth/http/browser_routes.ts`, `auth/http/account_flow_routes.ts`, `auth/oauth.ts`, `auth/providers/*`                                                                  | `trellis_oauth_states`; secure cookie; provider configuration                                                                   | Provider login/callback and account-flow provider login                                       | Authorization code with PKCE, provider-bound CAS state consumption, safe callback validation, stable principal linking, and no token persistence                                                                               | `platform::auth::browser::oauth` and account-flow service                                          | migrate                                                      |
| Local login                      | `auth/http/browser_routes.ts`, `auth/local_credentials/*`                                                                                                               | `user_identities`, `users`, `local_credentials`; browser-flow KV                                                                | `POST /auth/login/local`                                                                      | Normalize username, use uniform failures, enforce lockout/account state, upgrade hash profile, and continue the consent flow                                                                                                   | `platform::auth::account::{local_credentials,users}` and browser-flow service                      | migrate                                                      |
| Local registration               | `auth/http/browser_routes.ts`, `auth/storage/portals.ts`                                                                                                                | Atomic `users`, `user_identities`, and `local_credentials` writes                                                               | `POST /auth/flow/:flowId/register/local`                                                      | Permit only under portal policy; atomically create stable principal, profile, local identity, and credential                                                                                                                   | account and browser domain services                                                                | replace with clean Rust model                                |
| First-admin bootstrap            | `auth/account_flows/bootstrap.ts`, `bootstrap/register.ts`                                                                                                              | `users`, `user_identities`, `account_flows`                                                                                     | Structured startup output and built-in account-flow URL                                       | Create or reuse one expiring single-use flow only when no active administrator exists; log no default credential                                                                                                               | `platform::auth::account::first_admin`                                                             | replace with clean Rust model                                |
| Durable account flows            | `auth/session/account_flows.ts`, `auth/account_flows/*`, `auth/http/account_flow_routes.ts`                                                                             | Hashed tokens in `account_flows`                                                                                                | Account-flow HTTP routes; `Auth.Users.IdentityLink.Create`, `Auth.Users.PasswordReset.Create` | Target-bound, expiring, single-consumption first-admin, identity-link, and password-reset workflows with atomic completion                                                                                                     | `platform::auth::account::flows`                                                                   | migrate                                                      |
| User account and profile         | `auth/session/users.ts`, `auth/storage/sessions_users_approvals.ts`                                                                                                     | `users`; profile observations duplicated in identities and sessions                                                             | `Auth.Users.Resolve/List/Get/Create/Update`                                                   | Stable user principal plus non-authority human metadata, bounded administration, lifecycle checks, and last-admin safety                                                                                                       | principal repository plus `platform::auth::account::users`                                         | replace with clean Rust model                                |
| Provider identities              | `auth/identity.ts`, `auth/account_flows/oauth_completion.ts`, `auth/session/users.ts`                                                                                   | Unique `(provider, subject)` in `user_identities`                                                                               | `Auth.Users.IdentityLink.Create`, `Auth.UserIdentities.List/Unlink`                           | Provider subjects link to stable user principals, cannot move between users, and keep observations separate from account profile authority                                                                                     | existing provider-identity repository plus account identity-link service                           | migrate                                                      |
| Local credentials                | `auth/local_credentials/passwords.ts`, `auth/storage/sessions_users_approvals.ts`                                                                                       | `local_credentials`                                                                                                             | Login, password change/reset, first-admin completion                                          | Argon2id only, versioned bounded profile, library verification, no plaintext, atomic reset, and post-login profile upgrade                                                                                                     | `platform::auth::account::local_credentials`                                                       | replace with clean Rust model                                |
| Login portals                    | `auth/storage/portals.ts`, `auth/admin/portals_rpc.ts`                                                                                                                  | Portal and login-settings/default-policy tables                                                                                 | `Auth.Portals.List/Get/Put/Remove`, `Auth.Portals.LoginSettings.Get/Update`                   | Preserve non-removable built-in portal, disabled state, provider/local-registration policy, and required-nullable settings                                                                                                     | `platform::auth::browser::portals`                                                                 | migrate                                                      |
| Portal routing                   | `auth/storage/portals.ts`, `auth/http/portal_flow.ts`                                                                                                                   | Login and deployment portal routes                                                                                              | Portal route RPCs; browser/activation selection                                               | Deterministic participant/origin/deployment selection with priority and built-in fallback; routes never confer authority                                                                                                       | portal repository and selector                                                                     | replace with clean Rust model                                |
| Built-in portal assets           | `auth/http/builtin_portal.ts`; source under `js/portals/login`                                                                                                          | Compiled static output                                                                                                          | `/_trellis/portal/*`, `/_trellis/assets/*`                                                    | Preserve existing UI without a production filesystem mount                                                                                                                                                                     | Rust embedded-asset HTTP adapter; Svelte remains source                                            | migrate                                                      |
| Identity consent and authority   | `auth/approval/*`, `auth/http/support.ts`, `auth/session/bind.ts`, `auth/storage/sessions_users_approvals.ts`                                                           | `identity_authorities`, `identity_grants`, capability assignments, serialized session evidence                                  | Flow approval/bind; legacy identity-grant RPCs                                                | Decide exact needs digest, `GrantSetV1`, capabilities, and consent metadata; update one current identity authority and reconcile before bind                                                                                   | existing authority/materialization services plus immutable proposals/decisions                     | replace with clean Rust model                                |
| Sessions                         | `auth/session/*`, `auth/http/session_logout_routes.ts`, `auth/storage/sessions_users_approvals.ts`                                                                      | `sessions` columns plus serialized session JSON                                                                                 | Session RPCs; HTTP logout                                                                     | One creation/rebind/revoke path for all principal kinds, stable ID, exact participant evidence, server-owned inbox, bounded list, durable revocation before kick, safe return validation, and provider logout URL construction | existing session repository plus `platform::auth::sessions`                                        | replace with clean Rust model                                |
| Session reconnect                | `auth/bootstrap/client.ts`, `auth/callout/user_reconnect.ts`, client auth helpers                                                                                       | `sessions`, users, capability groups, contracts, deployment/instance and activation state                                       | Client bootstrap and NATS reconnect                                                           | Address by session ID, verify proof with stored key, and reevaluate current issuable state on every bootstrap/connect                                                                                                          | client-bootstrap service and Auth Callout                                                          | replace with clean Rust model                                |
| Client bootstrap                 | `auth/bootstrap/client.ts`, `auth/http/bootstrap_routes.ts`                                                                                                             | Sessions, users, capabilities, contracts                                                                                        | `POST /bootstrap/client`                                                                      | Return exact participant, profile, inbox, transports, bindings, and connect metadata after coherent issuance; return no context                                                                                                | `platform::auth::bootstrap::client`                                                                | replace with clean Rust model                                |
| Service bootstrap                | `auth/bootstrap/service.ts`, `auth/http/bootstrap_routes.ts`                                                                                                            | Deployment/instance, contract, plan, materialization, offer, and binding tables                                                 | `POST /bootstrap/service`                                                                     | Verify provisioned identity, validate exact participant, plan authority, reconcile dependencies/resources, create a distinct session, and return stable states                                                                 | `platform::auth::bootstrap::service`                                                               | replace with clean Rust model                                |
| Device bootstrap                 | `auth/bootstrap/device.ts`, `auth/device_activation/http.ts`                                                                                                            | Device/deployment/activation, contract, authority, and materialization tables                                                   | `POST /auth/devices/connect-info`, `Auth.Devices.ConnectInfo.Get`                             | Replace connect-info with `POST /bootstrap/device`; verify identity, paired active deployment/instance, admin approval, and required delegation; create a distinct session and return no context                               | `platform::auth::bootstrap::device`                                                                | replace with clean Rust model                                |
| Device activation flow           | `auth/device_activation/http.ts`, `auth/device_activation/operation.ts`                                                                                                 | `trellis_browser_flows`; provisioning, activation, review tables                                                                | Activation request/wait; device resolve operation                                             | Proof-protected bounded flow, deployment-selected portal, cancellation-safe wait, and stable terminal states                                                                                                                   | `platform::auth::deployment::device_activation` plus operations runtime                            | migrate                                                      |
| Service deployments              | `auth/admin/service_rpc.ts`, `auth/storage/services.ts`                                                                                                                 | `service_deployments`, legacy deployment authority                                                                              | `Auth.Deployments.*` with `kind=service`                                                      | Manage lifecycle; update eligibility, reconcile, and revoke runtime access after commit                                                                                                                                        | deployment repository and unified admin domain service                                             | replace with clean Rust model                                |
| Service instances                | `auth/admin/service_rpc.ts`, `auth/storage/services.ts`                                                                                                                 | `service_instances`; sessions and connection KV                                                                                 | `Auth.ServiceInstances.*`                                                                     | Provision public identity, reject reassignment, manage lifecycle, and revoke sessions/connections after commit                                                                                                                 | instance, principal, session, and provisioning repositories                                        | replace with clean Rust model                                |
| Device deployments               | `auth/admin/rpc.ts`, `auth/storage/devices_activation.ts`                                                                                                               | `device_deployments`, legacy authority and portal route                                                                         | `Auth.Deployments.*` with `kind=device`                                                       | Manage eligibility and review/delegation policy without treating policy as authority                                                                                                                                           | deployment repository and unified admin domain service                                             | replace with clean Rust model                                |
| Device instances                 | `auth/admin/rpc.ts`, `auth/storage/devices_activation.ts`                                                                                                               | `device_instances`; overlapping activation state                                                                                | `Auth.Devices.Provision/List/Enable/Disable/Remove`                                           | Bind immutable identity to deployment/principal, keep instance/device lifecycle separate, and revoke runtime access after commit                                                                                               | instance, device, principal, and provisioning repositories                                         | replace with clean Rust model                                |
| Device provisioning secrets      | `auth/admin/rpc.ts`, `auth/storage/devices_activation.ts`                                                                                                               | `device_provisioning_secrets`                                                                                                   | Device provision and first identity binding                                                   | Return raw secret once, store only a hash, consume atomically, and never reactivate revoked identity                                                                                                                           | `platform::auth::deployment::device_provisioning`                                                  | replace with clean Rust model                                |
| Device activation reviews        | `auth/admin/rpc.ts`, `auth/device_activation/operation.ts`                                                                                                              | `device_activation_reviews`; operation state                                                                                    | Review RPCs and events                                                                        | Keep administrative approval distinct from user delegation; record idempotent decision and complete operation                                                                                                                  | activation proposal/decision repository and operation handler                                      | replace with clean Rust model                                |
| Device user delegation           | `auth/device_activation/*`, `auth/admin/rpc.ts`                                                                                                                         | `device_activations`; sessions and connection KV                                                                                | Device-authority list/revoke/resolve and events                                               | Deployment-scoped required/active/revoked delegation with expiry; revocation commits before kick                                                                                                                               | existing delegation repository plus activation service                                             | replace with clean Rust model                                |
| Deployment authority proposals   | `auth/contract_proposal_analysis.ts`, `auth/deployment_authority_plan.ts`, `auth/admin/authority_rpc.ts`                                                                | `deployment_authority_plans` and legacy authority children                                                                      | Deployment-authority plan/accept/reject/reconcile RPCs                                        | Derive one stable lineage per deployment + participant; reuse only equivalent pending plans, preserve terminal history, expose initial plans before acceptance, and atomically decide desired authority                        | `platform::auth::authority::proposals` plus authority repository                                   | replace with clean Rust model                                |
| Implementation offers            | `auth/bootstrap/service.ts`, `catalog/runtime.ts`, callout disconnect handling                                                                                          | `implementation_offers`; connection liveness                                                                                    | Indirect bootstrap/catalog behavior                                                           | Preserve provider availability and exact API evidence without allowing offers to mutate desired authority                                                                                                                      | dependency evidence plus operational provider presence                                             | replace with clean Rust model                                |
| Dependency selection             | `catalog/uses.ts`, `catalog/runtime.ts`, proposal analysis                                                                                                              | Implicit contract/offer lookup                                                                                                  | Planning, bootstrap, catalog status                                                           | Resolve exact required/optional API digests and provider evidence at authority scope                                                                                                                                           | participant resolution and dependency-evidence repositories                                        | replace with clean Rust model                                |
| Resource binding                 | `catalog/resources.ts`, authority reconciler, `catalog/rpc.ts`                                                                                                          | Physical JetStream resources and binding JSON                                                                                   | Reconciliation, internal bindings RPC, bootstrap                                              | Provision through an adapter, persist structured evidence, expose exact bindings, and never derive authority from physical names                                                                                               | resource provider, evidence repository, runtime-binding projection                                 | replace with clean Rust model                                |
| NATS Auth Callout                | `auth/callout/callout.ts`                                                                                                                                               | `$SYS.REQ.USER.AUTH`; signing/XKey material                                                                                     | NATS auth request/response JWT                                                                | Verify fresh replay-protected proof against stored key, resolve issuance, compile exact permissions, issue bounded JWT, seal response, and fail safely                                                                         | `platform::auth::callout` under `platform.owner`                                                   | migrate                                                      |
| Transport permission compilation | `auth/callout/permissions.ts`, legacy materialization/catalog helpers                                                                                                   | Persisted delegated subjects and materialized ACLs                                                                              | Embedded in callout JWT                                                                       | Project exact grants, descriptors, bindings, inbox, and narrow built-ins; subjects never create grants                                                                                                                         | pure `platform::auth::callout::permissions`                                                        | replace with clean Rust model                                |
| Connection presence              | `auth/callout/callout.ts`, `auth/session/connections.ts`, `auth/session/rpc.ts`                                                                                         | `trellis_connections`; disconnect advisories                                                                                    | `Auth.Connections.List`; connection events                                                    | Track bounded operational presence, never authority; remove exact entries on disconnect                                                                                                                                        | `platform::auth::sessions::connections` KV repository and watcher                                  | migrate                                                      |
| Session revoke and kick          | `auth/session/revoke*.ts`, `auth/callout/kick.ts`, `auth/session/rpc.ts`                                                                                                | Sessions; connection KV; NATS system kick                                                                                       | Session revoke/logout, connection kick, events                                                | Commit revocation first, then use one exact bounded kick path; kick failure cannot restore authority                                                                                                                           | session/connection services and outbox                                                             | migrate                                                      |
| Transitional request validation  | `auth/session/rpc.ts`, `auth/registration/session.ts`                                                                                                                   | Sessions, users, capability groups, service/device deployment and instance state, device activation, and in-memory replay state | `Auth.Requests.Validate`                                                                      | Validate actual subject/reply/request/time/payload proof against issuance and generated descriptor permissions                                                                                                                 | `platform::auth::rpc::request_validation`                                                          | migrate; remove in local-validation milestone                |
| Transitional event validation    | `auth/session/rpc.ts`, `auth/registration/session.ts`                                                                                                                   | Retained session validity evidence                                                                                              | `Auth.Events.Validate`                                                                        | Validate event identity/time/subject/payload and historical session interval with typed denial                                                                                                                                 | `platform::auth::rpc::event_validation`                                                            | migrate; retain until event-auth cutover                     |
| Auth transition events           | `auth/callout/callout.ts`, `auth/registration/session.ts`, `auth/session/revoke.ts`, `auth/device_activation/operation.ts`, `auth/admin/rpc.ts`, reconciliation storage | Direct session, connection, and device events; authority reconciliation history                                                 | Current Auth session/connection/device events and internal authority history                  | Couple durable changes, including authority transitions, to deterministic outbox rows; publish at least once and emit no secrets or no-op duplicates                                                                           | existing authorization outbox plus auth event publisher                                            | replace with clean Rust model                                |
| Auth API contract source         | `contracts/trellis_auth.ts` and generated auth SDKs                                                                                                                     | Current `trellis.contract.v1` manifest                                                                                          | Current Auth RPC/operation/event vocabulary                                                   | Replace the definition with source-owned `trellis.api.v1` and `trellis.participant.v1` artifacts; regenerate Rust and TypeScript SDKs without legacy aliases or compatibility artifacts                                        | source artifacts and generator pipeline                                                            | keep legacy source only as test reference after regeneration |
| Capability catalog               | `auth/session/users.ts`, `auth/registration/approval_users.ts`                                                                                                          | `deployment_authority_capability_definitions` and built-in definitions                                                          | `Auth.Capabilities.List`                                                                      | Expose a read-only catalog derived from installed API artifacts and built-in platform capability definitions; it is not mutable authority                                                                                      | generated artifact catalog view                                                                    | migrate                                                      |
| Mutable capability groups        | `auth/capability_groups.ts`, user and portal policy helpers                                                                                                             | Capability groups and assignments                                                                                               | `Auth.CapabilityGroups.*`                                                                     | Remove the mutable public group API entirely; any needed approval restriction is checked-in policy and cannot create grants                                                                                                    | no replacement public surface                                                                      | intentionally retire                                         |
| Deployment grant overrides       | `auth/grants/policy.ts`, temporary registration adapters                                                                                                                | `deployment_authority_grant_overrides`                                                                                          | Grant-override RPCs                                                                           | No second grant source; retained policy may only restrict exact proposals                                                                                                                                                      | restrictive proposal policy                                                                        | intentionally retire                                         |
| Catalog issue resolution         | Catalog modules and auth registration                                                                                                                                   | Catalog issue projection                                                                                                        | `Auth.CatalogIssues.Resolve`                                                                  | Catalog issues remain catalog/platform responsibility                                                                                                                                                                          | catalog surface                                                                                    | intentionally retire from auth                               |
| Event-consumer listing           | Catalog/resource registration                                                                                                                                           | Participant resource bindings                                                                                                   | `Auth.EventConsumers.List`                                                                    | Event-consumer bindings remain participant/resource management                                                                                                                                                                 | resource/catalog surface                                                                           | intentionally retire from auth                               |
| Contexts and trust               | No accepted runtime owner                                                                                                                                               | None in Rust auth service                                                                                                       | Context/trust/refresh surfaces                                                                | Signing, trust, context storage, refresh, and runtime cache are outside this cutover                                                                                                                                           | later authorization-context runtime                                                                | defer                                                        |
| Ordinary local validation        | Runtimes call central validators                                                                                                                                        | None                                                                                                                            | Router request verification                                                                   | Keep central validation until context issuance; do not add premature local proof-v2 enforcement                                                                                                                                | later runtime/router integration                                                                   | defer                                                        |

## Runtime Cutover Rules

The Rust service uses one composition root after `platform.owner` acquisition:

```text
platform storage and migrations
  -> authorization/account/provisioning repositories
  -> shared domain services
  -> startup reconciliation and first-admin check
  -> auth flow and connection KV resources
  -> Auth Callout, RPC/operation/event, and HTTP adapters
  -> platform readiness
```

HTTP, RPC, operation, Auth Callout, and background paths call the same domain
services. Handlers do not execute SQL directly. Session creation, authority
acceptance, and connection kick each have one mutation path.

The TypeScript control-plane auth registration is disabled in supported
`platform` and `all` deployments. There is no shared database, dual write,
read-through, proxy, fallback, or concurrent callout responder.

## Implementation Architecture

### Source Artifacts And Generation

The Rust runtime crate owns two authored JSON inputs:

```text
rust/crates/runtime/trellis.api.json
rust/crates/runtime/trellis.participant.json
```

The first is the `trellis.auth@v1` `trellis.api.v1` artifact. The second is the
Rust auth runtime's `trellis.participant.v1` artifact. Repository preparation
discovers, lints, parses, and generates directly from these formats. It does not
convert them to or from `trellis.contract.v1`.

The API artifact owns public RPCs, operations, events, schemas, errors,
capability definitions, and consent metadata. Transitional request and event
validators are `internal: true` RPC definitions. The generator emits their typed
vocabulary only to private runtime adapters. They are absent from public Auth
clients and participant facades. Legacy `trellis.contract.v1` Auth outputs are
deleted rather than retained or aliased.

The participant artifact implements the exact Auth API digest and declares the
runtime's built-in API uses, private consumers, queues, state, and resources.
Generated Rust and TypeScript artifacts derive from the same two authored files.
The legacy TypeScript Auth contract is not an alias or compatibility source.

### Service Composition

`platform::auth` remains the only auth subsystem. It contains the accepted
authorization state and adds cohesive account, workflow, browser, bootstrap,
session, callout, HTTP, and RPC modules as their behavior is implemented. File
layout follows actual cohesion rather than pre-creating one file per planned
operation.

One cloneable `AuthService` composes:

- the existing authorization store and materialization service;
- account, workflow, and provisioning repository ports implemented by the same
  in-memory and SQLite stores;
- exact API and participant artifact lookup;
- short-lived flow, replay, and connection-presence KV ports;
- configured OAuth/OIDC provider clients;
- resource provisioning and binding resolution;
- NATS system connection control and canonical event publication.

HTTP, RPC, operation, callout, and background adapters call this service. They
do not execute SQL, reconstruct authority, or call one another over public
transport merely because they share a process.

### Durable Storage And Transactions

`V1002__auth_service_cutover.sql` adds companion records around the accepted
V1001 authority core. V1001 principals, provider links, participant bindings,
sessions, desired authority, deployments, instances, devices, delegations,
dependency/resource evidence, materialization, and authorization transition
outbox remain canonical.

V1002 adds only the missing product records:

- user profiles and local credentials;
- login portals, login settings, and portal routes;
- hashed single-use account flows;
- immutable authority proposal payloads and decision records;
- service and device provisioned identity metadata;
- hashed one-time device provisioning secrets;
- device activation review workflow records;
- bounded idempotency results for state-changing proof requests;
- one post-commit action outbox for normal events and deterministic connection
  kick intents not represented by the existing authorization transition outbox.

V1002 also normalizes accepted M7 deployment-authority keys to the deterministic
deployment + participant lineage and updates authority-keyed evidence and
materialization rows in the same migration. Proposal rows store normalized
deployment identity. Their semantic digest is unique only while pending;
terminal rows remain immutable history and may coexist with a later proposal of
the same semantics.

Repositories expose aggregate transactions rather than table-shaped CRUD. Local
registration, account-flow completion, browser bind/session creation, authority
acceptance, provisioning-secret consumption, activation decision, and session
revocation plus post-commit actions each have one atomic repository operation.
Every state-changing proof transaction atomically claims a key scoped by proof
purpose, authenticated principal or key, and request ID; compares the exact
request digest; performs the mutation; and stores the replayable result. Session
revocation inserts deterministic event and kick actions in that same
transaction. A critical dispatcher retries kick actions idempotently after
restart; kick failure cannot restore or roll back revocation.

Required-nullable values are required JSON members and SQL `NULL` when absent.

### Bootstrap And Connect Proofs

`trellis-protocol` owns one explicit proof format per purpose:

- user auth request initiation;
- client reconnect bootstrap;
- service bootstrap;
- device bootstrap;
- NATS connect;
- unauthenticated session self-control where required.

Each proof binds its fixed domain and version, canonical public-key identity,
request ID, issued-at Unix milliseconds, exact operation fields, and raw or
canonical request digest where listed below. Service and device bootstrap bind
both provisioned identity and new session keys. NATS connect binds stable
session ID, key ID, participant digest, and the NATS server challenge nonce
supplied to the client's auth callback. The callout verifies exact equality with
`client_info.nonce` in the encrypted NATS authorization request before replay
admission or JWT issuance.

Proof inputs use deterministic length-prefixed framing shared with the accepted
authorization protocol primitive. Ad hoc string concatenation is not accepted.
Wire parsers reject unknown fields and noncanonical keys before signature
verification.

Every transcript begins with the UTF-8 proof format and fixed purpose, then
encodes the following fields in this exact order. Strings use UTF-8, timestamps
use canonical ASCII decimal, Ed25519 keys use their decoded canonical 32 bytes,
NATS keys use their decoded canonical key bytes, and digests use raw 32-byte
SHA-256 values. Every field is length-prefixed.

| Purpose              | Fixed transcript fields after format and purpose                                                                                                                                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| User auth request    | request ID, issued-at, new session public key, new session NATS User NKey, participant ID, participant digest, redirect target, canonical request digest                                                                                          |
| Client bootstrap     | request ID, issued-at, session ID, session key ID, session NATS User NKey, expected participant digest or empty, expected needs digest or empty, canonical request digest                                                                         |
| Service bootstrap    | request ID, issued-at, deployment ID, instance ID, provisioned identity key ID, new session public key, new session NATS User NKey, participant ID, participant digest, canonical request digest                                                  |
| Device bootstrap     | request ID, issued-at, deployment ID, instance ID, device identity key ID, new session public key, new session NATS User NKey, participant ID, participant digest, activation or provisioning challenge digest or empty, canonical request digest |
| NATS connect         | request ID, issued-at, session ID, session key ID, session NATS User NKey, participant digest, NATS challenge nonce                                                                                                                               |
| Session self-control | request ID, issued-at, session ID, session key ID, canonical request digest                                                                                                                                                                       |

For HTTP purposes, the canonical request digest is SHA-256 over the Trellis
canonical JSON encoding of the complete request object with the signature member
removed. Required-nullable members remain present as JSON `null`. Unknown
members are rejected before digest verification. NATS connect has no payload
digest because every accepted token field and the client-visible server nonce is
present directly in its transcript. NATS Server generates the ephemeral
`user_nkey`; it is unavailable to the client signer. The callout response JWT is
still bound to that key because NATS Server rejects any response whose subject
does not equal its generated key. Shared Rust and WASM/TypeScript vectors fix
every byte and reject reordered purposes, changed fields, and noncanonical keys.

Bootstrap returns a deny-all Auth-account bootstrap JWT whose subject is the
session public key encoded canonically as a NATS User NKey. It does not return a
second private key. Rust and TypeScript authenticators use the existing session
private key to return all of:

```text
jwt        session-bound deny-all Auth-account JWT
nkey       canonical NATS User NKey for the session public key
sig        standard Ed25519 signature over the raw NATS challenge nonce
auth_token Trellis NATS connect proof bound to the same nonce
```

The callout validates the bootstrap JWT issuer, expiry, deny-all permissions,
and subject; requires `connect_opts.nkey` to equal the stored session public
key's canonical User NKey; verifies `connect_opts.sig` over `client_info.nonce`;
and then verifies the separate Trellis transcript signature before replay
admission. The static shared credential is retired from client bootstrap. The
session-bound bootstrap JWT is only account-routing material and grants no
useful permission without a successful callout response.

NKey assertions in bootstrap and connect requests are validated by decoding the
canonical User NKey and requiring its raw Ed25519 public bytes to equal the
corresponding canonical base64url session public key. The server does not trust
a caller-provided NKey as a second identity and does not need the session
private seed to reconstruct or validate the mapping.

The default proof policy accepts an issued-at value no more than 30 seconds old
and no more than 30 seconds in the future. Configuration may tighten these
limits; it cannot exceed a documented five-minute hard maximum. Replay records
remain present for at least `maximum_age + maximum_future_skew` after first
admission, and durable idempotency results remain at least that long. A replay
entry cannot expire while the corresponding signed proof could still pass
freshness validation.

State-changing HTTP requests keep bounded durable idempotency results so a
matching request ID and digest can return the committed result after restart.
The claim and result are part of the business transaction, never a separate
best-effort write. The same ID with changed content fails. Short-lived NATS
connect replay uses a CAS-backed Trellis KV record because connection attempts
do not own a durable business mutation.

Browser and OAuth scratch state remains in Trellis KV. A mutation-owning flow
step first CAS-claims the KV record with a claim ID and request digest, then
commits the same claim, mutation, and result in SQLite, then marks the KV record
completed or consumed. A retry observing a claim checks the durable result and
either completes the KV transition or safely resumes an uncommitted claim. It
never repeats a committed mutation.

OAuth state distinguishes `active`, `exchange_started`, `restart_required`, and
`completed`. Callback handling CAS-transitions to `exchange_started` before the
provider exchange. If the normalized provider identity and linked-account result
commit, retry completes from that durable result. If exchange began but no
durable result exists after interruption or unknown external outcome, the flow
becomes `restart_required` and requires a fresh provider authorization. Provider
codes and tokens are never replayed or persisted.

### HTTP Runtime

Auth routes merge into the existing Axum listener. The platform does not start a
second HTTP server. `platform::start` returns a platform startup result
containing its `SubsystemHandle` and an erased-state `Router<()>`. Subsystem
startup retains that platform-specific result instead of erasing it into the
ordinary handle list. The supervisor merges the platform router with health
routes before it starts the sole listener, only after all selected subsystems
are ready.

The platform owner exposes an acceptance/fence guard shared by the router and
store. V1002 stores the current owner ID and immutable NATS lease acquisition
fence in one singleton platform row. A successor replaces that row before it
starts auth work. Every auth write uses a bounded immediate SQLite transaction,
checks the expected persisted generation at start and immediately before commit,
and checks a conservative in-memory lease-valid-until deadline at both points.
Transactions refuse to start without enough remaining lease time for the fixed
maximum transaction duration and roll back if that budget is exhausted.

On ownership loss, the supervisor closes the guard, aborts the HTTP listener and
in-flight auth work without graceful mutation drain, aborts platform tasks, and
terminates the process. Graceful bounded HTTP drain applies only to ordinary
signal shutdown while ownership remains held. The persisted generation prevents
a stale process from committing after a successor has fenced the shared store;
the lease deadline prevents commits after the stale owner's last defensible
ownership window even before a successor opens the database.

The implementation uses established libraries for security-sensitive protocol
work:

- `oauth2` and `openidconnect` for OAuth/OIDC authorization code, PKCE,
  discovery, token, and ID-token behavior;
- RustCrypto `argon2` for Argon2id credentials and profile verification;
- `tower-http` for standard HTTP middleware;
- Axum body limits and an Axum-compatible governor middleware for bounded
  source-keyed rate limiting.

Origin, redirect, cookie, and flow-local checks remain explicit auth-domain
rules because generic CORS middleware cannot establish flow ownership. Public
errors contain no credentials, provider tokens, proofs, or internal storage
details.

### NATS Runtime

After `platform.owner` acquisition, platform startup opens the existing
Trellis-account client plus dedicated auth-account and system-account clients.
The Auth-account client owns only callout subscription and replies. The system
client owns only connection advisories and exact kick requests. The Trellis
client owns platform RPC/event subjects and Trellis KV resources.

Startup loads the configured callout issuer, target, and XKey seeds and reuses
the repository's established `nats-jwt-rs` and XKey-capable `nkeys` stack.
Before subscribing, it validates key types and verifies that the auth issuer key
matches the Auth account represented by auth credentials and that the target key
matches the Trellis account represented by Trellis credentials. Invalid or
mismatched material fails startup.

The callout performs:

```text
connect request
  -> proof and replay validation
  -> resolve_issuable_state
  -> exact artifact and binding lookup
  -> deterministic transport permission compilation
  -> expiry-bounded NATS user JWT
  -> sealed auth response
  -> operational connection presence
```

Permission compilation consumes only exact grant atoms, exact API descriptors,
structured resource bindings, the authoritative inbox prefix, and a narrow
participant-kind-dependent built-in list. Subjects, ACLs, current responders,
and physical resource names do not create permission atoms.

Auth Callout, disconnect watcher, outbox publisher, and reconciliation run as
critical platform tasks. Unexpected task completion or ownership loss stops all
auth acceptance and terminates the owning runtime through the existing
supervisor path.

### Portal Assets

The current Svelte portal remains the authored UI. Repository preparation builds
its static output reproducibly into generated artifacts. The Rust release binary
embeds that output, including content-hashed assets. SPA fallback applies only
beneath `/_trellis/portal/*`; hashed asset misses and unknown auth/bootstrap/API
paths remain ordinary `404` responses. Production has no source-tree or mounted
asset dependency. A configured development override directory may replace the
embedded files without changing production behavior, and canonical-path checks
prevent traversal or symlink escape outside that directory.

## Authority Replacement Rules

The following legacy data can inform behavior and tests but is never imported as
current machine authority:

- serialized session capability or subject snapshots;
- identity-grant subjects;
- mutable capability-group expansion;
- deployment grant overrides that expand authority;
- persisted NATS publish/subscribe ACLs;
- implementation-offer liveness without exact authority-scoped evidence;
- resource bucket, stream, consumer, or object-store names.

Current authority remains the accepted identity or deployment authority
projection defined in `rust-authorization-state.md`. Transport permissions are a
short-lived edge projection from issuable state, exact API descriptors, and
structured resource bindings.

## Milestone 9 Trust And Context Integration Inventory

This appendix records the Milestone 9 extension boundary and its implemented
owner. Rust remains the only runtime Auth owner. Milestone 10 local request
validation and Milestone 11 local event validation remain deferred.

### Issuance, Bootstrap, Persistence, And Consumption

TypeScript remains a supported client-library implementation, not a second
runtime authorization engine. It verifies and persists only its own Rust-issued
context through Rust/WASM, refreshes that context and route-only JWT,
reconnects, and projects browser/Svelte state. It does not watch
`manifest.current` or revocations for local request authorization, resolve
arbitrary caller contexts, maintain validator readiness, or receive internal
registry-watch permissions. Rust owns those runtime validation and
registry-observation boundaries.

| Item                                  | Current owner and shape                                                                                                                                                                                                                                                                                                                                     | Milestone 9 change                                                                                                                                                                                                                                               | Root pin and manifest-floor persistence                                                                                                                                                       | Refresh owner                                                                                                       | Deferred use                                                                                                                     |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `IssuableAuthorizationState`          | `rust/crates/runtime/src/platform/auth/domain.rs` is assembled by `AuthorizationStateService::resolve_issuable_state` in `service.rs`. It contains the principal, session/key/inbox, exact participant, authority reference, deployment/instance IDs, effective grants/capabilities, physical binding evidence, expiry bounds, and materialization version. | Add a deterministic issuance snapshot token over all authorization-relevant source records. Map this state, excluding physical resource names, into the accepted signed context protocol. Recheck the token in the context-commit transaction.                   | Server trust floor is the V1003 SQLite singleton. No client persistence applies at this boundary.                                                                                             | `AuthorizationContextIssuer` resolves, retries bounded snapshot conflicts, reuses or signs, commits, and publishes. | Ordinary requests continue through `Auth.Requests.Validate` until M10; events continue through `Auth.Events.Validate` until M11. |
| NATS bootstrap JWT                    | `NatsBootstrapIssuer` in `platform/auth/http.rs` signs a deny-all Auth-account user JWT. Successful bind/bootstrap and refresh responses return `{jwt, jwtExpiresAt, servers?}` route material.                                                                                                                                                             | The JWT remains route-selection material only and grants no authority. Its lifetime is bounded by session/authority/delegation/context state and `maximum_bootstrap_jwt_lifetime_seconds`. Replays and refresh regenerate it only after current issuance checks. | Not persisted as trust authority. Clients persist it atomically with the current context; reconnect presents the independently current context digest.                                        | Context refresh atomically replaces the context bundle and renews the bootstrap JWT.                                | JWT transport permissions remain compiled from current Rust issuance, not from local context validation.                         |
| Browser bind response                 | `BindResponse` in `platform/auth/http.rs` is `{serverNow, session, nats, redirectTarget}` after session creation and `resolve_issuable_state`.                                                                                                                                                                                                              | Add one minimal `AuthorizationContextBundle`; do not report bind ready until its immutable registry entry round-trips. Bind replay reuses or reissues from current state.                                                                                        | Browser IndexedDB stores the installation root pin, manifest floor, and current bundle beside the existing remembered session state.                                                          | TypeScript browser context cache schedules refresh; Svelte projects its state.                                      | Portal consent remains server-owned; ordinary caller proof emission remains M8 until M10.                                        |
| Client bootstrap response             | `ClientBootstrapResponse` in `platform/auth/http.rs` returns session/participant/profile/effective grants/resource bindings/runtime bindings plus NATS material. Replay stores only the session ID and re-resolves issuance.                                                                                                                                | Add the context bundle after current-state revalidation and registry publication. Profile and resource physical names stay outside the signed context.                                                                                                           | Browser uses IndexedDB; Rust CLI uses its private admin-session file; explicit ephemeral clients may use memory only.                                                                         | Rust or TypeScript client cache according to the calling library.                                                   | Existing RPC/event proof paths remain unchanged.                                                                                 |
| Service bootstrap response            | `BootstrapResponse` in `platform/auth/http.rs` returns `ready` with session, authorization evidence, resource runtime bindings, and NATS material; non-ready states return those fields as null. Rust parses it in `trellis/src/client/client.rs`; TypeScript parses it in `server/service.ts`.                                                             | Add the bundle only to `ready`. Verify trust/context and registry publication before service readiness. Non-ready states never receive a context.                                                                                                                | Rust service options gain a persistent session/trust store; TypeScript gains an explicit storage adapter. Current process-local random session state is not sufficient for M9 restart floors. | Connected Rust and TypeScript service runtimes own proactive refresh and atomic context swap.                       | Service routers keep central request/event validation until M10/M11.                                                             |
| Device bootstrap response             | The same Rust `BootstrapResponse` is consumed by `trellis/src/client/client.rs` and `js/packages/trellis/device.ts`; current TS and Rust clients create a fresh session key and retain context only in process.                                                                                                                                             | Add the bundle only after administrative activation, delegation eligibility, coherent issuance, and registry publication.                                                                                                                                        | Device runtime storage persists its own session key reference, bundle, root pin, and manifest floor through an explicit adapter; no other session context is stored.                          | Connected device runtime owns proactive refresh and atomic swap.                                                    | Device ordinary traffic stays on transitional validators through M10/M11.                                                        |
| Session bootstrap response umbrella   | `BootstrapResponse`, `ClientBootstrapResponse`, and `BindResponse` are three private Rust HTTP structs with duplicated NATS/session projection.                                                                                                                                                                                                             | Introduce one source-owned reusable bundle projection shared by all four successful bind/bootstrap paths without merging their distinct product responses.                                                                                                       | Persistence is selected by each client/runtime below, never by the HTTP model.                                                                                                                | Shared refresh client and cache semantics; transport-specific scheduling stays in each runtime.                     | No bulk manifest/certificate history is added to any response.                                                                   |
| NATS connect proof and token          | `SessionProofInputV1::nats_connect` is used by Rust `trellis/src/client/client.rs` and TS `auth/session_auth.ts`; both emit `trellis.nats-connect-token.v1` containing request/time/session/participant/proof but no context digest.                                                                                                                        | Add `natsConnectContext`, require the current context digest in the token and transcript, and reject the old proof in supported M9 runtime connections.                                                                                                          | The reconnect callback reads the atomically current persisted/in-memory bundle.                                                                                                               | Refresh swaps the digest before a later reconnect callback signs.                                                   | The old pure constructor/vector remains only as accepted protocol history.                                                       |
| NATS Auth Callout                     | `platform/auth_callout.rs` verifies the deny-all JWT, NATS nonce signature, M8 connect proof, replay, current issuance, and participant binding before compiling permissions and recording connection presence.                                                                                                                                             | Load the durable published active context by digest, verify all context/session/current-issuance bindings, bound JWT expiry by context and trust, and record context ID/digest in presence.                                                                      | Server uses V1003 trust floor and durable contexts; it never trusts a KV root as a replacement.                                                                                               | No refresh occurs inside callout; clients refresh over HTTP before reconnect.                                       | Current Rust issuance still determines transport permissions.                                                                    |
| `AdminSessionState`                   | `trellis/src/auth/models.rs`; stored privately at the CLI config path `admin-session.json` by `auth/session_store.rs`. It contains URL, servers, session seed/key/ID, participant digest, inbox, bootstrap JWT, and optional expiry.                                                                                                                        | The current context and complete trust floor use the adjacent private atomic `authorization-context.json`; logout clears context while retaining trust. Admin reconnect verifies both before NATS.                                                               | Private sidecar file with restrictive permissions; explicit reset is required to replace the root floor.                                                                                      | Rust authorization-context refresh client owned by CLI/session library.                                             | Generated ordinary RPC requests remain transitional M8 requests.                                                                 |
| Rust user connect options             | `UserConnectOptions` in `trellis/src/client/client.rs` accepts servers, bootstrap JWT, session/inbox, seed, participant digest, and timeout.                                                                                                                                                                                                                | Require a verified current context bundle/cache and sign `natsConnectContext`.                                                                                                                                                                                   | Caller-supplied persistent state for ordinary apps; `AdminSessionState` for CLI.                                                                                                              | Rust cache/refresh client.                                                                                          | No local Router authorization in M9.                                                                                             |
| Rust service/device connect options   | `ServiceConnectWithContractOptions` and `DeviceConnectOptions` bootstrap with identity/session seeds, then retain session/NATS state in the connected `TrellisClient`; no durable manifest floor exists.                                                                                                                                                    | Carry persistent trust/context state, verify lazy manifest/certificate fetches, and refresh before expiry.                                                                                                                                                       | New explicit Rust session store; memory only when explicitly ephemeral/test.                                                                                                                  | Connected runtime.                                                                                                  | Request/event signing remains unchanged apart from NATS reconnect.                                                               |
| TypeScript user connect options       | `client_connect.ts` resolves a browser or supplied session-key identity, parses bind/client bootstrap, and keeps `{participantDigest, sessionId, jwt}` only for the connection.                                                                                                                                                                             | Parse/persist the bundle, verify through WASM-backed trust cache, and supply current context digest to the authenticator.                                                                                                                                        | Browser IndexedDB or explicit adapter for Node/Deno session-key clients.                                                                                                                      | `AuthorizationContextCache` scheduler.                                                                              | `createConnectedClient` keeps existing ordinary proof behavior.                                                                  |
| TypeScript service connect options    | `server/service.ts` creates fresh process-local session auth, parses ready bootstrap into connect/binding data, and connects with the M8 authenticator.                                                                                                                                                                                                     | Add explicit storage, own-context trust verification through Rust/WASM, scheduled refresh with route-JWT renewal, and context-bound reconnect. No runtime validator watcher or readiness subsystem is added.                                                     | Caller-provided service storage adapter; memory only for explicit ephemeral/tests.                                                                                                            | Connected `TrellisServiceSession`.                                                                                  | Service request/event validators remain central.                                                                                 |
| TypeScript device connect options     | `device.ts` derives the device identity, creates fresh process-local session auth, parses ready bootstrap, and connects with M8 auth.                                                                                                                                                                                                                       | Persist/verify root floor and bundle and reconnect with its context digest.                                                                                                                                                                                      | Caller-provided device storage adapter associated with the existing device identity.                                                                                                          | Connected device runtime.                                                                                           | No event-context emission yet.                                                                                                   |
| Browser session storage               | `auth/browser/storage.ts` stores the extractable session seed/key metadata and session ID in IndexedDB; `auth/browser/session.ts` owns remembered versus temporary mode.                                                                                                                                                                                    | Add one deployment-origin-scoped atomic record containing root key id/digest, minimum manifest generation/digest, and current bundle. Concurrent tabs serialize updates; clearing auth/context retains trust.                                                    | Existing IndexedDB database for remembered sessions; explicit ephemeral mode uses memory only.                                                                                                | Browser context cache.                                                                                              | OAuth tokens and other sessions' tokens remain excluded.                                                                         |
| Service runtime persistent auth state | No accepted M8 persistent store exists; Rust and TypeScript service sessions are process-local and bootstrap a fresh session.                                                                                                                                                                                                                               | Add an explicit storage port containing only own-session key reference/seed, session ID, root pin/floor, and current bundle.                                                                                                                                     | Deployment-selected private file/store; no hidden global fallback.                                                                                                                            | Service runtime.                                                                                                    | Persisted local request replay state is M10 work.                                                                                |
| Svelte provider/session state         | `trellis-svelte` currently projects portal/auth flow and device activation lifecycle; it has no trust/context cache or status.                                                                                                                                                                                                                              | Add a rune-backed context projection with loading/ready/refreshing/degraded/expired/authRequired, expiry/refresh times, trust generation, nullable last error, and `refreshNow()`. No UI redesign.                                                               | Delegates persistence to the TypeScript cache/storage adapter.                                                                                                                                | Svelte controller delegates network/timers to the TS cache.                                                         | It does not locally authorize requests/events.                                                                                   |

### Authorization-Relevant Mutation Inventory

Every row below selects and revokes affected active contexts in the same
authoritative SQLite transaction. SQLite is the sole complete persistence
implementation for trust, contexts, and revocations; in-memory auth fixtures do
not duplicate those semantics. Deterministic `context_revoke` post-commit
actions publish after commit; failure cannot roll back the domain mutation.

| Mutation                                       | Current authoritative path                                                                                                                            | Milestone 9 invalidation coupling                                                                                                                                                                       |
| ---------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Session revoke/logout                          | `AuthService::revoke_session` -> `AuthSessionRepository::revoke_session`; HTTP logout and Auth RPC call that shared path.                             | Revoke every active context for the session and enqueue deterministic revocations before kick/event actions.                                                                                            |
| Session expire                                 | `SessionRepository::expire_session` exists in memory/SQLite; expiry is also checked during `resolve_issuable_state`.                                  | Seal production expiry behind an aggregate transaction that expires/revokes session contexts. The janitor may classify already elapsed contexts but is not the primary reverse scan.                    |
| Session rebind                                 | `SessionRepository::rebind_session` updates key/participant/session state; browser identity changes may create replacement sessions.                  | Revoke contexts tied to the previous key or participant in the same rebind/replacement transaction.                                                                                                     |
| Password change                                | `AuthService::change_password` -> aggregate `PasswordChange`; it updates the credential and revokes sibling sessions.                                 | Revoke contexts for every sibling session revoked by that transaction.                                                                                                                                  |
| Password reset                                 | `AuthService::complete_password_reset` -> aggregate `PasswordResetCompletion`.                                                                        | Revoke contexts for every session disabled by reset in the same completion transaction.                                                                                                                 |
| Principal disable/revoke                       | `PrincipalRepository::update_principal_authorization_state`; user/deployment administration reaches it through Rust Auth RPC aggregate paths.         | Revoke all active contexts for the principal whenever authorization state/version changes. Profile-only edits do nothing.                                                                               |
| Identity authority decision/version change     | `AuthService::decide_authority_proposal` and `AuthRpcProcessor::identity_authority_revoke` update desired authority and enqueue reconciliation.       | Revoke contexts for the authority on any state/version change, including expansion, in the authority transaction.                                                                                       |
| Deployment authority decision/version change   | `AuthService::decide_authority_proposal` updates desired deployment authority and supersedes plans atomically.                                        | Revoke every active context for the stable deployment-authority lineage in that decision transaction.                                                                                                   |
| Materialization semantic change/unavailability | `AuthorizationStateService::reconcile_authority` -> repository reconciliation transaction using `materializer.rs`.                                    | When materialization version, effective grants/capabilities, availability, or expiry changes, revoke contexts for that authority in the same reconciliation transaction. Semantic no-ops do not revoke. |
| Deployment disable/remove/expiry               | `DeploymentProfileRepository::put_deployment_profile` via Auth RPC lifecycle handlers; live expiry is checked during issuance.                        | Revoke deployment contexts atomically on disable/remove/expiry transition. No-op profile writes do nothing.                                                                                             |
| Instance disable/revoke/stale/remove           | `ProvisioningRepository::mutate_provisioned_instance`, called by Auth RPC lifecycle handlers.                                                         | Revoke contexts selected by exact instance ID in that aggregate mutation.                                                                                                                               |
| Device disable/revoke/remove                   | Device lifecycle uses the same provisioned-instance aggregate and device/principal records.                                                           | Revoke exact device principal/deployment/instance contexts in the lifecycle transaction.                                                                                                                |
| Device delegation revoke/expiry decision       | `ProvisioningRepository::mutate_device_delegation`, called by Auth RPC and the activation operation.                                                  | Revoke contexts for the device/deployment delegation when state or expiry changes.                                                                                                                      |
| Participant binding replacement                | Authority proposal acceptance and session rebind select exact participant artifact/needs digests; bindings are immutable records addressed by digest. | Revoke affected session/authority contexts when the selected binding changes. Merely inserting an unrelated immutable binding does nothing.                                                             |
| Issuer revocation                              | Runtime startup accepts only a verified current root-signed manifest and durable generation floor.                                                    | Manifest-floor advancement marks all unexpired contexts from newly revoked issuers revoked and enqueues registry actions while retaining certificates for history.                                      |

Context expiry remains the cryptographic upper bound. Registry revocation is
prompt distribution, not a source of broader authority. Profile display-name,
email/image changes, session last-seen coalescing, unrelated principals or
authorities, outbox retries, and semantic no-op reconciliation do not change an
issuance snapshot token and do not revoke contexts.
