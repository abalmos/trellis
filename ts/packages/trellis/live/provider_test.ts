import {
  headers as natsHeaders,
  type Msg,
  type NatsConnection,
} from "@nats-io/nats-core";
import { assertEquals } from "@std/assert";

import { encodeEventSubjectParameterToken } from "../helpers.ts";
import { base64urlEncode } from "../auth/utils.ts";
import { LIVE_VERSION } from "./client_open.ts";
import { LiveFeedProvider, type LiveProviderIdentity } from "./provider.ts";

function digest(kind: string): string {
  const bytes = new Uint8Array(32);
  for (let i = 0; i < kind.length; i++) bytes[i] = kind.charCodeAt(i);
  return base64urlEncode(bytes);
}

const BASE_SUBJECT = `feed.v1.${encodeEventSubjectParameterToken("api")}.${
  encodeEventSubjectParameterToken("deploy")
}.Watch`;

function identity(kind: string): LiveProviderIdentity {
  return {
    connectionId: `${kind}-conn`,
    sessionKey: `${kind}-session`,
    principalId: `${kind}-principal`,
    participantId: `${kind}-participant`,
    deploymentId: `${kind}-deploy`,
    instanceId: `${kind}-instance`,
    contextDigest: digest(kind),
  };
}

function msg(args: {
  data: Uint8Array;
  reply?: string;
  headers?: ReturnType<typeof natsHeaders>;
  subject?: string;
}): Msg {
  return {
    subject: args.subject ?? "feed.watch",
    data: args.data,
    reply: args.reply,
    headers: args.headers,
    respond: () => false,
    json: () => ({}),
    string: () => "",
    sid: 1,
  } as unknown as Msg;
}

Deno.test("NX04 unsigned and foreign control are dropped without reflection", async () => {
  const published: { subject: string; data: Uint8Array }[] = [];
  const nats = {
    publish(subject: string, data?: Uint8Array) {
      if (data) published.push({ subject, data });
    },
    info: { max_payload: 1_048_576 },
  } as unknown as NatsConnection;
  const provider = new LiveFeedProvider({
    nats,
    identity: identity("provider"),
    sign: async () => new Uint8Array(64),
  });
  const owner = identity("owner");
  let sourceStarts = 0;
  await provider.offer(
    msg({ data: new Uint8Array(), reply: "_INBOX.owner" }),
    BASE_SUBJECT,
    { openId: "open-1", receiveMaxPayloadBytes: 1_048_576 },
    owner,
    async () => {
      sourceStarts += 1;
    },
  );
  const offerJson = JSON.parse(
    new TextDecoder().decode(published.at(-1)!.data),
  );
  assertEquals(offerJson.kind, "feed");
  const sessionId = offerJson.sessionId as string;
  const encode = (value: unknown) =>
    new TextEncoder().encode(JSON.stringify(value));
  const closeBody = encode({
    format: LIVE_VERSION,
    type: "control",
    sessionId,
    controlSeq: "1",
    action: "close",
    reason: "cancelled",
    receivedSeq: "0",
    consumedSeq: "0",
  });
  const allow = async () => true;

  await provider.handleControl(msg({ data: closeBody }), allow);
  await provider.handleControl(
    msg({
      data: closeBody,
      reply: "_INBOX.attacker",
      headers: natsHeaders(),
    }),
    allow,
  );
  const foreign = natsHeaders();
  foreign.set("proof", "not-a-proof");
  foreign.set("authorization-context", "attacker-digest");
  foreign.set("session-key", "attacker-session");
  await provider.handleControl(
    msg({
      data: closeBody,
      reply: "_INBOX.attacker",
      headers: foreign,
    }),
    allow,
  );
  assertEquals(sourceStarts, 0);

  const ownerHeaders = natsHeaders();
  ownerHeaders.set("proof", "owner-proof");
  ownerHeaders.set("authorization-context", owner.contextDigest);
  ownerHeaders.set("session-key", owner.sessionKey);
  const activateBody = encode({
    format: LIVE_VERSION,
    type: "control",
    sessionId,
    controlSeq: "1",
    action: "activate",
    receivedSeq: "0",
    consumedSeq: "0",
  });
  await provider.handleControl(
    msg({
      data: activateBody,
      reply: "_INBOX.owner",
      headers: ownerHeaders,
    }),
    allow,
  );
  const challenge = published
    .map((frame) => JSON.parse(new TextDecoder().decode(frame.data)))
    .find((value) => value.type === "challenge");
  assertEquals(typeof challenge.challengeId, "string");
  const pulseBody = encode({
    format: LIVE_VERSION,
    type: "control",
    sessionId,
    controlSeq: "2",
    action: "pulse",
    challengeId: challenge.challengeId,
    receivedSeq: "0",
    consumedSeq: "0",
  });
  await provider.handleControl(
    msg({
      data: pulseBody,
      reply: "_INBOX.owner",
      headers: ownerHeaders,
    }),
    allow,
  );
  assertEquals(sourceStarts, 1);
  await provider.handleControl(
    msg({
      data: pulseBody,
      reply: "_INBOX.owner",
      headers: ownerHeaders,
    }),
    allow,
  );
  assertEquals(sourceStarts, 1);
});

Deno.test("operation-watch offers advertise operation-watch kind", async () => {
  const encodedOffers: Uint8Array[] = [];
  const nats = {
    publish(_subject: string, data?: Uint8Array) {
      if (data) encodedOffers.push(data);
    },
    info: { max_payload: 1_048_576 },
  } as unknown as NatsConnection;
  const provider = new LiveFeedProvider({
    nats,
    identity: identity("provider"),
    sign: async () => new Uint8Array(64),
  });
  const operationSubject = `operation.v1.${
    encodeEventSubjectParameterToken("api")
  }.${encodeEventSubjectParameterToken("deploy")}.Run`;
  await provider.offer(
    msg({ data: new Uint8Array(), reply: "_INBOX.owner" }),
    operationSubject,
    { openId: "open-2", receiveMaxPayloadBytes: 1_048_576 },
    identity("owner"),
    async () => {},
    "operation-watch",
  );
  const offerJson = JSON.parse(new TextDecoder().decode(encodedOffers.at(-1)!));
  assertEquals(offerJson.kind, "operation-watch");
  assertEquals(offerJson.baseSubject, operationSubject);
});
