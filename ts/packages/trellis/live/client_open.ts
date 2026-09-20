import {
  headers as natsHeaders,
  type Msg,
  type NatsConnection,
  type Subscription,
} from "@nats-io/nats-core";
import { encodeEventSubjectParameterToken } from "../helpers.ts";
import {
  liveConstants,
  type LiveControlResponse,
  liveDataSubject,
  liveGenerateNonce,
  liveObserveSubject,
  type LiveOfferWire,
  liveParseControlResponse,
  liveParseFrame,
  liveParseOffer,
  liveVerifyServerProof,
} from "./protocol.ts";
import {
  type LiveClock,
  LiveDeadlines,
  productionLiveClock,
} from "./deadlines.ts";
import { ConsumerCore, LiveSubscription } from "./subscription.ts";
import {
  LiveCancellation,
  type LiveCloseReceipt,
  LiveEnd,
  LiveStreamError,
} from "./types.ts";
import type { LiveSessionManager } from "./manager.ts";

export const LIVE_VERSION = "trellis.live.v1";

const C = liveConstants();

/** Live session kind advertised on a signed offer. */
export type LiveSessionKind = "feed" | "operation-watch";

/** Fresh request proof material for one live control or open. */
export type LiveProof = {
  proof: string;
  iat: number;
  requestId: string;
  contextDigest: string;
};

/** Resolved and retained provider context for offer verification. */
export type LiveVerifiedContext = {
  context: {
    sessionKey: string;
    connectionId: string;
    principalId: string;
    participantId: string;
    deploymentId?: string | null;
  };
  contextDigest: string;
};

/** Host dependencies supplied by the owning connection. */
export type LiveOpenHost<T> = {
  nats: NatsConnection;
  inboxPrefix: string;
  timeoutMs: number;
  sessionKey: string;
  live: LiveSessionManager;
  createRequestProof: (
    subject: string,
    payload: string,
    reply: string,
  ) => Promise<LiveProof>;
  /** Required: resolves and retains the provider context by digest. */
  resolveContext: (digest: string) => Promise<LiveVerifiedContext>;
  /** The consumer's installed provider deployment for this API, if bound. */
  selectedProviderDeploymentId?: string;
  decodeEvent: (value: unknown) => T | undefined;
  /** Internal monotonic clock; production connections omit it. */
  clock?: LiveClock;
};

function inbox(prefix: string): string {
  return `${prefix}.${liveGenerateNonce()}`;
}

function proofHeaders(proof: LiveProof, sessionKey: string) {
  const headers = natsHeaders();
  headers.set("proof", proof.proof);
  headers.set("iat", String(proof.iat));
  headers.set("request-id", proof.requestId);
  headers.set("authorization-context", proof.contextDigest);
  headers.set("session-key", sessionKey);
  return headers;
}

function sleepUntil(clock: LiveClock, deadlineMs: number): Promise<void> {
  return new Promise((resolve) => {
    const clear = clock.scheduleAt(deadlineMs, () => {
      clear();
      resolve();
    });
  });
}

async function withTimeout<U>(
  clock: LiveClock,
  timeoutMs: number,
  work: Promise<U>,
): Promise<U | undefined> {
  let clear: (() => void) | undefined;
  const timeout = new Promise<undefined>((resolve) => {
    clear = clock.scheduleAt(
      clock.nowMs() + timeoutMs,
      () => resolve(undefined),
    );
  });
  try {
    return await Promise.race([work, timeout]);
  } finally {
    clear?.();
  }
}

async function request(
  host: LiveOpenHost<unknown>,
  subject: string,
  payload: string,
): Promise<{ msg: Msg; requestId: string }> {
  const reply = inbox(host.inboxPrefix);
  const proof = await host.createRequestProof(subject, payload, reply);
  const clock = host.clock ?? productionLiveClock;
  const sub = host.nats.subscribe(reply);
  try {
    host.nats.publish(subject, payload, {
      headers: proofHeaders(proof, host.sessionKey),
      reply,
    });
    const received = await withTimeout(
      clock,
      host.timeoutMs,
      sub[Symbol.asyncIterator]().next().then((next) =>
        next.done ? undefined : next.value
      ),
    );
    if (!received) {
      throw new LiveStreamError("setup_timeout", "live request timed out");
    }
    return { msg: received, requestId: proof.requestId };
  } finally {
    sub.unsubscribe();
  }
}

