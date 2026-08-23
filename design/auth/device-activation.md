---
title: Device Activation
description: Preregistered device activation, portal routing, connect info, and first online activation flow.
order: 15
---

# Design: Device Activation

## Prerequisites

- [trellis-auth.md](./trellis-auth.md) - auth architecture and principal model
- [auth-api.md](./auth-api.md) - auth HTTP and RPC surfaces
- [auth-protocol.md](./auth-protocol.md) - proofs, connect payloads, and
  pre-auth wait rules
- [../contracts/trellis-api-participants.md](./../contracts/trellis-api-participants.md) -
  device lineage, presented contract, and implementation-offer rules

## Context

Trellis needs an activation flow for preregistered devices that:

- have their own durable identity
- may be offline during setup
- may have constrained input
- can send an outbound activation URL or QR payload to a phone or browser
- may later gain more product-specific business logic in the portal flow
- use normal Trellis runtime auth with the device identity key once they are
  online

This design makes `device` the primary architecture term for this activation
model.

Key decisions:

- `device` is the primary architecture term for this activation model
- activated devices are preregistered against deployment-owned device
  deployments
- the client does not choose a flow type or deployment during normal activation
- Trellis resolves the device instance, device deployment, and activation portal
  policy from preregistered records
- the built-in device activation portal is the Trellis-owned app contract
  `trellis.portal.activation@v1`
- the activation portal is still a browser web app; if it calls Trellis after
  login, it does so as the logged-in user rather than as a service
- devices present a contract proposal at runtime; deployments validate requested
  needs against deployment authority and materialized authority
- device deployments do not carry a separate rollout-target digest field
- device review is a first-class optional gate controlled by `reviewMode`
- the provisioning/admin path may generate the device root secret locally, but
  Trellis stores only `publicIdentityKey` plus activation-only secret material
  rather than the root secret itself

## Design

### 1) Preregistered device instances are the primary path

Known device activation starts from a preregistered instance record.

The expected lifecycle is:

1. an admin or manufacturing/provisioning process provisions the device instance
   by `publicIdentityKey` and `activationKey`
2. that instance is attached to a device deployment
3. a user later activates the device through an authenticated portal flow
4. the activated device reconnects later by asking Trellis for current connect
   info

Unknown or self-registering devices may be added later as a separate extension.
They are not the primary v1 model.

### 2) Device identity is the durable principal

Each activated device is its own Trellis principal.

- the device later authenticates with its own identity key, not as the user who
  activated it
- the user identity and the device identity are intentionally separate
- any short confirmation code is only a local setup signal; it is never the
  device's online credential

Each device starts from one root secret:

```text
deviceRootSecret: 32 random bytes
```

The device derives purpose-specific keys with HKDF-SHA256:

```text
identitySeed  = HKDF-SHA256(ikm=deviceRootSecret, salt="", info="trellis/device-identity/v1", L=32)
activationKey = HKDF-SHA256(ikm=deviceRootSecret, salt="", info="trellis/device-activate/v1", L=32)
```

The durable public identity key is:

```text
identityPrivateKey = Ed25519Seed(identitySeed)
publicIdentityKey  = Ed25519Public(identityPrivateKey)
```

Rules:

- `identityPrivateKey` is the real online credential for activated devices
- `activationKey` derives device-display confirmation evidence and may also be
  used by the client-only offline QR helpers
- Trellis stores the activation evidence needed to verify the confirmation code,
  but it does not need the device root secret or `identitySeed`
- if Trellis needs a stable instance id, it derives that id from
  `publicIdentityKey`
- clients do not pass a separate user-chosen instance identifier in the normal
  path

### 3) Device deployments define rollout and review policy

`DeviceDeployment` is a deployment-owned record used during activation and
online auth.

```json
{
  "deploymentId": "reader.default",
  "authority": {
    "contractIds": ["acme.reader@v1"],
    "capabilities": ["acme.reader::read"]
  },
  "contractHistory": [
    { "contractDigest": "<digest-v1>", "action": "accepted_update" },
    { "contractDigest": "<digest-v2>", "action": "accepted_update" }
  ],
  "reviewMode": "none",
  "disabled": false
}
```

Rules:

- `deploymentId` is the stable server-side identifier attached to the device
  instance and activation record
