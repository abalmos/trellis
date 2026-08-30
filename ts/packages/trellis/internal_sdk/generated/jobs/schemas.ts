// Generated from ./rust/crates/jobs-runtime/.trellis/generated/protocol/apis/trellis.jobs@v1.json
export const JobsCancelRequestSchema = {
  "description":
    "Jobs admin ids are globally addressable; callers identify jobs by id only.",
  "properties": {
    "id": { "minLength": 1, "type": "string" },
    "reason": { "minLength": 1, "type": "string" },
  },
  "required": ["id"],
  "type": "object",
} as const;

export const JobsCancelResponseSchema = {
  "properties": {
    "job": {
      "properties": {
        "completedAt": { "format": "date-time", "type": "string" },
        "concurrency": {
          "properties": {
            "heartbeatAt": { "format": "date-time", "type": "string" },
            "key": { "minLength": 1, "type": "string" },
            "keyHash": { "minLength": 1, "type": "string" },
            "leaseExpiresAt": { "format": "date-time", "type": "string" },
            "staleTakeoverCount": { "minimum": 0, "type": "integer" },
          },
          "required": ["key", "keyHash"],
          "type": "object",
        },
        "context": {
          "properties": {
            "requestId": { "minLength": 1, "type": "string" },
            "traceId": { "pattern": "^[0-9a-f]{32}$", "type": "string" },
            "traceparent": {
              "pattern": "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
              "type": "string",
            },
            "tracestate": { "minLength": 1, "type": "string" },
          },
          "required": ["requestId", "traceId", "traceparent"],
          "type": "object",
        },
        "createdAt": { "format": "date-time", "type": "string" },
        "deadline": { "format": "date-time", "type": "string" },
        "errorDetail": {
          "properties": {
            "causes": { "items": { "type": "object" }, "type": "array" },
            "fingerprint": { "minLength": 1, "type": "string" },
            "firstSeen": { "format": "date-time", "type": "string" },
            "message": { "type": "string" },
            "occurrenceCount": { "minimum": 0, "type": "integer" },
            "stack": { "type": "string" },
            "type": { "type": "string" },
            "worker": {
              "properties": {
                "instanceId": { "type": "string" },
                "runtime": { "type": "string" },
                "service": { "type": "string" },
                "version": { "type": "string" },
              },
              "type": "object",
            },
          },
          "required": ["message", "fingerprint"],
          "type": "object",
        },
        "id": { "minLength": 1, "type": "string" },
        "lastError": { "type": "string" },
        "lineage": {
          "properties": {
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "relatedKeys": { "items": { "type": "string" }, "type": "array" },
            "rootJobId": { "type": "string" },
          },
          "type": "object",
        },
        "logs": {
          "items": {
            "properties": {
              "level": {
                "anyOf": [{ "const": "info", "type": "string" }, {
                  "const": "warn",
                  "type": "string",
                }, { "const": "error", "type": "string" }],
              },
              "message": { "type": "string" },
              "timestamp": { "format": "date-time", "type": "string" },
            },
            "required": ["timestamp", "level", "message"],
            "type": "object",
          },
          "type": "array",
        },
        "maxTries": { "minimum": 1, "type": "integer" },
        "payload": {},
        "progress": {
          "properties": {
            "current": { "minimum": 0, "type": "integer" },
            "message": { "type": "string" },
            "step": { "type": "string" },
            "total": { "minimum": 0, "type": "integer" },
          },
          "type": "object",
        },
        "queuePolicy": {
          "properties": {
            "existingJobId": { "minLength": 1, "type": "string" },
            "outcome": { "minLength": 1, "type": "string" },
            "reason": { "minLength": 1, "type": "string" },
            "replacedJobId": { "minLength": 1, "type": "string" },
          },
          "required": ["outcome"],
          "type": "object",
        },
        "result": {},
        "service": { "minLength": 1, "type": "string" },
        "startedAt": { "format": "date-time", "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "retry", "type": "string" },
            { "const": "completed", "type": "string" },
            { "const": "failed", "type": "string" },
            { "const": "cancelled", "type": "string" },
            { "const": "skipped", "type": "string" },
            { "const": "stale", "type": "string" },
            { "const": "expired", "type": "string" },
            { "const": "dead", "type": "string" },
            { "const": "dismissed", "type": "string" },
          ],
        },
        "tries": { "minimum": 0, "type": "integer" },
        "trigger": {
          "properties": {
            "id": { "type": "string" },
            "kind": {
              "anyOf": [
                { "const": "schedule", "type": "string" },
                { "const": "operation", "type": "string" },
                { "const": "rpc", "type": "string" },
                { "const": "event", "type": "string" },
                { "const": "manualReplay", "type": "string" },
                { "const": "serviceCode", "type": "string" },
                { "const": "parentJob", "type": "string" },
              ],
            },
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "requestId": { "type": "string" },
            "subject": { "type": "string" },
            "traceId": { "type": "string" },
          },
          "required": ["kind"],
          "type": "object",
        },
        "type": { "minLength": 1, "type": "string" },
        "updatedAt": { "format": "date-time", "type": "string" },
        "waitingOn": {
          "items": {
            "properties": {
              "id": { "minLength": 1, "type": "string" },
              "label": { "minLength": 1, "type": "string" },
              "startedAt": { "format": "date-time", "type": "string" },
              "target": {
                "properties": {
                  "id": { "minLength": 1, "type": "string" },
                  "key": { "minLength": 1, "type": "string" },
                  "kind": {
                    "anyOf": [{ "const": "job", "type": "string" }, {
                      "const": "operation",
                      "type": "string",
                    }, { "const": "external", "type": "string" }],
                  },
                  "label": { "minLength": 1, "type": "string" },
                  "operation": { "minLength": 1, "type": "string" },
                  "operationId": { "minLength": 1, "type": "string" },
                  "service": { "minLength": 1, "type": "string" },
                  "system": { "minLength": 1, "type": "string" },
                  "type": { "minLength": 1, "type": "string" },
                },
                "required": ["kind"],
                "type": "object",
              },
            },
            "required": ["id", "target", "startedAt"],
            "type": "object",
          },
          "type": "array",
        },
      },
      "required": [
        "id",
        "context",
        "service",
        "type",
        "state",
        "payload",
        "createdAt",
        "updatedAt",
        "tries",
        "maxTries",
      ],
      "type": "object",
    },
  },
  "required": ["job"],
  "type": "object",
} as const;

export const JobsDismissDLQRequestSchema = {
  "description":
    "Jobs admin ids are globally addressable; callers identify jobs by id only.",
  "properties": {
    "id": { "minLength": 1, "type": "string" },
    "reason": { "minLength": 1, "type": "string" },
  },
  "required": ["id"],
  "type": "object",
} as const;

