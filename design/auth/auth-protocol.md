---
title: Auth Protocol
description: Language-neutral auth protocol rules for proofs, connect tokens, auth callout, reply validation, and auth state.
order: 20
---

# Design: Auth Protocol

## Prerequisites

- [trellis-auth.md](./trellis-auth.md) - auth architecture and trust model
- [../contracts/trellis-contracts-catalog.md](./../contracts/trellis-contracts-catalog.md) -
  contract-driven permission derivation
- [../operations/trellis-operations.md](./../operations/trellis-operations.md) -
  operation watch and streaming reply semantics

## Scope

This document defines the language-neutral Trellis auth protocol.

It covers:

- cryptographic encodings and signatures
- pinned authorization roots, issuer certificates, and issuer manifests
- signed authorization contexts and context-bound request proofs
- NATS connect token shapes
- auth callout behavior
- RPC proof verification
- pre-auth device wait verification
- reply-subject validation and streaming reply rules
- internal auth state records required for protocol behavior

It does not define public HTTP and RPC endpoint schemas; those live in
`auth-api.md` and, for the activated-device lifecycle, `device-activation.md`.

## Cryptographic Primitives

| Notation    | Definition                                     |
| ----------- | ---------------------------------------------- |
| `hash(x)`   | SHA-256 digest of x                            |
| `sign(k,x)` | Ed25519 signature of x using key k             |
| Encoding    | base64url without padding (RFC 4648 section 5) |

Canonical byte encoding for signatures:

| Value type       | Encoding                                           |
| ---------------- | -------------------------------------------------- |
| Strings          | UTF-8 bytes                                        |
| Numbers          | JSON safe integers in the canonical request object |
| Proof transcript | Domain-separated, length-prefixed binary fields    |

Every integer carried by a signed JSON object or cross-language proof must be
within the exactly interoperable JSON safe-integer range:

```text
-9_007_199_254_740_991 through +9_007_199_254_740_991
```

This applies to validity times, manifest generations, revocation times,
authority versions, request `iat`, verification-policy time/generation inputs,
and integers nested anywhere in signed extensions. Positive counters retain
their positive/nonzero requirements. Verifiers reject unsafe values before
canonicalization or signature acceptance and report the exact RFC 6901 path.

All Trellis clients, including Rust CLIs and future non-TypeScript clients, must
match this encoding exactly.

Identity-derived account ids are no longer used for canonical users. User
accounts have generated `userId` values; provider identity ids may be derived
from provider and subject, but they are not account ids.

The signed value is always the exact UTF-8 bytes as transmitted. No URL
normalization is applied before signing.

## Connect Token Shapes

After bootstrap or browser binding, every client receives a deny-all
Auth-account JWT whose subject is the session public key encoded as a NATS User
NKey. The client uses the same session private key for the standard NATS nonce
signature and a separately domain-separated Trellis connect proof.

The `auth_token` is a `trellis.nats-connect-token.v1` object:

```ts
{
  format: "trellis.nats-connect-token.v1",
  requestId: string,
  issuedAt: number,
  sessionId: string,
  participantDigest: string,
  proof: SessionProofV1
}
```

Rules:

- `SessionProofV1` uses the `NatsConnect` purpose and binds the exact server
  nonce, session id, participant digest, request id, issue time, and session
  NKey
- the proof transcript is built from protocol-owned length-prefixed frames; no
  concatenated-signature format is accepted
- the standard NATS nonce signature is padded standard Base64; Trellis proof
  fields use unpadded Base64URL
- the callout verifies that the bootstrap JWT subject, stored session key, proof
  signer, and server-supplied `user_nkey` are the same identity
- each connect or reconnect uses a fresh request id and nonce-bound proof;
  consumed proof replays fail closed through CAS-backed replay state
- clients with unstable clocks derive `issuedAt` from bootstrap `serverNow`

## Auth Callout Behavior

When NATS calls `$SYS.REQ.USER.AUTH`:

1. Decode the encrypted request by requiring `Nats-Server-Xkey`, decrypting the
   payload, and extracting `user_nkey` plus `connect_opts.auth_token`.
2. Validate the `trellis.nats-connect-token.v1` envelope, deny-all bootstrap
   JWT, standard nonce signature, and nonce-bound `trellis.session-proof.v1`.
3. CAS-admit the proof replay key and resolve current issuable session,
   principal, participant, authority, deployment/instance, and device/delegation
   state.
