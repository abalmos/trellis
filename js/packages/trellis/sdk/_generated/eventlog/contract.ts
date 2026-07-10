// Generated from ./generated/contracts/manifests/trellis.eventlog@v1.json
import type {
  ContractDependencyUse,
  SdkContractModule,
  TrellisContractV1,
  UseSpec,
} from "../../../index.ts";
import { API } from "./api.ts";

const CONTRACT_MODULE_METADATA = Symbol.for(
  "@qlever-llc/trellis/contracts/contract-module",
);

export const CONTRACT_ID = "trellis.eventlog@v1" as const;
export const CONTRACT_DIGEST =
  "dbrWayHMjo4A774xb0ovz2hd5qG6lfMBqXKxQ8wYyTw" as const;
export const CONTRACT = {
  "capabilities": {
    "trellis.eventlog::events.read": {
      "description": "View projected Trellis events and event consumer health.",
      "displayName": "Read Event Log data",
    },
    "trellis.eventlog::events.stream": {
      "description": "Subscribe to Event Log live invalidation frames.",
      "displayName": "Stream Event Log changes",
    },
  },
  "description":
    "Read-only Event Log API for Trellis event stream observability.",
  "displayName": "Trellis Event Log",
  "docs": {
    "markdown":
      "Provides read-only event, consumer-health, metrics, and live invalidation surfaces for Trellis events.",
    "summary": "Event stream observability APIs.",
  },
  "errors": {
    "NotFoundError": {
      "schema": { "schema": "NotFoundErrorData" },
      "type": "NotFoundError",
    },
  },
  "feeds": {
    "EventLog.Watch": {
      "capabilities": { "subscribe": ["trellis.eventlog::events.stream"] },
      "docs": {
        "markdown":
          "Streams ready and invalidation frames for Event Log clients.",
        "summary": "Watch event changes.",
      },
      "event": { "schema": "EventLogWatchFrame" },
      "input": { "schema": "EventLogWatchRequest" },
      "subject": "feeds.v1.EventLog.Watch",
      "version": "v1",
    },
  },
  "format": "trellis.contract.v1",
  "id": "trellis.eventlog@v1",
  "kind": "service",
  "rpc": {
    "EventLog.Consumers.Inspect": {
      "capabilities": { "call": ["trellis.eventlog::events.read"] },
      "errors": [{ "type": "UnexpectedError" }, { "type": "ValidationError" }, {
        "type": "NotFoundError",
      }],
      "input": { "schema": "EventLogConsumersInspectRequest" },
      "output": { "schema": "EventLogConsumersInspectResponse" },
      "subject": "rpc.v1.EventLog.Consumers.Inspect",
      "version": "v1",
    },
    "EventLog.Consumers.Query": {
      "capabilities": { "call": ["trellis.eventlog::events.read"] },
      "errors": [{ "type": "UnexpectedError" }, { "type": "ValidationError" }],
      "input": { "schema": "EventLogConsumersQueryRequest" },
      "output": { "schema": "EventLogConsumersQueryResponse" },
      "subject": "rpc.v1.EventLog.Consumers.Query",
      "version": "v1",
    },
    "EventLog.Inspect": {
      "capabilities": { "call": ["trellis.eventlog::events.read"] },
      "errors": [{ "type": "UnexpectedError" }, { "type": "ValidationError" }, {
        "type": "NotFoundError",
      }],
      "input": { "schema": "EventLogInspectRequest" },
      "output": { "schema": "EventLogInspectResponse" },
      "subject": "rpc.v1.EventLog.Inspect",
      "version": "v1",
    },
    "EventLog.Metrics": {
      "capabilities": { "call": ["trellis.eventlog::events.read"] },
      "errors": [{ "type": "UnexpectedError" }, { "type": "ValidationError" }],
      "input": { "schema": "EventLogMetricsRequest" },
      "output": { "schema": "EventLogMetricsResponse" },
      "subject": "rpc.v1.EventLog.Metrics",
      "version": "v1",
    },
    "EventLog.Query": {
      "capabilities": { "call": ["trellis.eventlog::events.read"] },
      "errors": [{ "type": "UnexpectedError" }, { "type": "ValidationError" }],
      "input": { "schema": "EventLogQueryRequest" },
      "output": { "schema": "EventLogQueryResponse" },
      "subject": "rpc.v1.EventLog.Query",
      "version": "v1",
    },
  },
  "schemas": {
    "EventConsumerStatusRow": {
      "properties": {
        "ackPending": { "type": "integer" },
        "ackWaitMs": { "type": "integer" },
        "concurrency": { "type": "integer" },
        "consumerName": { "type": "string" },
        "contractId": { "type": "string" },
        "deploymentId": { "type": "string" },
        "filterSubjects": { "items": { "type": "string" }, "type": "array" },
        "group": { "type": "string" },
        "maxDeliver": { "type": "integer" },
        "oldestPendingAt": { "type": "string" },
        "oldestPendingEventId": { "type": "string" },
        "pending": { "type": "integer" },
        "redelivered": { "type": "integer" },
        "status": {
          "anyOf": [
            { "const": "current" },
            { "const": "processing" },
            { "const": "behind" },
            { "const": "saturated" },
            { "const": "inactive" },
            { "const": "failing" },
            { "const": "missing" },
            { "const": "orphaned" },
          ],
        },
        "stream": { "type": "string" },
        "waitingPulls": { "type": "integer" },
      },
      "required": [
        "stream",
        "consumerName",
        "filterSubjects",
        "status",
        "pending",
        "ackPending",
        "waitingPulls",
      ],
      "type": "object",
    },
    "EventLogConsumersInspectRequest": {
      "properties": {
        "consumerName": { "type": "string" },
        "stream": { "type": "string" },
      },
      "required": ["consumerName"],
      "type": "object",
    },
    "EventLogConsumersInspectResponse": {
      "additionalProperties": true,
      "type": "object",
    },
    "EventLogConsumersQueryRequest": {
      "properties": {
        "contractId": { "type": "string" },
        "deploymentId": { "type": "string" },
        "limit": { "maximum": 500, "minimum": 1, "type": "integer" },
        "offset": { "minimum": 0, "type": "integer" },
        "ownerContractId": { "type": "string" },
        "status": {
          "items": {
            "anyOf": [
              { "const": "current" },
              { "const": "processing" },
              { "const": "behind" },
              { "const": "saturated" },
              { "const": "inactive" },
              { "const": "failing" },
              { "const": "missing" },
              { "const": "orphaned" },
            ],
          },
          "type": "array",
        },
        "subject": { "type": "string" },
      },
      "required": ["limit"],
      "type": "object",
    },
    "EventLogConsumersQueryResponse": {
      "properties": {
        "consumers": {
          "items": { "schema": "EventConsumerStatusRow" },
          "type": "array",
        },
        "limit": { "type": "integer" },
        "offset": { "type": "integer" },
        "total": { "type": "integer" },
      },
      "required": ["consumers", "total", "offset", "limit"],
      "type": "object",
    },
    "EventLogInspectRequest": {
      "properties": {
        "eventId": { "type": "string" },
        "streamSequence": { "type": "integer" },
      },
      "type": "object",
    },
    "EventLogInspectResponse": {
      "additionalProperties": true,
      "type": "object",
    },
    "EventLogMetricsRequest": {
      "properties": {
        "window": {
          "anyOf": [{ "const": "15m" }, { "const": "1h" }, { "const": "6h" }, {
            "const": "24h",
          }, { "const": "7d" }],
        },
      },
      "type": "object",
    },
    "EventLogMetricsResponse": {
      "properties": {
        "buckets": {
          "items": { "additionalProperties": true, "type": "object" },
          "type": "array",
        },
        "summary": {
          "properties": {
            "byResolution": {
              "properties": {
                "malformed": { "type": "integer" },
                "resolved": { "type": "integer" },
                "unresolved": { "type": "integer" },
              },
              "type": "object",
            },
            "byVerificationStatus": {
              "properties": {
                "auth-unavailable": { "type": "integer" },
                "invalid-signature": { "type": "integer" },
                "missing-proof": { "type": "integer" },
                "missing-session": { "type": "integer" },
                "outside-session-window": { "type": "integer" },
                "subject-denied": { "type": "integer" },
                "verified": { "type": "integer" },
              },
              "type": "object",
            },
            "eventTypes": {
              "items": {
                "properties": {
                  "count": { "type": "integer" },
                  "ownerContractId": { "type": "string" },
                  "ownerEventName": { "type": "string" },
                },
                "required": ["ownerContractId", "ownerEventName", "count"],
                "type": "object",
              },
              "type": "array",
            },
            "payloadSizeBytes": { "type": "integer" },
            "total": { "type": "integer" },
            "uniqueSubjects": { "type": "integer" },
          },
          "required": [
            "total",
            "uniqueSubjects",
            "payloadSizeBytes",
            "byResolution",
            "byVerificationStatus",
            "eventTypes",
          ],
          "type": "object",
        },
      },
      "required": ["summary", "buckets"],
      "type": "object",
    },
    "EventLogQueryRequest": {
      "properties": {
        "consumerDeploymentId": { "type": "string" },
        "consumerName": { "type": "string" },
        "excludeEventTypes": {
          "items": {
            "properties": {
              "ownerContractId": { "type": "string" },
              "ownerEventName": { "type": "string" },
            },
            "required": ["ownerContractId", "ownerEventName"],
            "type": "object",
          },
          "type": "array",
        },
        "includeEventTypes": {
          "items": {
            "properties": {
              "ownerContractId": { "type": "string" },
              "ownerEventName": { "type": "string" },
            },
            "required": ["ownerContractId", "ownerEventName"],
            "type": "object",
          },
          "type": "array",
        },
        "limit": { "maximum": 500, "minimum": 1, "type": "integer" },
        "offset": { "minimum": 0, "type": "integer" },
        "ownerContractId": { "type": "string" },
        "ownerEventName": { "type": "string" },
        "publisherContractId": { "type": "string" },
        "publisherDeploymentId": { "type": "string" },
        "resolution": {
          "items": {
            "anyOf": [{ "const": "resolved" }, { "const": "unresolved" }, {
              "const": "malformed",
            }],
          },
          "type": "array",
        },
        "search": { "type": "string" },
        "sort": { "additionalProperties": true, "type": "object" },
        "subject": { "type": "string" },
        "verificationStatus": {
          "items": {
            "anyOf": [
              { "const": "verified" },
              { "const": "missing-proof" },
              { "const": "invalid-signature" },
              { "const": "missing-session" },
              { "const": "subject-denied" },
              { "const": "outside-session-window" },
              { "const": "auth-unavailable" },
            ],
          },
          "type": "array",
        },
        "window": {
          "anyOf": [{ "const": "15m" }, { "const": "1h" }, { "const": "6h" }, {
            "const": "24h",
          }, { "const": "7d" }],
        },
      },
      "required": ["limit"],
      "type": "object",
    },
    "EventLogQueryResponse": {
      "properties": {
        "events": { "items": { "schema": "EventLogRow" }, "type": "array" },
        "limit": { "type": "integer" },
        "offset": { "type": "integer" },
        "total": { "type": "integer" },
      },
      "required": ["events", "total", "offset", "limit"],
      "type": "object",
    },
    "EventLogRow": {
      "properties": {
        "eventId": { "type": "string" },
        "eventTime": { "type": "string" },
        "headerCount": { "type": "integer" },
        "ownerContractId": { "type": "string" },
        "ownerEventName": { "type": "string" },
        "payloadSizeBytes": { "type": "integer" },
        "publisherContractDigest": { "type": "string" },
        "publisherContractId": { "type": "string" },
        "publisherDeploymentId": { "type": "string" },
        "publisherInstanceId": { "type": "string" },
        "publisherKind": {
          "anyOf": [{ "const": "service" }, { "const": "device" }, {
            "const": "user",
          }],
        },
        "resolution": {
          "anyOf": [{ "const": "resolved" }, { "const": "unresolved" }, {
            "const": "malformed",
          }],
        },
        "streamSequence": { "type": "integer" },
        "subject": { "type": "string" },
        "traceId": { "type": "string" },
        "verificationStatus": {
          "anyOf": [
            { "const": "verified" },
            { "const": "missing-proof" },
            { "const": "invalid-signature" },
            { "const": "missing-session" },
            { "const": "subject-denied" },
            { "const": "outside-session-window" },
            { "const": "auth-unavailable" },
          ],
        },
      },
      "required": [
        "eventId",
        "eventTime",
        "streamSequence",
        "subject",
        "resolution",
        "verificationStatus",
        "payloadSizeBytes",
        "headerCount",
      ],
      "type": "object",
    },
    "EventLogWatchFrame": { "additionalProperties": true, "type": "object" },
    "EventLogWatchRequest": { "additionalProperties": true, "type": "object" },
    "NotFoundErrorData": {
      "additionalProperties": true,
      "properties": {
        "context": { "additionalProperties": true, "type": "object" },
        "id": { "type": "string" },
        "message": { "type": "string" },
        "type": { "const": "NotFoundError" },
      },
      "required": ["type", "message", "id"],
      "type": "object",
    },
  },
  "uses": {
    "required": {
      "auth": {
        "contract": "trellis.auth@v1",
        "rpc": { "call": ["Auth.EventConsumers.List"] },
      },
      "health": {
        "contract": "trellis.health@v1",
        "events": { "publish": ["Health.Heartbeat"] },
      },
    },
  },
} as TrellisContractV1;