/** Open one live Feed session and return a prepared handle. */
export async function openLiveFeed<T>(
  host: LiveOpenHost<T>,
  subject: string,
  inputJson: string,
): Promise<LiveSubscription<T>> {
  const openId = liveGenerateNonce();
  const maxPayload = Number(host.nats.info?.max_payload ?? C.windowBytes);
  const body = JSON.stringify({
    format: LIVE_VERSION,
    type: "open",
    openId,
    receiveMaxPayloadBytes: maxPayload,
    input: JSON.parse(inputJson),
  });
  return await openLiveSession(host, subject, subject, "feed", body, openId);
}

/** Open one Operation-watch live session on the existing control route. */
export async function openLiveOperationWatch<T>(
  host: LiveOpenHost<T>,
  operationSubject: string,
  controlSubject: string,
  args: { operationId: string; includeUpdates?: boolean },
): Promise<LiveSubscription<T>> {
  const openId = liveGenerateNonce();
  const maxPayload = Number(host.nats.info?.max_payload ?? C.windowBytes);
  const body = JSON.stringify({
    action: "watch",
    operationId: args.operationId,
    ...(args.includeUpdates ? { includeUpdates: true } : {}),
    observation: {
      format: LIVE_VERSION,
      type: "open",
      openId,
      receiveMaxPayloadBytes: maxPayload,
    },
  });
  return await openLiveSession(
    host,
    controlSubject,
    operationSubject,
    "operation-watch",
    body,
    openId,
  );
}

async function openLiveSession<T>(
  host: LiveOpenHost<T>,
  requestSubject: string,
  expectedBaseSubject: string,
  expectedKind: LiveSessionKind,
  body: string,
  openId: string,
): Promise<LiveSubscription<T>> {
  if (!host.live.isAvailable()) {
    throw new LiveStreamError("disconnected", "live manager is unavailable");
  }
  const permit = host.live.admitConsumer();
  const clock = host.clock ?? productionLiveClock;
  try {
    const response = await request(host, requestSubject, body);
    const offer = await verifyOffer(
      host,
      expectedBaseSubject,
      openId,
      expectedKind,
      response.msg,
      response.requestId,
    );
    const core = new ConsumerCore<T>(offer.sessionId);
    const cancellation = new LiveCancellation();
    const session = { seq: 0n };
    const closeFn = (): Promise<LiveCloseReceipt> =>
      closeExchange(host, offer, core, session, clock);
    const subscription = new LiveSubscription(
      core,
      cancellation,
      closeFn,
      permit,
    );
    void runPump(host, core, cancellation, offer, session, clock);
    return subscription;
  } catch (cause) {
    permit[Symbol.dispose]();
    throw cause;
  }
}