4. Compile permissions from exact `GrantSetV1` atoms plus matching API
   descriptors and physical resource evidence. A subject or binding never
   creates an atom.
5. Sign the target-account JWT for the server-generated `user_nkey`, bounded by
   current session, authority, and delegation expiry, then record connection
   presence.

All principal kinds use this same pipeline. Expected denials return fixed reason
codes. Unexpected storage, crypto, provider, and topology causes are logged
internally and return only `internal_error`.

## Server-Relative Time

Bootstrap and connect-info responses that expect `iat`-based runtime auth SHOULD
return `serverNow`.

Clients SHOULD:

1. record request start and end time locally
2. estimate midpoint clock offset from `serverNow`
3. compute future `iat` values from corrected server-relative time
4. retry once after `iat_out_of_range` when a fresh `serverNow` is returned

Clients MUST NOT loop forever on repeated `iat_out_of_range`.

Auth callout payload field names use canonical snake_case names such as:

- `user_nkey`
- `server_id`
- `client_info`
- `connect_opts`

CamelCase aliases are not part of the Trellis protocol.

The auth-callout request and response MUST be XKey-encrypted. Plaintext
auth-callout payloads are not supported.

## Permission Derivation

The auth callout derives permissions from:

- current issuable authorization state
- the exact `GrantSetV1` accepted for the bound participant
- exact `trellis.api.v1` descriptors
- typed materialized resource bindings
- the session reply inbox and narrow built-in subjects

Rules:

- inbox subscribe permission always includes `${inboxPrefix}.>`
- services receive resource publish/subscribe subjects only when the exact
  participant-resource atom and its matching materialized binding are both
  present; optional evidence contributes only the corresponding optional atoms
- operation-control publish permissions are derived only from operation
  `observe`/`cancel` capabilities; `call` authorizes starting an operation but
  does not authorize publishing to its control subject
- auth-callout denial paths return explicit deny responses and MUST NOT mint a
  partially scoped user JWT when the known manifest, session, deployment
  authority, materialized authority, or resource state needed for permission
  derivation is unavailable
- unexpected auth-callout exceptions are logged with internal details but return
  a stable generic external error such as `internal_error`
- operation streaming replies use `jwt.resp.max = OPERATION_RESPONSE_MAX`
- `OPERATION_RESPONSE_MAX` MUST be greater than `1` and SHOULD default to
  `65535`

## Authorization Context And Local Request Authorization

Milestone 9 implements the authority proof below for bootstrap, refresh, and
every NATS connect/reconnect. Ordinary authenticated runtime requests continue
to use transitional `Auth.Requests.Validate` until Milestone 10, when they will
use the second proof locally:

1. **Authority proof:** a pinned authorization root verifies a current
   generation-numbered issuer manifest and a directly root-signed issuer
   certificate; the active issuer verifies a short-lived authorization context.
2. **Possession proof:** the session private key bound into that context signs
   the exact context digest, subject, reply subject, raw payload hash, issue
   time, and request id.

The trust hierarchy is:

```text
pinned root
  -> root-signed current issuer manifest
  -> root-signed issuer certificate selected by exact signed digest
  -> issuer-signed short-lived authorization context
  -> session-key-signed request proof v2
```

All signed JSON security objects use strict recognized top-level fields. Signed
forward-compatible data belongs in `extensions`; names in the canonical
`critical` set fail closed unless understood. Each signature uses a distinct
domain and covers SHA-256 of length-prefixed domain bytes plus RFC 8785
canonical unsigned JSON. Key ids are derived as unpadded base64url SHA-256
digests of the raw 32-byte Ed25519 public key.

The root-signed manifest is the authoritative current issuer registry. Consumers
durably supply the root key id, canonical root digest, minimum accepted
generation, and canonical manifest digest at that generation. This rejects an
older valid manifest, same-generation equivocation, and root replacement across
process restarts. Clearing a session or context retains the floor; changing the
root requires an explicit trust reset. Multiple active issuers permit overlap
during key rotation. A revoked or omitted issuer is untrusted, and each entry
binds the exact complete signed certificate digest. A writable distribution
store therefore cannot create issuer authority without a root signature.

Server startup creates or exact-confirms the root, active certificate, and
generation-addressed manifest before advancing SQLite or `manifest.current`.
SQLite acceptance and removed-issuer context revocation commit together;
`manifest.current` is the final CAS-protected step. Startup and `trellis check`
reconcile the configured file generation and digest against both the durable
SQLite floor and the highest verified immutable registry history, so an expired
historical manifest still prevents rollback.

