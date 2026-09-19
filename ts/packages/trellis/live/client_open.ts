import {
  headers as natsHeaders,
  type Msg,
  type NatsConnection,
} from "@nats-io/nats-core";
import { encodeEventSubjectParameterToken } from "../helpers.ts";
import {
  liveDataSubject,
  liveGenerateNonce,
  liveObserveSubject,
  liveParseFrame,
  liveVerifyServerProof,
} from "./protocol.ts";
import { ConsumerCore, LiveSubscription } from "./subscription.ts";
import { LiveCancellation, LiveEnd, LiveStreamError } from "./types.ts";
import type { LiveSessionManager } from "./manager.ts";

export const LIVE_VERSION = "trellis.live.v1";

export type LiveProof = {
  proof: string;
  iat: number;
  requestId: string;
  contextDigest: string;
};

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
  resolveContext?: (
    digest: string,
  ) => Promise<{ context: { sessionKey: string } }>;
  decodeEvent: (value: unknown) => T;
};

type LiveOfferWire = {
  type: string;
  openId: string;
  requestId: string;
  sessionId: string;
  baseSubject: string;
  dataSubject: string;
  controlSubject: string;
  provider: { sessionKey: string; connectionId: string };
  consumer: { connectionId: string };
  limits: { maxDataBodyBytes: number };
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

async function request(
  host: LiveOpenHost<unknown>,
  subject: string,
  payload: string,
): Promise<Msg> {
  const reply = inbox(host.inboxPrefix);
  const proof = await host.createRequestProof(subject, payload, reply);
  const sub = host.nats.subscribe(reply);
  try {
    host.nats.publish(subject, payload, {
      headers: proofHeaders(proof, host.sessionKey),
      reply,
    });
    const timeout = new Promise<"timeout">((resolve) =>
      setTimeout(() => resolve("timeout"), host.timeoutMs)
    );
    const next = await Promise.race([
      sub[Symbol.asyncIterator]().next(),
      timeout,
    ]);
    if (next === "timeout" || next.done) {
      throw new LiveStreamError("setup_timeout", "live request timed out");
    }
    return next.value;
  } finally {
    sub.unsubscribe();
  }
}

/** Open one live session and return a prepared handle. First next() activates. */
export async function openLiveFeed<T>(
  host: LiveOpenHost<T>,
  subject: string,
  inputJson: string,
): Promise<LiveSubscription<T>> {
  if (!host.live.isAvailable()) {
    throw new LiveStreamError("disconnected", "live manager is unavailable");
  }
  const permit = host.live.admitConsumer();
  const openId = liveGenerateNonce();
  const maxPayload = Number(host.nats.info?.max_payload ?? 1_048_576);
  const body = JSON.stringify({
    format: LIVE_VERSION,
    type: "open",
    openId,
    receiveMaxPayloadBytes: maxPayload,
    input: JSON.parse(inputJson),
  });
  const response = await request(host, subject, body);
  const offer = await verifyOffer(host, subject, openId, response);
  const core = new ConsumerCore<T>(offer.sessionId);
  const cancellation = new LiveCancellation();
  const seq = { n: 0 };
  const closeExchange = cancellation.cancelled().then(async () => {
    try {
      await sendControl(host, offer, {
        format: LIVE_VERSION,
        type: "control",
        sessionId: offer.sessionId,
        controlSeq: String(++seq.n),
        action: "close",
        reason: "cancelled",
        receivedSeq: String(core.receivedSeq()),
        consumedSeq: String(core.consumedSeq()),
      });
    } catch {
      // Close is best-effort once the local handle is gone.
    }
  });
  const subscription = new LiveSubscription(
    core,
    cancellation,
    permit,
    closeExchange,
  );
  void runPump(host, core, cancellation, offer, seq);
  return subscription;
}

async function verifyOffer(
  host: LiveOpenHost<unknown>,
  baseSubject: string,
  openId: string,
  response: Msg,
): Promise<LiveOfferWire> {
  const value = JSON.parse(new TextDecoder().decode(response.data)) as
    & LiveOfferWire
    & {
      code?: string;
      type: string;
    };
  if (value.type !== "offer") {
    throw new LiveStreamError(
      value.code ?? "invalid_request",
      `live open rejected with '${value.code ?? value.type}'`,
    );
  }
  if (value.openId !== openId) {
    throw new LiveStreamError(
      "protocol_error",
      "offer answers a different logical open",
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
  if (host.resolveContext) {
    const verified = await host.resolveContext(contextDigest);
    if (verified.context.sessionKey !== sessionKey) {
      throw new LiveStreamError(
        "protocol_error",
        "offer context does not bind the signing session key",
      );
    }
  }
  if (
    encodeEventSubjectParameterToken(sessionKey) !== value.provider.sessionKey
  ) {
    throw new LiveStreamError(
      "protocol_error",
      "offer signer does not match its advertised identity",
    );
  }
  if (
    liveDataSubject(
      value.provider.connectionId,
      value.consumer.connectionId,
      value.sessionId,
    ) !== value.dataSubject
  ) {
    throw new LiveStreamError(
      "protocol_error",
      "offer data subject is not canonical",
    );
  }
  if (
    liveObserveSubject(
      value.baseSubject,
      value.provider.connectionId,
      value.sessionId,
    ) !==
      value.controlSubject
  ) {
    throw new LiveStreamError(
      "protocol_error",
      "offer control subject is not canonical",
    );
  }
  if (value.baseSubject !== baseSubject) {
    throw new LiveStreamError(
      "protocol_error",
      "offer base subject does not match",
    );
  }
  return value;
}

async function runPump<T>(
  host: LiveOpenHost<T>,
  core: ConsumerCore<T>,
  cancellation: LiveCancellation,
  offer: LiveOfferWire,
  seq: { n: number },
): Promise<void> {
  await Promise.race([core.start.promise, cancellation.cancelled()]);
  if (cancellation.aborted) return;
  const data = host.nats.subscribe(offer.dataSubject);
  try {
    await sendControl(host, offer, {
      format: LIVE_VERSION,
      type: "control",
      sessionId: offer.sessionId,
      controlSeq: String(++seq.n),
      action: "activate",
      receivedSeq: "0",
      consumedSeq: "0",
    });
    void cancellation.cancelled().then(() => data.unsubscribe());
    for await (const message of data) {
      if (cancellation.aborted) return;
      const headers = message.headers;
      const proof = headers?.get("trellis-live-proof");
      const contextDigest = headers?.get("authorization-context");
      const sessionKey = headers?.get("session-key");
      if (!proof || !contextDigest || !sessionKey) continue;
      try {
        liveVerifyServerProof(
          proof,
          contextDigest,
          message.subject,
          message.data,
          sessionKey,
        );
      } catch {
        continue;
      }
      let frame;
      try {
        frame = liveParseFrame(message.data);
      } catch {
        continue;
      }
      if (frame.sessionId !== offer.sessionId) continue;
      if (frame.type === "data") {
        try {
          core.admit({
            value: host.decodeEvent(frame.value),
            encodedLen: message.data.length,
          });
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
        await sendControl(host, offer, {
          format: LIVE_VERSION,
          type: "control",
          sessionId: offer.sessionId,
          controlSeq: String(++seq.n),
          action: "pulse",
          challengeId: frame.challengeId,
          receivedSeq: String(core.receivedSeq()),
          consumedSeq: String(core.consumedSeq()),
        });
      } else if (frame.type === "end") {
        if (frame.terminal.reason === "complete") {
          core.commitEnd(new LiveEnd("complete"));
        } else {
          core.discardQueue();
          core.commitEnd(
            new LiveEnd(
              frame.terminal.reason,
              frame.terminal.error
                ? new LiveStreamError(
                  frame.terminal.error.code,
                  frame.terminal.error.message,
                )
                : undefined,
            ),
          );
        }
        return;
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
    data.unsubscribe();
  }
}

async function sendControl(
  host: LiveOpenHost<unknown>,
  offer: LiveOfferWire,
  control: Record<string, string>,
): Promise<void> {
  const payload = JSON.stringify(control);
  await request(host, offer.controlSubject, payload);
}