async function verifyOffer(
  host: LiveOpenHost<unknown>,
  baseSubject: string,
  openId: string,
  expectedKind: LiveSessionKind,
  response: Msg,
  requestId: string,
): Promise<LiveOfferWire> {
  let offer: LiveOfferWire;
  try {
    offer = liveParseOffer(response.data);
  } catch (cause) {
    // A signed open-error or legacy finite envelope is a setup failure.
    const decoded = safeCode(response.data);
    throw new LiveStreamError(
      decoded ?? "invalid_request",
      `live open rejected with '${decoded ?? "invalid_request"}'`,
    );
  }
  if (offer.kind !== expectedKind) {
    throw new LiveStreamError("protocol_error", "offer session kind mismatch");
  }
  if (offer.openId !== openId) {
    throw new LiveStreamError(
      "protocol_error",
      "offer answers a different logical open",
    );
  }
  if (offer.requestId !== requestId) {
    throw new LiveStreamError(
      "protocol_error",
      "offer answers a different opening request",
    );
  }
  const contextDigest = response.headers?.get("authorization-context");
  const sessionKey = response.headers?.get("session-key");
  const proof = response.headers?.get("trellis-live-proof");
  if (!contextDigest || !sessionKey || !proof) {
    throw new LiveStreamError("protocol_error", "offer omitted proof headers");
  }
  liveVerifyServerProof(
    proof,
    contextDigest,
    response.subject,
    response.data,
    sessionKey,
  );
  const verified = await host.resolveContext(contextDigest);
  if (verified.context.sessionKey !== sessionKey) {
    throw new LiveStreamError(
      "protocol_error",
      "offer context does not bind the signing session key",
    );
  }
  if (verified.contextDigest !== contextDigest) {
    throw new LiveStreamError(
      "protocol_error",
      "offer context digest is not the verified digest",
    );
  }
  if (
    encodeEventSubjectParameterToken(sessionKey) !== offer.provider.sessionKey
  ) {
    throw new LiveStreamError(
      "protocol_error",
      "offer signer does not match its advertised identity",
    );
  }
  if (offer.provider.connectionId !== verified.context.connectionId) {
    throw new LiveStreamError(
      "protocol_error",
      "offer provider connection does not match its verified context",
    );
  }
  if (offer.provider.principalId !== verified.context.principalId) {
    throw new LiveStreamError(
      "protocol_error",
      "offer provider principal does not match its verified context",
    );
  }
  const selectedDeployment = host.selectedProviderDeploymentId ??
    verified.context.deploymentId ?? undefined;
  if (
    !selectedDeployment || offer.provider.deploymentId !== selectedDeployment
  ) {
    throw new LiveStreamError(
      "protocol_error",
      "offer provider is not the selected deployment for this API",
    );
  }
  if (
    liveDataSubject(
      offer.provider.connectionId,
      offer.consumer.connectionId,
      offer.sessionId,
    ) !== offer.dataSubject
  ) {
    throw new LiveStreamError(
      "protocol_error",
      "offer data subject is not canonical",
    );
  }
  if (
    liveObserveSubject(
      offer.baseSubject,
      offer.provider.connectionId,
      offer.sessionId,
    ) !== offer.controlSubject
  ) {
    throw new LiveStreamError(
      "protocol_error",
      "offer control subject is not canonical",
    );
  }
  if (offer.baseSubject !== baseSubject) {
    throw new LiveStreamError(
      "protocol_error",
      "offer base subject does not match",
    );
  }
  if (
    offer.limits.windowFrames !== C.windowFrames ||
    offer.limits.windowBytes !== C.windowBytes
  ) {
    throw new LiveStreamError(
      "protocol_error",
      "offer limits do not match the shared protocol",
    );
  }
  return offer;
}

function safeCode(raw: Uint8Array): string | undefined {
  try {
    const value = JSON.parse(new TextDecoder().decode(raw)) as {
      code?: string;
    };
    return typeof value.code === "string" ? value.code : undefined;
  } catch {
    return undefined;
  }
}

/** Bounded callback ingress preserving delivery order. */
class MsgIngress {
  #queue: Msg[] = [];
  #bytes = 0;
  #messageWaiter: (() => void) | undefined;
  #closed = false;