A previously verified manifest remains subject to the policy supplied for each
context decision. Raising the durable minimum generation invalidates stale
verified or cloned handles immediately; context verification rechecks the
manifest generation and returns `ManifestRollback` at `/generation`.

Context issuance computes and distributes `refreshAt = expiresAt - refreshLead

- jitter(contextDigest)`, where jitter is deterministic, bounded, and can only
  move refresh earlier. The protocol implementation owns this calculation.
  Client runtimes consume the distributed value directly, so restart and
  reconnect do not produce refresh storms or repeatedly call a server that is
  not yet willing to replace the context.

The refresh request always contains `currentContextDigest`, but the value is
nullable. A client with a valid context sends its digest; a client whose context
or route JWT has expired sends `null` while proving possession of the retained
session key and pinned trust floor. Success returns `serverNow`, a context, and
a renewed deny-all route JWT plus its expiry as one atomic installation. Clients
derive a midpoint clock offset from `serverNow`, schedule against corrected
server-relative time, and reschedule even when refresh validly returns the same
context digest. Only terminal session/authority failures clear session recovery
state.

The issuer decides reuse inside the same SQLite transaction that commits the
context. Equivalent concurrent requests reuse one record, and each session has
at most two active overlapping contexts: the current lease and one replacement
for reconnect handoff. Publication actions are keyed by immutable context digest
and deduplicated transactionally.

The signed context binds the stable principal, exact participant artifact and
accepted-needs digests, durable identity/deployment authority record and
version, session id and public key, reply-inbox prefix, exact
`trellis.grant-set.v1`, and canonical platform capability keys. Its validity is
short-lived and entirely contained by both the issuer certificate and current
manifest. Capabilities may authorize built-in/platform surfaces during
migration, but never expand exact permission atoms.

`maximum_context_bytes` and `maximumContextBytes` always mean the UTF-8 byte
length of canonical complete signed-context JSON. Issuance, protocol parsing,
WASM verification, and the context-registry value limit enforce that same unit;
the base64url transport token length is not the configured unit.

Verified caller metadata exposes the source authority record and version,
deployment and instance ids, participant artifact and needs digests, context and
session identity, validity bounds, grants, capabilities, and immutable signed
context needed by auditing and later runtime integration.

Request proof v2 input is:

```text
LP("trellis.authorization-request-proof.v2")
LP(raw 32-byte signed-context digest)
LP(exact subject UTF-8)
LP(exact reply subject UTF-8, or empty)
LP(SHA-256(raw payload bytes actually received))
LP(ASCII decimal iat)
LP(request-id UTF-8)
```

The receiver verifies the manifest, certificate, context, and session-key proof
locally; computes the raw payload hash locally; validates a nonempty reply
subject against the signed inbox prefix; and enforces required exact permission
atoms and platform capabilities as subsets of the signed context. It then
inserts `(contextId, requestId)` into a local replay cache until context expiry.
Replay cache storage and runtime integration follow the pure protocol milestone.

NATS authentication verifies `natsConnectContext`, the complete trust chain, the
immutable registry record, and fresh issuable state before compiling a transport
JWT. Short context expiry bounds already-issued authority; immediate
session/authority revocation atomically publishes a separate revocation record,
refuses refresh and reconnect, and kicks current transport connections.
Validators subscribe before loading complete manifest-pointer and revocation
snapshots, become healthy only after both snapshots succeed, and gate Auth
Callout startup on that readiness. They resnapshot after watch or manifest
changes, invalidate stale verified contexts, and resolve exact manifest,
certificate, context, and revocation records lazily by immutable key. Connection
presence records the context id and digest used for admission.

## Transitional RPC Message Signing

The following request proof and `Auth.Requests.Validate` flow describes current
migration-baseline implementation behavior, not the target 0.11 protocol. It
remains temporarily so existing runtimes can migrate; no new ordinary request
path should depend on it.

Each authenticated RPC includes proof of session-key ownership. Contract digest
binding is established earlier during connect, bootstrap, or session creation;
per-request RPC proofs do not carry or sign `contractDigest`.

Proof input:

