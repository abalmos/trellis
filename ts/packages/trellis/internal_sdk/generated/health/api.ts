// Generated from ./rust/crates/runtime/.trellis/generated/protocol/apis/trellis.health@v1.json

export const API_ID = "trellis.health@v1" as const;
export const API_DIGEST =
  "didAZZoUrnCyj4EF01tWRMkd2UlHrMvYnwWyGmBg5Uk" as const;
export const API = {
  "capabilities": {
    "trellis.health::read": {
      "allows": [{
        "action": "subscribe",
        "target": {
          "api": "trellis.health@v1",
          "kind": "apiSurface",
          "name": "Health.StatusChanged",
          "surface": "event",
        },
      }, {
        "action": "subscribe",
        "target": {
          "api": "trellis.health@v1",
          "kind": "apiSurface",
          "name": "Health.Watch",
          "surface": "feed",
        },
      }, {
        "action": "call",
        "target": {
          "api": "trellis.health@v1",
          "kind": "apiSurface",
          "name": "Health.Inspect",
          "surface": "rpc",
        },
      }, {
        "action": "call",
        "target": {
          "api": "trellis.health@v1",
          "kind": "apiSurface",
          "name": "Health.Metrics",
          "surface": "rpc",
        },
      }, {
        "action": "call",
        "target": {
          "api": "trellis.health@v1",
          "kind": "apiSurface",
          "name": "Health.Query",
          "surface": "rpc",
        },
      }],
    },
  },
  "consent": {
    "trellis.health::read": {
      "consequence": "",
      "description": "View current and historical participant health state.",
      "title": "Read participant health",
    },
  },
  "description":
    "Trellis-managed participant health projection and operational history.",
  "displayName": "Trellis Health",
  "docs": {
    "markdown":
      "Provides current participant health, instance inspection, bounded metrics, invalidation feeds, and durable status transitions. Periodic heartbeat samples use a private runtime transport and are not contract events.",
    "summary": "Participant health administration APIs.",
  },
  "errors": {
    "NotFoundError": { "schema": { "schema": "NotFoundErrorData" } },
    "UnexpectedError": {},
    "ValidationError": {},
  },
  "events": {
    "Health.StatusChanged": {
      "docs": {
        "markdown":
          "Emitted only when an instance effective status changes; periodic heartbeat samples are not emitted as events.",
        "summary": "Observe effective health transitions.",
      },
      "event": { "schema": "HealthStatusChangedEvent" },
      "version": "v1",
    },
  },
  "exports": { "schemas": ["HealthHeartbeatSample"] },
  "feeds": {
    "Health.Watch": {
      "docs": {
        "markdown":
          "Streams projection revisions and affected participant identities so clients can refresh authoritative snapshots.",
        "summary": "Watch health projection invalidations.",
      },
      "event": { "schema": "HealthWatchFrame" },
      "input": { "schema": "HealthWatchRequest" },
      "version": "v1",
    },
  },
  "format": "trellis.api.v1",
  "id": "trellis.health@v1",
  "rpc": {
    "Health.Inspect": {
      "docs": {
        "markdown":
          "Returns latest instance samples and bounded status intervals for one participant contract.",
        "summary": "Inspect participant health.",
      },
      "errors": ["NotFoundError", "UnexpectedError", "ValidationError"],
      "input": { "schema": "HealthInspectRequest" },
      "output": { "schema": "HealthInspectResponse" },
      "version": "v1",
    },
    "Health.Metrics": {
      "docs": {
        "markdown":
          "Returns time-bucketed availability, status duration, sample, check, and latency aggregates.",
        "summary": "Read participant health metrics.",
      },
      "errors": ["UnexpectedError", "ValidationError"],
      "input": { "schema": "HealthMetricsRequest" },
      "output": { "schema": "HealthMetricsResponse" },
      "version": "v1",
    },
    "Health.Query": {
      "docs": {
        "markdown":
          "Returns a server-authoritative, paginated health summary grouped by participant contract.",
        "summary": "Query participant health.",
      },
      "errors": ["UnexpectedError", "ValidationError"],
      "input": { "schema": "HealthQueryRequest" },
      "output": { "schema": "HealthQueryResponse" },
      "version": "v1",
    },
  },
  "schemas": {
    "HealthHeartbeatSample": {
      "properties": {
        "checks": {
          "items": {
            "properties": {
              "error": { "maxLength": 1024, "type": "string" },
              "info": { "type": "object" },
              "latencyMs": {
                "maximum": 3600000,
                "minimum": 0,
                "type": "number",
              },
              "name": { "maxLength": 128, "minLength": 1, "type": "string" },
              "status": { "enum": ["ok", "failed"], "type": "string" },
              "summary": { "maxLength": 1024, "type": "string" },
            },
            "required": ["name", "status", "latencyMs"],
            "type": "object",
          },
          "maxItems": 64,
          "type": "array",
        },
        "participant": {
          "properties": {
            "contractDigest": {
              "maxLength": 256,
              "minLength": 1,
              "type": "string",
            },
            "contractId": {
              "maxLength": 256,
              "minLength": 1,
              "type": "string",
            },
            "info": { "type": "object" },
            "instanceId": {
              "maxLength": 128,
              "minLength": 1,
              "type": "string",
            },
            "kind": { "enum": ["service", "device"], "type": "string" },
            "name": { "maxLength": 256, "minLength": 1, "type": "string" },
            "publishIntervalMs": {
              "maximum": 600000,
              "minimum": 1000,
              "type": "integer",
            },
            "runtime": {
              "enum": ["deno", "node", "rust", "unknown"],
              "type": "string",
            },
            "runtimeVersion": { "maxLength": 256, "type": "string" },
            "startedAt": { "format": "date-time", "type": "string" },
            "version": { "maxLength": 256, "type": "string" },
          },
          "required": [
            "name",
            "kind",
            "instanceId",
            "contractId",
            "contractDigest",
            "startedAt",
            "publishIntervalMs",
            "runtime",
          ],
          "type": "object",
        },
        "reportedStatus": {
          "enum": ["healthy", "degraded", "unhealthy"],
          "type": "string",
        },
        "sample": {
          "properties": {
            "id": { "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$", "type": "string" },
            "time": { "format": "date-time", "type": "string" },
          },
          "required": ["id", "time"],
          "type": "object",
        },
        "summary": { "maxLength": 1024, "type": "string" },
      },
      "required": ["sample", "participant", "reportedStatus", "checks"],
      "type": "object",
    },
    "HealthInspectRequest": {
      "properties": {
        "contractId": { "maxLength": 256, "minLength": 1, "type": "string" },
        "historyLimit": { "maximum": 500, "minimum": 1, "type": "integer" },
        "historySince": { "format": "date-time", "type": "string" },
        "instanceId": { "maxLength": 128, "minLength": 1, "type": "string" },
        "participantKind": { "enum": ["service", "device"], "type": "string" },
      },
      "required": ["participantKind", "contractId"],
      "type": "object",
    },
    "HealthInspectResponse": {
      "properties": {
        "asOf": { "format": "date-time", "type": "string" },
        "history": {
          "items": {
            "properties": {
              "checks": {
                "items": {
                  "properties": {
                    "name": { "type": "string" },
                    "status": { "enum": ["ok", "failed"], "type": "string" },
                  },
                  "required": ["name", "status"],
                  "type": "object",
                },
                "type": "array",
              },
              "effectiveStatus": {
                "enum": ["healthy", "degraded", "unhealthy", "offline"],
                "type": "string",
              },
              "endedAt": { "format": "date-time", "type": "string" },
              "instanceId": { "type": "string" },
              "intervalId": { "minimum": 1, "type": "integer" },
              "reason": {
                "enum": [
                  "first-sample",
                  "heartbeat-change",
                  "heartbeat-resumed",
                  "deadline-expired",
                ],
                "type": "string",
              },
              "reportedStatus": {
                "enum": ["healthy", "degraded", "unhealthy"],
                "type": "string",
              },
              "startedAt": { "format": "date-time", "type": "string" },
            },
            "required": [
              "intervalId",
              "instanceId",
              "startedAt",
              "reportedStatus",
              "effectiveStatus",
              "checks",
              "reason",
            ],
            "type": "object",
          },
          "type": "array",
        },
        "instances": {
          "items": {
            "properties": {
              "ageMs": { "minimum": 0, "type": "integer" },
              "contractDigest": { "type": "string" },
              "deploymentId": { "type": "string" },
              "effectiveStatus": {
                "enum": ["healthy", "degraded", "unhealthy", "offline"],
                "type": "string",
              },
              "heartbeatDeadline": { "format": "date-time", "type": "string" },
              "instanceId": { "type": "string" },
              "latestSample": {
                "properties": {
                  "checks": {
                    "items": {
                      "properties": {
                        "error": { "maxLength": 1024, "type": "string" },
                        "info": { "type": "object" },
                        "latencyMs": {
                          "maximum": 3600000,
                          "minimum": 0,
                          "type": "number",
                        },
                        "name": {
                          "maxLength": 128,
                          "minLength": 1,
                          "type": "string",
                        },
                        "status": {
                          "enum": ["ok", "failed"],
                          "type": "string",
                        },
                        "summary": { "maxLength": 1024, "type": "string" },
                      },
                      "required": ["name", "status", "latencyMs"],
                      "type": "object",
                    },
                    "maxItems": 64,
                    "type": "array",
                  },
                  "participant": {
                    "properties": {
                      "contractDigest": {
                        "maxLength": 256,
                        "minLength": 1,
                        "type": "string",
                      },
                      "contractId": {
                        "maxLength": 256,
                        "minLength": 1,
                        "type": "string",
                      },
                      "info": { "type": "object" },
                      "instanceId": {
                        "maxLength": 128,
                        "minLength": 1,
                        "type": "string",
                      },
                      "kind": {
                        "enum": ["service", "device"],
                        "type": "string",
                      },
                      "name": {
                        "maxLength": 256,
                        "minLength": 1,
                        "type": "string",
                      },
                      "publishIntervalMs": {
                        "maximum": 600000,
                        "minimum": 1000,
                        "type": "integer",
                      },
                      "runtime": {
                        "enum": ["deno", "node", "rust", "unknown"],
                        "type": "string",
                      },
                      "runtimeVersion": { "maxLength": 256, "type": "string" },
                      "startedAt": { "format": "date-time", "type": "string" },
                      "version": { "maxLength": 256, "type": "string" },
                    },
                    "required": [
                      "name",
                      "kind",
                      "instanceId",
                      "contractId",
                      "contractDigest",
                      "startedAt",
                      "publishIntervalMs",
                      "runtime",
                    ],
                    "type": "object",
                  },
                  "reportedStatus": {
                    "enum": ["healthy", "degraded", "unhealthy"],
                    "type": "string",
                  },
                  "sample": {
                    "properties": {
                      "id": {
                        "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$",
                        "type": "string",
                      },
                      "time": { "format": "date-time", "type": "string" },
                    },
                    "required": ["id", "time"],
                    "type": "object",
                  },
                  "summary": { "maxLength": 1024, "type": "string" },
                },
                "required": [
                  "sample",
                  "participant",
                  "reportedStatus",
                  "checks",
                ],
                "type": "object",
              },
              "observedAt": { "format": "date-time", "type": "string" },
              "reportedStatus": {
                "enum": ["healthy", "degraded", "unhealthy"],
                "type": "string",
              },
              "startedAt": { "format": "date-time", "type": "string" },
            },
            "required": [
              "instanceId",
              "deploymentId",
              "contractDigest",
              "reportedStatus",
              "effectiveStatus",
              "observedAt",
              "heartbeatDeadline",
              "ageMs",
              "startedAt",
              "latestSample",
            ],
            "type": "object",
          },
          "type": "array",
        },
        "participant": {
          "properties": {
            "contractId": { "type": "string" },
            "effectiveStatus": {
              "enum": ["healthy", "degraded", "unhealthy", "offline"],
              "type": "string",
            },
            "offlineInstances": { "minimum": 0, "type": "integer" },
            "onlineInstances": { "minimum": 0, "type": "integer" },
            "participantKind": {
              "enum": ["service", "device"],
              "type": "string",
            },
            "participantName": { "type": "string" },
          },
          "required": [
            "participantKind",
            "contractId",
            "participantName",
            "effectiveStatus",
            "onlineInstances",
            "offlineInstances",
          ],
          "type": "object",
        },
        "projection": {
          "properties": {
            "completeSince": { "format": "date-time", "type": "string" },
            "gapDetected": { "type": "boolean" },
            "lastStreamSequence": { "minimum": 0, "type": "integer" },
            "retainedFrom": { "format": "date-time", "type": "string" },
            "revision": { "minimum": 0, "type": "integer" },
          },
          "required": ["lastStreamSequence", "revision", "gapDetected"],
          "type": "object",
        },
      },
      "required": ["participant", "instances", "history", "asOf", "projection"],
      "type": "object",
    },
    "HealthMetricsRequest": {
      "properties": {
        "checkNames": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 64,
          "type": "array",
          "uniqueItems": true,
        },
        "contractId": { "maxLength": 256, "minLength": 1, "type": "string" },
        "end": { "format": "date-time", "type": "string" },
        "instanceIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
          "uniqueItems": true,
        },
        "participantKind": { "enum": ["service", "device"], "type": "string" },
        "start": { "format": "date-time", "type": "string" },
        "stepMs": { "minimum": 300000, "type": "integer" },
      },
      "required": ["start", "end", "stepMs", "participantKind", "contractId"],
      "type": "object",
    },
    "HealthMetricsResponse": {
      "properties": {
        "asOf": { "format": "date-time", "type": "string" },
        "projection": {
          "properties": {
            "completeSince": { "format": "date-time", "type": "string" },
            "gapDetected": { "type": "boolean" },
            "lastStreamSequence": { "minimum": 0, "type": "integer" },
            "retainedFrom": { "format": "date-time", "type": "string" },
            "revision": { "minimum": 0, "type": "integer" },
          },
          "required": ["lastStreamSequence", "revision", "gapDetected"],
          "type": "object",
        },
        "series": {
          "items": {
            "properties": {
              "buckets": {
                "items": {
                  "properties": {
                    "checks": {
                      "items": {
                        "properties": {
                          "failedCount": { "minimum": 0, "type": "integer" },
                          "latencyAverageMs": {
                            "minimum": 0,
                            "type": "number",
                          },
                          "latencyMaxMs": { "minimum": 0, "type": "number" },
                          "name": { "type": "string" },
                          "okCount": { "minimum": 0, "type": "integer" },
                          "sampleCount": { "minimum": 0, "type": "integer" },
                        },
                        "required": [
                          "name",
                          "sampleCount",
                          "okCount",
                          "failedCount",
                          "latencyAverageMs",
                          "latencyMaxMs",
                        ],
                        "type": "object",
                      },
                      "type": "array",
                    },
                    "degradedMs": { "minimum": 0, "type": "integer" },
                    "end": { "format": "date-time", "type": "string" },
                    "healthyMs": { "minimum": 0, "type": "integer" },
                    "observedMs": { "minimum": 0, "type": "integer" },
                    "offlineMs": { "minimum": 0, "type": "integer" },
                    "sampleCount": { "minimum": 0, "type": "integer" },
                    "start": { "format": "date-time", "type": "string" },
                    "unhealthyMs": { "minimum": 0, "type": "integer" },
                  },
                  "required": [
                    "start",
                    "end",
                    "observedMs",
                    "sampleCount",
                    "healthyMs",
                    "degradedMs",
                    "unhealthyMs",
                    "offlineMs",
                    "checks",
                  ],
                  "type": "object",
                },
                "type": "array",
              },
              "contractId": { "type": "string" },
              "instanceId": { "type": "string" },
              "participantKind": {
                "enum": ["service", "device"],
                "type": "string",
              },
            },
            "required": [
              "participantKind",
              "contractId",
              "instanceId",
              "buckets",
            ],
            "type": "object",
          },
          "type": "array",
        },
        "summary": {
          "properties": {
            "availability": { "maximum": 1, "minimum": 0, "type": "number" },
            "observedMs": { "minimum": 0, "type": "integer" },
            "onlineMs": { "minimum": 0, "type": "integer" },
            "sampleCount": { "minimum": 0, "type": "integer" },
            "transitions": { "minimum": 0, "type": "integer" },
          },
          "required": ["observedMs", "onlineMs", "sampleCount", "transitions"],
          "type": "object",
        },
      },
      "required": ["series", "summary", "asOf", "projection"],
      "type": "object",
    },
    "HealthProjectionDiagnostics": {
      "properties": {
        "completeSince": { "format": "date-time", "type": "string" },
        "gapDetected": { "type": "boolean" },
        "lastStreamSequence": { "minimum": 0, "type": "integer" },
        "retainedFrom": { "format": "date-time", "type": "string" },
        "revision": { "minimum": 0, "type": "integer" },
      },
      "required": ["lastStreamSequence", "revision", "gapDetected"],
      "type": "object",
    },
    "HealthQueryRequest": {
      "properties": {
        "contractIds": {
          "items": { "maxLength": 256, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
          "uniqueItems": true,
        },
        "deploymentIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
          "uniqueItems": true,
        },
        "limit": { "maximum": 200, "minimum": 1, "type": "integer" },
        "offset": { "minimum": 0, "type": "integer" },
        "participantKinds": {
          "items": { "enum": ["service", "device"], "type": "string" },
          "maxItems": 2,
          "type": "array",
          "uniqueItems": true,
        },
        "search": { "maxLength": 256, "type": "string" },
        "statuses": {
          "items": {
            "enum": ["healthy", "degraded", "unhealthy", "offline"],
            "type": "string",
          },
          "maxItems": 4,
          "type": "array",
          "uniqueItems": true,
        },
      },
      "type": "object",
    },
    "HealthQueryResponse": {
      "properties": {
        "asOf": { "format": "date-time", "type": "string" },
        "count": { "minimum": 0, "type": "integer" },
        "entries": {
          "items": {
            "properties": {
              "contractDigests": {
                "items": { "maxLength": 128, "minLength": 1, "type": "string" },
                "type": "array",
                "uniqueItems": true,
              },
              "contractId": { "type": "string" },
              "deploymentIds": {
                "items": { "maxLength": 128, "minLength": 1, "type": "string" },
                "type": "array",
                "uniqueItems": true,
              },
              "effectiveStatus": {
                "enum": ["healthy", "degraded", "unhealthy", "offline"],
                "type": "string",
              },
              "lastSeenAt": { "format": "date-time", "type": "string" },
              "offlineInstances": { "minimum": 0, "type": "integer" },
              "onlineInstances": { "minimum": 0, "type": "integer" },
              "participantKind": {
                "enum": ["service", "device"],
                "type": "string",
              },
              "participantName": { "type": "string" },
              "runtimes": { "items": { "type": "string" }, "type": "array" },
              "versions": { "items": { "type": "string" }, "type": "array" },
            },
            "required": [
              "participantKind",
              "contractId",
              "participantName",
              "effectiveStatus",
              "deploymentIds",
              "contractDigests",
              "onlineInstances",
              "offlineInstances",
              "lastSeenAt",
              "versions",
              "runtimes",
            ],
            "type": "object",
          },
          "type": "array",
        },
        "limit": { "minimum": 1, "type": "integer" },
        "offset": { "minimum": 0, "type": "integer" },
        "projection": {
          "properties": {
            "completeSince": { "format": "date-time", "type": "string" },
            "gapDetected": { "type": "boolean" },
            "lastStreamSequence": { "minimum": 0, "type": "integer" },
            "retainedFrom": { "format": "date-time", "type": "string" },
            "revision": { "minimum": 0, "type": "integer" },
          },
          "required": ["lastStreamSequence", "revision", "gapDetected"],
          "type": "object",
        },
      },
      "required": ["entries", "count", "limit", "offset", "asOf", "projection"],
      "type": "object",
    },
    "HealthStatusChangedEvent": {
      "properties": {
        "changedAt": { "format": "date-time", "type": "string" },
        "header": {
          "properties": {
            "id": { "maxLength": 128, "minLength": 1, "type": "string" },
            "time": { "format": "date-time", "type": "string" },
          },
          "required": ["id", "time"],
          "type": "object",
        },
        "lastSeenAt": { "format": "date-time", "type": "string" },
        "participant": {
          "properties": {
            "contractId": { "type": "string" },
            "deploymentId": { "type": "string" },
            "instanceId": { "type": "string" },
            "kind": { "enum": ["service", "device"], "type": "string" },
            "name": { "type": "string" },
          },
          "required": [
            "kind",
            "contractId",
            "instanceId",
            "deploymentId",
            "name",
          ],
          "type": "object",
        },
        "previousStatus": {
          "enum": ["healthy", "degraded", "unhealthy", "offline"],
          "type": "string",
        },
        "reason": {
          "enum": ["heartbeat-change", "heartbeat-resumed", "deadline-expired"],
          "type": "string",
        },
        "reportedStatus": {
          "enum": ["healthy", "degraded", "unhealthy"],
          "type": "string",
        },
        "status": {
          "enum": ["healthy", "degraded", "unhealthy", "offline"],
          "type": "string",
        },
        "summary": { "maxLength": 1024, "type": "string" },
      },
      "required": [
        "header",
        "participant",
        "previousStatus",
        "status",
        "reportedStatus",
        "reason",
        "changedAt",
        "lastSeenAt",
      ],
      "type": "object",
    },
    "HealthWatchFrame": {
      "anyOf": [{
        "properties": {
          "projectionRevision": { "minimum": 0, "type": "integer" },
          "type": { "const": "ready", "type": "string" },
        },
        "required": ["type", "projectionRevision"],
        "type": "object",
      }, {
        "properties": {
          "changes": {
            "items": {
              "properties": {
                "contractId": { "type": "string" },
                "deploymentId": { "type": "string" },
                "instanceId": { "type": "string" },
                "participantKind": {
                  "enum": ["service", "device"],
                  "type": "string",
                },
              },
              "required": [
                "participantKind",
                "contractId",
                "instanceId",
                "deploymentId",
              ],
              "type": "object",
            },
            "maxItems": 100,
            "type": "array",
          },
          "projectionRevision": { "minimum": 0, "type": "integer" },
          "type": { "const": "healthInvalidated", "type": "string" },
        },
        "required": ["type", "projectionRevision"],
        "type": "object",
      }],
    },
    "HealthWatchRequest": {
      "properties": {
        "contractIds": {
          "items": { "maxLength": 256, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
          "uniqueItems": true,
        },
        "deploymentIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
          "uniqueItems": true,
        },
        "instanceIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
          "uniqueItems": true,
        },
        "participantKinds": {
          "items": { "enum": ["service", "device"], "type": "string" },
          "maxItems": 2,
          "type": "array",
          "uniqueItems": true,
        },
      },
      "type": "object",
    },
    "NotFoundErrorData": {
      "properties": {
        "context": { "additionalProperties": true, "type": "object" },
        "id": { "minLength": 1, "type": "string" },
        "message": { "type": "string" },
        "resource": { "minLength": 1, "type": "string" },
        "traceId": { "type": "string" },
        "type": { "const": "NotFoundError", "type": "string" },
      },
      "required": ["type", "resource", "id", "message"],
      "type": "object",
    },
  },
  "version": "1.0.0",
} as const;
