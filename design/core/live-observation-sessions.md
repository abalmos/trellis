---
title: Live Observation Sessions
description: Signed, connection-owned transport for Feed and Operation live observation.
---

# Live Observation Sessions

A **live observation session** is the ephemeral, caller-specific live transport
behind a Feed `Watch` or an Operation observation. It replaces the older
indefinite request/reply reply-inbox stream. Ordinary RPCs and finite Operation
controls remain request/reply.

This document is the architecture contract for that transport. The exact wire
schemas, constants, and error codes live in the shared protocol crate
(`trellis-protocol`) and are consumed by generated TypeScript and Rust clients.

## Model

- A **Feed** is an ephemeral caller-specific live view. Closing an observation
  ends only that view.
- An **Operation** is durable work. Its observer is ephemeral and does not own
  the executor; closing an observation never cancels, restarts, or changes the
  ownership of durable execution.
- One authenticated connection owns one live session manager. There is no
  central live-session database and no relay.
- Live transport is **bounded but not durable**: it flow-controls and
  acknowledges lifecycle transitions over Core NATS, does not retransmit
  application data, and does not promise exactly-once business processing.

## Wire flow

1. The consumer sends one bounded opening request to the bound Feed base with a
   fresh request-proof reply inbox. An Operation watch opening is carried on the
   existing Operation control route.
2. The provider authenticates the exact Feed `Subscribe` (or Operation
   `Observe`) authority, reserves a session, and returns a **signed offer** over
   one finite reply. No producer starts here.
3. The first consumer iteration installs a distinct **exact** data subscription
   and sends `activate`. The provider enters `ACTIVATING`, returns an activating
   acknowledgement, and publishes one signed delivery-path challenge.
4. The consumer answers the challenge with a signed `pulse`. A fresh, verified
   pulse acknowledgement **commits `ACTIVE`**, and only then does the provider
   invoke the source factory exactly once.
5. Application data flows on the signed, connection-scoped data subject.
   Owner-directed controls carry liveness, cumulative credit, and closure.

A prepared handle that is never iterated expires with its reservation and never
starts a producer.

## Subjects

Deployment-bound subject families are **singular** `feed.v1` and `operation.v1`:

```text
Feed base:        feed.v1.<b64(apiId)>.<b64(providerDeploymentId)>.<action>
Owner controls:   <base>.observe.<b64(P)>.<sessionId>
Live delivery:    live.v1.data.<b64(P)>.<b64(C)>.<sessionId>
```

Each consumer subscribes to its **exact** data subject, never the whole wildcard
it is permitted to receive. Owner controls are handled only by the replica that
accepted the session; opening routes keep their queue groups, but owner-control
subscriptions do not.

## Identity and authentication

Provider-origin messages (offer, control response, challenge, data, end) are
signed over the **actual NATS subject and raw received bytes** with the
`trellis-live-server-proof.v1` domain. The offer's identity fields use the
canonical subject-token projection of the full runtime key; proof headers and
retained identities use the full runtime key. A frame is accepted only when the
actual subject, current covered context, pinned provider tuple, session id, and
signature all verify.

An identity-preserving authorization refresh on an unchanged transport epoch
does not close the session, reset sequence/credit, or reinvoke the handler. A
changed runtime key, logical connection id, selected deployment, or removed
route ends the session. Invalid or foreign frames are discarded without
extending liveness and without closing an otherwise valid session.

## Credit and buffering

- The provider window is 64 outstanding frames and 1 MiB of exact encoded `DATA`
  bodies. Cost is the whole serialized frame body, not a surrogate.
- The consumer releases credit when a validated item is **handed to the
  application**, not when bytes arrive. It sends cumulative credit after 16
  newly consumed frames or 50 ms, whichever comes first, piggybacked on a
  pending pulse.
- Wire cursors are checked u64 values serialized as canonical decimal strings.
- A verified sequence gap, or a challenge/end that implies an unreceived
  trailing sequence, closes the session as `delivery_gap`. Duplicates neither
  deliver twice nor inflate credit.

## Liveness and closure

- After activation the provider schedules one challenge every 10 s; an
  unanswered challenge is re-sent unchanged every 2 s. Only a fresh challenge
  answer extends the peer-inactivity deadline; ordinary data and duplicate
  controls do not.
- A quiet, owned, authorized session stays active indefinitely. There is no
  total session age limit and no lifetime message quota.
- A stalled application backlog (outstanding data with no consumption progress)
  closes as `consumer_slow`. A quiet empty stream has no consumption deadline.
- Normal source completion publishes `end` after every admitted frame; the
  consumer acknowledges and drains already-queued items in bounded `DRAINING`
  before resolving `complete`.
- Explicit close and end-ack share one bounded best-effort exchange with fresh
  proofs per retry and a single total budget. `session_not_found`, transport
  loss, or no response leaves remote cleanup **unconfirmed**.

## Observation versus execution

- Closing an Operation observation never calls cancellation, writes
  `cancelRequestedAt`, or changes the executor lease.
- Reopening an Operation observation reconciles the current durable snapshot;
  transient updates missed while disconnected are not replayed.
- Feed and Operation observers never receive an automatic producer restart or a
  silent data-loss fallback.

## Observability

Live session, end, handshake, buffer, frame, rejection, and cleanup-pending
instruments are emitted by the real session owners. Labels are bounded; session
and Operation ids never become metric dimensions. No span is held open for the
lifetime of a session, and no per-frame spans are emitted.

## Testing

- Live monotonic deadlines are tested through the production deadline owner
  under a controllable local clock. This is virtual elapsed time, not a real
  multi-hour soak.
- NATS response-permission expiry and independent static live delivery are
  tested against an isolated broker with a deliberately short response
  allowance.
- Cross-language paths are exercised with bounded real signed traffic and real
  lifecycle events on normal builds.
- An application-owned idle handle remains active by design; garbage collection
  is never the cleanup mechanism.