```ts
function buildProofInput(
  sessionKey: string,
  subject: string,
  payloadHash: Uint8Array,
  iat: number,
  requestId: string,
): Uint8Array {
  const enc = new TextEncoder();
  const sessionKeyBytes = enc.encode(sessionKey);
  const subjectBytes = enc.encode(subject);
  const iatBytes = enc.encode(String(iat));
  const requestIdBytes = enc.encode(requestId);

  const buf = new Uint8Array(
    4 + sessionKeyBytes.length + 4 + subjectBytes.length + 4 +
      payloadHash.length + 4 + iatBytes.length + 4 + requestIdBytes.length,
  );
  const view = new DataView(buf.buffer);

  let offset = 0;
  view.setUint32(offset, sessionKeyBytes.length);
  offset += 4;
  buf.set(sessionKeyBytes, offset);
  offset += sessionKeyBytes.length;
  view.setUint32(offset, subjectBytes.length);
  offset += 4;
  buf.set(subjectBytes, offset);
  offset += subjectBytes.length;
  view.setUint32(offset, payloadHash.length);
  offset += 4;
  buf.set(payloadHash, offset);
  offset += payloadHash.length;
  view.setUint32(offset, iatBytes.length);
  offset += 4;
  buf.set(iatBytes, offset);
  offset += iatBytes.length;
  view.setUint32(offset, requestIdBytes.length);
  offset += 4;
  buf.set(requestIdBytes, offset);

  return buf;
}

payloadHash = SHA256(payload);
proof = ed25519_sign(
  sessionKeyPrivate,
  SHA256(buildProofInput(sessionKey, subject, payloadHash, iat, requestId)),
);
```

Rules:

- receivers MUST compute `payloadHash` from the raw request body they actually
  received
- receivers MUST NOT trust a caller-supplied payload hash header
- clients MUST send `iat` and `request-id` headers with every signed RPC request
- verifiers MUST include the corrected `iat` value and `requestId` in the proof
  input and reject proofs whose `iat` is outside the configured freshness window
- auth MUST reject replay of the same `requestId` for the same session while the
  replay cache entry is live
- receivers MUST verify the request against the stored authenticated
  session/principal state created at connect, bootstrap, or session binding time
- length-prefixing is mandatory and prevents boundary attacks

Required message headers:

```text
session-key: <sessionKey>
proof: <base64url(ed25519 signature)>
iat: <unix seconds, corrected to server-relative time when available>
request-id: <unique request id for this session>
```

Verification steps:

1. Extract `session-key`, `proof`, `iat`, and `request-id`
2. Compute `payloadHash = SHA256(raw_request_body)`
3. Reconstruct proof input and verify signature using `session-key` as the
   public key
4. Call `rpc.Auth.Requests.Validate` with `sessionKey`, `proof`, `subject`, raw
   `payloadHash`, `iat`, `requestId`, and required capabilities for session
   lookup, replay detection, stored contract/principal context, and capability
   checking

Target runtimes replace step 4 with the local signed-context decision described
above. Historical proof v1 text remains here only to make that transition
explicit.

## Pre-Auth Device Wait Verification

Before an activated device is activated it cannot use normal authenticated RPCs,
but an online device may still wait for activation completion by calling
`POST /auth/devices/activate/wait`.

That endpoint uses an identity-key proof rather than a session-key proof.

Proof input:

```ts
function buildDeviceWaitProofInput(
  flowId: string,
  publicIdentityKey: string,
  nonce: string,
  iat: number,
  contractDigest: string,
): Uint8Array {
  const enc = new TextEncoder();
  const flowIdBytes = enc.encode(flowId);
  const publicIdentityKeyBytes = enc.encode(publicIdentityKey);
  const nonceBytes = enc.encode(nonce);
  const iatBytes = enc.encode(String(iat));
  const contractDigestBytes = enc.encode(contractDigest);

  const buf = new Uint8Array(
    4 + flowIdBytes.length +
      4 + publicIdentityKeyBytes.length +
      4 + nonceBytes.length +
      4 + iatBytes.length +
      4 + contractDigestBytes.length,
  );
  const view = new DataView(buf.buffer);

  let offset = 0;
  view.setUint32(offset, flowIdBytes.length);
  offset += 4;
  buf.set(flowIdBytes, offset);
  offset += flowIdBytes.length;

  view.setUint32(offset, publicIdentityKeyBytes.length);
  offset += 4;
  buf.set(publicIdentityKeyBytes, offset);
  offset += publicIdentityKeyBytes.length;

  view.setUint32(offset, nonceBytes.length);
  offset += 4;
  buf.set(nonceBytes, offset);
  offset += nonceBytes.length;

  view.setUint32(offset, iatBytes.length);
  offset += 4;
  buf.set(iatBytes, offset);
  offset += iatBytes.length;

  view.setUint32(offset, contractDigestBytes.length);
  offset += 4;
  buf.set(contractDigestBytes, offset);

  return buf;
}

sig = ed25519_sign(
  identityPrivateKey,
  SHA256(
    buildDeviceWaitProofInput(
      flowId,
      publicIdentityKey,
      nonce,
      iat,
      contractDigest,
    ),
  ),
);
```

