import {
  headers as natsHeaders,
  type Msg,
  type NatsConnection,
} from "@nats-io/nats-core";
import { encodeEventSubjectParameterToken } from "../helpers.ts";
import { base64urlEncode } from "../auth/utils.ts";
import {
  liveDataSubject,
  liveGenerateNonce,
  liveObserveSubject,
  liveObserveWildcardSubject,
  liveParseControl,
  liveServerProofDigest,
} from "./protocol.ts";
import { LIVE_VERSION, type LiveSessionKind } from "./client_open.ts";
import type { LiveControlWire } from "./protocol.ts";

const WINDOW_FRAMES = 64;
const WINDOW_BYTES = 1_048_576;
const OPEN_RESERVATION_MS = 15_000;
const HEARTBEAT_INTERVAL_MS = 10_000;
const PEER_INACTIVITY_MS = 35_000;
const CONSUMER_STALL_MS = 35_000;

export type LiveProviderIdentity = {
  connectionId: string;
  sessionKey: string;
  principalId: string;
  participantId: string;
  deploymentId: string;
  instanceId: string;
  contextDigest: string;
};

export type LiveProviderHost = {
  nats: NatsConnection;
  identity: LiveProviderIdentity;
  sign: (digest: Uint8Array) => Promise<Uint8Array>;
};

type ProviderSession = {
  sessionId: string;
  openId: string;
  baseSubject: string;
  dataSubject: string;
  controlSubject: string;
  consumer: LiveProviderIdentity;
  phase: "offered" | "activating" | "active" | "closed";
  sourceStarted: boolean;
  seq: number;
  startSource: () => void;
  abort: AbortController;
};

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

async function publishSigned(
  host: LiveProviderHost,
  subject: string,
  body: Uint8Array,
  reply?: string,
): Promise<void> {
  const headers = await liveHeaders(host, reply ?? subject, body);
  host.nats.publish(reply ?? subject, body, { headers });
}

/** In-process live provider: delayed source start after Pulse. */
export class LiveFeedProvider {
  readonly #host: LiveProviderHost;
  readonly #sessions = new Map<string, ProviderSession>();

  constructor(host: LiveProviderHost) {
    this.#host = host;
  }

  wildcardSubject(baseSubject: string): string {
    return liveObserveWildcardSubject(
      baseSubject,
      this.#host.identity.connectionId,
    );
  }

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
    const dataSubject = liveDataSubject(
      this.#host.identity.connectionId,
      consumer.connectionId,
      sessionId,
    );
    const controlSubject = liveObserveSubject(
      baseSubject,
      this.#host.identity.connectionId,
      sessionId,
    );
    const abort = new AbortController();
    const session: ProviderSession = {
      sessionId,
      openId: open.openId,
      baseSubject,
      dataSubject,
      controlSubject,
      consumer,
      phase: "offered",
      sourceStarted: false,
      seq: 0,
      abort,
      startSource: () => {
        if (session.sourceStarted) return;
        session.sourceStarted = true;
        session.phase = "active";
        void startSource({
          signal: abort.signal,
          emit: async (value) => {
            session.seq += 1;
            const body = jsonBytes({
              format: LIVE_VERSION,
              type: "data",
              sessionId,
              seq: String(session.seq),
              value,
            });
            const headers = await liveHeaders(this.#host, dataSubject, body);
            this.#host.nats.publish(dataSubject, body, { headers });
          },
        }).then(async () => {
          const body = jsonBytes({
            format: LIVE_VERSION,
            type: "end",
            sessionId,
            finalSeq: String(session.seq),
            terminal: { reason: "complete", error: null },
          });
          const headers = await liveHeaders(this.#host, dataSubject, body);
          this.#host.nats.publish(dataSubject, body, { headers });
          session.phase = "closed";
        }).catch(() => {
          session.phase = "closed";
        });
      },
    };
    this.#sessions.set(sessionId, session);
    const offer = {
      format: LIVE_VERSION,
      type: "offer",
      kind,
      openId: open.openId,
      requestId: msg.headers?.get("request-id") ?? "",
      sessionId,
      baseSubject,
      dataSubject,
      controlSubject,
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
        maxDataBodyBytes: Math.min(
          open.receiveMaxPayloadBytes,
          Number(this.#host.nats.info?.max_payload ?? WINDOW_BYTES),
        ),
        windowFrames: WINDOW_FRAMES,
        windowBytes: WINDOW_BYTES,
        reservationMs: OPEN_RESERVATION_MS,
        heartbeatIntervalMs: HEARTBEAT_INTERVAL_MS,
        peerInactivityMs: PEER_INACTIVITY_MS,
        consumerStallMs: CONSUMER_STALL_MS,
      },
    };
    if (!msg.reply) return;
    await publishSigned(this.#host, msg.reply, jsonBytes(offer), msg.reply);
  }

  async handleControl(
    msg: Msg,
    authenticate?: (msg: Msg) => Promise<boolean>,
  ): Promise<void> {
    if (!msg.reply || !msg.headers) return;
    const proof = msg.headers.get("proof");
    const contextDigest = msg.headers.get("authorization-context");
    const sessionKey = msg.headers.get("session-key");
    if (!proof || !contextDigest || !sessionKey) return;
    let control: LiveControlWire;
    try {
      control = liveParseControl(msg.data);
    } catch {
      return;
    }
    const session = this.#sessions.get(control.sessionId);
    if (!session) return;
    if (sessionKey !== session.consumer.sessionKey) return;
    if (contextDigest !== session.consumer.contextDigest) return;
    if (authenticate && !(await authenticate(msg))) return;
    if (control.action === "activate") {
      session.phase = "activating";
      const body = jsonBytes({
        format: LIVE_VERSION,
        type: "challenge",
        sessionId: session.sessionId,
        challengeId: liveGenerateNonce(),
        lastSentSeq: "0",
      });
      const headers = await liveHeaders(this.#host, session.dataSubject, body);
      this.#host.nats.publish(session.dataSubject, body, { headers });
    } else if (control.action === "pulse") {
      session.startSource();
    } else if (control.action === "close") {
      session.abort.abort();
      session.phase = "closed";
      this.#sessions.delete(session.sessionId);
    }
    if (!msg.reply) return;
    const ack = jsonBytes({
      format: LIVE_VERSION,
      type: "control-ack",
      sessionId: session.sessionId,
      controlSeq: control.controlSeq,
      requestId: msg.headers?.get("request-id") ?? "",
      action: control.action,
      state: session.phase === "closed" ? "closed" : session.phase,
      acceptedReceivedSeq: control.receivedSeq ?? "0",
      acceptedConsumedSeq: control.consumedSeq ?? "0",
      terminal: null,
      cleanup: null,
    });
    await publishSigned(this.#host, msg.reply, ack, msg.reply);
  }
}

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