  push(msg: Msg): boolean {
    if (this.#queue.length + 1 > C.windowFrames + 8) return false;
    if (
      this.#bytes + msg.data.length > C.windowBytes + 8 * C.maxControlBodyBytes
    ) {
      return false;
    }
    this.#queue.push(msg);
    this.#bytes += msg.data.length;
    const waiter = this.#messageWaiter;
    this.#messageWaiter = undefined;
    waiter?.();
    return true;
  }

  /** Shift the oldest retained message; never waits and never discards. */
  tryNext(): Msg | undefined {
    const msg = this.#queue.shift();
    if (msg) this.#bytes -= msg.data.length;
    return msg;
  }

  /** Resolve once a new message is retained (or the ingress is closed). */
  waitMessage(): Promise<void> {
    if (this.#closed || this.#queue.length > 0) return Promise.resolve();
    return new Promise<void>((resolve) => {
      this.#messageWaiter = resolve;
    });
  }

  close(): void {
    this.#closed = true;
    const waiter = this.#messageWaiter;
    this.#messageWaiter = undefined;
    waiter?.();
  }
}

async function runPump<T>(
  host: LiveOpenHost<T>,
  core: ConsumerCore<T>,
  cancellation: LiveCancellation,
  offer: LiveOfferWire,
  session: { seq: bigint },
  clock: LiveClock,
): Promise<void> {
  await Promise.race([core.waitStart(), cancellation.cancelled()]);
  if (cancellation.aborted) return;
  const ingress = new MsgIngress();
  let subscription: Subscription | undefined;
  try {
    subscription = host.nats.subscribe(offer.dataSubject, {
      callback: (error, msg) => {
        if (error) {
          ingress.close();
          return;
        }
        if (!ingress.push(msg)) {
          core.commitEnd(
            new LiveEnd(
              "consumer_slow",
              new LiveStreamError(
                "consumer_slow",
                "bounded live ingress exceeded",
              ),
            ),
          );
        }
      },
    });
    if (!(await flush(host.nats))) {
      core.commitEnd(
        new LiveEnd(
          "disconnected",
          new LiveStreamError(
            "disconnected",
            "live data subscription could not be flushed",
          ),
        ),
      );
      return;
    }
    core.setPhase("activating");
    const deadlines = LiveDeadlines.prepared(clock.nowMs());

    // Activation: bounded fresh-proof retries within the reservation.
    const reservationDeadline = clock.nowMs() + C.openReservationMs;
    const activateSeq = nextSeq(session);
    let activated = false;
    while (!activated) {
      const response = await controlAttempt(host, offer, {
        action: "activate",
        controlSeq: activateSeq.toString(),
        receivedSeq: "0",
        consumedSeq: "0",
      });
      if (
        response && response.kind === "ack" &&
        response.body.state !== "closed"
      ) {
        activated = true;
        break;
      }
      if (response && response.kind === "error") {
        core.commitEnd(
          new LiveEnd(
            "setup_timeout",
            new LiveStreamError(response.body.code, "live activation rejected"),
          ),
        );
        return;
      }
      if (clock.nowMs() >= reservationDeadline) {
        core.commitEnd(
          new LiveEnd(
            "setup_timeout",
            new LiveStreamError(
              "setup_timeout",
              "live activation did not complete within the reservation",
            ),
          ),
        );
        return;
      }
    }

    let lastCreditSent = 0n;
    let nextExpected = 1n;
    while (true) {
      core.commitDrainIfComplete();
      if (core.committedEnd()) return;
      if (core.phase === "draining") {
        const stall = createDeadlineWaiter(clock, deadlines.nextDue());
        try {
          const winner = await Promise.race([
            cancellation.cancelled().then(() => "cancel" as const),
            new Promise<"end">((resolve) => core.onEnd(() => resolve("end"))),
            stall.promise.then(() => "timer" as const),
          ]);
          if (winner === "timer") {
            core.discardQueue();
            core.commitEnd(
              new LiveEnd(
                "consumer_slow",
                new LiveStreamError(
                  "consumer_slow",
                  "live draining queue was not consumed",
                ),
              ),
            );
          }
        } finally {
          stall.dispose();
        }
        return;
      }

      // Recover any consumption that advanced while the pump was busy.
      const consumedNow = core.consumedSeq();
      if (consumedNow > lastCreditSent) {
        deadlines.noteConsumption(
          clock.nowMs(),
          Number(consumedNow - lastCreditSent),
        );
      }

      let winner: "cancel" | "credit" | "timer" | "msg";
      let message: Msg | undefined = ingress.tryNext();
      if (message) {
        winner = "msg";
      } else {
        const waiter = createDeadlineWaiter(clock, deadlines.nextDue());
        try {
          winner = await Promise.race([
            cancellation.cancelled().then(() => "cancel" as const),
            core.waitCredit().then(() => "credit" as const),
            waiter.promise.then(() => "timer" as const),
            ingress.waitMessage().then(() => "msg" as const),
          ]);
        } finally {
          waiter.dispose();
        }
        if (winner === "msg") {
          message = ingress.tryNext();
          if (!message) continue;
        }
      }
      if (winner === "cancel") {
        core.discardQueue();
        core.commitEnd(new LiveEnd("cancelled"));
        return;
      }
      if (winner === "credit") {
        continue;
      }
      if (winner === "timer") {
        const action = deadlines.evaluate(clock.nowMs());
        switch (action) {
          case "reservation_expired":
            core.commitEnd(
              new LiveEnd(
                "setup_timeout",
                new LiveStreamError(
                  "setup_timeout",
                  "live activation did not complete within the reservation",
                ),
              ),
            );
            return;
          case "peer_inactive":
            core.commitEnd(
              new LiveEnd(
                "peer_lost",
                new LiveStreamError(
                  "peer_lost",
                  "live provider silent past the inactivity bound",
                ),
              ),
            );
            return;
          case "credit_due": {
            const consumed = core.consumedSeq();
            deadlines.creditSent();
            if (consumed > lastCreditSent) {
              lastCreditSent = consumed;
              await controlAttempt(host, offer, {
                action: "ack",
                controlSeq: nextSeq(session).toString(),
                receivedSeq: core.receivedSeq().toString(),
                consumedSeq: consumed.toString(),
              });
            }
            break;
          }
          default:
            break;
        }
        continue;
      }
      const msg = message;
      if (!msg) continue;
      let frame;
      try {
        frame = liveParseFrame(msg.data, offer.limits.maxDataBodyBytes);
      } catch {
        continue;
      }
      if (!verifyProviderFrame(offer, msg, frame.sessionId)) continue;
      if (frame.type === "data") {
        const seq = BigInt(frame.seq);
        if (seq > nextExpected) {
          core.commitEnd(
            new LiveEnd(
              "delivery_gap",
              new LiveStreamError("delivery_gap", "live delivery sequence gap"),
            ),
          );
          return;
        }
        if (seq < nextExpected) continue;
        nextExpected = seq + 1n;
        try {
          const value = host.decodeEvent(frame.value);
          if (value === undefined) {
            core.releaseFiltered();
            continue;
          }
          if (!core.admit({ value, encodedLen: msg.data.length })) {
            core.commitEnd(
              new LiveEnd(
                "consumer_slow",
                new LiveStreamError(
                  "consumer_slow",
                  "bounded live ingress exceeded",
                ),
              ),
            );
            return;
          }
        } catch (error) {
          core.commitEnd(
            new LiveEnd(
              "protocol_error",
              error instanceof LiveStreamError
                ? error
                : new LiveStreamError("protocol_error", String(error)),
            ),
          );
          return;
        }
      } else if (frame.type === "challenge") {
        if (BigInt(frame.lastSentSeq) > core.receivedSeq()) {
          core.commitEnd(
            new LiveEnd(
              "delivery_gap",
              new LiveStreamError(
                "delivery_gap",
                "live challenge referred to unreceived data",
              ),
            ),
          );
          return;
        }
        const response = await controlAttempt(host, offer, {
          action: "pulse",
          controlSeq: nextSeq(session).toString(),
          challengeId: frame.challengeId,
          receivedSeq: core.receivedSeq().toString(),
          consumedSeq: core.consumedSeq().toString(),
        });
        if (
          response && response.kind === "ack" &&
          response.body.state === "active"
        ) {
          if (core.phase === "activating") {
            deadlines.commitActive(clock.nowMs(), false);
            core.setPhase("active");
          } else {
            deadlines.freshRoundTrip(clock.nowMs(), false);
          }
        }
      } else if (frame.type === "end") {
        if (BigInt(frame.finalSeq) !== core.receivedSeq()) {
          core.commitEnd(
            new LiveEnd(
              "delivery_gap",
              new LiveStreamError(
                "delivery_gap",
                "live end did not follow the complete sequence",
              ),
            ),
          );
          return;
        }
        await controlAttempt(host, offer, {
          action: "end-ack",
          controlSeq: nextSeq(session).toString(),
          finalSeq: frame.finalSeq,
          receivedSeq: core.receivedSeq().toString(),
          consumedSeq: core.consumedSeq().toString(),
        });
        const end = frame.terminal.error
          ? new LiveEnd(
            frame.terminal.reason,
            new LiveStreamError(
              frame.terminal.error.code,
              frame.terminal.error.message,
            ),
          )
          : new LiveEnd(frame.terminal.reason);
        if (end.isComplete()) {
          core.setPendingEnd(end);
          core.setPhase("draining");
          deadlines.beginDraining();
          deadlines.dataAdmitted(clock.nowMs());
        } else {
          core.commitEnd(end);
          return;
        }
      }
    }
  } catch (error) {
    core.commitEnd(
      new LiveEnd(
        "disconnected",
        new LiveStreamError("disconnected", String(error)),
      ),
    );
  } finally {
    ingress.close();
    subscription?.unsubscribe();
  }
}

type DeadlineWaiter = { promise: Promise<void>; dispose: () => void };

function createDeadlineWaiter(
  clock: LiveClock,
  deadlineMs: number | undefined,
): DeadlineWaiter {
  if (deadlineMs === undefined) {
    return { promise: new Promise<void>(() => {}), dispose: () => {} };
  }
  let disposed = false;
  let clear: (() => void) | undefined;
  const promise = new Promise<void>((resolve) => {
    clear = clock.scheduleAt(deadlineMs, () => {
      if (!disposed) resolve();
    });
  });
  return {
    promise,
    dispose: () => {
      disposed = true;
      clear?.();
    },
  };
}

function nextSeq(session: { seq: bigint }): bigint {
  session.seq += 1n;
  return session.seq;
}

function verifyProviderFrame(
  offer: LiveOfferWire,
  msg: Msg,
  sessionId: string,
): boolean {
  const headers = msg.headers;
  const proof = headers?.get("trellis-live-proof");
  const contextDigest = headers?.get("authorization-context");
  const sessionKey = headers?.get("session-key");
  if (!proof || !contextDigest || !sessionKey) return false;
  if (
    encodeEventSubjectParameterToken(sessionKey) !== offer.provider.sessionKey
  ) {
    return false;
  }
  if (sessionId !== offer.sessionId) return false;
  try {
    liveVerifyServerProof(
      proof,
      contextDigest,
      msg.subject,
      msg.data,
      sessionKey,
    );
    return true;
  } catch {
    return false;
  }
}

async function controlAttempt(
  host: LiveOpenHost<unknown>,
  offer: LiveOfferWire,
  control: Record<string, string>,
): Promise<LiveControlResponse | undefined> {
  const reply = inbox(host.inboxPrefix);
  const payload = JSON.stringify({
    format: LIVE_VERSION,
    type: "control",
    sessionId: offer.sessionId,
    ...control,
  });
  const proof = await host.createRequestProof(
    offer.controlSubject,
    payload,
    reply,
  );
  const clock = host.clock ?? productionLiveClock;
  const sub = host.nats.subscribe(reply);
  try {
    host.nats.publish(offer.controlSubject, payload, {
      headers: proofHeaders(proof, host.sessionKey),
      reply,
    });
    const received = await withTimeout(
      clock,
      host.timeoutMs,
      sub[Symbol.asyncIterator]().next().then((next) =>
        next.done ? undefined : next.value
      ),
    );
    if (!received) return undefined;
    const response = liveParseControlResponse(received.data);
    const contextDigest = received.headers?.get("authorization-context");
    const sessionKey = received.headers?.get("session-key");
    const proofHeader = received.headers?.get("trellis-live-proof");
    if (!contextDigest || !sessionKey || !proofHeader) return undefined;
    if (
      encodeEventSubjectParameterToken(sessionKey) !== offer.provider.sessionKey
    ) {
      return undefined;
    }
    liveVerifyServerProof(
      proofHeader,
      contextDigest,
      received.subject,
      received.data,
      sessionKey,
    );
    if (response.body.sessionId !== offer.sessionId) return undefined;
    if (response.body.controlSeq !== control.controlSeq) return undefined;
    if (response.kind === "ack" && response.body.action !== control.action) {
      return undefined;
    }
    return response;
  } catch {
    return undefined;
  } finally {
    sub.unsubscribe();
  }
}

async function closeExchange<T>(
  host: LiveOpenHost<T>,
  offer: LiveOfferWire,
  core: ConsumerCore<T>,
  session: { seq: bigint },
  clock: LiveClock,
): Promise<LiveCloseReceipt> {
  const end = core.committedEnd() ?? new LiveEnd("cancelled");
  const deadline = clock.nowMs() + C.closeExchangeMs;
  let cleanup: "complete" | "incomplete" | "unknown" = "unknown";
  while (clock.nowMs() < deadline) {
    const response = await controlAttempt(host, offer, {
      action: "close",
      reason: "cancelled",
      controlSeq: nextSeq(session).toString(),
      receivedSeq: core.receivedSeq().toString(),
      consumedSeq: core.consumedSeq().toString(),
    });
    if (response && response.kind === "ack") {
      cleanup = response.body.cleanup === "complete"
        ? "complete"
        : response.body.cleanup === "incomplete"
        ? "incomplete"
        : "unknown";
      break;
    }
    if (
      response && response.kind === "error" &&
      response.body.code === "session_not_found"
    ) {
      break;
    }
    const remaining = deadline - clock.nowMs();
    if (remaining <= 0) break;
    await sleepUntil(
      clock,
      clock.nowMs() + Math.min(C.closeRetryMs, remaining),
    );
  }
  const remote = cleanup === "unknown" ? "unconfirmed" : "confirmed";
  return { end, remote, cleanup };
}

async function flush(nats: NatsConnection): Promise<boolean> {
  try {
    await nats.flush();
    return true;
  } catch {
    return false;
  }
}