Rules:

- the endpoint MUST reject if `abs(now - iat) > 30s`
- the endpoint MUST verify `sig` using the supplied `publicIdentityKey`
- the endpoint MUST include the signed `flowId` in the proof input and load the
  browser flow directly by that id
- the endpoint MUST include the exact `contractDigest` in the proof input
- the endpoint MUST match the direct flow lookup against `publicIdentityKey` and
  `nonce`; QR and MAC bearer semantics remain the intended protection for the
  browser-to-flow handoff
- the endpoint MUST NOT create a device session or issue transport credentials
  directly
- the endpoint is a bounded long poll for setup only; it is not a general
  pre-auth RPC mechanism

## Reply-Subject Validation

Services MUST validate that a reply subject matches the caller's inbox prefix.

```ts
if (!msg.reply?.startsWith(callerInboxPrefix + ".")) {
  throw new AuthError("Reply subject mismatch");
}
```

This prevents confused deputy attacks.

## Operation Streaming Replies

Unary RPCs use one reply. Operations may use multiple replies to the same
validated caller inbox subject.

Rules:

- Trellis MUST permit bounded multi-response publishing to a reply subject that
  was supplied on an authenticated request and passed reply-subject validation
- this capability applies only to a reply subject derived from a request the
  service actually received
- it is not a general publish grant to arbitrary inbox subjects
- operation `watch()` and streamed `wait()` responses use this mechanism
- ordinary unary RPCs still respond once by convention even when the transport
  permission can support more than one response

## Error Codes

Public Auth RPCs use their declared `AuthError`, `ValidationError`, or
`UnexpectedError` envelope with stable codes and safe messages. Transitional
internal validators retain the reason codes below where existing runtimes depend
on them.

| Scenario                     | Reason Code                   |
| ---------------------------- | ----------------------------- |
| SessionKey header missing    | `missing_session_key`         |
| Session not found            | `session_not_found`           |
| Session expired              | `session_expired`             |
| Invalid signature            | `invalid_signature`           |
| SessionKey mismatch in OAuth | `oauth_session_key_mismatch`  |
| Session already bound        | `session_already_bound`       |
| AuthToken already used       | `authtoken_already_used`      |
| Timestamp out of range       | `iat_out_of_range`            |
| Identity grant required      | `approval_required`           |
| Contract changed             | `contract_changed`            |
| User inactive                | `user_inactive`               |
| User not found               | `user_not_found`              |
| Unknown service              | `unknown_service`             |
| Service disabled             | `service_disabled`            |
| Unknown device               | `unknown_device`              |
| Device activation revoked    | `device_activation_revoked`   |
| Device deployment not found  | `device_deployment_not_found` |
| Device deployment disabled   | `device_deployment_disabled`  |
| Reply mismatch               | `reply_subject_mismatch`      |
| Missing capabilities         | `insufficient_permissions`    |

Internal storage, SQL, crypto, provider, and topology causes are never public,
even after connection authentication. They are recorded in structured server
logs and collapse to `internal_error` at HTTP, RPC, operation, and callout
boundaries.

Browser clients treat `session_not_found` as an authentication-required state,
not as a page-local application error. Revoked browser sessions therefore
re-enter the normal login redirect flow so the app can preserve its current
return path and show sign-in UX. Non-browser clients may surface the same
`AuthError` directly.

## Internal State Model

Rust-owned principal, provider-identity, session, desired-authority,
runtime-evidence, and materialized-authority records are specified in
[rust-authorization-state.md](./rust-authorization-state.md). Those records and
the unsigned issuable-state query precede external auth/bootstrap cutover and
signed authorization-context issuance. The browser-flow records below describe
the retained external flow behavior until that cutover; they are not the Rust
authority storage model.