- `authority` stores deployment-owned desired authority
- each `contractId` identifies one contract lineage
- `contractHistory` records accepted authority update and authority migration
  history for the deployment; it is audit metadata, not authority
- activated devices present a contract proposal; auth checks that derived
  requested needs fit deployment authority and that reconciliation has produced
  the required materialized authority
- unknown or authority-incompatible presented contracts are rejected instead of
  falling back to another digest in the deployment
- `reviewMode: "required"` means portal completion creates or resumes a pending
  review rather than activating immediately
- device deployments require `reviewMode: "none" | "required"`; service
  deployments require `reviewMode: null`
- `requiresDeviceDelegation` independently controls user delegation and does not
  imply or bypass administrative review
- there is no separate rollout-target digest field

### 4) Activated devices may not request resources for now

Activated devices are consumer-only for now.

Rules:

- activated-device contracts may use `rpc`, `operations`, `events.subscribe`,
  and `uses`
- activated-device contracts may not declare `resources`
- activated-device contracts may not rely on installed resource bindings

### 5) Portal resolution is handled by Trellis

The client does not pass `flowType`, `deploymentId`, or `portalId` in the normal
path.

Routing rules:

- app and CLI login flows resolve portal routing from auth-owned global login
  route selectors keyed by app identity, then fall back to the built-in Trellis
  login portal
- activated-device flows resolve portal routing from device deployment
  authority, then fall back to the built-in Trellis device portal

This is automatic resolution in the sense that callers do not choose the portal
explicitly. It is still explicit on the server side because Trellis relies on
auth-owned login route selectors, stored deployment authority metadata for
device flows, device-deployment records, and the built-in Trellis fallback.

### 6) Known-device activation uses one auth-owned operation

Known preregistered device activation uses one requester-visible auth-owned
operation: `Auth.DeviceUserAuthorities.Resolve`.

Happy path without review:

```mermaid
sequenceDiagram
    participant W as Device
    participant U as User Browser
    participant T as Trellis Auth
    participant P as Portal

    W->>T: POST /bootstrap/device (proof-bound initial bootstrap)
    T-->>W: Return activationUrl with flowId
    W->>U: Show activation URL or QR payload
    U->>P: Open /_trellis/portal/devices/activate?flowId=...
    U->>P: Authenticate and complete portal business logic
    P->>T: Activate known device instance
    W->>T: Retry POST /bootstrap/device with fresh request proof
    T-->>W: Return ready session/context/NATS evidence
```

If portal-side business logic is long-running, the portal may still use its own
async workflow around that auth-owned operation. If the portal calls Trellis
during that work, it does so using a normal user-authenticated browser app
contract rather than service credentials or portal-specific contract machinery.

If `reviewMode` is `required`, the activation flow inserts an auth-owned
pending-review step:

- `Auth.DeviceUserAuthorities.Resolve` creates or resumes a review record
  instead of activating immediately
- auth emits `events.v1.Auth.DeviceUserAuthorities.ReviewRequested` for reviewer
  automation
- a service or privileged user with `trellis.auth::device.review` or `admin`
  decides the review through auth RPCs
- the built-in portal and custom portals observe review and completion through
  the operation's `progress`, `watch()`, and `wait()` semantics until it becomes
  `activated`, `rejected`, or `expired`
- `/bootstrap/device` creates or resumes the durable review; the authenticated
  `Resolve` start transition verifies the device confirmation code and claims
  the review for exactly one user
- administrative approval of an unclaimed review does not create required user
  delegation or make the device ready; the claiming user completes that
  transition later
- `Resolve` get and snapshot projection are read-only, server-side `wait()` is
  notifier-driven rather than a SQLite polling loop, and operation cancellation
  is unsupported

### 7) Device records

The flow uses four durable record families, one short-lived browser flow record,
and one auth-owned secret record.

`AuthBrowserFlow(kind="device_activation")` preserves QR context across login or
account creation.

```json
{
  "flowId": "01KS755ZXTHRWQEXM1VGAMM7BF",
  "kind": "device_activation",
  "deviceActivation": {
    "instanceId": "dev_...",
    "deploymentId": "reader.default",
    "publicIdentityKey": "<base64url>",
    "nonce": "<base64url>",
    "qrMac": "<base64url>"
  },
  "createdAt": "2026-04-05T12:00:00Z",
  "expiresAt": "2026-04-05T12:30:00Z"
}
```