export const JobsDismissDLQResponseSchema = {
  "properties": {
    "job": {
      "properties": {
        "completedAt": { "format": "date-time", "type": "string" },
        "concurrency": {
          "properties": {
            "heartbeatAt": { "format": "date-time", "type": "string" },
            "key": { "minLength": 1, "type": "string" },
            "keyHash": { "minLength": 1, "type": "string" },
            "leaseExpiresAt": { "format": "date-time", "type": "string" },
            "staleTakeoverCount": { "minimum": 0, "type": "integer" },
          },
          "required": ["key", "keyHash"],
          "type": "object",
        },
        "context": {
          "properties": {
            "requestId": { "minLength": 1, "type": "string" },
            "traceId": { "pattern": "^[0-9a-f]{32}$", "type": "string" },
            "traceparent": {
              "pattern": "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
              "type": "string",
            },
            "tracestate": { "minLength": 1, "type": "string" },
          },
          "required": ["requestId", "traceId", "traceparent"],
          "type": "object",
        },
        "createdAt": { "format": "date-time", "type": "string" },
        "deadline": { "format": "date-time", "type": "string" },
        "errorDetail": {
          "properties": {
            "causes": { "items": { "type": "object" }, "type": "array" },
            "fingerprint": { "minLength": 1, "type": "string" },
            "firstSeen": { "format": "date-time", "type": "string" },
            "message": { "type": "string" },
            "occurrenceCount": { "minimum": 0, "type": "integer" },
            "stack": { "type": "string" },
            "type": { "type": "string" },
            "worker": {
              "properties": {
                "instanceId": { "type": "string" },
                "runtime": { "type": "string" },
                "service": { "type": "string" },
                "version": { "type": "string" },
              },
              "type": "object",
            },
          },
          "required": ["message", "fingerprint"],
          "type": "object",
        },
        "id": { "minLength": 1, "type": "string" },
        "lastError": { "type": "string" },
        "lineage": {
          "properties": {
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "relatedKeys": { "items": { "type": "string" }, "type": "array" },
            "rootJobId": { "type": "string" },
          },
          "type": "object",
        },
        "logs": {
          "items": {
            "properties": {
              "level": {
                "anyOf": [{ "const": "info", "type": "string" }, {
                  "const": "warn",
                  "type": "string",
                }, { "const": "error", "type": "string" }],
              },
              "message": { "type": "string" },
              "timestamp": { "format": "date-time", "type": "string" },
            },
            "required": ["timestamp", "level", "message"],
            "type": "object",
          },
          "type": "array",
        },
        "maxTries": { "minimum": 1, "type": "integer" },
        "payload": {},
        "progress": {
          "properties": {
            "current": { "minimum": 0, "type": "integer" },
            "message": { "type": "string" },
            "step": { "type": "string" },
            "total": { "minimum": 0, "type": "integer" },
          },
          "type": "object",
        },
        "queuePolicy": {
          "properties": {
            "existingJobId": { "minLength": 1, "type": "string" },
            "outcome": { "minLength": 1, "type": "string" },
            "reason": { "minLength": 1, "type": "string" },
            "replacedJobId": { "minLength": 1, "type": "string" },
          },
          "required": ["outcome"],
          "type": "object",
        },
        "result": {},
        "service": { "minLength": 1, "type": "string" },
        "startedAt": { "format": "date-time", "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "retry", "type": "string" },
            { "const": "completed", "type": "string" },
            { "const": "failed", "type": "string" },
            { "const": "cancelled", "type": "string" },
            { "const": "skipped", "type": "string" },
            { "const": "stale", "type": "string" },
            { "const": "expired", "type": "string" },
            { "const": "dead", "type": "string" },
            { "const": "dismissed", "type": "string" },
          ],
        },
        "tries": { "minimum": 0, "type": "integer" },
        "trigger": {
          "properties": {
            "id": { "type": "string" },
            "kind": {
              "anyOf": [
                { "const": "schedule", "type": "string" },
                { "const": "operation", "type": "string" },
                { "const": "rpc", "type": "string" },
                { "const": "event", "type": "string" },
                { "const": "manualReplay", "type": "string" },
                { "const": "serviceCode", "type": "string" },
                { "const": "parentJob", "type": "string" },
              ],
            },
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "requestId": { "type": "string" },
            "subject": { "type": "string" },
            "traceId": { "type": "string" },
          },
          "required": ["kind"],
          "type": "object",
        },
        "type": { "minLength": 1, "type": "string" },
        "updatedAt": { "format": "date-time", "type": "string" },
        "waitingOn": {
          "items": {
            "properties": {
              "id": { "minLength": 1, "type": "string" },
              "label": { "minLength": 1, "type": "string" },
              "startedAt": { "format": "date-time", "type": "string" },
              "target": {
                "properties": {
                  "id": { "minLength": 1, "type": "string" },
                  "key": { "minLength": 1, "type": "string" },
                  "kind": {
                    "anyOf": [{ "const": "job", "type": "string" }, {
                      "const": "operation",
                      "type": "string",
                    }, { "const": "external", "type": "string" }],
                  },
                  "label": { "minLength": 1, "type": "string" },
                  "operation": { "minLength": 1, "type": "string" },
                  "operationId": { "minLength": 1, "type": "string" },
                  "service": { "minLength": 1, "type": "string" },
                  "system": { "minLength": 1, "type": "string" },
                  "type": { "minLength": 1, "type": "string" },
                },
                "required": ["kind"],
                "type": "object",
              },
            },
            "required": ["id", "target", "startedAt"],
            "type": "object",
          },
          "type": "array",
        },
      },
      "required": [
        "id",
        "context",
        "service",
        "type",
        "state",
        "payload",
        "createdAt",
        "updatedAt",
        "tries",
        "maxTries",
      ],
      "type": "object",
    },
  },
  "required": ["job"],
  "type": "object",
} as const;

export const JobsGetKeyRequestSchema = {
  "properties": {
    "key": { "minLength": 1, "type": "string" },
    "service": { "minLength": 1, "type": "string" },
    "type": { "minLength": 1, "type": "string" },
  },
  "required": ["service", "type", "key"],
  "type": "object",
} as const;