function assertSelectedKeysExist(
  kind: "rpc" | "operations" | "events" | "feeds",
  keys: readonly string[] | undefined,
  api: Record<string, unknown>,
) {
  if (!keys) {
    return;
  }

  for (const key of keys) {
    if (!Object.hasOwn(api, key)) {
      throw new Error(
        `Contract '${CONTRACT_ID}' does not expose ${kind} key '${key}'`,
      );
    }
  }
}

function assertValidUseSpec(spec: UseSpec<typeof API.owned>) {
  assertSelectedKeysExist("rpc", spec.rpc?.call, API.owned.rpc);
  assertSelectedKeysExist(
    "operations",
    spec.operations?.call,
    API.owned.operations,
  );
  assertSelectedKeysExist("events", spec.events?.publish, API.owned.events);
  assertSelectedKeysExist("events", spec.events?.subscribe, API.owned.events);
  assertSelectedKeysExist("feeds", spec.feeds?.subscribe, API.owned.feeds);
}

export const sdk: SdkContractModule<typeof CONTRACT_ID, typeof API.owned> = {
  CONTRACT_ID,
  CONTRACT_DIGEST,
  CONTRACT,
  API,
  use: <const TSpec extends UseSpec<typeof API.owned>>(spec: TSpec) => {
    assertValidUseSpec(spec);

    const dependencyUse = {
      contract: CONTRACT_ID,
      ...(spec.rpc?.call ? { rpc: { call: [...spec.rpc.call] } } : {}),
      ...(spec.operations?.call
        ? { operations: { call: [...spec.operations.call] } }
        : {}),
      ...((spec.events?.publish || spec.events?.subscribe)
        ? {
          events: {
            ...(spec.events.publish
              ? { publish: [...spec.events.publish] }
              : {}),
            ...(spec.events.subscribe
              ? { subscribe: [...spec.events.subscribe] }
              : {}),
          },
        }
        : {}),
      ...(spec.feeds?.subscribe
        ? { feeds: { subscribe: [...spec.feeds.subscribe] } }
        : {}),
    };

    Object.defineProperty(dependencyUse, CONTRACT_MODULE_METADATA, {
      value: sdk,
      enumerable: false,
    });

    return dependencyUse as ContractDependencyUse<
      typeof CONTRACT_ID,
      typeof API.owned,
      TSpec
    >;
  },
};

export const use = sdk.use;