## Browser Flow Protocol

The portal-owned browser login UX uses `flowId` as the browser-visible
identifier. The flow is proof-bound to the initiating session key and exact
participant; there is no second portal-authored authority token.
Trellis-generated account ids use `usr_` plus a ULID, and auth-owned review ids
use their semantic prefix plus a ULID. Trellis ships a built-in portal served by
the Trellis HTTP server from static assets. Login portal records and route
selectors are global auth-owned routing config; the built-in login portal record
is visible, non-removable, and non-replaceable. Device deployments may carry
deployment-owned portal-route metadata for device flows. Neither form is
standalone portal authority. Device activation uses the same browser-visible
`flowId` concept with `kind: "device_activation"` flow records rather than a
separate public identifier. Portals are web apps, not service-authenticated
principals; if a portal later continues as a Trellis app after login, it does so
under a normal user session.

Flow summary:

1. `POST /auth/requests` validates the signed login-init request and exact
   participant/API presentation, derives one server-owned required/optional
   consent proposal, and creates a Trellis-owned browser flow plus a short
   `flowId`-based `loginUrl` when current authority cannot bind immediately.
2. `GET /auth/login/:provider` requires `flowId` and stores the provider choice
   in the same browser flow. The provider must be allowed by the selected login
   portal policy. If the referenced login flow is expired but still carries an
   app `redirectTo`, auth redirects to that app URL without adding an auth error
   so the app can restart its current auth request.
3. OIDC start creates PKCE/nonce state bound to a state-specific `HttpOnly`,
   `SameSite=Lax` browser cookie and the exact portal-policy digest. Callback
   verifies browser possession and current provider/registration policy before
   CAS claim and exchange. Identity-link flows never self-register.
4. `GET /auth/flow/:flowId` returns the current state, exact consent view and
   digest, effective providers, registration policy, authenticated profile, and
   validated redirect target. For a known expired browser flow, the expired
   state may include `returnLocation` so portals can return to the originating
   app without showing a transient expiration screen; missing flows do not
   receive an invented return URL.
5. `POST /auth/flow/:flowId/approval` accepts only the current consent-view
   digest, selected server-issued optional bundle ids, decision, and idempotency
   key. The server re-resolves the participant and rejects stale wording,
   unknown bundles, caller-authored grants/capabilities, and reserved authority.
6. `POST /auth/flow/:flowId/bind` completes the browser bind from
   `{ sessionKey, sig }`.

When a caller's participant changes, it starts the normal auth request flow with
the current canonical participant and referenced API artifacts. Human wording
may change the consent-view digest but does not change the machine proposal
digest. Auth may bind immediately only when current exact identity authority
covers the resolved required request; otherwise it returns a normal browser
flow.

Bind proof rules:

- browser flow creation uses the purpose-specific `trellis.session-proof.v1`
  request transcript over the complete request with its signature removed
- bind consumes the server-owned proof-bound flow with a durable idempotency
  key; it does not accept a second concatenated-signature format
- browser clients treat flow claims as internal auth-service state rather than a
  fragment-delivered public contract

Runtime storage responsibilities:

| Storage                       | Logical contents                                                                                                                                                                                                                                                     | TTL                             |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------- |
| SQL                           | Users, credentials, sessions, principals, desired and materialized authority, proposals/decisions, deployments, instances, devices, delegations, portals/routes, provisioning records, idempotency results, post-commit actions, and hashed account-management flows | Durable, with explicit expiries |
| `trellis_auth_oauth` KV       | PKCE/nonce state, browser-binding digest, portal-policy digest, CAS claim/result, and terminal unknown-outcome state                                                                                                                                                 | 15 min                          |
| `trellis_auth_browser` KV     | Proof-bound browser flow and exact server-owned consent proposal keyed by `flowId`                                                                                                                                                                                   | Browser-flow TTL                |
| `trellis_auth_replay` KV      | Short-lived NATS connect-proof replay admissions                                                                                                                                                                                                                     | Proof window                    |
| `trellis_auth_connections` KV | Active connection presence keyed by NATS user key                                                                                                                                                                                                                    | 120 s                           |

Ephemeral browser-binding and account-flow bearer secrets are stored by digest,
not raw value. OAuth state ids are public correlation ids; the separate cookie
secret supplies browser possession.