`DeviceInstance` is the preregistered known device record.

```json
{
  "instanceId": "dev_...",
  "publicIdentityKey": "<base64url>",
  "deploymentId": "reader.default",
  "metadata": {
    "name": "Front Desk Reader",
    "serialNumber": "SN-123",
    "modelNumber": "MX-10",
    "assetTag": "asset-42"
  },
  "state": "registered",
  "createdAt": "2026-04-05T11:00:00Z",
  "activatedAt": null,
  "revokedAt": null
}
```

Rules:

- `metadata` is optional operator-provided string metadata for CLI and console
  experiences
- Trellis understands `name`, `serialNumber`, and `modelNumber` for default
  admin display, but the map may also include deployment-specific opaque keys
- auth, activation, and connect-info decisions do not depend on this metadata
- device instances do not store authority; connect-info and runtime auth resolve
  the presented contract proposal against enabled device deployment authority
  and materialized authority

`DeviceProvisioningSecret` is the auth-owned activation secret material keyed by
`instanceId`.

```json
{
  "instanceId": "dev_...",
  "activationKey": "<base64url>",
  "createdAt": "2026-04-05T11:00:00Z"
}
```

`DeviceActivationReview` tracks optional gated review.

```json
{
  "reviewId": "dar_01KS755ZXTHRWQEXM1VGAMM7BG",
  "flowId": "01KS755ZXTHRWQEXM1VGAMM7BF",
  "instanceId": "dev_...",
  "publicIdentityKey": "<base64url>",
  "deploymentId": "reader.default",
  "state": "pending",
  "requestedAt": "2026-04-05T12:03:00Z",
  "expiresAt": "2026-04-05T12:18:00Z",
  "decidedAt": null,
  "reason": null
}
```

Device activation browser `flowId` values are ULIDs. Review ids use `dar_`
followed by a ULID.

`DeviceActivationRecord` is the final auth decision for that instance once
activation is granted. It also keeps the activating user identity when the
device was activated through a browser or review flow so `Auth.Sessions.Me` can
surface that user later.

```json
{
  "instanceId": "dev_...",
  "publicIdentityKey": "<base64url>",
  "deploymentId": "reader.default",
  "activatedBy": {
    "origin": "github",
    "id": "123"
  },
  "state": "activated",
  "activatedAt": "2026-04-05T12:08:00Z",
  "revokedAt": null
}
```

### 8) Offline QR enrollment is not a server route

The client libraries retain helpers for constructing and validating this
device-authored payload:

```json
{
  "v": 1,
  "publicIdentityKey": "<base64url>",
  "nonce": "<base64url>",
  "qrMac": "<base64url>"
}
```

Trellis 0.12 has no server endpoint that accepts this QR payload or verifies its
MAC. Offline QR enrollment is future work. The current online workflow starts
with the device's proof-bound `POST /bootstrap/device` request and uses the
returned activation URL, durable review id, and device-derived confirmation
code. Do not add or call the retired activation-request endpoint.

### 9) Online activation wait

Before activation, a device cannot use authenticated RPCs. Device-side wait
helpers therefore repeat `POST /bootstrap/device`, creating a fresh request id,
timestamp, session proof, and session key on every attempt. There is no separate
activation wait route.

Response model:

```ts
type DeviceBootstrapResponse =
  | {
    state: "activation_pending";
    serverNow: number;
    activation: {
      state: "pending";
      reviewId: string;
      activationUrl: string;
      expiresAt: number;
      retryAfterMs: number;
    };
  }
  | {
    state: "ready";
    serverNow: number;
    // exact retained bootstrap session, context, routes, and runtime evidence
  };
```

Rules:

- each bootstrap response projects the authoritative review `expiresAt`; clients
  derive a monotonic deadline from `expiresAt - serverNow`
- claim, administrative decision, and delegation-completion transactions reject
  expired reviews using server time; public review boundaries durably project
  due pending reviews as `expired`
- a later bootstrap may create a replacement review with a new review id and
  confirmation evidence; callers do not silently follow that replacement
- bootstrap proof construction and verification are canonical only in
  [auth-protocol.md](./auth-protocol.md); this document intentionally does not
  duplicate the algorithm
