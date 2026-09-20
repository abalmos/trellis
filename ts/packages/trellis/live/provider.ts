import {
  headers as natsHeaders,
  type Msg,
  type NatsConnection,
  type Subscription,
} from "@nats-io/nats-core";
import { encodeEventSubjectParameterToken } from "../helpers.ts";
import { base64urlEncode } from "../auth/utils.ts";
import {
  liveConstants,
  type LiveControlResponse,
  type LiveControlWire,
  liveDataSubject,
  liveGenerateNonce,
  liveNegotiateMaxDataBodyBytes,
  liveObserveSubject,
  liveObserveWildcardSubject,
  type LiveOfferWire,
  liveParseControl,
  liveServerProofDigest,
} from "./protocol.ts";
import { LIVE_VERSION, type LiveSessionKind } from "./client_open.ts";
import {
  type LiveClock,
  LiveDeadlines,
  LiveTimer,
  productionLiveClock,
} from "./deadlines.ts";
import { type LiveCloseReceipt, LiveEnd, LiveStreamError } from "./types.ts";
import {
  recordCatalogCounter,
  recordCatalogUpDown,
} from "../telemetry/metrics.ts";

const C = liveConstants();

type ProviderPhase =
  | "offered"
  | "activating"
  | "active"
  | "draining"
  | "closing"
  | "closed";

/** Provider host identity used to sign and address live messages. */
export type LiveProviderIdentity = {
  connectionId: string;
  sessionKey: string;
  principalId: string;
  participantId: string;
  deploymentId: string;
  instanceId: string;
  contextDigest: string;
};

/** Provider host dependencies supplied by the owning connection. */
export type LiveProviderHost = {
  nats: NatsConnection;
  identity: LiveProviderIdentity;
  sign: (digest: Uint8Array) => Promise<Uint8Array>;
  /** Internal monotonic clock; production connections omit it. */
  clock?: LiveClock;
};

type Outstanding = { seq: bigint; bytes: number };

type Challenge = { id: string; lastSentSeq: string };

type ControlOutcome = {
  state: "activating" | "active" | "closed";
  terminal: LiveEnd | undefined;
  cleanup: "complete" | "incomplete" | null;
  challenge: Challenge | undefined;
  startSource: boolean;
};

class ProviderSessionRecord {
  readonly sessionId: string;
  readonly openId: string;
  readonly kind: LiveSessionKind;
  readonly baseSubject: string;
  readonly dataSubject: string;
  readonly controlSubject: string;
  readonly consumer: LiveProviderIdentity;
  maxDataBodyBytes: number;
  phase: ProviderPhase = "offered";
  readonly deadlines: LiveDeadlines;
  readonly timer: LiveTimer;
  challenge: Challenge | undefined;
  readonly outstanding: Outstanding[] = [];
  outstandingBytes = 0;
  highestSent = 0n;
  highestConsumed = 0n;
  highestReceived = 0n;
  sourceStarted = false;
  readonly abort = new AbortController();
  emitInFlight = false;
  closed = false;
  counted = false;
  lastControlSeq = 0n;
  lastControlHash: string | undefined;
  lastControlResponse: Record<string, unknown> | undefined;
  startSource: () => void = () => {};

  constructor(
    sessionId: string,
    openId: string,
    kind: LiveSessionKind,
    baseSubject: string,
    providerConnectionId: string,
    consumer: LiveProviderIdentity,
    clock: LiveClock,
    onDue: () => void,
  ) {
    this.sessionId = sessionId;
    this.openId = openId;
    this.kind = kind;
    this.baseSubject = baseSubject;
    this.consumer = consumer;
    this.dataSubject = liveDataSubject(
      providerConnectionId,
      consumer.connectionId,
      sessionId,
    );
    this.controlSubject = liveObserveSubject(
      baseSubject,
      providerConnectionId,
      sessionId,
    );
    this.maxDataBodyBytes = C.windowBytes;
    this.deadlines = LiveDeadlines.reserved(clock.nowMs());
    this.timer = new LiveTimer(clock, onDue);
  }

