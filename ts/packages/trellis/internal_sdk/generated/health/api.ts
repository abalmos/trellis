// Generated from ./rust/crates/runtime/.trellis/artifacts/apis/trellis.health@v1.json

export const API_ID = "trellis.health@v1" as const;
export const API_DIGEST =
  "8GFaxY8gvKvagqRIF43iaRpUN_40ukCylk_nwmFaZkg" as const;
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
  "description":
    "Trellis-managed participant health projection and operational history.",
  "displayName": "Trellis Health",
  "errors": {
    "NotFoundError": { "schema": { "schema": "NotFoundErrorData" } },
    "UnexpectedError": {},
    "ValidationError": {},
  },
  "events": {
    "Health.StatusChanged": {
      "event": { "schema": "HealthStatusChangedEvent" },
      "version": "v1",
    },
  },
  "exports": { "schemas": ["HealthHeartbeatSample"] },
  "feeds": {
    "Health.Watch": {
      "event": { "schema": "HealthWatchFrame" },
      "input": { "schema": "HealthWatchRequest" },
      "version": "v1",
    },
  },
  "format": "trellis.api.v1",
  "id": "trellis.health@v1",
  "rpc": {
    "Health.Inspect": {
      "errors": ["NotFoundError", "UnexpectedError", "ValidationError"],
      "input": { "schema": "HealthInspectRequest" },
      "output": { "schema": "HealthInspectResponse" },
      "version": "v1",
    },
    "Health.Metrics": {
      "errors": ["UnexpectedError", "ValidationError"],
      "input": { "schema": "HealthMetricsRequest" },
      "output": { "schema": "HealthMetricsResponse" },
      "version": "v1",
    },
    "Health.Query": {
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
              "info": { "properties": {}, "type": "object" },
              "latencyMs": {
                "maximum": 3600000,
                "minimum": 0,
                "type": "number",
              },
              "name": { "maxLength": 128, "minLength": 1, "type": "string" },
              "status": {
                "anyOf": [{ "const": "ok", "type": "string" }, {
                  "const": "failed",
                  "type": "string",
                }],
              },
              "summary": { "maxLength": 1024, "type": "string" },
            },
            "required": ["latencyMs", "name", "status"],
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
            "info": { "properties": {}, "type": "object" },
            "instanceId": {
              "maxLength": 128,
              "minLength": 1,
              "type": "string",
            },
            "kind": {
              "anyOf": [{ "const": "service", "type": "string" }, {
                "const": "device",
                "type": "string",
              }],
            },
            "name": { "maxLength": 256, "minLength": 1, "type": "string" },
            "publishIntervalMs": {
              "maximum": 600000,
              "minimum": 1000,
              "type": "integer",
            },
            "runtime": {
              "anyOf": [
                { "const": "deno", "type": "string" },
                { "const": "node", "type": "string" },
                { "const": "rust", "type": "string" },
                { "const": "unknown", "type": "string" },
              ],
            },
            "runtimeVersion": { "maxLength": 256, "type": "string" },
            "startedAt": { "format": "date-time", "type": "string" },
            "version": { "maxLength": 256, "type": "string" },
          },
          "required": [
            "contractDigest",
            "contractId",
            "instanceId",
            "kind",
            "name",
            "publishIntervalMs",
            "runtime",
            "startedAt",
          ],
          "type": "object",
        },
        "reportedStatus": {
          "anyOf": [{ "const": "healthy", "type": "string" }, {
            "const": "degraded",
            "type": "string",
          }, { "const": "unhealthy", "type": "string" }],
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
      "required": ["checks", "participant", "reportedStatus", "sample"],
      "type": "object",
    },
    "HealthHeartbeatSamplechecksItem": {
      "properties": {
        "error": { "maxLength": 1024, "type": "string" },
        "info": { "properties": {}, "type": "object" },
        "latencyMs": { "maximum": 3600000, "minimum": 0, "type": "number" },
        "name": { "maxLength": 128, "minLength": 1, "type": "string" },
        "status": {
          "anyOf": [{ "const": "ok", "type": "string" }, {
            "const": "failed",
            "type": "string",
          }],
        },
        "summary": { "maxLength": 1024, "type": "string" },
      },
      "required": ["latencyMs", "name", "status"],
      "type": "object",
    },
    "HealthHeartbeatSamplechecksIteminfo": {
      "properties": {},
      "type": "object",
    },
    "HealthHeartbeatSampleparticipant": {
      "properties": {
        "contractDigest": {
          "maxLength": 256,
          "minLength": 1,
          "type": "string",
        },
        "contractId": { "maxLength": 256, "minLength": 1, "type": "string" },
        "info": { "properties": {}, "type": "object" },
        "instanceId": { "maxLength": 128, "minLength": 1, "type": "string" },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "name": { "maxLength": 256, "minLength": 1, "type": "string" },
        "publishIntervalMs": {
          "maximum": 600000,
          "minimum": 1000,
          "type": "integer",
        },
        "runtime": {
          "anyOf": [
            { "const": "deno", "type": "string" },
            { "const": "node", "type": "string" },
            { "const": "rust", "type": "string" },
            { "const": "unknown", "type": "string" },
          ],
        },
        "runtimeVersion": { "maxLength": 256, "type": "string" },
        "startedAt": { "format": "date-time", "type": "string" },
        "version": { "maxLength": 256, "type": "string" },
      },
      "required": [
        "contractDigest",
        "contractId",
        "instanceId",
        "kind",
        "name",
        "publishIntervalMs",
        "runtime",
        "startedAt",
      ],
      "type": "object",
    },
    "HealthHeartbeatSampleparticipantinfo": {
      "properties": {},
      "type": "object",
    },
    "HealthHeartbeatSamplesample": {
      "properties": {
        "id": { "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$", "type": "string" },
        "time": { "format": "date-time", "type": "string" },
      },
      "required": ["id", "time"],
      "type": "object",
    },
    "HealthInspectRequest": {
      "properties": {
        "contractId": { "maxLength": 256, "minLength": 1, "type": "string" },
        "historyLimit": { "maximum": 500, "minimum": 1, "type": "integer" },
        "historySince": { "format": "date-time", "type": "string" },
        "instanceId": { "maxLength": 128, "minLength": 1, "type": "string" },
        "participantKind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
      },
      "required": ["contractId", "participantKind"],
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
                    "status": {
                      "anyOf": [{ "const": "ok", "type": "string" }, {
                        "const": "failed",
                        "type": "string",
                      }],
                    },
                  },
                  "required": ["name", "status"],
                  "type": "object",
                },
                "type": "array",
              },
              "effectiveStatus": {
                "anyOf": [
                  { "const": "healthy", "type": "string" },
                  { "const": "degraded", "type": "string" },
                  { "const": "unhealthy", "type": "string" },
                  { "const": "offline", "type": "string" },
                ],
              },
              "endedAt": { "format": "date-time", "type": "string" },
              "instanceId": { "type": "string" },
              "intervalId": { "minimum": 1, "type": "integer" },
              "reason": {
                "anyOf": [
                  { "const": "first-sample", "type": "string" },
                  { "const": "heartbeat-change", "type": "string" },
                  { "const": "heartbeat-resumed", "type": "string" },
                  { "const": "deadline-expired", "type": "string" },
                ],
              },
              "reportedStatus": {
                "anyOf": [{ "const": "healthy", "type": "string" }, {
                  "const": "degraded",
                  "type": "string",
                }, { "const": "unhealthy", "type": "string" }],
              },
              "startedAt": { "format": "date-time", "type": "string" },
            },
            "required": [
              "checks",
              "effectiveStatus",
              "instanceId",
              "intervalId",
              "reason",
              "reportedStatus",
              "startedAt",
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
                "anyOf": [
                  { "const": "healthy", "type": "string" },
                  { "const": "degraded", "type": "string" },
                  { "const": "unhealthy", "type": "string" },
                  { "const": "offline", "type": "string" },
                ],
              },
              "heartbeatDeadline": { "format": "date-time", "type": "string" },
              "instanceId": { "type": "string" },
              "latestSample": {
                "properties": {
                  "checks": {
                    "items": {
                      "properties": {
                        "error": { "maxLength": 1024, "type": "string" },
                        "info": { "properties": {}, "type": "object" },
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
                          "anyOf": [{ "const": "ok", "type": "string" }, {
                            "const": "failed",
                            "type": "string",
                          }],
                        },
                        "summary": { "maxLength": 1024, "type": "string" },
                      },
                      "required": ["latencyMs", "name", "status"],
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
                      "info": { "properties": {}, "type": "object" },
                      "instanceId": {
                        "maxLength": 128,
                        "minLength": 1,
                        "type": "string",
                      },
                      "kind": {
                        "anyOf": [{ "const": "service", "type": "string" }, {
                          "const": "device",
                          "type": "string",
                        }],
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
                        "anyOf": [
                          { "const": "deno", "type": "string" },
                          { "const": "node", "type": "string" },
                          { "const": "rust", "type": "string" },
                          { "const": "unknown", "type": "string" },
                        ],
                      },
                      "runtimeVersion": { "maxLength": 256, "type": "string" },
                      "startedAt": { "format": "date-time", "type": "string" },
                      "version": { "maxLength": 256, "type": "string" },
                    },
                    "required": [
                      "contractDigest",
                      "contractId",
                      "instanceId",
                      "kind",
                      "name",
                      "publishIntervalMs",
                      "runtime",
                      "startedAt",
                    ],
                    "type": "object",
                  },
                  "reportedStatus": {
                    "anyOf": [{ "const": "healthy", "type": "string" }, {
                      "const": "degraded",
                      "type": "string",
                    }, { "const": "unhealthy", "type": "string" }],
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
                  "checks",
                  "participant",
                  "reportedStatus",
                  "sample",
                ],
                "type": "object",
              },
              "observedAt": { "format": "date-time", "type": "string" },
              "reportedStatus": {
                "anyOf": [{ "const": "healthy", "type": "string" }, {
                  "const": "degraded",
                  "type": "string",
                }, { "const": "unhealthy", "type": "string" }],
              },
              "startedAt": { "format": "date-time", "type": "string" },
            },
            "required": [
              "ageMs",
              "contractDigest",
              "deploymentId",
              "effectiveStatus",
              "heartbeatDeadline",
              "instanceId",
              "latestSample",
              "observedAt",
              "reportedStatus",
              "startedAt",
            ],
            "type": "object",
          },
          "type": "array",
        },
        "participant": {
          "properties": {
            "contractId": { "type": "string" },
            "effectiveStatus": {
              "anyOf": [
                { "const": "healthy", "type": "string" },
                { "const": "degraded", "type": "string" },
                { "const": "unhealthy", "type": "string" },
                { "const": "offline", "type": "string" },
              ],
            },
            "offlineInstances": { "minimum": 0, "type": "integer" },
            "onlineInstances": { "minimum": 0, "type": "integer" },
            "participantKind": {
              "anyOf": [{ "const": "service", "type": "string" }, {
                "const": "device",
                "type": "string",
              }],
            },
            "participantName": { "type": "string" },
          },
          "required": [
            "contractId",
            "effectiveStatus",
            "offlineInstances",
            "onlineInstances",
            "participantKind",
            "participantName",
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
          "required": ["gapDetected", "lastStreamSequence", "revision"],
          "type": "object",
        },
      },
      "required": ["asOf", "history", "instances", "participant", "projection"],
      "type": "object",
    },
    "HealthInspectResponsehistoryItem": {
      "properties": {
        "checks": {
          "items": {
            "properties": {
              "name": { "type": "string" },
              "status": {
                "anyOf": [{ "const": "ok", "type": "string" }, {
                  "const": "failed",
                  "type": "string",
                }],
              },
            },
            "required": ["name", "status"],
            "type": "object",
          },
          "type": "array",
        },
        "effectiveStatus": {
          "anyOf": [
            { "const": "healthy", "type": "string" },
            { "const": "degraded", "type": "string" },
            { "const": "unhealthy", "type": "string" },
            { "const": "offline", "type": "string" },
          ],
        },
        "endedAt": { "format": "date-time", "type": "string" },
        "instanceId": { "type": "string" },
        "intervalId": { "minimum": 1, "type": "integer" },
        "reason": {
          "anyOf": [
            { "const": "first-sample", "type": "string" },
            { "const": "heartbeat-change", "type": "string" },
            { "const": "heartbeat-resumed", "type": "string" },
            { "const": "deadline-expired", "type": "string" },
          ],
        },
        "reportedStatus": {
          "anyOf": [{ "const": "healthy", "type": "string" }, {
            "const": "degraded",
            "type": "string",
          }, { "const": "unhealthy", "type": "string" }],
        },
        "startedAt": { "format": "date-time", "type": "string" },
      },
      "required": [
        "checks",
        "effectiveStatus",
        "instanceId",
        "intervalId",
        "reason",
        "reportedStatus",
        "startedAt",
      ],
      "type": "object",
    },
    "HealthInspectResponsehistoryItemchecksItem": {
      "properties": {
        "name": { "type": "string" },
        "status": {
          "anyOf": [{ "const": "ok", "type": "string" }, {
            "const": "failed",
            "type": "string",
          }],
        },
      },
      "required": ["name", "status"],
      "type": "object",
    },
    "HealthInspectResponseinstancesItem": {
      "properties": {
        "ageMs": { "minimum": 0, "type": "integer" },
        "contractDigest": { "type": "string" },
        "deploymentId": { "type": "string" },
        "effectiveStatus": {
          "anyOf": [
            { "const": "healthy", "type": "string" },
            { "const": "degraded", "type": "string" },
            { "const": "unhealthy", "type": "string" },
            { "const": "offline", "type": "string" },
          ],
        },
        "heartbeatDeadline": { "format": "date-time", "type": "string" },
        "instanceId": { "type": "string" },
        "latestSample": {
          "properties": {
            "checks": {
              "items": {
                "properties": {
                  "error": { "maxLength": 1024, "type": "string" },
                  "info": { "properties": {}, "type": "object" },
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
                    "anyOf": [{ "const": "ok", "type": "string" }, {
                      "const": "failed",
                      "type": "string",
                    }],
                  },
                  "summary": { "maxLength": 1024, "type": "string" },
                },
                "required": ["latencyMs", "name", "status"],
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
                "info": { "properties": {}, "type": "object" },
                "instanceId": {
                  "maxLength": 128,
                  "minLength": 1,
                  "type": "string",
                },
                "kind": {
                  "anyOf": [{ "const": "service", "type": "string" }, {
                    "const": "device",
                    "type": "string",
                  }],
                },
                "name": { "maxLength": 256, "minLength": 1, "type": "string" },
                "publishIntervalMs": {
                  "maximum": 600000,
                  "minimum": 1000,
                  "type": "integer",
                },
                "runtime": {
                  "anyOf": [
                    { "const": "deno", "type": "string" },
                    { "const": "node", "type": "string" },
                    { "const": "rust", "type": "string" },
                    { "const": "unknown", "type": "string" },
                  ],
                },
                "runtimeVersion": { "maxLength": 256, "type": "string" },
                "startedAt": { "format": "date-time", "type": "string" },
                "version": { "maxLength": 256, "type": "string" },
              },
              "required": [
                "contractDigest",
                "contractId",
                "instanceId",
                "kind",
                "name",
                "publishIntervalMs",
                "runtime",
                "startedAt",
              ],
              "type": "object",
            },
            "reportedStatus": {
              "anyOf": [{ "const": "healthy", "type": "string" }, {
                "const": "degraded",
                "type": "string",
              }, { "const": "unhealthy", "type": "string" }],
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
          "required": ["checks", "participant", "reportedStatus", "sample"],
          "type": "object",
        },
        "observedAt": { "format": "date-time", "type": "string" },
        "reportedStatus": {
          "anyOf": [{ "const": "healthy", "type": "string" }, {
            "const": "degraded",
            "type": "string",
          }, { "const": "unhealthy", "type": "string" }],
        },
        "startedAt": { "format": "date-time", "type": "string" },
      },
      "required": [
        "ageMs",
        "contractDigest",
        "deploymentId",
        "effectiveStatus",
        "heartbeatDeadline",
        "instanceId",
        "latestSample",
        "observedAt",
        "reportedStatus",
        "startedAt",
      ],
      "type": "object",
    },
    "HealthInspectResponseinstancesItemlatestSample": {
      "properties": {
        "checks": {
          "items": {
            "properties": {
              "error": { "maxLength": 1024, "type": "string" },
              "info": { "properties": {}, "type": "object" },
              "latencyMs": {
                "maximum": 3600000,
                "minimum": 0,
                "type": "number",
              },
              "name": { "maxLength": 128, "minLength": 1, "type": "string" },
              "status": {
                "anyOf": [{ "const": "ok", "type": "string" }, {
                  "const": "failed",
                  "type": "string",
                }],
              },
              "summary": { "maxLength": 1024, "type": "string" },
            },
            "required": ["latencyMs", "name", "status"],
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
            "info": { "properties": {}, "type": "object" },
            "instanceId": {
              "maxLength": 128,
              "minLength": 1,
              "type": "string",
            },
            "kind": {
              "anyOf": [{ "const": "service", "type": "string" }, {
                "const": "device",
                "type": "string",
              }],
            },
            "name": { "maxLength": 256, "minLength": 1, "type": "string" },
            "publishIntervalMs": {
              "maximum": 600000,
              "minimum": 1000,
              "type": "integer",
            },
            "runtime": {
              "anyOf": [
                { "const": "deno", "type": "string" },
                { "const": "node", "type": "string" },
                { "const": "rust", "type": "string" },
                { "const": "unknown", "type": "string" },
              ],
            },
            "runtimeVersion": { "maxLength": 256, "type": "string" },
            "startedAt": { "format": "date-time", "type": "string" },
            "version": { "maxLength": 256, "type": "string" },
          },
          "required": [
            "contractDigest",
            "contractId",
            "instanceId",
            "kind",
            "name",
            "publishIntervalMs",
            "runtime",
            "startedAt",
          ],
          "type": "object",
        },
        "reportedStatus": {
          "anyOf": [{ "const": "healthy", "type": "string" }, {
            "const": "degraded",
            "type": "string",
          }, { "const": "unhealthy", "type": "string" }],
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
      "required": ["checks", "participant", "reportedStatus", "sample"],
      "type": "object",
    },
    "HealthInspectResponseinstancesItemlatestSamplechecksItem": {
      "properties": {
        "error": { "maxLength": 1024, "type": "string" },
        "info": { "properties": {}, "type": "object" },
        "latencyMs": { "maximum": 3600000, "minimum": 0, "type": "number" },
        "name": { "maxLength": 128, "minLength": 1, "type": "string" },
        "status": {
          "anyOf": [{ "const": "ok", "type": "string" }, {
            "const": "failed",
            "type": "string",
          }],
        },
        "summary": { "maxLength": 1024, "type": "string" },
      },
      "required": ["latencyMs", "name", "status"],
      "type": "object",
    },
    "HealthInspectResponseinstancesItemlatestSamplechecksIteminfo": {
      "properties": {},
      "type": "object",
    },
    "HealthInspectResponseinstancesItemlatestSampleparticipant": {
      "properties": {
        "contractDigest": {
          "maxLength": 256,
          "minLength": 1,
          "type": "string",
        },
        "contractId": { "maxLength": 256, "minLength": 1, "type": "string" },
        "info": { "properties": {}, "type": "object" },
        "instanceId": { "maxLength": 128, "minLength": 1, "type": "string" },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "name": { "maxLength": 256, "minLength": 1, "type": "string" },
        "publishIntervalMs": {
          "maximum": 600000,
          "minimum": 1000,
          "type": "integer",
        },
        "runtime": {
          "anyOf": [
            { "const": "deno", "type": "string" },
            { "const": "node", "type": "string" },
            { "const": "rust", "type": "string" },
            { "const": "unknown", "type": "string" },
          ],
        },
        "runtimeVersion": { "maxLength": 256, "type": "string" },
        "startedAt": { "format": "date-time", "type": "string" },
        "version": { "maxLength": 256, "type": "string" },
      },
      "required": [
        "contractDigest",
        "contractId",
        "instanceId",
        "kind",
        "name",
        "publishIntervalMs",
        "runtime",
        "startedAt",
      ],
      "type": "object",
    },
    "HealthInspectResponseinstancesItemlatestSampleparticipantinfo": {
      "properties": {},
      "type": "object",
    },
    "HealthInspectResponseinstancesItemlatestSamplesample": {
      "properties": {
        "id": { "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$", "type": "string" },
        "time": { "format": "date-time", "type": "string" },
      },
      "required": ["id", "time"],
      "type": "object",
    },
    "HealthInspectResponseparticipant": {
      "properties": {
        "contractId": { "type": "string" },
        "effectiveStatus": {
          "anyOf": [
            { "const": "healthy", "type": "string" },
            { "const": "degraded", "type": "string" },
            { "const": "unhealthy", "type": "string" },
            { "const": "offline", "type": "string" },
          ],
        },
        "offlineInstances": { "minimum": 0, "type": "integer" },
        "onlineInstances": { "minimum": 0, "type": "integer" },
        "participantKind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "participantName": { "type": "string" },
      },
      "required": [
        "contractId",
        "effectiveStatus",
        "offlineInstances",
        "onlineInstances",
        "participantKind",
        "participantName",
      ],
      "type": "object",
    },
    "HealthInspectResponseprojection": {
      "properties": {
        "completeSince": { "format": "date-time", "type": "string" },
        "gapDetected": { "type": "boolean" },
        "lastStreamSequence": { "minimum": 0, "type": "integer" },
        "retainedFrom": { "format": "date-time", "type": "string" },
        "revision": { "minimum": 0, "type": "integer" },
      },
      "required": ["gapDetected", "lastStreamSequence", "revision"],
      "type": "object",
    },
    "HealthMetricsRequest": {
      "properties": {
        "checkNames": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 64,
          "type": "array",
        },
        "contractId": { "maxLength": 256, "minLength": 1, "type": "string" },
        "end": { "format": "date-time", "type": "string" },
        "instanceIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
        },
        "participantKind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "start": { "format": "date-time", "type": "string" },
        "stepMs": { "minimum": 300000, "type": "integer" },
      },
      "required": ["contractId", "end", "participantKind", "start", "stepMs"],
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
          "required": ["gapDetected", "lastStreamSequence", "revision"],
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
                          "failedCount",
                          "latencyAverageMs",
                          "latencyMaxMs",
                          "name",
                          "okCount",
                          "sampleCount",
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
                    "checks",
                    "degradedMs",
                    "end",
                    "healthyMs",
                    "observedMs",
                    "offlineMs",
                    "sampleCount",
                    "start",
                    "unhealthyMs",
                  ],
                  "type": "object",
                },
                "type": "array",
              },
              "contractId": { "type": "string" },
              "instanceId": { "type": "string" },
              "participantKind": {
                "anyOf": [{ "const": "service", "type": "string" }, {
                  "const": "device",
                  "type": "string",
                }],
              },
            },
            "required": [
              "buckets",
              "contractId",
              "instanceId",
              "participantKind",
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
      "required": ["asOf", "projection", "series", "summary"],
      "type": "object",
    },
    "HealthMetricsResponseprojection": {
      "properties": {
        "completeSince": { "format": "date-time", "type": "string" },
        "gapDetected": { "type": "boolean" },
        "lastStreamSequence": { "minimum": 0, "type": "integer" },
        "retainedFrom": { "format": "date-time", "type": "string" },
        "revision": { "minimum": 0, "type": "integer" },
      },
      "required": ["gapDetected", "lastStreamSequence", "revision"],
      "type": "object",
    },
    "HealthMetricsResponseseriesItem": {
      "properties": {
        "buckets": {
          "items": {
            "properties": {
              "checks": {
                "items": {
                  "properties": {
                    "failedCount": { "minimum": 0, "type": "integer" },
                    "latencyAverageMs": { "minimum": 0, "type": "number" },
                    "latencyMaxMs": { "minimum": 0, "type": "number" },
                    "name": { "type": "string" },
                    "okCount": { "minimum": 0, "type": "integer" },
                    "sampleCount": { "minimum": 0, "type": "integer" },
                  },
                  "required": [
                    "failedCount",
                    "latencyAverageMs",
                    "latencyMaxMs",
                    "name",
                    "okCount",
                    "sampleCount",
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
              "checks",
              "degradedMs",
              "end",
              "healthyMs",
              "observedMs",
              "offlineMs",
              "sampleCount",
              "start",
              "unhealthyMs",
            ],
            "type": "object",
          },
          "type": "array",
        },
        "contractId": { "type": "string" },
        "instanceId": { "type": "string" },
        "participantKind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
      },
      "required": ["buckets", "contractId", "instanceId", "participantKind"],
      "type": "object",
    },
    "HealthMetricsResponseseriesItembucketsItem": {
      "properties": {
        "checks": {
          "items": {
            "properties": {
              "failedCount": { "minimum": 0, "type": "integer" },
              "latencyAverageMs": { "minimum": 0, "type": "number" },
              "latencyMaxMs": { "minimum": 0, "type": "number" },
              "name": { "type": "string" },
              "okCount": { "minimum": 0, "type": "integer" },
              "sampleCount": { "minimum": 0, "type": "integer" },
            },
            "required": [
              "failedCount",
              "latencyAverageMs",
              "latencyMaxMs",
              "name",
              "okCount",
              "sampleCount",
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
        "checks",
        "degradedMs",
        "end",
        "healthyMs",
        "observedMs",
        "offlineMs",
        "sampleCount",
        "start",
        "unhealthyMs",
      ],
      "type": "object",
    },
    "HealthMetricsResponseseriesItembucketsItemchecksItem": {
      "properties": {
        "failedCount": { "minimum": 0, "type": "integer" },
        "latencyAverageMs": { "minimum": 0, "type": "number" },
        "latencyMaxMs": { "minimum": 0, "type": "number" },
        "name": { "type": "string" },
        "okCount": { "minimum": 0, "type": "integer" },
        "sampleCount": { "minimum": 0, "type": "integer" },
      },
      "required": [
        "failedCount",
        "latencyAverageMs",
        "latencyMaxMs",
        "name",
        "okCount",
        "sampleCount",
      ],
      "type": "object",
    },
    "HealthMetricsResponsesummary": {
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
    "HealthProjectionDiagnostics": {
      "properties": {
        "completeSince": { "format": "date-time", "type": "string" },
        "gapDetected": { "type": "boolean" },
        "lastStreamSequence": { "minimum": 0, "type": "integer" },
        "retainedFrom": { "format": "date-time", "type": "string" },
        "revision": { "minimum": 0, "type": "integer" },
      },
      "required": ["gapDetected", "lastStreamSequence", "revision"],
      "type": "object",
    },
    "HealthQueryRequest": {
      "properties": {
        "contractIds": {
          "items": { "maxLength": 256, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
        },
        "deploymentIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
        },
        "limit": { "maximum": 200, "minimum": 1, "type": "integer" },
        "offset": { "minimum": 0, "type": "integer" },
        "participantKinds": {
          "items": {
            "anyOf": [{ "const": "service", "type": "string" }, {
              "const": "device",
              "type": "string",
            }],
          },
          "maxItems": 2,
          "type": "array",
        },
        "search": { "maxLength": 256, "type": "string" },
        "statuses": {
          "items": {
            "anyOf": [
              { "const": "healthy", "type": "string" },
              { "const": "degraded", "type": "string" },
              { "const": "unhealthy", "type": "string" },
              { "const": "offline", "type": "string" },
            ],
          },
          "maxItems": 4,
          "type": "array",
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
              },
              "contractId": { "type": "string" },
              "deploymentIds": {
                "items": { "maxLength": 128, "minLength": 1, "type": "string" },
                "type": "array",
              },
              "effectiveStatus": {
                "anyOf": [
                  { "const": "healthy", "type": "string" },
                  { "const": "degraded", "type": "string" },
                  { "const": "unhealthy", "type": "string" },
                  { "const": "offline", "type": "string" },
                ],
              },
              "lastSeenAt": { "format": "date-time", "type": "string" },
              "offlineInstances": { "minimum": 0, "type": "integer" },
              "onlineInstances": { "minimum": 0, "type": "integer" },
              "participantKind": {
                "anyOf": [{ "const": "service", "type": "string" }, {
                  "const": "device",
                  "type": "string",
                }],
              },
              "participantName": { "type": "string" },
              "runtimes": { "items": { "type": "string" }, "type": "array" },
              "versions": { "items": { "type": "string" }, "type": "array" },
            },
            "required": [
              "contractDigests",
              "contractId",
              "deploymentIds",
              "effectiveStatus",
              "lastSeenAt",
              "offlineInstances",
              "onlineInstances",
              "participantKind",
              "participantName",
              "runtimes",
              "versions",
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
          "required": ["gapDetected", "lastStreamSequence", "revision"],
          "type": "object",
        },
      },
      "required": ["asOf", "count", "entries", "limit", "offset", "projection"],
      "type": "object",
    },
    "HealthQueryResponseentriesItem": {
      "properties": {
        "contractDigests": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "type": "array",
        },
        "contractId": { "type": "string" },
        "deploymentIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "type": "array",
        },
        "effectiveStatus": {
          "anyOf": [
            { "const": "healthy", "type": "string" },
            { "const": "degraded", "type": "string" },
            { "const": "unhealthy", "type": "string" },
            { "const": "offline", "type": "string" },
          ],
        },
        "lastSeenAt": { "format": "date-time", "type": "string" },
        "offlineInstances": { "minimum": 0, "type": "integer" },
        "onlineInstances": { "minimum": 0, "type": "integer" },
        "participantKind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "participantName": { "type": "string" },
        "runtimes": { "items": { "type": "string" }, "type": "array" },
        "versions": { "items": { "type": "string" }, "type": "array" },
      },
      "required": [
        "contractDigests",
        "contractId",
        "deploymentIds",
        "effectiveStatus",
        "lastSeenAt",
        "offlineInstances",
        "onlineInstances",
        "participantKind",
        "participantName",
        "runtimes",
        "versions",
      ],
      "type": "object",
    },
    "HealthQueryResponseprojection": {
      "properties": {
        "completeSince": { "format": "date-time", "type": "string" },
        "gapDetected": { "type": "boolean" },
        "lastStreamSequence": { "minimum": 0, "type": "integer" },
        "retainedFrom": { "format": "date-time", "type": "string" },
        "revision": { "minimum": 0, "type": "integer" },
      },
      "required": ["gapDetected", "lastStreamSequence", "revision"],
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
            "kind": {
              "anyOf": [{ "const": "service", "type": "string" }, {
                "const": "device",
                "type": "string",
              }],
            },
            "name": { "type": "string" },
          },
          "required": [
            "contractId",
            "deploymentId",
            "instanceId",
            "kind",
            "name",
          ],
          "type": "object",
        },
        "previousStatus": {
          "anyOf": [
            { "const": "healthy", "type": "string" },
            { "const": "degraded", "type": "string" },
            { "const": "unhealthy", "type": "string" },
            { "const": "offline", "type": "string" },
          ],
        },
        "reason": {
          "anyOf": [{ "const": "heartbeat-change", "type": "string" }, {
            "const": "heartbeat-resumed",
            "type": "string",
          }, { "const": "deadline-expired", "type": "string" }],
        },
        "reportedStatus": {
          "anyOf": [{ "const": "healthy", "type": "string" }, {
            "const": "degraded",
            "type": "string",
          }, { "const": "unhealthy", "type": "string" }],
        },
        "status": {
          "anyOf": [
            { "const": "healthy", "type": "string" },
            { "const": "degraded", "type": "string" },
            { "const": "unhealthy", "type": "string" },
            { "const": "offline", "type": "string" },
          ],
        },
        "summary": { "maxLength": 1024, "type": "string" },
      },
      "required": [
        "changedAt",
        "header",
        "lastSeenAt",
        "participant",
        "previousStatus",
        "reason",
        "reportedStatus",
        "status",
      ],
      "type": "object",
    },
    "HealthStatusChangedEventheader": {
      "properties": {
        "id": { "maxLength": 128, "minLength": 1, "type": "string" },
        "time": { "format": "date-time", "type": "string" },
      },
      "required": ["id", "time"],
      "type": "object",
    },
    "HealthStatusChangedEventparticipant": {
      "properties": {
        "contractId": { "type": "string" },
        "deploymentId": { "type": "string" },
        "instanceId": { "type": "string" },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "name": { "type": "string" },
      },
      "required": ["contractId", "deploymentId", "instanceId", "kind", "name"],
      "type": "object",
    },
    "HealthWatchFrame": {
      "anyOf": [{
        "properties": {
          "projectionRevision": { "minimum": 0, "type": "integer" },
          "type": { "const": "ready", "type": "string" },
        },
        "required": ["projectionRevision", "type"],
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
                  "anyOf": [{ "const": "service", "type": "string" }, {
                    "const": "device",
                    "type": "string",
                  }],
                },
              },
              "required": [
                "contractId",
                "deploymentId",
                "instanceId",
                "participantKind",
              ],
              "type": "object",
            },
            "maxItems": 100,
            "type": "array",
          },
          "projectionRevision": { "minimum": 0, "type": "integer" },
          "type": { "const": "healthInvalidated", "type": "string" },
        },
        "required": ["projectionRevision", "type"],
        "type": "object",
      }],
    },
    "HealthWatchFrameValue1": {
      "properties": {
        "projectionRevision": { "minimum": 0, "type": "integer" },
        "type": { "const": "ready", "type": "string" },
      },
      "required": ["projectionRevision", "type"],
      "type": "object",
    },
    "HealthWatchFrameValue2": {
      "properties": {
        "changes": {
          "items": {
            "properties": {
              "contractId": { "type": "string" },
              "deploymentId": { "type": "string" },
              "instanceId": { "type": "string" },
              "participantKind": {
                "anyOf": [{ "const": "service", "type": "string" }, {
                  "const": "device",
                  "type": "string",
                }],
              },
            },
            "required": [
              "contractId",
              "deploymentId",
              "instanceId",
              "participantKind",
            ],
            "type": "object",
          },
          "maxItems": 100,
          "type": "array",
        },
        "projectionRevision": { "minimum": 0, "type": "integer" },
        "type": { "const": "healthInvalidated", "type": "string" },
      },
      "required": ["projectionRevision", "type"],
      "type": "object",
    },
    "HealthWatchFrameValue2changesItem": {
      "properties": {
        "contractId": { "type": "string" },
        "deploymentId": { "type": "string" },
        "instanceId": { "type": "string" },
        "participantKind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
      },
      "required": [
        "contractId",
        "deploymentId",
        "instanceId",
        "participantKind",
      ],
      "type": "object",
    },
    "HealthWatchRequest": {
      "properties": {
        "contractIds": {
          "items": { "maxLength": 256, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
        },
        "deploymentIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
        },
        "instanceIds": {
          "items": { "maxLength": 128, "minLength": 1, "type": "string" },
          "maxItems": 100,
          "type": "array",
        },
        "participantKinds": {
          "items": {
            "anyOf": [{ "const": "service", "type": "string" }, {
              "const": "device",
              "type": "string",
            }],
          },
          "maxItems": 2,
          "type": "array",
        },
      },
      "type": "object",
    },
    "NotFoundErrorData": {
      "properties": {
        "context": { "properties": {}, "type": "object" },
        "id": { "minLength": 1, "type": "string" },
        "message": { "type": "string" },
        "resource": { "minLength": 1, "type": "string" },
        "traceId": { "type": "string" },
        "type": { "const": "NotFoundError", "type": "string" },
      },
      "required": ["id", "message", "resource", "type"],
      "type": "object",
    },
    "NotFoundErrorDatacontext": { "properties": {}, "type": "object" },
  },
  "version": "1.0.0",
} as const;