Browser flows are keyed by raw `flowId` because the flow identifier is
browser-visible and used to fetch auth-owned portal state. Device activation
records persist for the lifetime of the activated device unless revoked. Browser
login uses auth-owned global login portal records and route selectors. Device
activation routing remains deployment-owned authority state.

The complete server-owned `consent` value is immutable for the lifetime of the
browser flow. CAS state transitions may advance lifecycle/result fields but must
reject changes to the consent view, consent-view digest, machine proposal
digest, optional bundles, or capability definitions.

Provider chooser state returns only effective providers after selected portal
policy. `allowedFederatedProviders: null` allows all configured providers, `[]`
allows none, and a non-empty array allows only that configured subset.

### Browser Flow Record

```ts
{
  format: "trellis.auth-browser-flow.v1";
  flowId: string;
  state: "choose_provider" | "authenticated" | "approval_required" |
    "approval_denied" | "approved" | "consumed" | "expired";
  requestId: string;
  requestDigest: string;
  participantId: string;
  participantArtifactDigest: string;
  participantNeedsDigest: string;
  consent: ServerOwnedConsentProposal;
  sessionPublicKey: string;
  sessionNkey: string;
  portalId: string;
  redirectTarget: string | null;
  principalId: string | null;
  createdAt: number;
  expiresAt: number;
  version: number;
}
```

### Session Object

All principal kinds use the same durable session record: generated session id,
principal id/kind, exact participant artifact and needs digests, session public
key and key id, server-owned inbox prefix, created/last-seen/expiry/revocation
times, and optional deployment/instance binding. Authority and transport ACLs
are not copied into the session; issuance resolves current authority on every
credential refresh.

### Identity Authority

User/app, CLI, native, and device-user authority uses the same immutable
proposal and terminal-decision workflow as deployment authority. Acceptance
updates the current desired-authority projection and inserts its outbox record
in one transaction. The accepted record binds the stable principal, exact
participant artifact, needs digest, `GrantSetV1`, and expiry.

Mutable capability groups, deployment grant overrides, stored NATS subject ACLs,
and contract-era identity-grant objects are not part of the protocol.

### Users Projection

This account projection is Trellis-local and is updated by Trellis-managed
flows. `userId` is generated by Trellis and is not derived from provider
`origin`/`id` values. Local-user creation stores `username` as the subject of
the local identity, not as the account id.

Account linking adds provider identities to the same Trellis user account.
Multiple OIDC identities may be linked to one Trellis account. A Trellis account
may have at most one local username/password identity; an OIDC identity may link
to a local identity only when the target account does not already have a local
identity.

Local password-reset flows are bound to that existing local identity. The reset
flow record stores the target identity id and local username; portals may not
choose or change the username during reset completion.

Authorization is not stored on the user profile. First-admin bootstrap creates
and accepts authority for the exact built-in administration participant; its
mandatory grants are derived from that artifact.

### Active Connections

```ts
{
  serverId: string;
  clientId: number;
  connectedAt: string;
}
```

Rules:

- key is `<sessionKey>.<scopeId>.<user_nkey>` where `scopeId` is `userId` for
  user sessions, the service principal for service runtime sessions, and
  `instanceId` for device runtime sessions
- disconnect cleanup is best-effort plus TTL-backed self-healing

## Event Authorization

The `trellis` service publishes `events.v1.Auth.*` as part of `trellis.auth@v1`.

Events:

- `events.v1.Auth.Connections.Opened`
- `events.v1.Auth.Connections.Closed`
- `events.v1.Auth.Sessions.Revoked`
- `events.v1.Auth.Connections.Kicked`
- `events.v1.Auth.DeviceUserAuthorities.Requested`
- `events.v1.Auth.DeviceUserAuthorities.ReviewRequested`
- `events.v1.Auth.DeviceUserAuthorities.Approved`
- `events.v1.Auth.DeviceUserAuthorities.Resolved`

Rules:

- services may subscribe only if the presented contract proposal fits service
  deployment authority, reconciliation has produced the needed materialized
  authority, and the contract declares the events in grouped `uses.required` or
  `uses.optional` entries that are active and authorized
- extra manual capability flags are not the contract boundary
- user sessions must never receive service-only capabilities

## Non-Goals

- defining HTTP endpoint and RPC request/response payloads
- defining TypeScript or Rust client library APIs
- deployment configuration, rate limiting, root-key rotation, or HA runbooks
- issuer private-key storage, context issuance/distribution, runtime
  replay-cache implementation, and event-proof v2
