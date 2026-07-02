import { assertEquals } from "@std/assert";
import { mapDeadEventFromAdvisory, parseMaxDeliveriesAdvisory } from "./rpc.ts";

// Retained unit coverage: advisory spelling normalization and dead-event mapping
// are pure Jobs admin projection helpers; live tests own NATS delivery behavior.

Deno.test("Jobs admin projection parses advisory variants", () => {
  const timestamp = "2026-03-28T12:05:00.000Z";
  const context = {
    requestId: "request-job-1",
    traceId: "0123456789abcdef0123456789abcdef",
    traceparent: "00-0123456789abcdef0123456789abcdef-0123456789abcdef-01",
  };
  const expectedAdvisory = {
    stream: "JOBS_WORK",
    consumer: "documents-document-process",
    streamSeq: 41,
    deliveries: 5,
    timestamp,
  };
  const workEvent = {
    jobId: "job-1",
    service: "documents",
    jobType: "document-process",
    eventType: "created",
    state: "pending",
    context,
    tries: 1,
    maxTries: 4,
    timestamp: "2026-03-28T11:00:00.000Z",
  } satisfies Parameters<typeof mapDeadEventFromAdvisory>[1];
  const expectedEvent = {
    jobId: "job-1",
    service: "documents",
    jobType: "document-process",
    eventType: "dead",
    state: "dead",
    previousState: "pending",
    context,
    tries: 5,
    maxTries: 4,
    error:
      "max deliveries exceeded: stream=JOBS_WORK consumer=documents-document-process deliveries=5",
    timestamp,
  } satisfies NonNullable<ReturnType<typeof mapDeadEventFromAdvisory>>;

  assertEquals(
    parseMaxDeliveriesAdvisory({
      stream: "JOBS_WORK",
      consumer: "documents-document-process",
      timestamp,
    }),
    undefined,
  );

  for (
    const raw of [
      { stream_seq: 41, deliveries: 5 },
      { streamSeq: 41, deliveries: 5 },
      { stream_seq: 41, num_deliveries: 5 },
      { streamSeq: 41, num_deliveries: 5 },
    ]
  ) {
    const advisory = parseMaxDeliveriesAdvisory({
      stream: "JOBS_WORK",
      consumer: "documents-document-process",
      timestamp,
      ...raw,
    });
    assertEquals(advisory, expectedAdvisory);
    if (!advisory) throw new Error("expected advisory");
    assertEquals(
      mapDeadEventFromAdvisory(undefined, workEvent, advisory),
      expectedEvent,
    );
  }

  const terminalJob = {
    id: "job-1",
    service: "documents",
    type: "document-process",
    state: "completed",
    context,
    payload: { id: "job-1" },
    createdAt: "2026-03-28T11:00:00.000Z",
    updatedAt: "2026-03-28T11:10:00.000Z",
    tries: 1,
    maxTries: 4,
  } satisfies NonNullable<Parameters<typeof mapDeadEventFromAdvisory>[0]>;

  assertEquals(
    mapDeadEventFromAdvisory(terminalJob, workEvent, expectedAdvisory),
    undefined,
  );
});