export const JobsGetKeyResponseSchema = {
  "properties": {
    "active": {
      "items": {
        "properties": {
          "heartbeatAgeMs": { "minimum": 0, "type": "integer" },
          "heartbeatAt": { "format": "date-time", "type": "string" },
          "instanceId": { "type": "string" },
          "jobId": { "minLength": 1, "type": "string" },
          "leaseExpiresAt": { "format": "date-time", "type": "string" },
          "startedAt": { "format": "date-time", "type": "string" },
        },
        "required": [
          "jobId",
          "instanceId",
          "startedAt",
          "heartbeatAt",
          "heartbeatAgeMs",
          "leaseExpiresAt",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "key": { "minLength": 1, "type": "string" },
    "keyHash": { "minLength": 1, "type": "string" },
    "latestPolicyReason": { "minLength": 1, "type": "string" },
    "queued": {
      "items": {
        "properties": {
          "createdAt": { "format": "date-time", "type": "string" },
          "jobId": { "minLength": 1, "type": "string" },
        },
        "required": ["jobId", "createdAt"],
        "type": "object",
      },
      "type": "array",
    },
    "queuedDepth": { "minimum": 0, "type": "integer" },
    "service": { "minLength": 1, "type": "string" },
    "staleTakeoverCount": { "minimum": 0, "type": "integer" },
    "type": { "minLength": 1, "type": "string" },
  },
  "required": [
    "service",
    "type",
    "key",
    "keyHash",
    "active",
    "queued",
    "queuedDepth",
    "staleTakeoverCount",
  ],
  "type": "object",
} as const;

export const JobsInspectRequestSchema = {
  "description":
    "Jobs admin ids are globally addressable; callers identify jobs by id only.",
  "properties": { "id": { "minLength": 1, "type": "string" } },
  "required": ["id"],
  "type": "object",
} as const;

export const JobsInspectResponseSchema = {
  "properties": {
    "attempts": {
      "items": {
        "properties": {
          "endedAt": { "format": "date-time", "type": "string" },
          "error": {
            "properties": {
              "causes": { "items": { "type": "object" }, "type": "array" },
              "fingerprint": { "minLength": 1, "type": "string" },
              "firstSeen": { "format": "date-time", "type": "string" },
              "message": { "type": "string" },
              "occurrenceCount": { "minimum": 0, "type": "integer" },
              "stack": { "type": "string" },
              "type": { "type": "string" },
              "worker": {
                "properties": {
                  "instanceId": { "type": "string" },
                  "runtime": { "type": "string" },
                  "service": { "type": "string" },
                  "version": { "type": "string" },
                },
                "type": "object",
              },
            },
            "required": ["message", "fingerprint"],
            "type": "object",
          },
          "startedAt": { "format": "date-time", "type": "string" },
          "state": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "retry", "type": "string" },
              { "const": "completed", "type": "string" },
              { "const": "failed", "type": "string" },
              { "const": "cancelled", "type": "string" },
              { "const": "skipped", "type": "string" },
              { "const": "stale", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "dead", "type": "string" },
              { "const": "dismissed", "type": "string" },
            ],
          },
          "try": { "minimum": 0, "type": "integer" },
        },
        "required": ["try", "startedAt"],
        "type": "object",
      },
      "type": "array",
    },
    "errors": {
      "items": {
        "properties": {
          "causes": { "items": { "type": "object" }, "type": "array" },
          "fingerprint": { "minLength": 1, "type": "string" },
          "firstSeen": { "format": "date-time", "type": "string" },
          "message": { "type": "string" },
          "occurrenceCount": { "minimum": 0, "type": "integer" },
          "stack": { "type": "string" },
          "type": { "type": "string" },
          "worker": {
            "properties": {
              "instanceId": { "type": "string" },
              "runtime": { "type": "string" },
              "service": { "type": "string" },
              "version": { "type": "string" },
            },
            "type": "object",
          },
        },
        "required": ["message", "fingerprint"],
        "type": "object",
      },
      "type": "array",
    },
    "job": {
      "properties": {
        "completedAt": { "format": "date-time", "type": "string" },
        "concurrency": {
          "properties": {
            "heartbeatAt": { "format": "date-time", "type": "string" },
            "key": { "minLength": 1, "type": "string" },
            "keyHash": { "minLength": 1, "type": "string" },
            "leaseExpiresAt": { "format": "date-time", "type": "string" },
            "staleTakeoverCount": { "minimum": 0, "type": "integer" },
          },
          "required": ["key", "keyHash"],
          "type": "object",
        },
        "context": {
          "properties": {
            "requestId": { "minLength": 1, "type": "string" },
            "traceId": { "pattern": "^[0-9a-f]{32}$", "type": "string" },
            "traceparent": {
              "pattern": "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
              "type": "string",
            },
            "tracestate": { "minLength": 1, "type": "string" },
          },
          "required": ["requestId", "traceId", "traceparent"],
          "type": "object",
        },
        "createdAt": { "format": "date-time", "type": "string" },
        "deadline": { "format": "date-time", "type": "string" },
        "errorDetail": {
          "properties": {
            "causes": { "items": { "type": "object" }, "type": "array" },
            "fingerprint": { "minLength": 1, "type": "string" },
            "firstSeen": { "format": "date-time", "type": "string" },
            "message": { "type": "string" },
            "occurrenceCount": { "minimum": 0, "type": "integer" },
            "stack": { "type": "string" },
            "type": { "type": "string" },
            "worker": {
              "properties": {
                "instanceId": { "type": "string" },
                "runtime": { "type": "string" },
                "service": { "type": "string" },
                "version": { "type": "string" },
              },
              "type": "object",
            },
          },
          "required": ["message", "fingerprint"],
          "type": "object",
        },
        "id": { "minLength": 1, "type": "string" },
        "lastError": { "type": "string" },
        "lineage": {
          "properties": {
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "relatedKeys": { "items": { "type": "string" }, "type": "array" },
            "rootJobId": { "type": "string" },
          },
          "type": "object",
        },
        "logs": {
          "items": {
            "properties": {
              "level": {
                "anyOf": [{ "const": "info", "type": "string" }, {
                  "const": "warn",
                  "type": "string",
                }, { "const": "error", "type": "string" }],
              },
              "message": { "type": "string" },
              "timestamp": { "format": "date-time", "type": "string" },
            },
            "required": ["timestamp", "level", "message"],
            "type": "object",
          },
          "type": "array",
        },
        "maxTries": { "minimum": 1, "type": "integer" },
        "payload": {},
        "progress": {
          "properties": {
            "current": { "minimum": 0, "type": "integer" },
            "message": { "type": "string" },
            "step": { "type": "string" },
            "total": { "minimum": 0, "type": "integer" },
          },
          "type": "object",
        },
        "queuePolicy": {
          "properties": {
            "existingJobId": { "minLength": 1, "type": "string" },
            "outcome": { "minLength": 1, "type": "string" },
            "reason": { "minLength": 1, "type": "string" },
            "replacedJobId": { "minLength": 1, "type": "string" },
          },
          "required": ["outcome"],
          "type": "object",
        },
        "result": {},
        "service": { "minLength": 1, "type": "string" },
        "startedAt": { "format": "date-time", "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "retry", "type": "string" },
            { "const": "completed", "type": "string" },
            { "const": "failed", "type": "string" },
            { "const": "cancelled", "type": "string" },
            { "const": "skipped", "type": "string" },
            { "const": "stale", "type": "string" },
            { "const": "expired", "type": "string" },
            { "const": "dead", "type": "string" },
            { "const": "dismissed", "type": "string" },
          ],
        },
        "tries": { "minimum": 0, "type": "integer" },
        "trigger": {
          "properties": {
            "id": { "type": "string" },
            "kind": {
              "anyOf": [
                { "const": "schedule", "type": "string" },
                { "const": "operation", "type": "string" },
                { "const": "rpc", "type": "string" },
                { "const": "event", "type": "string" },
                { "const": "manualReplay", "type": "string" },
                { "const": "serviceCode", "type": "string" },
                { "const": "parentJob", "type": "string" },
              ],
            },
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "requestId": { "type": "string" },
            "subject": { "type": "string" },
            "traceId": { "type": "string" },
          },
          "required": ["kind"],
          "type": "object",
        },
        "type": { "minLength": 1, "type": "string" },
        "updatedAt": { "format": "date-time", "type": "string" },
        "waitingOn": {
          "items": {
            "properties": {
              "id": { "minLength": 1, "type": "string" },
              "label": { "minLength": 1, "type": "string" },
              "startedAt": { "format": "date-time", "type": "string" },
              "target": {
                "properties": {
                  "id": { "minLength": 1, "type": "string" },
                  "key": { "minLength": 1, "type": "string" },
                  "kind": {
                    "anyOf": [{ "const": "job", "type": "string" }, {
                      "const": "operation",
                      "type": "string",
                    }, { "const": "external", "type": "string" }],
                  },
                  "label": { "minLength": 1, "type": "string" },
                  "operation": { "minLength": 1, "type": "string" },
                  "operationId": { "minLength": 1, "type": "string" },
                  "service": { "minLength": 1, "type": "string" },
                  "system": { "minLength": 1, "type": "string" },
                  "type": { "minLength": 1, "type": "string" },
                },
                "required": ["kind"],
                "type": "object",
              },
            },
            "required": ["id", "target", "startedAt"],
            "type": "object",
          },
          "type": "array",
        },
      },
      "required": [
        "id",
        "context",
        "service",
        "type",
        "state",
        "payload",
        "createdAt",
        "updatedAt",
        "tries",
        "maxTries",
      ],
      "type": "object",
    },
    "lineage": {
      "properties": {
        "operationId": { "type": "string" },
        "parentJobId": { "type": "string" },
        "relatedKeys": { "items": { "type": "string" }, "type": "array" },
        "rootJobId": { "type": "string" },
      },
      "type": "object",
    },
    "related": {
      "items": {
        "properties": {
          "completedAt": { "format": "date-time", "type": "string" },
          "context": {
            "properties": {
              "requestId": { "minLength": 1, "type": "string" },
              "traceId": { "pattern": "^[0-9a-f]{32}$", "type": "string" },
              "traceparent": {
                "pattern":
                  "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
                "type": "string",
              },
              "tracestate": { "minLength": 1, "type": "string" },
            },
            "required": ["requestId", "traceId", "traceparent"],
            "type": "object",
          },
          "createdAt": { "format": "date-time", "type": "string" },
          "errorFingerprint": { "type": "string" },
          "id": { "minLength": 1, "type": "string" },
          "lastError": { "type": "string" },
          "lineage": {
            "properties": {
              "operationId": { "type": "string" },
              "parentJobId": { "type": "string" },
              "relatedKeys": { "items": { "type": "string" }, "type": "array" },
              "rootJobId": { "type": "string" },
            },
            "type": "object",
          },
          "matchedBy": {
            "anyOf": [
              { "const": "trace", "type": "string" },
              { "const": "parent", "type": "string" },
              { "const": "root", "type": "string" },
              { "const": "operation", "type": "string" },
              { "const": "concurrency", "type": "string" },
              { "const": "wait", "type": "string" },
            ],
          },
          "maxTries": { "minimum": 1, "type": "integer" },
          "progress": {
            "properties": {
              "current": { "minimum": 0, "type": "integer" },
              "message": { "type": "string" },
              "step": { "type": "string" },
              "total": { "minimum": 0, "type": "integer" },
            },
            "type": "object",
          },
          "queueAgeMs": { "minimum": 0, "type": "integer" },
          "queueKey": { "type": "string" },
          "runtimeBand": { "type": "string" },
          "runtimeMs": { "minimum": 0, "type": "integer" },
          "service": { "minLength": 1, "type": "string" },
          "startedAt": { "format": "date-time", "type": "string" },
          "state": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "retry", "type": "string" },
              { "const": "completed", "type": "string" },
              { "const": "failed", "type": "string" },
              { "const": "cancelled", "type": "string" },
              { "const": "skipped", "type": "string" },
              { "const": "stale", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "dead", "type": "string" },
              { "const": "dismissed", "type": "string" },
            ],
          },
          "tries": { "minimum": 0, "type": "integer" },
          "trigger": {
            "properties": {
              "id": { "type": "string" },
              "kind": {
                "anyOf": [
                  { "const": "schedule", "type": "string" },
                  { "const": "operation", "type": "string" },
                  { "const": "rpc", "type": "string" },
                  { "const": "event", "type": "string" },
                  { "const": "manualReplay", "type": "string" },
                  { "const": "serviceCode", "type": "string" },
                  { "const": "parentJob", "type": "string" },
                ],
              },
              "operationId": { "type": "string" },
              "parentJobId": { "type": "string" },
              "requestId": { "type": "string" },
              "subject": { "type": "string" },
              "traceId": { "type": "string" },
            },
            "required": ["kind"],
            "type": "object",
          },
          "type": { "minLength": 1, "type": "string" },
          "updatedAt": { "format": "date-time", "type": "string" },
          "waitingOn": {
            "items": {
              "properties": {
                "id": { "minLength": 1, "type": "string" },
                "label": { "minLength": 1, "type": "string" },
                "startedAt": { "format": "date-time", "type": "string" },
                "target": {
                  "properties": {
                    "id": { "minLength": 1, "type": "string" },
                    "key": { "minLength": 1, "type": "string" },
                    "kind": {
                      "anyOf": [{ "const": "job", "type": "string" }, {
                        "const": "operation",
                        "type": "string",
                      }, { "const": "external", "type": "string" }],
                    },
                    "label": { "minLength": 1, "type": "string" },
                    "operation": { "minLength": 1, "type": "string" },
                    "operationId": { "minLength": 1, "type": "string" },
                    "service": { "minLength": 1, "type": "string" },
                    "system": { "minLength": 1, "type": "string" },
                    "type": { "minLength": 1, "type": "string" },
                  },
                  "required": ["kind"],
                  "type": "object",
                },
              },
              "required": ["id", "target", "startedAt"],
              "type": "object",
            },
            "type": "array",
          },
        },
        "required": [
          "id",
          "service",
          "type",
          "state",
          "createdAt",
          "updatedAt",
          "tries",
          "maxTries",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "timeline": {
      "items": {
        "properties": {
          "error": { "type": "string" },
          "errorDetail": {
            "properties": {
              "causes": { "items": { "type": "object" }, "type": "array" },
              "fingerprint": { "minLength": 1, "type": "string" },
              "firstSeen": { "format": "date-time", "type": "string" },
              "message": { "type": "string" },
              "occurrenceCount": { "minimum": 0, "type": "integer" },
              "stack": { "type": "string" },
              "type": { "type": "string" },
              "worker": {
                "properties": {
                  "instanceId": { "type": "string" },
                  "runtime": { "type": "string" },
                  "service": { "type": "string" },
                  "version": { "type": "string" },
                },
                "type": "object",
              },
            },
            "required": ["message", "fingerprint"],
            "type": "object",
          },
          "logs": {
            "items": {
              "properties": {
                "level": {
                  "anyOf": [{ "const": "info", "type": "string" }, {
                    "const": "warn",
                    "type": "string",
                  }, { "const": "error", "type": "string" }],
                },
                "message": { "type": "string" },
                "timestamp": { "format": "date-time", "type": "string" },
              },
              "required": ["timestamp", "level", "message"],
              "type": "object",
            },
            "type": "array",
          },
          "message": { "type": "string" },
          "previousState": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "retry", "type": "string" },
              { "const": "completed", "type": "string" },
              { "const": "failed", "type": "string" },
              { "const": "cancelled", "type": "string" },
              { "const": "skipped", "type": "string" },
              { "const": "stale", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "dead", "type": "string" },
              { "const": "dismissed", "type": "string" },
            ],
          },
          "progress": {
            "properties": {
              "current": { "minimum": 0, "type": "integer" },
              "message": { "type": "string" },
              "step": { "type": "string" },
              "total": { "minimum": 0, "type": "integer" },
            },
            "type": "object",
          },
          "projected": { "type": "boolean" },
          "rawEvent": {},
          "reason": { "type": "string" },
          "sequence": { "minimum": 0, "type": "integer" },
          "state": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "retry", "type": "string" },
              { "const": "completed", "type": "string" },
              { "const": "failed", "type": "string" },
              { "const": "cancelled", "type": "string" },
              { "const": "skipped", "type": "string" },
              { "const": "stale", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "dead", "type": "string" },
              { "const": "dismissed", "type": "string" },
            ],
          },
          "timestamp": { "format": "date-time", "type": "string" },
          "tries": { "minimum": 0, "type": "integer" },
          "type": { "minLength": 1, "type": "string" },
          "waitEdge": {
            "properties": {
              "id": { "minLength": 1, "type": "string" },
              "label": { "minLength": 1, "type": "string" },
              "startedAt": { "format": "date-time", "type": "string" },
              "target": {
                "properties": {
                  "id": { "minLength": 1, "type": "string" },
                  "key": { "minLength": 1, "type": "string" },
                  "kind": {
                    "anyOf": [{ "const": "job", "type": "string" }, {
                      "const": "operation",
                      "type": "string",
                    }, { "const": "external", "type": "string" }],
                  },
                  "label": { "minLength": 1, "type": "string" },
                  "operation": { "minLength": 1, "type": "string" },
                  "operationId": { "minLength": 1, "type": "string" },
                  "service": { "minLength": 1, "type": "string" },
                  "system": { "minLength": 1, "type": "string" },
                  "type": { "minLength": 1, "type": "string" },
                },
                "required": ["kind"],
                "type": "object",
              },
            },
            "required": ["id", "target", "startedAt"],
            "type": "object",
          },
          "workerInstanceId": { "type": "string" },
        },
        "required": ["sequence", "type", "state", "timestamp"],
        "type": "object",
      },
      "type": "array",
    },
    "trigger": {
      "properties": {
        "id": { "type": "string" },
        "kind": {
          "anyOf": [
            { "const": "schedule", "type": "string" },
            { "const": "operation", "type": "string" },
            { "const": "rpc", "type": "string" },
            { "const": "event", "type": "string" },
            { "const": "manualReplay", "type": "string" },
            { "const": "serviceCode", "type": "string" },
            { "const": "parentJob", "type": "string" },
          ],
        },
        "operationId": { "type": "string" },
        "parentJobId": { "type": "string" },
        "requestId": { "type": "string" },
        "subject": { "type": "string" },
        "traceId": { "type": "string" },
      },
      "required": ["kind"],
      "type": "object",
    },
  },
  "required": ["job", "timeline", "attempts", "related", "errors"],
  "type": "object",
} as const;