  outstandingEmpty(): boolean {
    return this.outstanding.length === 0;
  }
}

function jsonBytes(value: unknown): Uint8Array {
  return new TextEncoder().encode(JSON.stringify(value));
}

async function liveHeaders(
  host: LiveProviderHost,
  subject: string,
  body: Uint8Array,
) {
  const digest = liveServerProofDigest(
    host.identity.contextDigest,
    subject,
    body,
  );
  const proof = base64urlEncode(await host.sign(digest));
  const headers = natsHeaders();
  headers.set("authorization-context", host.identity.contextDigest);
  headers.set("session-key", host.identity.sessionKey);
  headers.set("trellis-live-proof", proof);
  return headers;
}

/**
 * In-process live provider engine.
 *
 * A session is reserved by an offer, activated by a delivery-path challenge
 * round trip, then streams signed DATA frames under consumer credit until the
 * source ends or a real deadline fires.
 */
export class LiveFeedProvider {
  readonly #host: LiveProviderHost;
  readonly #clock: LiveClock;
  readonly #sessions = new Map<string, ProviderSessionRecord>();
  readonly #tombstones = new Map<string, number>();

  constructor(host: LiveProviderHost) {
    this.#host = host;
    this.#clock = host.clock ?? productionLiveClock;
  }

  wildcardSubject(baseSubject: string): string {
    return liveObserveWildcardSubject(
      baseSubject,
      this.#host.identity.connectionId,
    );
  }

  /** Install the owner-control subscription for one exact route. */
  subscribeControl(baseSubject: string): Subscription {
    return this.#host.nats.subscribe(this.wildcardSubject(baseSubject));
  }

