// Generated from ./generated/contracts/manifests/trellis.eventlog@v1.json
export const EventLogConsumersInspectRequestSchema = {
  "additionalProperties": false,
  "properties": {
    "consumerName": { "type": "string" },
    "stream": { "type": "string" },
  },
  "required": ["consumerName"],
  "type": "object",
} as const;

export const EventLogConsumersInspectResponseSchema = {
  "additionalProperties": true,
  "type": "object",
} as const;

export const EventLogConsumersQueryRequestSchema = {
  "additionalProperties": false,
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
} as const;

export const EventLogConsumersQueryResponseSchema = {
  "additionalProperties": false,
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
} as const;

export const EventLogInspectRequestSchema = {
  "additionalProperties": false,
  "properties": {
    "eventId": { "type": "string" },
    "streamSequence": { "type": "integer" },
  },
  "type": "object",
} as const;

export const EventLogInspectResponseSchema = {
  "additionalProperties": true,
  "type": "object",
} as const;

export const EventLogMetricsRequestSchema = {
  "additionalProperties": false,
  "properties": {
    "window": {
      "anyOf": [{ "const": "15m" }, { "const": "1h" }, { "const": "6h" }, {
        "const": "24h",
      }, { "const": "7d" }],
    },
  },
  "type": "object",
} as const;

export const EventLogMetricsResponseSchema = {
  "additionalProperties": false,
  "properties": {
    "buckets": {
      "items": { "additionalProperties": true, "type": "object" },
      "type": "array",
    },
    "summary": {
      "additionalProperties": false,
      "properties": {
        "byResolution": {
          "additionalProperties": false,
          "properties": {
            "malformed": { "type": "integer" },
            "resolved": { "type": "integer" },
            "unresolved": { "type": "integer" },
          },
          "type": "object",
        },
        "byVerificationStatus": {
          "additionalProperties": false,
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
            "additionalProperties": false,
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
} as const;

export const EventLogQueryRequestSchema = {
  "additionalProperties": false,
  "properties": {
    "consumerDeploymentId": { "type": "string" },
    "consumerName": { "type": "string" },
    "excludeEventTypes": {
      "items": {
        "additionalProperties": false,
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
        "additionalProperties": false,
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
} as const;

export const EventLogQueryResponseSchema = {
  "additionalProperties": false,
  "properties": {
    "events": { "items": { "schema": "EventLogRow" }, "type": "array" },
    "limit": { "type": "integer" },
    "offset": { "type": "integer" },
    "total": { "type": "integer" },
  },
  "required": ["events", "total", "offset", "limit"],
  "type": "object",
} as const;

export const EventLogWatchFrameSchema = {
  "additionalProperties": true,
  "type": "object",
} as const;

export const EventLogWatchRequestSchema = {
  "additionalProperties": true,
  "type": "object",
} as const;

export const NotFoundErrorDataSchema = {
  "additionalProperties": true,
  "properties": {
    "context": { "additionalProperties": true, "type": "object" },
    "id": { "type": "string" },
    "message": { "type": "string" },
    "type": { "const": "NotFoundError" },
  },
  "required": ["type", "message", "id"],
  "type": "object",
} as const;