export const JobsListDLQRequestSchema = {
  "properties": {
    "limit": { "minimum": 1, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
    "service": { "minLength": 1, "type": "string" },
    "since": { "format": "date-time", "type": "string" },
    "type": { "minLength": 1, "type": "string" },
  },
  "required": ["limit"],
  "type": "object",
} as const;

export const JobsListDLQResponseSchema = {
  "properties": {
    "count": { "minimum": 0, "type": "integer" },
    "entries": {
      "items": {
        "properties": {
          "completedAt": { "format": "date-time", "type": "string" },
          "concurrency": {
            "properties": {
              "heartbeatAt": { "format": "date-time", "type": "string" },
              "key": { "minLength": 1, "type": "string" },
              "keyHash": { "minLength": 1, "type": "string" },
              "leaseExpiresAt": { "format": "date-time", "type": "string" },
              "staleTakeoverCount": { "minimum": 0, "type": "integer" },
            },
            "required": ["key", "keyHash"],
            "type": "object",
          },
          "context": {
            "properties": {
              "requestId": { "minLength": 1, "type": "string" },
              "traceId": { "pattern": "^[0-9a-f]{32}$", "type": "string" },
              "traceparent": {
                "pattern":
                  "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
                "type": "string",
              },
              "tracestate": { "minLength": 1, "type": "string" },
            },
            "required": ["requestId", "traceId", "traceparent"],
            "type": "object",
          },
          "createdAt": { "format": "date-time", "type": "string" },
          "deadline": { "format": "date-time", "type": "string" },
          "errorDetail": {
            "properties": {
              "causes": { "items": { "type": "object" }, "type": "array" },
              "fingerprint": { "minLength": 1, "type": "string" },
              "firstSeen": { "format": "date-time", "type": "string" },
              "message": { "type": "string" },
              "occurrenceCount": { "minimum": 0, "type": "integer" },
              "stack": { "type": "string" },
              "type": { "type": "string" },
              "worker": {
                "properties": {
                  "instanceId": { "type": "string" },
                  "runtime": { "type": "string" },
                  "service": { "type": "string" },
                  "version": { "type": "string" },
                },
                "type": "object",
              },
            },
            "required": ["message", "fingerprint"],
            "type": "object",
          },
          "id": { "minLength": 1, "type": "string" },
          "lastError": { "type": "string" },
          "lineage": {
            "properties": {
              "operationId": { "type": "string" },
              "parentJobId": { "type": "string" },
              "relatedKeys": { "items": { "type": "string" }, "type": "array" },
              "rootJobId": { "type": "string" },
            },
            "type": "object",
          },
          "logs": {
            "items": {
              "properties": {
                "level": {
                  "anyOf": [{ "const": "info", "type": "string" }, {
                    "const": "warn",
                    "type": "string",
                  }, { "const": "error", "type": "string" }],
                },
                "message": { "type": "string" },
                "timestamp": { "format": "date-time", "type": "string" },
              },
              "required": ["timestamp", "level", "message"],
              "type": "object",
            },
            "type": "array",
          },
          "maxTries": { "minimum": 1, "type": "integer" },
          "payload": {},
          "progress": {
            "properties": {
              "current": { "minimum": 0, "type": "integer" },
              "message": { "type": "string" },
              "step": { "type": "string" },
              "total": { "minimum": 0, "type": "integer" },
            },
            "type": "object",
          },
          "queuePolicy": {
            "properties": {
              "existingJobId": { "minLength": 1, "type": "string" },
              "outcome": { "minLength": 1, "type": "string" },
              "reason": { "minLength": 1, "type": "string" },
              "replacedJobId": { "minLength": 1, "type": "string" },
            },
            "required": ["outcome"],
            "type": "object",
          },
          "result": {},
          "service": { "minLength": 1, "type": "string" },
          "startedAt": { "format": "date-time", "type": "string" },
          "state": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "retry", "type": "string" },
              { "const": "completed", "type": "string" },
              { "const": "failed", "type": "string" },
              { "const": "cancelled", "type": "string" },
              { "const": "skipped", "type": "string" },
              { "const": "stale", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "dead", "type": "string" },
              { "const": "dismissed", "type": "string" },
            ],
          },
          "tries": { "minimum": 0, "type": "integer" },
          "trigger": {
            "properties": {
              "id": { "type": "string" },
              "kind": {
                "anyOf": [
                  { "const": "schedule", "type": "string" },
                  { "const": "operation", "type": "string" },
                  { "const": "rpc", "type": "string" },
                  { "const": "event", "type": "string" },
                  { "const": "manualReplay", "type": "string" },
                  { "const": "serviceCode", "type": "string" },
                  { "const": "parentJob", "type": "string" },
                ],
              },
              "operationId": { "type": "string" },
              "parentJobId": { "type": "string" },
              "requestId": { "type": "string" },
              "subject": { "type": "string" },
              "traceId": { "type": "string" },
            },
            "required": ["kind"],
            "type": "object",
          },
          "type": { "minLength": 1, "type": "string" },
          "updatedAt": { "format": "date-time", "type": "string" },
          "waitingOn": {
            "items": {
              "properties": {
                "id": { "minLength": 1, "type": "string" },
                "label": { "minLength": 1, "type": "string" },
                "startedAt": { "format": "date-time", "type": "string" },
                "target": {
                  "properties": {
                    "id": { "minLength": 1, "type": "string" },
                    "key": { "minLength": 1, "type": "string" },
                    "kind": {
                      "anyOf": [{ "const": "job", "type": "string" }, {
                        "const": "operation",
                        "type": "string",
                      }, { "const": "external", "type": "string" }],
                    },
                    "label": { "minLength": 1, "type": "string" },
                    "operation": { "minLength": 1, "type": "string" },
                    "operationId": { "minLength": 1, "type": "string" },
                    "service": { "minLength": 1, "type": "string" },
                    "system": { "minLength": 1, "type": "string" },
                    "type": { "minLength": 1, "type": "string" },
                  },
                  "required": ["kind"],
                  "type": "object",
                },
              },
              "required": ["id", "target", "startedAt"],
              "type": "object",
            },
            "type": "array",
          },
        },
        "required": [
          "id",
          "context",
          "service",
          "type",
          "state",
          "payload",
          "createdAt",
          "updatedAt",
          "tries",
          "maxTries",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "limit": { "minimum": 1, "type": "integer" },
    "nextOffset": { "minimum": 0, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
  },
  "required": ["entries", "count", "offset", "limit"],
  "type": "object",
} as const;

export const JobsListServicesRequestSchema = {
  "properties": {
    "limit": { "minimum": 1, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
  },
  "required": ["limit"],
  "type": "object",
} as const;

export const JobsListServicesResponseSchema = {
  "properties": {
    "count": { "minimum": 0, "type": "integer" },
    "entries": {
      "items": {
        "properties": {
          "healthy": { "type": "boolean" },
          "name": { "minLength": 1, "type": "string" },
          "workers": {
            "items": {
              "properties": {
                "concurrency": { "minimum": 1, "type": "integer" },
                "instanceId": { "minLength": 1, "type": "string" },
                "jobType": { "minLength": 1, "type": "string" },
                "service": { "minLength": 1, "type": "string" },
                "timestamp": { "format": "date-time", "type": "string" },
                "version": { "minLength": 1, "type": "string" },
              },
              "required": ["service", "jobType", "instanceId", "timestamp"],
              "type": "object",
            },
            "type": "array",
          },
        },
        "required": ["name", "healthy", "workers"],
        "type": "object",
      },
      "type": "array",
    },
    "limit": { "minimum": 1, "type": "integer" },
    "nextOffset": { "minimum": 0, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
  },
  "required": ["entries", "count", "offset", "limit"],
  "type": "object",
} as const;

export const JobsMetricsRequestSchema = {
  "properties": {
    "groupBy": {
      "anyOf": [
        { "const": "type", "type": "string" },
        { "const": "service", "type": "string" },
        { "const": "queueKey", "type": "string" },
        { "const": "state", "type": "string" },
        { "const": "trigger", "type": "string" },
      ],
    },
    "queueKey": { "type": "string" },
    "service": { "minLength": 1, "type": "string" },
    "state": {
      "items": {
        "anyOf": [
          { "const": "pending", "type": "string" },
          { "const": "active", "type": "string" },
          { "const": "retry", "type": "string" },
          { "const": "completed", "type": "string" },
          { "const": "failed", "type": "string" },
          { "const": "cancelled", "type": "string" },
          { "const": "skipped", "type": "string" },
          { "const": "stale", "type": "string" },
          { "const": "expired", "type": "string" },
          { "const": "dead", "type": "string" },
          { "const": "dismissed", "type": "string" },
        ],
      },
      "type": "array",
    },
    "step": {
      "anyOf": [
        { "const": "1m", "type": "string" },
        { "const": "5m", "type": "string" },
        { "const": "15m", "type": "string" },
        { "const": "1h", "type": "string" },
        { "const": "6h", "type": "string" },
        { "const": "1d", "type": "string" },
      ],
    },
    "trigger": { "type": "string" },
    "type": { "minLength": 1, "type": "string" },
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
  "required": ["window", "step", "groupBy"],
  "type": "object",
} as const;

export const JobsMetricsResponseSchema = {
  "properties": {
    "buckets": {
      "items": {
        "properties": {
          "end": { "format": "date-time", "type": "string" },
          "groups": {
            "items": {
              "properties": {
                "cancelled": { "minimum": 0, "type": "integer" },
                "completed": { "minimum": 0, "type": "integer" },
                "dead": { "minimum": 0, "type": "integer" },
                "dismissed": { "minimum": 0, "type": "integer" },
                "failed": { "minimum": 0, "type": "integer" },
                "key": { "type": "string" },
                "label": { "type": "string" },
                "queueWait": {
                  "properties": {
                    "count": { "minimum": 0, "type": "integer" },
                    "maxMs": { "minimum": 0, "type": "integer" },
                    "p50Ms": { "minimum": 0, "type": "integer" },
                    "p95Ms": { "minimum": 0, "type": "integer" },
                  },
                  "required": ["count"],
                  "type": "object",
                },
                "retried": { "minimum": 0, "type": "integer" },
                "runtime": {
                  "properties": {
                    "count": { "minimum": 0, "type": "integer" },
                    "maxMs": { "minimum": 0, "type": "integer" },
                    "p50Ms": { "minimum": 0, "type": "integer" },
                    "p95Ms": { "minimum": 0, "type": "integer" },
                  },
                  "required": ["count"],
                  "type": "object",
                },
                "started": { "minimum": 0, "type": "integer" },
                "submitted": { "minimum": 0, "type": "integer" },
              },
              "required": [
                "key",
                "label",
                "submitted",
                "started",
                "completed",
                "failed",
                "retried",
                "dead",
                "cancelled",
                "dismissed",
                "runtime",
                "queueWait",
              ],
              "type": "object",
            },
            "type": "array",
          },
          "start": { "format": "date-time", "type": "string" },
        },
        "required": ["start", "end", "groups"],
        "type": "object",
      },
      "type": "array",
    },
    "generatedAt": { "format": "date-time", "type": "string" },
    "groupBy": { "type": "string" },
    "step": { "type": "string" },
    "summary": {
      "items": {
        "properties": {
          "byState": { "additionalProperties": true, "type": "object" },
          "dead": { "minimum": 0, "type": "integer" },
          "failed": { "minimum": 0, "type": "integer" },
          "failureRate": { "minimum": 0, "type": "number" },
          "key": { "type": "string" },
          "label": { "type": "string" },
          "latestUpdatedAt": { "format": "date-time", "type": "string" },
          "oldestCreatedAt": { "format": "date-time", "type": "string" },
          "queueWait": {
            "properties": {
              "count": { "minimum": 0, "type": "integer" },
              "maxMs": { "minimum": 0, "type": "integer" },
              "p50Ms": { "minimum": 0, "type": "integer" },
              "p95Ms": { "minimum": 0, "type": "integer" },
            },
            "required": ["count"],
            "type": "object",
          },
          "queued": { "minimum": 0, "type": "integer" },
          "running": { "minimum": 0, "type": "integer" },
          "runtime": {
            "properties": {
              "count": { "minimum": 0, "type": "integer" },
              "maxMs": { "minimum": 0, "type": "integer" },
              "p50Ms": { "minimum": 0, "type": "integer" },
              "p95Ms": { "minimum": 0, "type": "integer" },
            },
            "required": ["count"],
            "type": "object",
          },
          "slow": { "minimum": 0, "type": "integer" },
          "total": { "minimum": 0, "type": "integer" },
        },
        "required": [
          "key",
          "label",
          "total",
          "byState",
          "runtime",
          "queueWait",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "window": { "type": "string" },
  },
  "required": [
    "window",
    "step",
    "groupBy",
    "generatedAt",
    "summary",
    "buckets",
  ],
  "type": "object",
} as const;

export const JobsQueryRequestSchema = {
  "properties": {
    "groupBy": {
      "anyOf": [
        { "const": "service", "type": "string" },
        { "const": "type", "type": "string" },
        { "const": "state", "type": "string" },
        { "const": "queueKey", "type": "string" },
        { "const": "trigger", "type": "string" },
        { "const": "runtimeBand", "type": "string" },
      ],
    },
    "limit": { "minimum": 1, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
    "queueKey": { "type": "string" },
    "runtimeBand": {
      "anyOf": [
        { "const": "queued", "type": "string" },
        { "const": "running", "type": "string" },
        { "const": "slow", "type": "string" },
        { "const": "terminal", "type": "string" },
      ],
    },
    "search": { "type": "string" },
    "service": { "minLength": 1, "type": "string" },
    "sort": {
      "properties": {
        "direction": {
          "anyOf": [{ "const": "asc", "type": "string" }, {
            "const": "desc",
            "type": "string",
          }],
        },
        "field": {
          "anyOf": [
            { "const": "updatedAt", "type": "string" },
            { "const": "queueAge", "type": "string" },
            { "const": "runtime", "type": "string" },
            { "const": "failureRate", "type": "string" },
            { "const": "retries", "type": "string" },
            { "const": "depth", "type": "string" },
          ],
        },
      },
      "required": ["field"],
      "type": "object",
    },
    "state": {
      "items": {
        "anyOf": [
          { "const": "pending", "type": "string" },
          { "const": "active", "type": "string" },
          { "const": "retry", "type": "string" },
          { "const": "completed", "type": "string" },
          { "const": "failed", "type": "string" },
          { "const": "cancelled", "type": "string" },
          { "const": "skipped", "type": "string" },
          { "const": "stale", "type": "string" },
          { "const": "expired", "type": "string" },
          { "const": "dead", "type": "string" },
          { "const": "dismissed", "type": "string" },
        ],
      },
      "type": "array",
    },
    "trigger": { "type": "string" },
    "type": { "minLength": 1, "type": "string" },
    "window": {
      "anyOf": [{ "const": "1h", "type": "string" }, {
        "const": "24h",
        "type": "string",
      }, { "const": "7d", "type": "string" }],
    },
  },
  "required": ["limit"],
  "type": "object",
} as const;

export const JobsQueryResponseSchema = {
  "properties": {
    "count": { "minimum": 0, "type": "integer" },
    "entries": {
      "items": {
        "properties": {
          "completedAt": { "format": "date-time", "type": "string" },
          "context": {
            "properties": {
              "requestId": { "minLength": 1, "type": "string" },
              "traceId": { "pattern": "^[0-9a-f]{32}$", "type": "string" },
              "traceparent": {
                "pattern":
                  "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
                "type": "string",
              },
              "tracestate": { "minLength": 1, "type": "string" },
            },
            "required": ["requestId", "traceId", "traceparent"],
            "type": "object",
          },
          "createdAt": { "format": "date-time", "type": "string" },
          "errorFingerprint": { "type": "string" },
          "id": { "minLength": 1, "type": "string" },
          "lastError": { "type": "string" },
          "lineage": {
            "properties": {
              "operationId": { "type": "string" },
              "parentJobId": { "type": "string" },
              "relatedKeys": { "items": { "type": "string" }, "type": "array" },
              "rootJobId": { "type": "string" },
            },
            "type": "object",
          },
          "maxTries": { "minimum": 1, "type": "integer" },
          "progress": {
            "properties": {
              "current": { "minimum": 0, "type": "integer" },
              "message": { "type": "string" },
              "step": { "type": "string" },
              "total": { "minimum": 0, "type": "integer" },
            },
            "type": "object",
          },
          "queueAgeMs": { "minimum": 0, "type": "integer" },
          "queueKey": { "type": "string" },
          "runtimeBand": { "type": "string" },
          "runtimeMs": { "minimum": 0, "type": "integer" },
          "service": { "minLength": 1, "type": "string" },
          "startedAt": { "format": "date-time", "type": "string" },
          "state": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "retry", "type": "string" },
              { "const": "completed", "type": "string" },
              { "const": "failed", "type": "string" },
              { "const": "cancelled", "type": "string" },
              { "const": "skipped", "type": "string" },
              { "const": "stale", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "dead", "type": "string" },
              { "const": "dismissed", "type": "string" },
            ],
          },
          "tries": { "minimum": 0, "type": "integer" },
          "trigger": {
            "properties": {
              "id": { "type": "string" },
              "kind": {
                "anyOf": [
                  { "const": "schedule", "type": "string" },
                  { "const": "operation", "type": "string" },
                  { "const": "rpc", "type": "string" },
                  { "const": "event", "type": "string" },
                  { "const": "manualReplay", "type": "string" },
                  { "const": "serviceCode", "type": "string" },
                  { "const": "parentJob", "type": "string" },
                ],
              },
              "operationId": { "type": "string" },
              "parentJobId": { "type": "string" },
              "requestId": { "type": "string" },
              "subject": { "type": "string" },
              "traceId": { "type": "string" },
            },
            "required": ["kind"],
            "type": "object",
          },
          "type": { "minLength": 1, "type": "string" },
          "updatedAt": { "format": "date-time", "type": "string" },
          "waitingOn": {
            "items": {
              "properties": {
                "id": { "minLength": 1, "type": "string" },
                "label": { "minLength": 1, "type": "string" },
                "startedAt": { "format": "date-time", "type": "string" },
                "target": {
                  "properties": {
                    "id": { "minLength": 1, "type": "string" },
                    "key": { "minLength": 1, "type": "string" },
                    "kind": {
                      "anyOf": [{ "const": "job", "type": "string" }, {
                        "const": "operation",
                        "type": "string",
                      }, { "const": "external", "type": "string" }],
                    },
                    "label": { "minLength": 1, "type": "string" },
                    "operation": { "minLength": 1, "type": "string" },
                    "operationId": { "minLength": 1, "type": "string" },
                    "service": { "minLength": 1, "type": "string" },
                    "system": { "minLength": 1, "type": "string" },
                    "type": { "minLength": 1, "type": "string" },
                  },
                  "required": ["kind"],
                  "type": "object",
                },
              },
              "required": ["id", "target", "startedAt"],
              "type": "object",
            },
            "type": "array",
          },
        },
        "required": [
          "id",
          "service",
          "type",
          "state",
          "createdAt",
          "updatedAt",
          "tries",
          "maxTries",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "groups": {
      "items": {
        "properties": {
          "count": { "minimum": 0, "type": "integer" },
          "depth": { "minimum": 0, "type": "integer" },
          "failureRate": { "minimum": 0, "type": "number" },
          "key": { "type": "string" },
          "label": { "type": "string" },
          "latestUpdatedAt": { "format": "date-time", "type": "string" },
          "oldestCreatedAt": { "format": "date-time", "type": "string" },
          "state": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "retry", "type": "string" },
              { "const": "completed", "type": "string" },
              { "const": "failed", "type": "string" },
              { "const": "cancelled", "type": "string" },
              { "const": "skipped", "type": "string" },
              { "const": "stale", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "dead", "type": "string" },
              { "const": "dismissed", "type": "string" },
            ],
          },
        },
        "required": ["key", "label", "count"],
        "type": "object",
      },
      "type": "array",
    },
    "limit": { "minimum": 1, "type": "integer" },
    "nextOffset": { "minimum": 0, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
    "stats": {
      "properties": {
        "byState": { "additionalProperties": true, "type": "object" },
        "dead": { "minimum": 0, "type": "integer" },
        "failed": { "minimum": 0, "type": "integer" },
        "queued": { "minimum": 0, "type": "integer" },
        "running": { "minimum": 0, "type": "integer" },
        "slow": { "minimum": 0, "type": "integer" },
        "total": { "minimum": 0, "type": "integer" },
      },
      "required": ["total", "byState"],
      "type": "object",
    },
  },
  "required": ["entries", "groups", "stats", "count", "offset", "limit"],
  "type": "object",
} as const;

export const JobsReplayDLQRequestSchema = {
  "description":
    "Jobs admin ids are globally addressable; callers identify jobs by id only.",
  "properties": {
    "id": { "minLength": 1, "type": "string" },
    "reason": { "minLength": 1, "type": "string" },
  },
  "required": ["id"],
  "type": "object",
} as const;

export const JobsReplayDLQResponseSchema = {
  "properties": {
    "job": {
      "properties": {
        "completedAt": { "format": "date-time", "type": "string" },
        "concurrency": {
          "properties": {
            "heartbeatAt": { "format": "date-time", "type": "string" },
            "key": { "minLength": 1, "type": "string" },
            "keyHash": { "minLength": 1, "type": "string" },
            "leaseExpiresAt": { "format": "date-time", "type": "string" },
            "staleTakeoverCount": { "minimum": 0, "type": "integer" },
          },
          "required": ["key", "keyHash"],
          "type": "object",
        },
        "context": {
          "properties": {
            "requestId": { "minLength": 1, "type": "string" },
            "traceId": { "pattern": "^[0-9a-f]{32}$", "type": "string" },
            "traceparent": {
              "pattern": "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
              "type": "string",
            },
            "tracestate": { "minLength": 1, "type": "string" },
          },
          "required": ["requestId", "traceId", "traceparent"],
          "type": "object",
        },
        "createdAt": { "format": "date-time", "type": "string" },
        "deadline": { "format": "date-time", "type": "string" },
        "errorDetail": {
          "properties": {
            "causes": { "items": { "type": "object" }, "type": "array" },
            "fingerprint": { "minLength": 1, "type": "string" },
            "firstSeen": { "format": "date-time", "type": "string" },
            "message": { "type": "string" },
            "occurrenceCount": { "minimum": 0, "type": "integer" },
            "stack": { "type": "string" },
            "type": { "type": "string" },
            "worker": {
              "properties": {
                "instanceId": { "type": "string" },
                "runtime": { "type": "string" },
                "service": { "type": "string" },
                "version": { "type": "string" },
              },
              "type": "object",
            },
          },
          "required": ["message", "fingerprint"],
          "type": "object",
        },
        "id": { "minLength": 1, "type": "string" },
        "lastError": { "type": "string" },
        "lineage": {
          "properties": {
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "relatedKeys": { "items": { "type": "string" }, "type": "array" },
            "rootJobId": { "type": "string" },
          },
          "type": "object",
        },
        "logs": {
          "items": {
            "properties": {
              "level": {
                "anyOf": [{ "const": "info", "type": "string" }, {
                  "const": "warn",
                  "type": "string",
                }, { "const": "error", "type": "string" }],
              },
              "message": { "type": "string" },
              "timestamp": { "format": "date-time", "type": "string" },
            },
            "required": ["timestamp", "level", "message"],
            "type": "object",
          },
          "type": "array",
        },
        "maxTries": { "minimum": 1, "type": "integer" },
        "payload": {},
        "progress": {
          "properties": {
            "current": { "minimum": 0, "type": "integer" },
            "message": { "type": "string" },
            "step": { "type": "string" },
            "total": { "minimum": 0, "type": "integer" },
          },
          "type": "object",
        },
        "queuePolicy": {
          "properties": {
            "existingJobId": { "minLength": 1, "type": "string" },
            "outcome": { "minLength": 1, "type": "string" },
            "reason": { "minLength": 1, "type": "string" },
            "replacedJobId": { "minLength": 1, "type": "string" },
          },
          "required": ["outcome"],
          "type": "object",
        },
        "result": {},
        "service": { "minLength": 1, "type": "string" },
        "startedAt": { "format": "date-time", "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "retry", "type": "string" },
            { "const": "completed", "type": "string" },
            { "const": "failed", "type": "string" },
            { "const": "cancelled", "type": "string" },
            { "const": "skipped", "type": "string" },
            { "const": "stale", "type": "string" },
            { "const": "expired", "type": "string" },
            { "const": "dead", "type": "string" },
            { "const": "dismissed", "type": "string" },
          ],
        },
        "tries": { "minimum": 0, "type": "integer" },
        "trigger": {
          "properties": {
            "id": { "type": "string" },
            "kind": {
              "anyOf": [
                { "const": "schedule", "type": "string" },
                { "const": "operation", "type": "string" },
                { "const": "rpc", "type": "string" },
                { "const": "event", "type": "string" },
                { "const": "manualReplay", "type": "string" },
                { "const": "serviceCode", "type": "string" },
                { "const": "parentJob", "type": "string" },
              ],
            },
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "requestId": { "type": "string" },
            "subject": { "type": "string" },
            "traceId": { "type": "string" },
          },
          "required": ["kind"],
          "type": "object",
        },
        "type": { "minLength": 1, "type": "string" },
        "updatedAt": { "format": "date-time", "type": "string" },
        "waitingOn": {
          "items": {
            "properties": {
              "id": { "minLength": 1, "type": "string" },
              "label": { "minLength": 1, "type": "string" },
              "startedAt": { "format": "date-time", "type": "string" },
              "target": {
                "properties": {
                  "id": { "minLength": 1, "type": "string" },
                  "key": { "minLength": 1, "type": "string" },
                  "kind": {
                    "anyOf": [{ "const": "job", "type": "string" }, {
                      "const": "operation",
                      "type": "string",
                    }, { "const": "external", "type": "string" }],
                  },
                  "label": { "minLength": 1, "type": "string" },
                  "operation": { "minLength": 1, "type": "string" },
                  "operationId": { "minLength": 1, "type": "string" },
                  "service": { "minLength": 1, "type": "string" },
                  "system": { "minLength": 1, "type": "string" },
                  "type": { "minLength": 1, "type": "string" },
                },
                "required": ["kind"],
                "type": "object",
              },
            },
            "required": ["id", "target", "startedAt"],
            "type": "object",
          },
          "type": "array",
        },
      },
      "required": [
        "id",
        "context",
        "service",
        "type",
        "state",
        "payload",
        "createdAt",
        "updatedAt",
        "tries",
        "maxTries",
      ],
      "type": "object",
    },
  },
  "required": ["job"],
  "type": "object",
} as const;

export const JobsRetryRequestSchema = {
  "description":
    "Jobs admin ids are globally addressable; callers identify jobs by id only.",
  "properties": {
    "id": { "minLength": 1, "type": "string" },
    "reason": { "minLength": 1, "type": "string" },
  },
  "required": ["id"],
  "type": "object",
} as const;

export const JobsRetryResponseSchema = {
  "properties": {
    "job": {
      "properties": {
        "completedAt": { "format": "date-time", "type": "string" },
        "concurrency": {
          "properties": {
            "heartbeatAt": { "format": "date-time", "type": "string" },
            "key": { "minLength": 1, "type": "string" },
            "keyHash": { "minLength": 1, "type": "string" },
            "leaseExpiresAt": { "format": "date-time", "type": "string" },
            "staleTakeoverCount": { "minimum": 0, "type": "integer" },
          },
          "required": ["key", "keyHash"],
          "type": "object",
        },
        "context": {
          "properties": {
            "requestId": { "minLength": 1, "type": "string" },
            "traceId": { "pattern": "^[0-9a-f]{32}$", "type": "string" },
            "traceparent": {
              "pattern": "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
              "type": "string",
            },
            "tracestate": { "minLength": 1, "type": "string" },
          },
          "required": ["requestId", "traceId", "traceparent"],
          "type": "object",
        },
        "createdAt": { "format": "date-time", "type": "string" },
        "deadline": { "format": "date-time", "type": "string" },
        "errorDetail": {
          "properties": {
            "causes": { "items": { "type": "object" }, "type": "array" },
            "fingerprint": { "minLength": 1, "type": "string" },
            "firstSeen": { "format": "date-time", "type": "string" },
            "message": { "type": "string" },
            "occurrenceCount": { "minimum": 0, "type": "integer" },
            "stack": { "type": "string" },
            "type": { "type": "string" },
            "worker": {
              "properties": {
                "instanceId": { "type": "string" },
                "runtime": { "type": "string" },
                "service": { "type": "string" },
                "version": { "type": "string" },
              },
              "type": "object",
            },
          },
          "required": ["message", "fingerprint"],
          "type": "object",
        },
        "id": { "minLength": 1, "type": "string" },
        "lastError": { "type": "string" },
        "lineage": {
          "properties": {
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "relatedKeys": { "items": { "type": "string" }, "type": "array" },
            "rootJobId": { "type": "string" },
          },
          "type": "object",
        },
        "logs": {
          "items": {
            "properties": {
              "level": {
                "anyOf": [{ "const": "info", "type": "string" }, {
                  "const": "warn",
                  "type": "string",
                }, { "const": "error", "type": "string" }],
              },
              "message": { "type": "string" },
              "timestamp": { "format": "date-time", "type": "string" },
            },
            "required": ["timestamp", "level", "message"],
            "type": "object",
          },
          "type": "array",
        },
        "maxTries": { "minimum": 1, "type": "integer" },
        "payload": {},
        "progress": {
          "properties": {
            "current": { "minimum": 0, "type": "integer" },
            "message": { "type": "string" },
            "step": { "type": "string" },
            "total": { "minimum": 0, "type": "integer" },
          },
          "type": "object",
        },
        "queuePolicy": {
          "properties": {
            "existingJobId": { "minLength": 1, "type": "string" },
            "outcome": { "minLength": 1, "type": "string" },
            "reason": { "minLength": 1, "type": "string" },
            "replacedJobId": { "minLength": 1, "type": "string" },
          },
          "required": ["outcome"],
          "type": "object",
        },
        "result": {},
        "service": { "minLength": 1, "type": "string" },
        "startedAt": { "format": "date-time", "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "retry", "type": "string" },
            { "const": "completed", "type": "string" },
            { "const": "failed", "type": "string" },
            { "const": "cancelled", "type": "string" },
            { "const": "skipped", "type": "string" },
            { "const": "stale", "type": "string" },
            { "const": "expired", "type": "string" },
            { "const": "dead", "type": "string" },
            { "const": "dismissed", "type": "string" },
          ],
        },
        "tries": { "minimum": 0, "type": "integer" },
        "trigger": {
          "properties": {
            "id": { "type": "string" },
            "kind": {
              "anyOf": [
                { "const": "schedule", "type": "string" },
                { "const": "operation", "type": "string" },
                { "const": "rpc", "type": "string" },
                { "const": "event", "type": "string" },
                { "const": "manualReplay", "type": "string" },
                { "const": "serviceCode", "type": "string" },
                { "const": "parentJob", "type": "string" },
              ],
            },
            "operationId": { "type": "string" },
            "parentJobId": { "type": "string" },
            "requestId": { "type": "string" },
            "subject": { "type": "string" },
            "traceId": { "type": "string" },
          },
          "required": ["kind"],
          "type": "object",
        },
        "type": { "minLength": 1, "type": "string" },
        "updatedAt": { "format": "date-time", "type": "string" },
        "waitingOn": {
          "items": {
            "properties": {
              "id": { "minLength": 1, "type": "string" },
              "label": { "minLength": 1, "type": "string" },
              "startedAt": { "format": "date-time", "type": "string" },
              "target": {
                "properties": {
                  "id": { "minLength": 1, "type": "string" },
                  "key": { "minLength": 1, "type": "string" },
                  "kind": {
                    "anyOf": [{ "const": "job", "type": "string" }, {
                      "const": "operation",
                      "type": "string",
                    }, { "const": "external", "type": "string" }],
                  },
                  "label": { "minLength": 1, "type": "string" },
                  "operation": { "minLength": 1, "type": "string" },
                  "operationId": { "minLength": 1, "type": "string" },
                  "service": { "minLength": 1, "type": "string" },
                  "system": { "minLength": 1, "type": "string" },
                  "type": { "minLength": 1, "type": "string" },
                },
                "required": ["kind"],
                "type": "object",
              },
            },
            "required": ["id", "target", "startedAt"],
            "type": "object",
          },
          "type": "array",
        },
      },
      "required": [
        "id",
        "context",
        "service",
        "type",
        "state",
        "payload",
        "createdAt",
        "updatedAt",
        "tries",
        "maxTries",
      ],
      "type": "object",
    },
  },
  "required": ["job"],
  "type": "object",
} as const;

export const JobsWatchFrameSchema = {
  "anyOf": [{
    "properties": {
      "kind": { "const": "ready", "type": "string" },
      "timestamp": { "format": "date-time", "type": "string" },
    },
    "required": ["kind", "timestamp"],
    "type": "object",
  }, {
    "properties": {
      "id": { "minLength": 1, "type": "string" },
      "kind": { "const": "jobChanged", "type": "string" },
      "service": { "minLength": 1, "type": "string" },
      "state": {
        "anyOf": [
          { "const": "pending", "type": "string" },
          { "const": "active", "type": "string" },
          { "const": "retry", "type": "string" },
          { "const": "completed", "type": "string" },
          { "const": "failed", "type": "string" },
          { "const": "cancelled", "type": "string" },
          { "const": "skipped", "type": "string" },
          { "const": "stale", "type": "string" },
          { "const": "expired", "type": "string" },
          { "const": "dead", "type": "string" },
          { "const": "dismissed", "type": "string" },
        ],
      },
      "type": { "minLength": 1, "type": "string" },
      "updatedAt": { "format": "date-time", "type": "string" },
    },
    "required": ["kind", "id", "service", "type", "state", "updatedAt"],
    "type": "object",
  }, {
    "properties": {
      "kind": { "const": "queryInvalidated", "type": "string" },
      "reason": {
        "anyOf": [{ "const": "matched-job-changed", "type": "string" }, {
          "const": "unknown-match",
          "type": "string",
        }],
      },
      "timestamp": { "format": "date-time", "type": "string" },
    },
    "required": ["kind", "reason", "timestamp"],
    "type": "object",
  }, {
    "properties": {
      "id": { "minLength": 1, "type": "string" },
      "kind": { "const": "jobInspectChanged", "type": "string" },
      "timestamp": { "format": "date-time", "type": "string" },
    },
    "required": ["kind", "id", "timestamp"],
    "type": "object",
  }],
} as const;

export const JobsWatchRequestSchema = {
  "properties": {
    "includeInitial": { "type": "boolean" },
    "jobId": { "minLength": 1, "type": "string" },
    "query": {
      "properties": {
        "groupBy": {
          "anyOf": [
            { "const": "service", "type": "string" },
            { "const": "type", "type": "string" },
            { "const": "state", "type": "string" },
            { "const": "queueKey", "type": "string" },
            { "const": "trigger", "type": "string" },
            { "const": "runtimeBand", "type": "string" },
          ],
        },
        "limit": { "minimum": 1, "type": "integer" },
        "offset": { "minimum": 0, "type": "integer" },
        "queueKey": { "type": "string" },
        "runtimeBand": {
          "anyOf": [
            { "const": "queued", "type": "string" },
            { "const": "running", "type": "string" },
            { "const": "slow", "type": "string" },
            { "const": "terminal", "type": "string" },
          ],
        },
        "search": { "type": "string" },
        "service": { "minLength": 1, "type": "string" },
        "sort": {
          "properties": {
            "direction": {
              "anyOf": [{ "const": "asc", "type": "string" }, {
                "const": "desc",
                "type": "string",
              }],
            },
            "field": {
              "anyOf": [
                { "const": "updatedAt", "type": "string" },
                { "const": "queueAge", "type": "string" },
                { "const": "runtime", "type": "string" },
                { "const": "failureRate", "type": "string" },
                { "const": "retries", "type": "string" },
                { "const": "depth", "type": "string" },
              ],
            },
          },
          "required": ["field"],
          "type": "object",
        },
        "state": {
          "items": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "retry", "type": "string" },
              { "const": "completed", "type": "string" },
              { "const": "failed", "type": "string" },
              { "const": "cancelled", "type": "string" },
              { "const": "skipped", "type": "string" },
              { "const": "stale", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "dead", "type": "string" },
              { "const": "dismissed", "type": "string" },
            ],
          },
          "type": "array",
        },
        "trigger": { "type": "string" },
        "type": { "minLength": 1, "type": "string" },
        "window": {
          "anyOf": [{ "const": "1h", "type": "string" }, {
            "const": "24h",
            "type": "string",
          }, { "const": "7d", "type": "string" }],
        },
      },
      "required": ["limit"],
      "type": "object",
    },
  },
  "type": "object",
} as const;

export const NotFoundErrorDataSchema = {
  "properties": {
    "context": { "additionalProperties": true, "type": "object" },
    "id": { "minLength": 1, "type": "string" },
    "jobId": { "minLength": 1, "type": "string" },
    "message": { "type": "string" },
    "resource": { "minLength": 1, "type": "string" },
    "traceId": { "type": "string" },
    "type": { "const": "NotFoundError", "type": "string" },
  },
  "required": ["id", "type", "message", "resource"],
  "type": "object",
} as const;
