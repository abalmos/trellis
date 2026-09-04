// Generated from ./rust/crates/eventlog-runtime/.trellis/artifacts/apis/trellis.eventlog@v1.json
export const EventLogConsumersInspectRequestSchema = {
  "properties": {
    "consumerName": { "type": "string" },
    "stream": { "type": "string" },
  },
  "required": ["consumerName"],
  "type": "object",
} as const;

export const EventLogConsumersInspectResponseSchema = {
  "properties": {},
  "type": "object",
} as const;

export const EventLogConsumersQueryRequestSchema = {
  "properties": {
    "contractId": { "type": "string" },
    "deploymentId": { "type": "string" },
    "limit": { "maximum": 500, "minimum": 1, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
    "ownerContractId": { "type": "string" },
    "status": {
      "items": {
        "anyOf": [
          { "const": "current", "type": "string" },
          { "const": "processing", "type": "string" },
          { "const": "behind", "type": "string" },
          { "const": "saturated", "type": "string" },
          { "const": "inactive", "type": "string" },
          { "const": "failing", "type": "string" },
          { "const": "missing", "type": "string" },
          { "const": "orphaned", "type": "string" },
        ],
      },
      "type": "array",
    },
    "subject": { "type": "string" },
  },
  "required": ["limit"],
  "type": "object",
} as const;

export const EventLogConsumersQueryResponseSchema = {
  "properties": {
    "consumers": {
      "items": {
        "properties": {
          "ackPending": { "type": "integer" },
          "ackWaitMs": { "type": "integer" },
          "consumerName": { "type": "string" },
          "contractId": { "type": "string" },
          "deploymentId": { "type": "string" },
          "filterSubjects": { "items": { "type": "string" }, "type": "array" },
          "group": { "type": "string" },
          "managedBy": {
            "anyOf": [{ "const": "authority", "type": "string" }, {
              "const": "platform",
              "type": "string",
            }, { "const": "external", "type": "string" }],
          },
          "maxDeliver": { "type": "integer" },
          "oldestPendingAt": { "type": "string" },
          "oldestPendingEventId": { "type": "string" },
          "pending": { "type": "integer" },
          "redelivered": { "type": "integer" },
          "status": {
            "anyOf": [
              { "const": "current", "type": "string" },
              { "const": "processing", "type": "string" },
              { "const": "behind", "type": "string" },
              { "const": "saturated", "type": "string" },
              { "const": "inactive", "type": "string" },
              { "const": "failing", "type": "string" },
              { "const": "missing", "type": "string" },
              { "const": "orphaned", "type": "string" },
            ],
          },
          "stream": { "type": "string" },
          "waitingPulls": { "type": "integer" },
        },
        "required": [
          "ackPending",
          "consumerName",
          "filterSubjects",
          "pending",
          "status",
          "stream",
          "waitingPulls",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "limit": { "type": "integer" },
    "offset": { "type": "integer" },
    "total": { "type": "integer" },
  },
  "required": ["consumers", "limit", "offset", "total"],
  "type": "object",
} as const;

export const EventLogInspectRequestSchema = {
  "properties": {
    "eventId": { "type": "string" },
    "streamSequence": { "type": "integer" },
  },
  "type": "object",
} as const;

export const EventLogInspectResponseSchema = {
  "properties": {},
  "type": "object",
} as const;

export const EventLogMetricsRequestSchema = {
  "properties": {
    "window": {
      "anyOf": [
        { "const": "15m", "type": "string" },
        { "const": "1h", "type": "string" },
        { "const": "6h", "type": "string" },
        { "const": "24h", "type": "string" },
        { "const": "7d", "type": "string" },
      ],
    },
  },
  "type": "object",
} as const;

export const EventLogMetricsResponseSchema = {
  "properties": {
    "buckets": {
      "items": {
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
          "integrityExceptions": { "type": "integer" },
          "payloadSizeBytes": { "type": "integer" },
          "start": { "type": "string" },
          "total": { "type": "integer" },
        },
        "required": [
          "byResolution",
          "byVerificationStatus",
          "integrityExceptions",
          "payloadSizeBytes",
          "start",
          "total",
        ],
        "type": "object",
      },
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
            "required": ["count", "ownerContractId", "ownerEventName"],
            "type": "object",
          },
          "type": "array",
        },
        "integrityExceptions": { "type": "integer" },
        "payloadSizeBytes": { "type": "integer" },
        "total": { "type": "integer" },
        "uniqueSubjects": { "type": "integer" },
      },
      "required": [
        "byResolution",
        "byVerificationStatus",
        "eventTypes",
        "integrityExceptions",
        "payloadSizeBytes",
        "total",
        "uniqueSubjects",
      ],
      "type": "object",
    },
  },
  "required": ["buckets", "summary"],
  "type": "object",
} as const;