- the locally derived confirmation code proves device-display possession to the
  authenticated user who starts `Auth.DeviceUserAuthorities.Resolve`
- a `ready` response is retained for the separate pure `connect_device` or
  `TrellisDevice.connect(...)` handoff; clients do not bootstrap a second
  session

### 10) Connect info is server-provided

Activated devices need current runtime connect information from Trellis both:

- when a caller explicitly asks to connect after activation completes
- on later startups when activation is already complete and the device wants to
  reconnect directly

Recommended shared response shape:

```ts
type DeviceConnectInfo = {
  instanceId: string;
  deploymentId: string;
  participant: {
    id: string;
    artifactDigest: string;
    needsDigest: string;
  };
  session: SessionRecord;
  nats: {
    jwt: string;
    servers: string[];
  };
  serverNow: number;
};
```

Rules:

- `POST /bootstrap/device` is the single bootstrap and reconnect boundary; the
  retired `/auth/devices/connect-info` preflight does not exist
- Trellis returns current endpoints, exact participant binding, effective
  grants, resource evidence, session metadata, and a deny-all JWT bound to the
  device's session NKey
- a pending activation returns pending review state and no usable credential
- reboot-safe storage keeps the device identity seed and durable activation
  identifiers, not transport topology or reusable shared credentials

### 11) Runtime auth presents a contract

Runtime auth happens after bootstrap returns `ready`. Device runtime is gated by
registration, active lifecycle state, any required administrative review and
user delegation, and a presented contract proposal whose requested needs fit
enabled device deployment authority and have converged into materialized
authority. Deployment authority never substitutes for activation readiness.

At connect time the device presents:

- identity-key proof
- exact `contractDigest`

Auth validates:

1. the known device instance by public identity key
2. lifecycle state and activation policy allow runtime connection, including an
   active delegation when the deployment requires one
3. the device deployment is present and enabled
4. the presented contract proposal derives requested needs that fit device
   deployment authority and are present in materialized authority

This keeps validation explicit while separating authority fit from activation
and implementation offer liveness. Administrative approval, required user
delegation, registration, lifecycle state, and materialized authority are
independent fail-closed inputs.

Lifecycle events are:

- `events.v1.Auth.DeviceUserAuthorities.Requested`
- `events.v1.Auth.DeviceUserAuthorities.ReviewRequested`
- `events.v1.Auth.DeviceUserAuthorities.Approved`
- `events.v1.Auth.DeviceUserAuthorities.Resolved`

## Client library boundary

Normal device, portal, and admin code SHOULD use Trellis client-library helpers
for the mechanical parts of device activation. Exact TypeScript declarations are
documented in the generated `/api` reference; exact Rust functions, structs, and
re-exports are documented in Rustdoc and generated SDK docs.

Rules:

- device-side helpers SHOULD derive the identity seed, public identity key, and
  activation key from the device root secret; applications persist only the
  device root secret directly
- activation helpers SHOULD build, encode, parse, and verify activation payloads
  and confirmation codes rather than forcing app code to reimplement byte
  layouts locally
- wait helpers own bounded polling through repeated `/bootstrap/device`
  requests, with a fresh request ID, timestamp, and proof on every attempt
- terminal rejected, expired, and disabled responses are errors; Rust surfaces
  them through typed `DeviceActivationError` variants
- activation helpers retain the exact ready bootstrap response for the separate
  pure runtime connection step instead of issuing a second device session
- portal and admin browser apps SHOULD prefer a typed device-activation client
  wrapper over manually spelling auth RPC method names and payload shapes
- authenticated portal-side activation starts the
  `Auth.DeviceUserAuthorities.Resolve` operation; review and completion are
  observed through operation progress and watch/wait semantics rather than a
  separate status-poll RPC
- the TypeScript device runtime connect helper is a pure runtime entrypoint; if
  Trellis says activation is still required it returns a transport error instead
  of starting activation on the caller's behalf
- the TypeScript device runtime connect helper accepts the root secret directly
  as bytes or a string form; storage, loading, generation, and rotation policy
  belong to the application
- the TypeScript device runtime connect helper accepts the same logger-or-false
  convention as service runtime helpers and should log distinct NATS lifecycle
  events for disconnect, reconnect attempts, reconnect success, stale
  connections, and connection errors