  /** Reserve one session, publish its signed offer and start its timer. */
  async offer(
    msg: Msg,
    baseSubject: string,
    open: { openId: string; receiveMaxPayloadBytes: number },
    consumer: LiveProviderIdentity,
    startSource: (session: {
      emit: (value: unknown) => Promise<void>;
      signal: AbortSignal;
    }) => Promise<void>,
    kind: LiveSessionKind = "feed",
  ): Promise<void> {
    const sessionId = liveGenerateNonce();
    const record = new ProviderSessionRecord(
      sessionId,
      open.openId,
      kind,
      baseSubject,
      this.#host.identity.connectionId,
      consumer,
      this.#clock,
      () => void this.#onDue(record),
    );
    record.startSource = () => {
      if (record.sourceStarted || record.phase === "closed") return;
      record.sourceStarted = true;
      record.phase = "active";
      // The provider side owns each accepted session: count it active until
      // #close records exactly one end and releases it.
      try {
        recordCatalogUpDown("trellis.feed.active", 1, {
          "trellis.side": "server",
        });
        record.counted = true;
      } catch {
        // Optional telemetry must never replace provider results.
      }
      void this.#runSource(record, startSource);
    };
    this.#sessions.set(sessionId, record);
    const maxDataBodyBytes = this.#negotiate(
      open.receiveMaxPayloadBytes,
    );
    record.maxDataBodyBytes = maxDataBodyBytes;
    const offer: LiveOfferWire = {
      format: LIVE_VERSION,
      type: "offer",
      kind,
      openId: open.openId,
      requestId: msg.headers?.get("request-id") ?? "",
      sessionId,
      baseSubject,
      dataSubject: record.dataSubject,
      controlSubject: record.controlSubject,
      provider: {
        connectionId: this.#host.identity.connectionId,
        sessionKey: encodeEventSubjectParameterToken(
          this.#host.identity.sessionKey,
        ),
        principalId: this.#host.identity.principalId,
        participantId: this.#host.identity.participantId,
        deploymentId: this.#host.identity.deploymentId,
        instanceId: this.#host.identity.instanceId,
      },
      consumer: {
        connectionId: consumer.connectionId,
        sessionKey: encodeEventSubjectParameterToken(consumer.sessionKey),
        principalId: consumer.principalId,
        participantId: consumer.participantId,
      },
      limits: {
        maxDataBodyBytes,
        windowFrames: C.windowFrames,
        windowBytes: C.windowBytes,
        reservationMs: C.openReservationMs,
        heartbeatIntervalMs: C.heartbeatIntervalMs,
        peerInactivityMs: C.peerInactivityMs,
        consumerStallMs: C.consumerStallMs,
      },
    };
    if (!msg.reply) return;
    await this.#publishSigned(msg.reply, jsonBytes(offer), msg.reply);
    // Arm the reservation deadline; the timer owns all policy afterwards.
    this.#armTimer(record);
  }

  /** Handle one authenticated owner control. */
  async handleControl(
    msg: Msg,
    authenticate: (msg: Msg) => Promise<boolean>,
  ): Promise<void> {
    if (!msg.reply || !msg.headers) return;
    const sessionKey = msg.headers.get("session-key");
    const contextDigest = msg.headers.get("authorization-context");
    if (!sessionKey || !contextDigest) return;
    let control: LiveControlWire;
    try {
      control = liveParseControl(msg.data);
    } catch {
      return;
    }
    const record = this.#sessions.get(control.sessionId);
    const now = this.#clock.nowMs();
    if (!record) {
      // A late duplicate gets a bounded, signed session-not-found receipt.
      await this.#publishControlError(
        msg.reply,
        control,
        "session_not_found",
      );
      return;
    }
    if (record.closed) {
      await this.#publishControlError(msg.reply, control, "session_not_found");
      return;
    }
    if (sessionKey !== record.consumer.sessionKey) return;
    if (contextDigest !== record.consumer.contextDigest) return;
    if (!(await authenticate(msg))) return;
    const outcome = this.#applyControl(record, control, now);
    const ack = this.#ackBody(record, control, msg, outcome);
    const published = await this.#tryPublishSigned(
      msg.reply,
      jsonBytes(ack),
      msg.reply,
    );
    if (!published) {
      // A failed acknowledgement handoff closes the reservation so a retry
      // can never start a second source.
      this.#fail(
        record,
        "peer_lost",
        "activation acknowledgement handoff failed",
      );
      return;
    }
    if (outcome.challenge) {
      await this.#publishChallenge(record);
    }
    if (outcome.startSource) {
      record.startSource();
    }
  }

  #applyControl(
    record: ProviderSessionRecord,
    control: LiveControlWire,
    nowMs: number,
  ): ControlOutcome {
    const hash = this.#controlHash(control);
    const seq = BigInt(control.controlSeq);
    if (
      record.lastControlResponse && seq === record.lastControlSeq &&
      hash === record.lastControlHash
    ) {
      // Logical replay: return the cached semantic outcome.
      return record.lastControlResponse as unknown as ControlOutcome;
    }
    const outcome = this.#transition(record, control, nowMs);
    record.lastControlSeq = seq;
    record.lastControlHash = hash;
    record.lastControlResponse = outcome as unknown as Record<string, unknown>;
    return outcome;
  }

  #transition(
    record: ProviderSessionRecord,
    control: LiveControlWire,
    nowMs: number,
  ): ControlOutcome {
    switch (control.action) {
      case "activate": {
        if (record.phase === "activating" && record.challenge) {
          return {
            state: "activating",
            terminal: undefined,
            cleanup: null,
            challenge: record.challenge,
            startSource: false,
          };
        }
        const challenge: Challenge = {
          id: liveGenerateNonce(),
          lastSentSeq: record.highestSent.toString(),
        };
        record.phase = "activating";
        record.challenge = challenge;
        record.deadlines.beginActivating(nowMs, challenge.id);
        this.#armTimer(record);
        return {
          state: "activating",
          terminal: undefined,
          cleanup: null,
          challenge,
          startSource: false,
        };
      }
      case "pulse": {
        if (record.deadlines.outstandingChallenge() !== control.challengeId) {
          return {
            state: this.#stateFor(record),
            terminal: undefined,
            cleanup: null,
            challenge: undefined,
            startSource: false,
          };
        }
        const activating = record.phase === "activating";
        if (activating) {
          record.deadlines.commitActive(nowMs, true);
          record.phase = "active";
        } else {
          record.deadlines.freshRoundTrip(nowMs, true);
        }
        record.challenge = undefined;
        this.#applyCredit(record, control);
        this.#armTimer(record);
        return {
          state: "active",
          terminal: undefined,
          cleanup: null,
          challenge: undefined,
          startSource: activating && !record.sourceStarted,
        };
      }
      case "ack": {
        this.#applyCredit(record, control);
        this.#armTimer(record);
        return {
          state: this.#stateFor(record),
          terminal: undefined,
          cleanup: null,
          challenge: undefined,
          startSource: false,
        };
      }
      case "close":
      case "end-ack": {
        this.#applyCredit(record, control);
        this.#close(record, "cancelled", null);
        return {
          state: "closed",
          terminal: new LiveEnd("cancelled"),
          cleanup: "complete",
          challenge: undefined,
          startSource: false,
        };
      }
      default:
        return {
          state: this.#stateFor(record),
          terminal: undefined,
          cleanup: null,
          challenge: undefined,
          startSource: false,
        };
    }
  }

  #stateFor(
    record: ProviderSessionRecord,
  ): "activating" | "active" | "closed" {
    if (record.phase === "closed" || record.phase === "closing") {
      return "closed";
    }
    if (record.phase === "active" || record.phase === "draining") {
      return "active";
    }
    return "activating";
  }

  #applyCredit(record: ProviderSessionRecord, control: LiveControlWire): void {
    const received = control.receivedSeq ? BigInt(control.receivedSeq) : 0n;
    const consumed = control.consumedSeq ? BigInt(control.consumedSeq) : 0n;
    if (consumed > received || received > record.highestSent) return;
    if (consumed < record.highestConsumed) return;
    const advanced = consumed > record.highestConsumed;
    record.highestConsumed = consumed;
    record.highestReceived = received;
    while (
      record.outstanding.length > 0 && record.outstanding[0].seq <= consumed
    ) {
      const frame = record.outstanding.shift();
      if (frame) record.outstandingBytes -= frame.bytes;
    }
    if (advanced) record.deadlines.noteStallReset(this.#clock.nowMs());
    if (record.outstandingEmpty()) record.deadlines.outstandingCleared();
  }

  #ackBody(
    record: ProviderSessionRecord,
    control: LiveControlWire,
    msg: Msg,
    outcome: ControlOutcome,
  ): LiveControlResponse["body"] & { type: "control-ack" } {
    return {
      format: LIVE_VERSION,
      type: "control-ack",
      sessionId: record.sessionId,
      controlSeq: control.controlSeq,
      requestId: msg.headers?.get("request-id") ?? "",
      action: control.action,
      state: outcome.state,
      acceptedReceivedSeq: record.highestReceived.toString(),
      acceptedConsumedSeq: record.highestConsumed.toString(),
      terminal: null,
      cleanup: outcome.cleanup,
    };
  }

  async #publishControlError(
    reply: string,
    control: LiveControlWire,
    code: string,
  ): Promise<void> {
    const body = {
      format: LIVE_VERSION,
      type: "control-error",
      sessionId: control.sessionId,
      controlSeq: control.controlSeq,
      requestId: "",
      code,
    };
    await this.#tryPublishSigned(reply, jsonBytes(body), reply);
  }

  #controlHash(control: LiveControlWire): string {
    // Logical identity excludes transport metadata (proof/request id/reply).
    return JSON.stringify(control);
  }

  /** Arm the timer for the record's next deadline. */
  #armTimer(record: ProviderSessionRecord): void {
    record.timer.arm(record.deadlines.nextDue());
  }

  /** Evaluate one due deadline and perform its action. */
  async #onDue(record: ProviderSessionRecord): Promise<void> {
    if (record.closed) {
      record.timer.dispose();
      return;
    }
    const now = this.#clock.nowMs();
    const action = record.deadlines.evaluate(now);
    switch (action) {
      case "reservation_expired":
        this.#fail(record, "setup_timeout", "live activation did not complete");
        return;
      case "challenge_retry":
        await this.#publishChallenge(record);
        record.deadlines.rearmChallengeRetry(this.#clock.nowMs());
        break;
      case "challenge_due": {
        const challenge: Challenge = {
          id: liveGenerateNonce(),
          lastSentSeq: record.highestSent.toString(),
        };
        record.challenge = challenge;
        record.deadlines.beginChallenge(this.#clock.nowMs(), challenge.id);
        await this.#publishChallenge(record);
        break;
      }
      case "peer_inactive":
        this.#fail(
          record,
          "peer_lost",
          "live consumer silent past the inactivity bound",
        );
        return;
      case "consumer_stalled":
        this.#fail(
          record,
          "consumer_slow",
          "live consumer did not consume outstanding data",
        );
        return;
      case "credit_due":
        record.deadlines.creditSent();
        break;
      case "close_exchange_elapsed":
        this.#finalize(record);
        return;
      case "cleanup_grace_elapsed":
        record.deadlines.markCleanupGraceElapsed();
        break;
      default:
        break;
    }
    this.#armTimer(record);
  }

  async #publishChallenge(record: ProviderSessionRecord): Promise<void> {
    const challenge = record.challenge;
    if (!challenge) return;
    const body = jsonBytes({
      format: LIVE_VERSION,
      type: "challenge",
      sessionId: record.sessionId,
      challengeId: challenge.id,
      lastSentSeq: challenge.lastSentSeq,
    });
    await this.#tryPublishFrame(record, body);
  }

  async #publishSigned(
    subject: string,
    body: Uint8Array,
    reply?: string,
  ): Promise<void> {
    const headers = await liveHeaders(this.#host, reply ?? subject, body);
    this.#host.nats.publish(reply ?? subject, body, { headers });
  }

  async #tryPublishSigned(
    subject: string,
    body: Uint8Array,
    reply?: string,
  ): Promise<boolean> {
    try {
      await this.#publishSigned(subject, body, reply);
      return true;
    } catch {
      return false;
    }
  }

  async #tryPublishFrame(
    record: ProviderSessionRecord,
    body: Uint8Array,
  ): Promise<boolean> {
    try {
      const headers = await liveHeaders(
        this.#host,
        record.dataSubject,
        body,
      );
      this.#host.nats.publish(record.dataSubject, body, { headers });
      return true;
    } catch {
      return false;
    }
  }

  #negotiate(consumerMaxPayloadBytes: number): number {
    const providerMax = Number(
      this.#host.nats.info?.max_payload ?? C.windowBytes,
    );
    return liveNegotiateMaxDataBodyBytes(
      consumerMaxPayloadBytes,
      providerMax,
    );
  }

  async #runSource(
    record: ProviderSessionRecord,
    startSource: (session: {
      emit: (value: unknown) => Promise<void>;
      signal: AbortSignal;
    }) => Promise<void>,
  ): Promise<void> {
    try {
      await startSource({
        signal: record.abort.signal,
        emit: async (value) => {
          await this.#emit(record, value);
        },
      });
      await this.#publishEnd(record, new LiveEnd("complete"));
    } catch (cause) {
      await this.#publishEnd(
        record,
        new LiveEnd(
          "source_error",
          new LiveStreamError(
            "source_failed",
            cause instanceof Error ? cause.message : String(cause),
          ),
        ),
      );
    }
  }

  async #emit(record: ProviderSessionRecord, value: unknown): Promise<void> {
    if (
      record.closed || record.phase === "closing" || record.phase === "closed"
    ) {
      throw new LiveStreamError("closed", "live session is closed");
    }
    if (record.emitInFlight) {
      throw new LiveStreamError(
        "concurrent_emit",
        "a live emit is already in flight",
      );
    }
    const maxDataBodyBytes = record.maxDataBodyBytes;
    const seq = record.highestSent + 1n;
    const body = jsonBytes({
      format: LIVE_VERSION,
      type: "data",
      sessionId: record.sessionId,
      seq: seq.toString(),
      value,
    });
    if (body.length > maxDataBodyBytes) {
      throw new LiveStreamError(
        "payload_too_large",
        "live data frame exceeds the negotiated body limit",
      );
    }
    record.emitInFlight = true;
    try {
      // Wait for credit while racing cancellation.
      while (
        record.outstanding.length + 1 > C.windowFrames ||
        record.outstandingBytes + body.length > C.windowBytes
      ) {
        if (record.abort.signal.aborted) {
          throw new LiveStreamError("cancelled", "live session was cancelled");
        }
        await new Promise<void>((resolve) => setTimeout(resolve, 0));
        if (record.closed) {
          throw new LiveStreamError("closed", "live session is closed");
        }
      }
      const published = await this.#tryPublishFrame(record, body);
      if (!published) {
        throw new LiveStreamError("source_failed", "live publication failed");
      }
      record.highestSent = seq;
      record.outstanding.push({ seq, bytes: body.length });
      record.outstandingBytes += body.length;
      record.deadlines.dataAdmitted(this.#clock.nowMs());
      this.#armTimer(record);
    } finally {
      record.emitInFlight = false;
    }
  }

  async #publishEnd(
    record: ProviderSessionRecord,
    end: LiveEnd,
  ): Promise<void> {
    const body = jsonBytes({
      format: LIVE_VERSION,
      type: "end",
      sessionId: record.sessionId,
      finalSeq: record.highestSent.toString(),
      terminal: {
        reason: end.reason,
        error: end.error
          ? { code: end.error.code, message: end.error.message }
          : null,
      },
    });
    await this.#tryPublishFrame(record, body);
    record.phase = "closing";
    record.deadlines.beginClosing(this.#clock.nowMs());
    this.#armTimer(record);
  }

  #fail(record: ProviderSessionRecord, code: string, message: string): void {
    void this.#sendFailure(record, code, message);
  }

  async #sendFailure(
    record: ProviderSessionRecord,
    code: string,
    message: string,
  ): Promise<void> {
    const end = new LiveEnd(
      code === "consumer_slow"
        ? "consumer_slow"
        : code === "peer_lost"
        ? "peer_lost"
        : code === "setup_timeout"
        ? "setup_timeout"
        : "source_error",
      new LiveStreamError(code, message),
    );
    await this.#publishEnd(record, end);
  }

  #close(
    record: ProviderSessionRecord,
    reason: string,
    error: LiveEnd | null,
  ): void {
    void error;
    if (record.closed) return;
    try {
      if (record.counted) {
        recordCatalogUpDown("trellis.feed.active", -1, {
          "trellis.side": "server",
        });
      }
      recordCatalogCounter("trellis.feed.ends", 1, {
        "trellis.side": "server",
        "trellis.reason": reason === "complete" || reason === "cancelled"
          ? reason
          : "error",
      });
    } catch {
      // Optional telemetry must never replace provider results.
    }
    record.abort.abort();
    record.closed = true;
    record.phase = "closed";
    record.deadlines.closed(this.#clock.nowMs());
    record.timer.dispose();
    this.#sessions.delete(record.sessionId);
    this.#tombstones.set(record.sessionId, this.#clock.nowMs() + C.tombstoneMs);
  }

  #finalize(record: ProviderSessionRecord): void {
    this.#close(record, "complete", null);
  }
}

/** Parse one bounded live Feed opening request. */
export function parseLiveOpen(
  raw: Uint8Array,
):
  | { openId: string; receiveMaxPayloadBytes: number; input: unknown }
  | undefined {
  try {
    const value = JSON.parse(new TextDecoder().decode(raw)) as {
      format?: string;
      type?: string;
      openId?: string;
      receiveMaxPayloadBytes?: number;
      input?: unknown;
    };
    if (value.format !== LIVE_VERSION || value.type !== "open") return;
    if (typeof value.openId !== "string") return;
    if (typeof value.receiveMaxPayloadBytes !== "number") return;
    return {
      openId: value.openId,
      receiveMaxPayloadBytes: value.receiveMaxPayloadBytes,
      input: value.input,
    };
  } catch {
    return;
  }
}