export const EventLogQueryRequestSchema = {
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
    "integrityExceptionOnly": { "type": "boolean" },
    "limit": { "maximum": 500, "minimum": 1, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
    "ownerContractId": { "type": "string" },
    "ownerEventName": { "type": "string" },
    "publisherDeploymentId": { "type": "string" },
    "publisherParticipantId": { "type": "string" },
    "resolution": {
      "items": {
        "anyOf": [{ "const": "resolved", "type": "string" }, {
          "const": "unresolved",
          "type": "string",
        }, { "const": "malformed", "type": "string" }],
      },
      "type": "array",
    },
    "search": { "type": "string" },
    "sort": { "properties": {}, "type": "object" },
    "subject": { "type": "string" },
    "verificationStatus": {
      "items": { "const": "verified", "type": "string" },
      "type": "array",
    },
    "window": {
      "anyOf": [
        { "const": "15m", "type": "string" },
        { "const": "1h", "type": "string" },
        { "const": "6h", "type": "string" },
        { "const": "24h", "type": "string" },
        { "const": "7d", "type": "string" },
      ],
    },
  },
  "required": ["limit"],
  "type": "object",
} as const;

export const EventLogQueryResponseSchema = {
  "properties": {
    "events": {
      "items": {
        "properties": {
          "eventId": { "type": "string" },
          "eventTime": { "type": "string" },
          "headerCount": { "type": "integer" },
          "ownerContractId": { "type": "string" },
          "ownerEventName": { "type": "string" },
          "payloadSizeBytes": { "type": "integer" },
          "publisherDeploymentId": { "type": "string" },
          "publisherInstanceId": { "type": "string" },
          "publisherKind": {
            "anyOf": [{ "const": "service", "type": "string" }, {
              "const": "device",
              "type": "string",
            }, { "const": "user", "type": "string" }],
          },
          "publisherParticipantDigest": { "type": "string" },
          "publisherParticipantId": { "type": "string" },
          "resolution": {
            "anyOf": [{ "const": "resolved", "type": "string" }, {
              "const": "unresolved",
              "type": "string",
            }, { "const": "malformed", "type": "string" }],
          },
          "streamSequence": { "type": "integer" },
          "subject": { "type": "string" },
          "traceId": { "type": "string" },
          "verificationStatus": { "const": "verified", "type": "string" },
        },
        "required": [
          "eventId",
          "eventTime",
          "headerCount",
          "payloadSizeBytes",
          "resolution",
          "streamSequence",
          "subject",
          "verificationStatus",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "limit": { "type": "integer" },
    "offset": { "type": "integer" },
    "total": { "type": "integer" },
  },
  "required": ["events", "limit", "offset", "total"],
  "type": "object",
} as const;

export const EventLogWatchFrameSchema = {
  "properties": {},
  "type": "object",
} as const;

export const EventLogWatchRequestSchema = {
  "properties": {},
  "type": "object",
} as const;

export const NotFoundErrorDataSchema = {
  "properties": {
    "context": { "properties": {}, "type": "object" },
    "id": { "type": "string" },
    "message": { "type": "string" },
    "type": { "const": "NotFoundError", "type": "string" },
  },
  "required": ["id", "message", "type"],
  "type": "object",
} as const;