- device runtime helpers SHOULD use current proof-bound bootstrap evidence on
  startup rather than persisting stale session evidence across restarts
- the TypeScript runtime connect helper publishes baseline samples automatically
  through its exact Auth-granted private health subject and exposes the same
  callback-based `health` helper surface used by services for enriching those
  samples
- Deno device runtimes MAY use the high-level device-user authority helper after
  registration when they need user-delegated authority; runtime connectivity
  itself is still controlled by lifecycle checks, deployment authority, and
  materialized authority
- callers do not manage or persist serialized local activation state directly
- Deno file-backed activation persistence stays internal to that
  activation-status helper, with storage-location overrides when the runtime
  needs to control the storage location
- online activation waiting observes server-owned review and delegation state;
  client-only offline QR helpers do not enable runtime access
- Rust activated-device code SHOULD use the Rust helpers for deterministic
  identity derivation, confirmation-code construction, proof-bound activation
  status/wait, ready-evidence handoff, and runtime connection rather than
  hand-written HKDF, HMAC, bootstrap-proof, or connection logic
- each Rust device bootstrap attempt generates a fresh Ed25519 session key;
  pending attempts discard the private seed, while ready evidence retains the
  successful seed for the exact activation-to-connection handoff
- Rust callers may use lower-level generated SDK surfaces for authenticated
  portal-side activation until a small typed convenience wrapper is available,
  but those calls still follow the `Auth.DeviceUserAuthorities.Resolve`
  operation model
- the Rust device runtime helper should follow the same service-style connect
  pattern as the TypeScript device runtime helper and remain a thin wrapper over
  the public auth HTTP and RPC surfaces

Implementation status:

- TypeScript provides the activated-device path through
  `checkDeviceActivation(...)`, its bounded wait helper, and the separate
  `TrellisDevice.connect(...)` runtime entry point
- Rust provides `DeviceActivationOptions`, `check_device_activation(...)`,
  `wait_for_device_activation(...)`, typed pending/session/error projections,
  and the separate pure `TrellisClient::connect_device(...)` runtime entry point
- both clients use only `/bootstrap/device`; neither launches a browser or calls
  a retired activation-request, wait, or connect-info route

### Minimal activated device example

```ts
import { isErr, TrellisDevice } from "@qlever-llc/trellis";
import { checkDeviceActivation } from "@qlever-llc/trellis/device/deno";
import { defineDeviceContract } from "@qlever-llc/trellis";

export const device = defineDeviceContract(() => ({
  id: "acme.demo-device@v1",
  displayName: "Demo Device",
  description: "A small activated device used for local Trellis demos.",
}));

export default device;

const authority = await checkDeviceActivation({
  trellisUrl,
  contract: device,
  rootSecret,
});

if (authority.status === "not_ready") {
  throw new Error(`Device user authority is not ready: ${authority.reason}`);
}

if (authority.status !== "activated") {
  console.info(authority.activationUrl);
  await authority.waitForOnlineApproval();
}

const trellis = await TrellisDevice.connect({
  trellisUrl,
  contract: device,
  rootSecret,
}).orThrow();

const me = await trellis.rpc.auth.sessionsMe({});
if (isErr(me)) throw me.error;
```

Rules:

- a normal activated-device participant may own no RPCs, operations, events, or
  resources at all; a small `uses`-only contract is valid
- requesting `Auth.Sessions.Me` from a device runtime is valid because device
  contracts receive baseline auth access automatically
- device-local UI and review flow handling belong around
  `checkDeviceActivation(...)`, not inside `connect()`
- demos and applications should check activation status first and then connect
  with a separate `TrellisDevice.connect(...)` call

Those helpers SHOULD own:

- deriving the identity seed, public identity key, and activation key from the
  device root secret
- building and parsing the activation payload
- signing fresh bootstrap requests and polling until activation resolves
- deriving and verifying the short confirmation code when used
- retaining the ready bootstrap response for the connection handoff
- wrapping the low-level HTTP and RPC surfaces into small typed convenience
  methods

Application code SHOULD still own:

- secure storage of the device root secret
- device-local UX such as serving or rendering the activation URL / QR
- reviewer automation and decision policy when `reviewMode` is enabled
- portal-side business logic and optional review policy

The wire protocol remains public and stable as an escape hatch, but it is not
the preferred normal integration surface.
