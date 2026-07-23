// Generated from ./generated/apis/trellis.auth@v1.json
export const AuthCapabilitiesListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "sourceApi": { "minLength": 1, "type": "string" },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthCapabilitiesListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "allows": {
            "items": {
              "properties": {
                "action": {
                  "enum": [
                    "call",
                    "invoke",
                    "observe",
                    "cancel",
                    "control",
                    "publish",
                    "subscribe",
                    "read",
                    "write",
                    "delete",
                    "submit",
                    "process",
                    "consume",
                  ],
                },
                "target": {
                  "anyOf": [{
                    "properties": {
                      "api": { "minLength": 1, "type": "string" },
                      "kind": { "const": "apiSurface" },
                      "name": { "minLength": 1, "type": "string" },
                      "surface": {
                        "enum": ["rpc", "operation", "event", "feed", "state"],
                      },
                    },
                    "required": ["kind", "api", "surface", "name"],
                    "type": "object",
                  }, {
                    "properties": {
                      "api": { "minLength": 1, "type": "string" },
                      "kind": { "const": "operationSignal" },
                      "operation": { "minLength": 1, "type": "string" },
                      "signal": { "minLength": 1, "type": "string" },
                    },
                    "required": ["kind", "api", "operation", "signal"],
                    "type": "object",
                  }, {
                    "properties": {
                      "kind": { "const": "participantResource" },
                      "name": { "minLength": 1, "type": "string" },
                      "participant": { "minLength": 1, "type": "string" },
                      "resource": {
                        "enum": [
                          "state",
                          "jobQueue",
                          "eventConsumer",
                          "kv",
                          "store",
                        ],
                      },
                    },
                    "required": ["kind", "participant", "resource", "name"],
                    "type": "object",
                  }],
                },
              },
              "required": ["target", "action"],
              "type": "object",
            },
            "type": "array",
          },
          "capability": { "minLength": 1, "type": "string" },
          "description": { "minLength": 1, "type": "string" },
          "displayName": { "minLength": 1, "type": "string" },
          "sourceApi": { "type": ["string", "null"] },
        },
        "required": [
          "capability",
          "displayName",
          "description",
          "allows",
          "sourceApi",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthConnectionsClosedEventSchema = {
  "properties": {
    "connectionId": { "minLength": 1, "type": "string" },
    "eventId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "participantId": { "minLength": 1, "type": "string" },
    "principalId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "eventId",
    "occurredAt",
    "sessionId",
    "principalId",
    "participantId",
    "connectionId",
    "reason",
  ],
  "type": "object",
} as const;

export const AuthConnectionsKickRequestSchema = {
  "properties": {
    "connectionId": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["connectionId", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthConnectionsKickResponseSchema = {
  "properties": {
    "connectionId": { "minLength": 1, "type": "string" },
    "kicked": { "type": "boolean" },
  },
  "required": ["connectionId", "kicked"],
  "type": "object",
} as const;

export const AuthConnectionsKickedEventSchema = {
  "properties": {
    "connectionId": { "minLength": 1, "type": "string" },
    "eventId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "participantId": { "minLength": 1, "type": "string" },
    "principalId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "eventId",
    "occurredAt",
    "sessionId",
    "principalId",
    "participantId",
    "connectionId",
    "reason",
  ],
  "type": "object",
} as const;

export const AuthConnectionsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthConnectionsListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "clientId": { "minLength": 1, "type": "string" },
          "connectedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "connectionId": { "minLength": 1, "type": "string" },
          "lastSeenAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "remoteAddress": { "type": ["string", "null"] },
          "serverId": { "minLength": 1, "type": "string" },
          "sessionId": { "minLength": 1, "type": "string" },
          "userNkey": { "minLength": 1, "type": "string" },
        },
        "required": [
          "connectionId",
          "sessionId",
          "serverId",
          "clientId",
          "userNkey",
          "remoteAddress",
          "connectedAt",
          "lastSeenAt",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthConnectionsOpenedEventSchema = {
  "properties": {
    "clientId": { "minLength": 1, "type": "string" },
    "connectionId": { "minLength": 1, "type": "string" },
    "eventId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "participantId": { "minLength": 1, "type": "string" },
    "principalId": { "minLength": 1, "type": "string" },
    "serverId": { "minLength": 1, "type": "string" },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "eventId",
    "occurredAt",
    "sessionId",
    "principalId",
    "participantId",
    "connectionId",
    "serverId",
    "clientId",
  ],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityAcceptMigrationRequestSchema = {
  "properties": {
    "expectedBaseAuthorityVersion": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 1,
        "type": "integer",
      }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "proposalId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": [
    "proposalId",
    "expectedBaseAuthorityVersion",
    "reason",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityAcceptMigrationResponseSchema = {
  "properties": {
    "authority": {
      "properties": {
        "acceptedNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "authorityId": { "minLength": 1, "type": "string" },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decision": {
          "anyOf": [{
            "properties": {
              "decidedAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "decidedBy": { "minLength": 1, "type": "string" },
              "reason": { "type": ["string", "null"] },
            },
            "required": ["decidedAt", "decidedBy", "reason"],
            "type": "object",
          }, { "type": "null" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "desiredCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "desiredGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "const": "deployment" },
        "materialization": {
          "anyOf": [{
            "properties": {
              "authorityId": { "minLength": 1, "type": "string" },
              "authorityKind": { "enum": ["identity", "deployment"] },
              "authorityVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "effectiveCapabilities": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "effectiveGrantSet": {
                "properties": {
                  "format": { "const": "trellis.grant-set.v1" },
                  "permissions": {
                    "items": {
                      "properties": {
                        "action": {
                          "enum": [
                            "call",
                            "invoke",
                            "observe",
                            "cancel",
                            "control",
                            "publish",
                            "subscribe",
                            "read",
                            "write",
                            "delete",
                            "submit",
                            "process",
                            "consume",
                          ],
                        },
                        "target": {
                          "anyOf": [{
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "apiSurface" },
                              "name": { "minLength": 1, "type": "string" },
                              "surface": {
                                "enum": [
                                  "rpc",
                                  "operation",
                                  "event",
                                  "feed",
                                  "state",
                                ],
                              },
                            },
                            "required": ["kind", "api", "surface", "name"],
                            "type": "object",
                          }, {
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "operationSignal" },
                              "operation": { "minLength": 1, "type": "string" },
                              "signal": { "minLength": 1, "type": "string" },
                            },
                            "required": ["kind", "api", "operation", "signal"],
                            "type": "object",
                          }, {
                            "properties": {
                              "kind": { "const": "participantResource" },
                              "name": { "minLength": 1, "type": "string" },
                              "participant": {
                                "minLength": 1,
                                "type": "string",
                              },
                              "resource": {
                                "enum": [
                                  "state",
                                  "jobQueue",
                                  "eventConsumer",
                                  "kv",
                                  "store",
                                ],
                              },
                            },
                            "required": [
                              "kind",
                              "participant",
                              "resource",
                              "name",
                            ],
                            "type": "object",
                          }],
                        },
                      },
                      "required": ["target", "action"],
                      "type": "object",
                    },
                    "type": "array",
                  },
                },
                "required": ["format", "permissions"],
                "type": "object",
              },
              "error": { "type": ["string", "null"] },
              "expiresAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "materializationId": { "minLength": 1, "type": "string" },
              "materializationVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "participantArtifactDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "participantId": { "minLength": 1, "type": "string" },
              "participantKind": {
                "enum": ["service", "app", "device", "agent"],
              },
              "participantNeedsDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "reconciledAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "state": { "enum": ["available", "unavailable", "error"] },
              "subjectId": { "minLength": 1, "type": "string" },
            },
            "required": [
              "materializationId",
              "authorityKind",
              "authorityId",
              "authorityVersion",
              "materializationVersion",
              "subjectId",
              "participantId",
              "participantKind",
              "participantArtifactDigest",
              "participantNeedsDigest",
              "effectiveGrantSet",
              "effectiveCapabilities",
              "state",
              "reconciledAt",
              "error",
              "expiresAt",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": { "enum": ["service", "device"] },
        "state": {
          "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "authorityId",
        "participantId",
        "participantArtifactDigest",
        "acceptedNeedsDigest",
        "desiredGrantSet",
        "desiredCapabilities",
        "state",
        "version",
        "createdAt",
        "updatedAt",
        "expiresAt",
        "decision",
        "materialization",
        "kind",
        "deploymentId",
        "participantKind",
      ],
      "type": "object",
    },
    "proposal": {
      "properties": {
        "authorityKind": { "enum": ["identity", "deployment"] },
        "baseAuthorityVersion": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": ["integer", "null"],
        },
        "classification": { "enum": ["initial", "update", "migration"] },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decisionAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "decisionBy": { "type": ["string", "null"] },
        "decisionReason": { "type": ["string", "null"] },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "proposalId": { "minLength": 1, "type": "string" },
        "proposedCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "proposedGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "reasons": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "state": {
          "enum": ["pending", "accepted", "rejected", "superseded", "expired"],
        },
        "subjectId": { "minLength": 1, "type": "string" },
      },
      "required": [
        "proposalId",
        "authorityKind",
        "subjectId",
        "participantId",
        "participantArtifactDigest",
        "participantNeedsDigest",
        "proposedGrantSet",
        "proposedCapabilities",
        "classification",
        "state",
        "reasons",
        "createdAt",
        "expiresAt",
        "decisionAt",
        "decisionBy",
        "decisionReason",
        "baseAuthorityVersion",
      ],
      "type": "object",
    },
  },
  "required": ["proposal", "authority"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityAcceptUpdateRequestSchema = {
  "properties": {
    "expectedBaseAuthorityVersion": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 1,
        "type": "integer",
      }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "proposalId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": [
    "proposalId",
    "expectedBaseAuthorityVersion",
    "reason",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityAcceptUpdateResponseSchema = {
  "properties": {
    "authority": {
      "properties": {
        "acceptedNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "authorityId": { "minLength": 1, "type": "string" },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decision": {
          "anyOf": [{
            "properties": {
              "decidedAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "decidedBy": { "minLength": 1, "type": "string" },
              "reason": { "type": ["string", "null"] },
            },
            "required": ["decidedAt", "decidedBy", "reason"],
            "type": "object",
          }, { "type": "null" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "desiredCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "desiredGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "const": "deployment" },
        "materialization": {
          "anyOf": [{
            "properties": {
              "authorityId": { "minLength": 1, "type": "string" },
              "authorityKind": { "enum": ["identity", "deployment"] },
              "authorityVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "effectiveCapabilities": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "effectiveGrantSet": {
                "properties": {
                  "format": { "const": "trellis.grant-set.v1" },
                  "permissions": {
                    "items": {
                      "properties": {
                        "action": {
                          "enum": [
                            "call",
                            "invoke",
                            "observe",
                            "cancel",
                            "control",
                            "publish",
                            "subscribe",
                            "read",
                            "write",
                            "delete",
                            "submit",
                            "process",
                            "consume",
                          ],
                        },
                        "target": {
                          "anyOf": [{
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "apiSurface" },
                              "name": { "minLength": 1, "type": "string" },
                              "surface": {
                                "enum": [
                                  "rpc",
                                  "operation",
                                  "event",
                                  "feed",
                                  "state",
                                ],
                              },
                            },
                            "required": ["kind", "api", "surface", "name"],
                            "type": "object",
                          }, {
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "operationSignal" },
                              "operation": { "minLength": 1, "type": "string" },
                              "signal": { "minLength": 1, "type": "string" },
                            },
                            "required": ["kind", "api", "operation", "signal"],
                            "type": "object",
                          }, {
                            "properties": {
                              "kind": { "const": "participantResource" },
                              "name": { "minLength": 1, "type": "string" },
                              "participant": {
                                "minLength": 1,
                                "type": "string",
                              },
                              "resource": {
                                "enum": [
                                  "state",
                                  "jobQueue",
                                  "eventConsumer",
                                  "kv",
                                  "store",
                                ],
                              },
                            },
                            "required": [
                              "kind",
                              "participant",
                              "resource",
                              "name",
                            ],
                            "type": "object",
                          }],
                        },
                      },
                      "required": ["target", "action"],
                      "type": "object",
                    },
                    "type": "array",
                  },
                },
                "required": ["format", "permissions"],
                "type": "object",
              },
              "error": { "type": ["string", "null"] },
              "expiresAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "materializationId": { "minLength": 1, "type": "string" },
              "materializationVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "participantArtifactDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "participantId": { "minLength": 1, "type": "string" },
              "participantKind": {
                "enum": ["service", "app", "device", "agent"],
              },
              "participantNeedsDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "reconciledAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "state": { "enum": ["available", "unavailable", "error"] },
              "subjectId": { "minLength": 1, "type": "string" },
            },
            "required": [
              "materializationId",
              "authorityKind",
              "authorityId",
              "authorityVersion",
              "materializationVersion",
              "subjectId",
              "participantId",
              "participantKind",
              "participantArtifactDigest",
              "participantNeedsDigest",
              "effectiveGrantSet",
              "effectiveCapabilities",
              "state",
              "reconciledAt",
              "error",
              "expiresAt",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": { "enum": ["service", "device"] },
        "state": {
          "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "authorityId",
        "participantId",
        "participantArtifactDigest",
        "acceptedNeedsDigest",
        "desiredGrantSet",
        "desiredCapabilities",
        "state",
        "version",
        "createdAt",
        "updatedAt",
        "expiresAt",
        "decision",
        "materialization",
        "kind",
        "deploymentId",
        "participantKind",
      ],
      "type": "object",
    },
    "proposal": {
      "properties": {
        "authorityKind": { "enum": ["identity", "deployment"] },
        "baseAuthorityVersion": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": ["integer", "null"],
        },
        "classification": { "enum": ["initial", "update", "migration"] },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decisionAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "decisionBy": { "type": ["string", "null"] },
        "decisionReason": { "type": ["string", "null"] },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "proposalId": { "minLength": 1, "type": "string" },
        "proposedCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "proposedGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "reasons": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "state": {
          "enum": ["pending", "accepted", "rejected", "superseded", "expired"],
        },
        "subjectId": { "minLength": 1, "type": "string" },
      },
      "required": [
        "proposalId",
        "authorityKind",
        "subjectId",
        "participantId",
        "participantArtifactDigest",
        "participantNeedsDigest",
        "proposedGrantSet",
        "proposedCapabilities",
        "classification",
        "state",
        "reasons",
        "createdAt",
        "expiresAt",
        "decisionAt",
        "decisionBy",
        "decisionReason",
        "baseAuthorityVersion",
      ],
      "type": "object",
    },
  },
  "required": ["proposal", "authority"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityGetRequestSchema = {
  "properties": { "authorityId": { "minLength": 1, "type": "string" } },
  "required": ["authorityId"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityGetResponseSchema = {
  "properties": {
    "authority": {
      "properties": {
        "acceptedNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "authorityId": { "minLength": 1, "type": "string" },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decision": {
          "anyOf": [{
            "properties": {
              "decidedAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "decidedBy": { "minLength": 1, "type": "string" },
              "reason": { "type": ["string", "null"] },
            },
            "required": ["decidedAt", "decidedBy", "reason"],
            "type": "object",
          }, { "type": "null" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "desiredCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "desiredGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "const": "deployment" },
        "materialization": {
          "anyOf": [{
            "properties": {
              "authorityId": { "minLength": 1, "type": "string" },
              "authorityKind": { "enum": ["identity", "deployment"] },
              "authorityVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "effectiveCapabilities": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "effectiveGrantSet": {
                "properties": {
                  "format": { "const": "trellis.grant-set.v1" },
                  "permissions": {
                    "items": {
                      "properties": {
                        "action": {
                          "enum": [
                            "call",
                            "invoke",
                            "observe",
                            "cancel",
                            "control",
                            "publish",
                            "subscribe",
                            "read",
                            "write",
                            "delete",
                            "submit",
                            "process",
                            "consume",
                          ],
                        },
                        "target": {
                          "anyOf": [{
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "apiSurface" },
                              "name": { "minLength": 1, "type": "string" },
                              "surface": {
                                "enum": [
                                  "rpc",
                                  "operation",
                                  "event",
                                  "feed",
                                  "state",
                                ],
                              },
                            },
                            "required": ["kind", "api", "surface", "name"],
                            "type": "object",
                          }, {
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "operationSignal" },
                              "operation": { "minLength": 1, "type": "string" },
                              "signal": { "minLength": 1, "type": "string" },
                            },
                            "required": ["kind", "api", "operation", "signal"],
                            "type": "object",
                          }, {
                            "properties": {
                              "kind": { "const": "participantResource" },
                              "name": { "minLength": 1, "type": "string" },
                              "participant": {
                                "minLength": 1,
                                "type": "string",
                              },
                              "resource": {
                                "enum": [
                                  "state",
                                  "jobQueue",
                                  "eventConsumer",
                                  "kv",
                                  "store",
                                ],
                              },
                            },
                            "required": [
                              "kind",
                              "participant",
                              "resource",
                              "name",
                            ],
                            "type": "object",
                          }],
                        },
                      },
                      "required": ["target", "action"],
                      "type": "object",
                    },
                    "type": "array",
                  },
                },
                "required": ["format", "permissions"],
                "type": "object",
              },
              "error": { "type": ["string", "null"] },
              "expiresAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "materializationId": { "minLength": 1, "type": "string" },
              "materializationVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "participantArtifactDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "participantId": { "minLength": 1, "type": "string" },
              "participantKind": {
                "enum": ["service", "app", "device", "agent"],
              },
              "participantNeedsDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "reconciledAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "state": { "enum": ["available", "unavailable", "error"] },
              "subjectId": { "minLength": 1, "type": "string" },
            },
            "required": [
              "materializationId",
              "authorityKind",
              "authorityId",
              "authorityVersion",
              "materializationVersion",
              "subjectId",
              "participantId",
              "participantKind",
              "participantArtifactDigest",
              "participantNeedsDigest",
              "effectiveGrantSet",
              "effectiveCapabilities",
              "state",
              "reconciledAt",
              "error",
              "expiresAt",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": { "enum": ["service", "device"] },
        "state": {
          "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "authorityId",
        "participantId",
        "participantArtifactDigest",
        "acceptedNeedsDigest",
        "desiredGrantSet",
        "desiredCapabilities",
        "state",
        "version",
        "createdAt",
        "updatedAt",
        "expiresAt",
        "decision",
        "materialization",
        "kind",
        "deploymentId",
        "participantKind",
      ],
      "type": "object",
    },
  },
  "required": ["authority"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "deploymentId": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "participantId": { "minLength": 1, "type": "string" },
    "state": {
      "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
    },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "acceptedNeedsDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "authorityId": { "minLength": 1, "type": "string" },
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "decision": {
            "anyOf": [{
              "properties": {
                "decidedAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": "integer",
                },
                "decidedBy": { "minLength": 1, "type": "string" },
                "reason": { "type": ["string", "null"] },
              },
              "required": ["decidedAt", "decidedBy", "reason"],
              "type": "object",
            }, { "type": "null" }],
          },
          "deploymentId": { "minLength": 1, "type": "string" },
          "desiredCapabilities": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "desiredGrantSet": {
            "properties": {
              "format": { "const": "trellis.grant-set.v1" },
              "permissions": {
                "items": {
                  "properties": {
                    "action": {
                      "enum": [
                        "call",
                        "invoke",
                        "observe",
                        "cancel",
                        "control",
                        "publish",
                        "subscribe",
                        "read",
                        "write",
                        "delete",
                        "submit",
                        "process",
                        "consume",
                      ],
                    },
                    "target": {
                      "anyOf": [{
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "apiSurface" },
                          "name": { "minLength": 1, "type": "string" },
                          "surface": {
                            "enum": [
                              "rpc",
                              "operation",
                              "event",
                              "feed",
                              "state",
                            ],
                          },
                        },
                        "required": ["kind", "api", "surface", "name"],
                        "type": "object",
                      }, {
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "operationSignal" },
                          "operation": { "minLength": 1, "type": "string" },
                          "signal": { "minLength": 1, "type": "string" },
                        },
                        "required": ["kind", "api", "operation", "signal"],
                        "type": "object",
                      }, {
                        "properties": {
                          "kind": { "const": "participantResource" },
                          "name": { "minLength": 1, "type": "string" },
                          "participant": { "minLength": 1, "type": "string" },
                          "resource": {
                            "enum": [
                              "state",
                              "jobQueue",
                              "eventConsumer",
                              "kv",
                              "store",
                            ],
                          },
                        },
                        "required": ["kind", "participant", "resource", "name"],
                        "type": "object",
                      }],
                    },
                  },
                  "required": ["target", "action"],
                  "type": "object",
                },
                "type": "array",
              },
            },
            "required": ["format", "permissions"],
            "type": "object",
          },
          "expiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "kind": { "const": "deployment" },
          "materialization": {
            "anyOf": [{
              "properties": {
                "authorityId": { "minLength": 1, "type": "string" },
                "authorityKind": { "enum": ["identity", "deployment"] },
                "authorityVersion": {
                  "maximum": 9007199254740991,
                  "minimum": 1,
                  "type": "integer",
                },
                "effectiveCapabilities": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
                "effectiveGrantSet": {
                  "properties": {
                    "format": { "const": "trellis.grant-set.v1" },
                    "permissions": {
                      "items": {
                        "properties": {
                          "action": {
                            "enum": [
                              "call",
                              "invoke",
                              "observe",
                              "cancel",
                              "control",
                              "publish",
                              "subscribe",
                              "read",
                              "write",
                              "delete",
                              "submit",
                              "process",
                              "consume",
                            ],
                          },
                          "target": {
                            "anyOf": [{
                              "properties": {
                                "api": { "minLength": 1, "type": "string" },
                                "kind": { "const": "apiSurface" },
                                "name": { "minLength": 1, "type": "string" },
                                "surface": {
                                  "enum": [
                                    "rpc",
                                    "operation",
                                    "event",
                                    "feed",
                                    "state",
                                  ],
                                },
                              },
                              "required": ["kind", "api", "surface", "name"],
                              "type": "object",
                            }, {
                              "properties": {
                                "api": { "minLength": 1, "type": "string" },
                                "kind": { "const": "operationSignal" },
                                "operation": {
                                  "minLength": 1,
                                  "type": "string",
                                },
                                "signal": { "minLength": 1, "type": "string" },
                              },
                              "required": [
                                "kind",
                                "api",
                                "operation",
                                "signal",
                              ],
                              "type": "object",
                            }, {
                              "properties": {
                                "kind": { "const": "participantResource" },
                                "name": { "minLength": 1, "type": "string" },
                                "participant": {
                                  "minLength": 1,
                                  "type": "string",
                                },
                                "resource": {
                                  "enum": [
                                    "state",
                                    "jobQueue",
                                    "eventConsumer",
                                    "kv",
                                    "store",
                                  ],
                                },
                              },
                              "required": [
                                "kind",
                                "participant",
                                "resource",
                                "name",
                              ],
                              "type": "object",
                            }],
                          },
                        },
                        "required": ["target", "action"],
                        "type": "object",
                      },
                      "type": "array",
                    },
                  },
                  "required": ["format", "permissions"],
                  "type": "object",
                },
                "error": { "type": ["string", "null"] },
                "expiresAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": ["integer", "null"],
                },
                "materializationId": { "minLength": 1, "type": "string" },
                "materializationVersion": {
                  "maximum": 9007199254740991,
                  "minimum": 1,
                  "type": "integer",
                },
                "participantArtifactDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "participantId": { "minLength": 1, "type": "string" },
                "participantKind": {
                  "enum": ["service", "app", "device", "agent"],
                },
                "participantNeedsDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "reconciledAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": ["integer", "null"],
                },
                "state": { "enum": ["available", "unavailable", "error"] },
                "subjectId": { "minLength": 1, "type": "string" },
              },
              "required": [
                "materializationId",
                "authorityKind",
                "authorityId",
                "authorityVersion",
                "materializationVersion",
                "subjectId",
                "participantId",
                "participantKind",
                "participantArtifactDigest",
                "participantNeedsDigest",
                "effectiveGrantSet",
                "effectiveCapabilities",
                "state",
                "reconciledAt",
                "error",
                "expiresAt",
              ],
              "type": "object",
            }, { "type": "null" }],
          },
          "participantArtifactDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "participantId": { "minLength": 1, "type": "string" },
          "participantKind": { "enum": ["service", "device"] },
          "state": {
            "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
          },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "authorityId",
          "participantId",
          "participantArtifactDigest",
          "acceptedNeedsDigest",
          "desiredGrantSet",
          "desiredCapabilities",
          "state",
          "version",
          "createdAt",
          "updatedAt",
          "expiresAt",
          "decision",
          "materialization",
          "kind",
          "deploymentId",
          "participantKind",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityPlanRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "expiresAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": ["integer", "null"],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "participantArtifact": { "type": "object" },
    "referencedApiArtifacts": {
      "items": { "type": "object" },
      "type": "array",
    },
  },
  "required": [
    "deploymentId",
    "participantArtifact",
    "referencedApiArtifacts",
    "expiresAt",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityPlanResponseSchema = {
  "properties": {
    "proposal": {
      "properties": {
        "authorityKind": { "enum": ["identity", "deployment"] },
        "baseAuthorityVersion": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": ["integer", "null"],
        },
        "classification": { "enum": ["initial", "update", "migration"] },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decisionAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "decisionBy": { "type": ["string", "null"] },
        "decisionReason": { "type": ["string", "null"] },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "proposalId": { "minLength": 1, "type": "string" },
        "proposedCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "proposedGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "reasons": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "state": {
          "enum": ["pending", "accepted", "rejected", "superseded", "expired"],
        },
        "subjectId": { "minLength": 1, "type": "string" },
      },
      "required": [
        "proposalId",
        "authorityKind",
        "subjectId",
        "participantId",
        "participantArtifactDigest",
        "participantNeedsDigest",
        "proposedGrantSet",
        "proposedCapabilities",
        "classification",
        "state",
        "reasons",
        "createdAt",
        "expiresAt",
        "decisionAt",
        "decisionBy",
        "decisionReason",
        "baseAuthorityVersion",
      ],
      "type": "object",
    },
  },
  "required": ["proposal"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityPlansGetRequestSchema = {
  "properties": { "proposalId": { "minLength": 1, "type": "string" } },
  "required": ["proposalId"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityPlansGetResponseSchema = {
  "properties": {
    "proposal": {
      "properties": {
        "authorityKind": { "enum": ["identity", "deployment"] },
        "baseAuthorityVersion": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": ["integer", "null"],
        },
        "classification": { "enum": ["initial", "update", "migration"] },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decisionAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "decisionBy": { "type": ["string", "null"] },
        "decisionReason": { "type": ["string", "null"] },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "proposalId": { "minLength": 1, "type": "string" },
        "proposedCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "proposedGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "reasons": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "state": {
          "enum": ["pending", "accepted", "rejected", "superseded", "expired"],
        },
        "subjectId": { "minLength": 1, "type": "string" },
      },
      "required": [
        "proposalId",
        "authorityKind",
        "subjectId",
        "participantId",
        "participantArtifactDigest",
        "participantNeedsDigest",
        "proposedGrantSet",
        "proposedCapabilities",
        "classification",
        "state",
        "reasons",
        "createdAt",
        "expiresAt",
        "decisionAt",
        "decisionBy",
        "decisionReason",
        "baseAuthorityVersion",
      ],
      "type": "object",
    },
  },
  "required": ["proposal"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityPlansListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "deploymentId": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "state": {
      "enum": ["pending", "accepted", "rejected", "superseded", "expired"],
    },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityPlansListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "authorityKind": { "enum": ["identity", "deployment"] },
          "baseAuthorityVersion": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": ["integer", "null"],
          },
          "classification": { "enum": ["initial", "update", "migration"] },
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "decisionAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "decisionBy": { "type": ["string", "null"] },
          "decisionReason": { "type": ["string", "null"] },
          "expiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "participantArtifactDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "participantId": { "minLength": 1, "type": "string" },
          "participantNeedsDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "proposalId": { "minLength": 1, "type": "string" },
          "proposedCapabilities": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "proposedGrantSet": {
            "properties": {
              "format": { "const": "trellis.grant-set.v1" },
              "permissions": {
                "items": {
                  "properties": {
                    "action": {
                      "enum": [
                        "call",
                        "invoke",
                        "observe",
                        "cancel",
                        "control",
                        "publish",
                        "subscribe",
                        "read",
                        "write",
                        "delete",
                        "submit",
                        "process",
                        "consume",
                      ],
                    },
                    "target": {
                      "anyOf": [{
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "apiSurface" },
                          "name": { "minLength": 1, "type": "string" },
                          "surface": {
                            "enum": [
                              "rpc",
                              "operation",
                              "event",
                              "feed",
                              "state",
                            ],
                          },
                        },
                        "required": ["kind", "api", "surface", "name"],
                        "type": "object",
                      }, {
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "operationSignal" },
                          "operation": { "minLength": 1, "type": "string" },
                          "signal": { "minLength": 1, "type": "string" },
                        },
                        "required": ["kind", "api", "operation", "signal"],
                        "type": "object",
                      }, {
                        "properties": {
                          "kind": { "const": "participantResource" },
                          "name": { "minLength": 1, "type": "string" },
                          "participant": { "minLength": 1, "type": "string" },
                          "resource": {
                            "enum": [
                              "state",
                              "jobQueue",
                              "eventConsumer",
                              "kv",
                              "store",
                            ],
                          },
                        },
                        "required": ["kind", "participant", "resource", "name"],
                        "type": "object",
                      }],
                    },
                  },
                  "required": ["target", "action"],
                  "type": "object",
                },
                "type": "array",
              },
            },
            "required": ["format", "permissions"],
            "type": "object",
          },
          "reasons": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "state": {
            "enum": [
              "pending",
              "accepted",
              "rejected",
              "superseded",
              "expired",
            ],
          },
          "subjectId": { "minLength": 1, "type": "string" },
        },
        "required": [
          "proposalId",
          "authorityKind",
          "subjectId",
          "participantId",
          "participantArtifactDigest",
          "participantNeedsDigest",
          "proposedGrantSet",
          "proposedCapabilities",
          "classification",
          "state",
          "reasons",
          "createdAt",
          "expiresAt",
          "decisionAt",
          "decisionBy",
          "decisionReason",
          "baseAuthorityVersion",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityReconcileRequestSchema = {
  "properties": {
    "authorityId": { "minLength": 1, "type": "string" },
    "expectedVersion": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 1,
        "type": "integer",
      }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
  },
  "required": ["authorityId", "expectedVersion", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityReconcileResponseSchema = {
  "properties": {
    "authority": {
      "properties": {
        "acceptedNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "authorityId": { "minLength": 1, "type": "string" },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decision": {
          "anyOf": [{
            "properties": {
              "decidedAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "decidedBy": { "minLength": 1, "type": "string" },
              "reason": { "type": ["string", "null"] },
            },
            "required": ["decidedAt", "decidedBy", "reason"],
            "type": "object",
          }, { "type": "null" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "desiredCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "desiredGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "const": "deployment" },
        "materialization": {
          "anyOf": [{
            "properties": {
              "authorityId": { "minLength": 1, "type": "string" },
              "authorityKind": { "enum": ["identity", "deployment"] },
              "authorityVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "effectiveCapabilities": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "effectiveGrantSet": {
                "properties": {
                  "format": { "const": "trellis.grant-set.v1" },
                  "permissions": {
                    "items": {
                      "properties": {
                        "action": {
                          "enum": [
                            "call",
                            "invoke",
                            "observe",
                            "cancel",
                            "control",
                            "publish",
                            "subscribe",
                            "read",
                            "write",
                            "delete",
                            "submit",
                            "process",
                            "consume",
                          ],
                        },
                        "target": {
                          "anyOf": [{
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "apiSurface" },
                              "name": { "minLength": 1, "type": "string" },
                              "surface": {
                                "enum": [
                                  "rpc",
                                  "operation",
                                  "event",
                                  "feed",
                                  "state",
                                ],
                              },
                            },
                            "required": ["kind", "api", "surface", "name"],
                            "type": "object",
                          }, {
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "operationSignal" },
                              "operation": { "minLength": 1, "type": "string" },
                              "signal": { "minLength": 1, "type": "string" },
                            },
                            "required": ["kind", "api", "operation", "signal"],
                            "type": "object",
                          }, {
                            "properties": {
                              "kind": { "const": "participantResource" },
                              "name": { "minLength": 1, "type": "string" },
                              "participant": {
                                "minLength": 1,
                                "type": "string",
                              },
                              "resource": {
                                "enum": [
                                  "state",
                                  "jobQueue",
                                  "eventConsumer",
                                  "kv",
                                  "store",
                                ],
                              },
                            },
                            "required": [
                              "kind",
                              "participant",
                              "resource",
                              "name",
                            ],
                            "type": "object",
                          }],
                        },
                      },
                      "required": ["target", "action"],
                      "type": "object",
                    },
                    "type": "array",
                  },
                },
                "required": ["format", "permissions"],
                "type": "object",
              },
              "error": { "type": ["string", "null"] },
              "expiresAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "materializationId": { "minLength": 1, "type": "string" },
              "materializationVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "participantArtifactDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "participantId": { "minLength": 1, "type": "string" },
              "participantKind": {
                "enum": ["service", "app", "device", "agent"],
              },
              "participantNeedsDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "reconciledAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "state": { "enum": ["available", "unavailable", "error"] },
              "subjectId": { "minLength": 1, "type": "string" },
            },
            "required": [
              "materializationId",
              "authorityKind",
              "authorityId",
              "authorityVersion",
              "materializationVersion",
              "subjectId",
              "participantId",
              "participantKind",
              "participantArtifactDigest",
              "participantNeedsDigest",
              "effectiveGrantSet",
              "effectiveCapabilities",
              "state",
              "reconciledAt",
              "error",
              "expiresAt",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": { "enum": ["service", "device"] },
        "state": {
          "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "authorityId",
        "participantId",
        "participantArtifactDigest",
        "acceptedNeedsDigest",
        "desiredGrantSet",
        "desiredCapabilities",
        "state",
        "version",
        "createdAt",
        "updatedAt",
        "expiresAt",
        "decision",
        "materialization",
        "kind",
        "deploymentId",
        "participantKind",
      ],
      "type": "object",
    },
  },
  "required": ["authority"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityRejectRequestSchema = {
  "properties": {
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "proposalId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["proposalId", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDeploymentAuthorityRejectResponseSchema = {
  "properties": {
    "proposal": {
      "properties": {
        "authorityKind": { "enum": ["identity", "deployment"] },
        "baseAuthorityVersion": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": ["integer", "null"],
        },
        "classification": { "enum": ["initial", "update", "migration"] },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decisionAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "decisionBy": { "type": ["string", "null"] },
        "decisionReason": { "type": ["string", "null"] },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "proposalId": { "minLength": 1, "type": "string" },
        "proposedCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "proposedGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "reasons": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "state": {
          "enum": ["pending", "accepted", "rejected", "superseded", "expired"],
        },
        "subjectId": { "minLength": 1, "type": "string" },
      },
      "required": [
        "proposalId",
        "authorityKind",
        "subjectId",
        "participantId",
        "participantArtifactDigest",
        "participantNeedsDigest",
        "proposedGrantSet",
        "proposedCapabilities",
        "classification",
        "state",
        "reasons",
        "createdAt",
        "expiresAt",
        "decisionAt",
        "decisionBy",
        "decisionReason",
        "baseAuthorityVersion",
      ],
      "type": "object",
    },
  },
  "required": ["proposal"],
  "type": "object",
} as const;

export const AuthDeploymentsCreateRequestSchema = {
  "properties": {
    "displayName": { "minLength": 1, "type": "string" },
    "expiresAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": ["integer", "null"],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "kind": { "enum": ["service", "device"] },
    "participantId": { "type": ["string", "null"] },
    "portalId": { "type": ["string", "null"] },
    "requiresDeviceDelegation": { "type": "boolean" },
  },
  "required": [
    "kind",
    "displayName",
    "participantId",
    "expiresAt",
    "requiresDeviceDelegation",
    "portalId",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthDeploymentsCreateResponseSchema = {
  "properties": {
    "deployment": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "disabledAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "enum": ["service", "device"] },
        "participantId": { "type": ["string", "null"] },
        "portalId": { "type": ["string", "null"] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "state": { "enum": ["active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "deploymentId",
        "kind",
        "displayName",
        "state",
        "participantId",
        "expiresAt",
        "requiresDeviceDelegation",
        "portalId",
        "createdAt",
        "updatedAt",
        "disabledAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["deployment"],
  "type": "object",
} as const;

export const AuthDeploymentsDisableRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["deploymentId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDeploymentsDisableResponseSchema = {
  "properties": {
    "deployment": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "disabledAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "enum": ["service", "device"] },
        "participantId": { "type": ["string", "null"] },
        "portalId": { "type": ["string", "null"] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "state": { "enum": ["active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "deploymentId",
        "kind",
        "displayName",
        "state",
        "participantId",
        "expiresAt",
        "requiresDeviceDelegation",
        "portalId",
        "createdAt",
        "updatedAt",
        "disabledAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["deployment", "mutation"],
  "type": "object",
} as const;

export const AuthDeploymentsEnableRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["deploymentId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDeploymentsEnableResponseSchema = {
  "properties": {
    "deployment": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "disabledAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "enum": ["service", "device"] },
        "participantId": { "type": ["string", "null"] },
        "portalId": { "type": ["string", "null"] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "state": { "enum": ["active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "deploymentId",
        "kind",
        "displayName",
        "state",
        "participantId",
        "expiresAt",
        "requiresDeviceDelegation",
        "portalId",
        "createdAt",
        "updatedAt",
        "disabledAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["deployment", "mutation"],
  "type": "object",
} as const;

export const AuthDeploymentsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "kind": { "enum": ["service", "device"] },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "state": { "enum": ["active", "disabled", "revoked"] },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthDeploymentsListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "deploymentId": { "minLength": 1, "type": "string" },
          "disabledAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "displayName": { "minLength": 1, "type": "string" },
          "expiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "kind": { "enum": ["service", "device"] },
          "participantId": { "type": ["string", "null"] },
          "portalId": { "type": ["string", "null"] },
          "requiresDeviceDelegation": { "type": "boolean" },
          "revokedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "state": { "enum": ["active", "disabled", "revoked"] },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "deploymentId",
          "kind",
          "displayName",
          "state",
          "participantId",
          "expiresAt",
          "requiresDeviceDelegation",
          "portalId",
          "createdAt",
          "updatedAt",
          "disabledAt",
          "revokedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthDeploymentsRemoveRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["deploymentId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDeploymentsRemoveResponseSchema = {
  "properties": {
    "deployment": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "disabledAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "enum": ["service", "device"] },
        "participantId": { "type": ["string", "null"] },
        "portalId": { "type": ["string", "null"] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "state": { "enum": ["active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "deploymentId",
        "kind",
        "displayName",
        "state",
        "participantId",
        "expiresAt",
        "requiresDeviceDelegation",
        "portalId",
        "createdAt",
        "updatedAt",
        "disabledAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["deployment", "mutation"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesApprovedEventSchema = {
  "properties": {
    "approvedBy": { "minLength": 1, "type": "string" },
    "deploymentId": { "minLength": 1, "type": "string" },
    "eventId": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
  },
  "required": [
    "eventId",
    "occurredAt",
    "deploymentId",
    "instanceId",
    "approvedBy",
  ],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "deploymentId": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "principalId": { "minLength": 1, "type": "string" },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "authority": {
            "anyOf": [{
              "properties": {
                "acceptedNeedsDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "authorityId": { "minLength": 1, "type": "string" },
                "createdAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": "integer",
                },
                "decision": {
                  "anyOf": [{
                    "properties": {
                      "decidedAt": {
                        "maximum": 9007199254740991,
                        "minimum": 0,
                        "type": "integer",
                      },
                      "decidedBy": { "minLength": 1, "type": "string" },
                      "reason": { "type": ["string", "null"] },
                    },
                    "required": ["decidedAt", "decidedBy", "reason"],
                    "type": "object",
                  }, { "type": "null" }],
                },
                "desiredCapabilities": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
                "desiredGrantSet": {
                  "properties": {
                    "format": { "const": "trellis.grant-set.v1" },
                    "permissions": {
                      "items": {
                        "properties": {
                          "action": {
                            "enum": [
                              "call",
                              "invoke",
                              "observe",
                              "cancel",
                              "control",
                              "publish",
                              "subscribe",
                              "read",
                              "write",
                              "delete",
                              "submit",
                              "process",
                              "consume",
                            ],
                          },
                          "target": {
                            "anyOf": [{
                              "properties": {
                                "api": { "minLength": 1, "type": "string" },
                                "kind": { "const": "apiSurface" },
                                "name": { "minLength": 1, "type": "string" },
                                "surface": {
                                  "enum": [
                                    "rpc",
                                    "operation",
                                    "event",
                                    "feed",
                                    "state",
                                  ],
                                },
                              },
                              "required": ["kind", "api", "surface", "name"],
                              "type": "object",
                            }, {
                              "properties": {
                                "api": { "minLength": 1, "type": "string" },
                                "kind": { "const": "operationSignal" },
                                "operation": {
                                  "minLength": 1,
                                  "type": "string",
                                },
                                "signal": { "minLength": 1, "type": "string" },
                              },
                              "required": [
                                "kind",
                                "api",
                                "operation",
                                "signal",
                              ],
                              "type": "object",
                            }, {
                              "properties": {
                                "kind": { "const": "participantResource" },
                                "name": { "minLength": 1, "type": "string" },
                                "participant": {
                                  "minLength": 1,
                                  "type": "string",
                                },
                                "resource": {
                                  "enum": [
                                    "state",
                                    "jobQueue",
                                    "eventConsumer",
                                    "kv",
                                    "store",
                                  ],
                                },
                              },
                              "required": [
                                "kind",
                                "participant",
                                "resource",
                                "name",
                              ],
                              "type": "object",
                            }],
                          },
                        },
                        "required": ["target", "action"],
                        "type": "object",
                      },
                      "type": "array",
                    },
                  },
                  "required": ["format", "permissions"],
                  "type": "object",
                },
                "expiresAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": ["integer", "null"],
                },
                "kind": { "const": "identity" },
                "materialization": {
                  "anyOf": [{
                    "properties": {
                      "authorityId": { "minLength": 1, "type": "string" },
                      "authorityKind": { "enum": ["identity", "deployment"] },
                      "authorityVersion": {
                        "maximum": 9007199254740991,
                        "minimum": 1,
                        "type": "integer",
                      },
                      "effectiveCapabilities": {
                        "items": { "minLength": 1, "type": "string" },
                        "type": "array",
                      },
                      "effectiveGrantSet": {
                        "properties": {
                          "format": { "const": "trellis.grant-set.v1" },
                          "permissions": {
                            "items": {
                              "properties": {
                                "action": {
                                  "enum": [
                                    "call",
                                    "invoke",
                                    "observe",
                                    "cancel",
                                    "control",
                                    "publish",
                                    "subscribe",
                                    "read",
                                    "write",
                                    "delete",
                                    "submit",
                                    "process",
                                    "consume",
                                  ],
                                },
                                "target": {
                                  "anyOf": [{
                                    "properties": {
                                      "api": {
                                        "minLength": 1,
                                        "type": "string",
                                      },
                                      "kind": { "const": "apiSurface" },
                                      "name": {
                                        "minLength": 1,
                                        "type": "string",
                                      },
                                      "surface": {
                                        "enum": [
                                          "rpc",
                                          "operation",
                                          "event",
                                          "feed",
                                          "state",
                                        ],
                                      },
                                    },
                                    "required": [
                                      "kind",
                                      "api",
                                      "surface",
                                      "name",
                                    ],
                                    "type": "object",
                                  }, {
                                    "properties": {
                                      "api": {
                                        "minLength": 1,
                                        "type": "string",
                                      },
                                      "kind": { "const": "operationSignal" },
                                      "operation": {
                                        "minLength": 1,
                                        "type": "string",
                                      },
                                      "signal": {
                                        "minLength": 1,
                                        "type": "string",
                                      },
                                    },
                                    "required": [
                                      "kind",
                                      "api",
                                      "operation",
                                      "signal",
                                    ],
                                    "type": "object",
                                  }, {
                                    "properties": {
                                      "kind": {
                                        "const": "participantResource",
                                      },
                                      "name": {
                                        "minLength": 1,
                                        "type": "string",
                                      },
                                      "participant": {
                                        "minLength": 1,
                                        "type": "string",
                                      },
                                      "resource": {
                                        "enum": [
                                          "state",
                                          "jobQueue",
                                          "eventConsumer",
                                          "kv",
                                          "store",
                                        ],
                                      },
                                    },
                                    "required": [
                                      "kind",
                                      "participant",
                                      "resource",
                                      "name",
                                    ],
                                    "type": "object",
                                  }],
                                },
                              },
                              "required": ["target", "action"],
                              "type": "object",
                            },
                            "type": "array",
                          },
                        },
                        "required": ["format", "permissions"],
                        "type": "object",
                      },
                      "error": { "type": ["string", "null"] },
                      "expiresAt": {
                        "maximum": 9007199254740991,
                        "minimum": 0,
                        "type": ["integer", "null"],
                      },
                      "materializationId": { "minLength": 1, "type": "string" },
                      "materializationVersion": {
                        "maximum": 9007199254740991,
                        "minimum": 1,
                        "type": "integer",
                      },
                      "participantArtifactDigest": {
                        "pattern": "^[A-Za-z0-9_-]{43}$",
                        "type": "string",
                      },
                      "participantId": { "minLength": 1, "type": "string" },
                      "participantKind": {
                        "enum": ["service", "app", "device", "agent"],
                      },
                      "participantNeedsDigest": {
                        "pattern": "^[A-Za-z0-9_-]{43}$",
                        "type": "string",
                      },
                      "reconciledAt": {
                        "maximum": 9007199254740991,
                        "minimum": 0,
                        "type": ["integer", "null"],
                      },
                      "state": {
                        "enum": ["available", "unavailable", "error"],
                      },
                      "subjectId": { "minLength": 1, "type": "string" },
                    },
                    "required": [
                      "materializationId",
                      "authorityKind",
                      "authorityId",
                      "authorityVersion",
                      "materializationVersion",
                      "subjectId",
                      "participantId",
                      "participantKind",
                      "participantArtifactDigest",
                      "participantNeedsDigest",
                      "effectiveGrantSet",
                      "effectiveCapabilities",
                      "state",
                      "reconciledAt",
                      "error",
                      "expiresAt",
                    ],
                    "type": "object",
                  }, { "type": "null" }],
                },
                "participantArtifactDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "participantId": { "minLength": 1, "type": "string" },
                "principalId": { "minLength": 1, "type": "string" },
                "state": {
                  "enum": [
                    "pending",
                    "accepted",
                    "rejected",
                    "revoked",
                    "stale",
                  ],
                },
                "updatedAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": "integer",
                },
                "version": {
                  "maximum": 9007199254740991,
                  "minimum": 1,
                  "type": "integer",
                },
              },
              "required": [
                "authorityId",
                "participantId",
                "participantArtifactDigest",
                "acceptedNeedsDigest",
                "desiredGrantSet",
                "desiredCapabilities",
                "state",
                "version",
                "createdAt",
                "updatedAt",
                "expiresAt",
                "decision",
                "materialization",
                "kind",
                "principalId",
              ],
              "type": "object",
            }, { "type": "null" }],
          },
          "device": {
            "properties": {
              "administrativeApproval": {
                "enum": ["pending", "approved", "rejected", "revoked"],
              },
              "createdAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "delegationExpiresAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "delegationRequired": { "type": "boolean" },
              "delegationState": { "enum": ["active", "missing", "revoked"] },
              "deploymentId": { "minLength": 1, "type": "string" },
              "identityKeyId": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": ["string", "null"],
              },
              "identityPublicKey": { "type": ["string", "null"] },
              "instanceId": { "minLength": 1, "type": "string" },
              "participantId": { "type": ["string", "null"] },
              "principalId": { "minLength": 1, "type": "string" },
              "state": { "enum": ["pending", "active", "disabled", "revoked"] },
              "updatedAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "version": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
            },
            "required": [
              "instanceId",
              "deploymentId",
              "principalId",
              "identityPublicKey",
              "identityKeyId",
              "participantId",
              "state",
              "administrativeApproval",
              "delegationRequired",
              "delegationState",
              "delegationExpiresAt",
              "createdAt",
              "updatedAt",
              "version",
            ],
            "type": "object",
          },
        },
        "required": ["device", "authority"],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesRequestedEventSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "eventId": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "userPrincipalId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "eventId",
    "occurredAt",
    "deploymentId",
    "instanceId",
    "userPrincipalId",
  ],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesResolveProgressSchema = {
  "properties": {
    "retryAfterMs": { "minimum": 0, "type": "integer" },
    "state": { "enum": ["waiting", "review_pending", "delegation_pending"] },
  },
  "required": ["state", "retryAfterMs"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesResolveRequestSchema = {
  "properties": { "flowId": { "minLength": 1, "type": "string" } },
  "required": ["flowId"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesResolveResponseSchema = {
  "properties": {
    "authority": {
      "anyOf": [{
        "properties": {
          "acceptedNeedsDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "authorityId": { "minLength": 1, "type": "string" },
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "decision": {
            "anyOf": [{
              "properties": {
                "decidedAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": "integer",
                },
                "decidedBy": { "minLength": 1, "type": "string" },
                "reason": { "type": ["string", "null"] },
              },
              "required": ["decidedAt", "decidedBy", "reason"],
              "type": "object",
            }, { "type": "null" }],
          },
          "desiredCapabilities": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "desiredGrantSet": {
            "properties": {
              "format": { "const": "trellis.grant-set.v1" },
              "permissions": {
                "items": {
                  "properties": {
                    "action": {
                      "enum": [
                        "call",
                        "invoke",
                        "observe",
                        "cancel",
                        "control",
                        "publish",
                        "subscribe",
                        "read",
                        "write",
                        "delete",
                        "submit",
                        "process",
                        "consume",
                      ],
                    },
                    "target": {
                      "anyOf": [{
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "apiSurface" },
                          "name": { "minLength": 1, "type": "string" },
                          "surface": {
                            "enum": [
                              "rpc",
                              "operation",
                              "event",
                              "feed",
                              "state",
                            ],
                          },
                        },
                        "required": ["kind", "api", "surface", "name"],
                        "type": "object",
                      }, {
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "operationSignal" },
                          "operation": { "minLength": 1, "type": "string" },
                          "signal": { "minLength": 1, "type": "string" },
                        },
                        "required": ["kind", "api", "operation", "signal"],
                        "type": "object",
                      }, {
                        "properties": {
                          "kind": { "const": "participantResource" },
                          "name": { "minLength": 1, "type": "string" },
                          "participant": { "minLength": 1, "type": "string" },
                          "resource": {
                            "enum": [
                              "state",
                              "jobQueue",
                              "eventConsumer",
                              "kv",
                              "store",
                            ],
                          },
                        },
                        "required": ["kind", "participant", "resource", "name"],
                        "type": "object",
                      }],
                    },
                  },
                  "required": ["target", "action"],
                  "type": "object",
                },
                "type": "array",
              },
            },
            "required": ["format", "permissions"],
            "type": "object",
          },
          "expiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "kind": { "const": "identity" },
          "materialization": {
            "anyOf": [{
              "properties": {
                "authorityId": { "minLength": 1, "type": "string" },
                "authorityKind": { "enum": ["identity", "deployment"] },
                "authorityVersion": {
                  "maximum": 9007199254740991,
                  "minimum": 1,
                  "type": "integer",
                },
                "effectiveCapabilities": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
                "effectiveGrantSet": {
                  "properties": {
                    "format": { "const": "trellis.grant-set.v1" },
                    "permissions": {
                      "items": {
                        "properties": {
                          "action": {
                            "enum": [
                              "call",
                              "invoke",
                              "observe",
                              "cancel",
                              "control",
                              "publish",
                              "subscribe",
                              "read",
                              "write",
                              "delete",
                              "submit",
                              "process",
                              "consume",
                            ],
                          },
                          "target": {
                            "anyOf": [{
                              "properties": {
                                "api": { "minLength": 1, "type": "string" },
                                "kind": { "const": "apiSurface" },
                                "name": { "minLength": 1, "type": "string" },
                                "surface": {
                                  "enum": [
                                    "rpc",
                                    "operation",
                                    "event",
                                    "feed",
                                    "state",
                                  ],
                                },
                              },
                              "required": ["kind", "api", "surface", "name"],
                              "type": "object",
                            }, {
                              "properties": {
                                "api": { "minLength": 1, "type": "string" },
                                "kind": { "const": "operationSignal" },
                                "operation": {
                                  "minLength": 1,
                                  "type": "string",
                                },
                                "signal": { "minLength": 1, "type": "string" },
                              },
                              "required": [
                                "kind",
                                "api",
                                "operation",
                                "signal",
                              ],
                              "type": "object",
                            }, {
                              "properties": {
                                "kind": { "const": "participantResource" },
                                "name": { "minLength": 1, "type": "string" },
                                "participant": {
                                  "minLength": 1,
                                  "type": "string",
                                },
                                "resource": {
                                  "enum": [
                                    "state",
                                    "jobQueue",
                                    "eventConsumer",
                                    "kv",
                                    "store",
                                  ],
                                },
                              },
                              "required": [
                                "kind",
                                "participant",
                                "resource",
                                "name",
                              ],
                              "type": "object",
                            }],
                          },
                        },
                        "required": ["target", "action"],
                        "type": "object",
                      },
                      "type": "array",
                    },
                  },
                  "required": ["format", "permissions"],
                  "type": "object",
                },
                "error": { "type": ["string", "null"] },
                "expiresAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": ["integer", "null"],
                },
                "materializationId": { "minLength": 1, "type": "string" },
                "materializationVersion": {
                  "maximum": 9007199254740991,
                  "minimum": 1,
                  "type": "integer",
                },
                "participantArtifactDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "participantId": { "minLength": 1, "type": "string" },
                "participantKind": {
                  "enum": ["service", "app", "device", "agent"],
                },
                "participantNeedsDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "reconciledAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": ["integer", "null"],
                },
                "state": { "enum": ["available", "unavailable", "error"] },
                "subjectId": { "minLength": 1, "type": "string" },
              },
              "required": [
                "materializationId",
                "authorityKind",
                "authorityId",
                "authorityVersion",
                "materializationVersion",
                "subjectId",
                "participantId",
                "participantKind",
                "participantArtifactDigest",
                "participantNeedsDigest",
                "effectiveGrantSet",
                "effectiveCapabilities",
                "state",
                "reconciledAt",
                "error",
                "expiresAt",
              ],
              "type": "object",
            }, { "type": "null" }],
          },
          "participantArtifactDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "participantId": { "minLength": 1, "type": "string" },
          "principalId": { "minLength": 1, "type": "string" },
          "state": {
            "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
          },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "authorityId",
          "participantId",
          "participantArtifactDigest",
          "acceptedNeedsDigest",
          "desiredGrantSet",
          "desiredCapabilities",
          "state",
          "version",
          "createdAt",
          "updatedAt",
          "expiresAt",
          "decision",
          "materialization",
          "kind",
          "principalId",
        ],
        "type": "object",
      }, { "type": "null" }],
    },
    "device": {
      "properties": {
        "administrativeApproval": {
          "enum": ["pending", "approved", "rejected", "revoked"],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": { "enum": ["active", "missing", "revoked"] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": ["string", "null"],
        },
        "identityPublicKey": { "type": ["string", "null"] },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["pending", "active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "administrativeApproval",
        "delegationRequired",
        "delegationState",
        "delegationExpiresAt",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "review": {
      "properties": {
        "confirmationCode": { "minLength": 1, "type": "string" },
        "decidedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "decidedBy": { "type": ["string", "null"] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "devicePrincipalId": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "reason": { "type": ["string", "null"] },
        "requestedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "reviewId": { "minLength": 1, "type": "string" },
        "state": {
          "enum": ["pending", "approved", "rejected", "expired", "revoked"],
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "reviewId",
        "deploymentId",
        "instanceId",
        "devicePrincipalId",
        "state",
        "confirmationCode",
        "requestedAt",
        "expiresAt",
        "decidedAt",
        "decidedBy",
        "reason",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["device", "review", "authority"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesResolvedEventSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "eventId": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "state": { "minLength": 1, "type": "string" },
  },
  "required": ["eventId", "occurredAt", "deploymentId", "instanceId", "state"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesReviewRequestedEventSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "eventId": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "reviewId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "eventId",
    "occurredAt",
    "deploymentId",
    "instanceId",
    "reviewId",
  ],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesReviewsDecideRequestSchema = {
  "properties": {
    "decision": { "enum": ["approve", "reject"] },
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
    "reviewId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "reviewId",
    "decision",
    "expectedVersion",
    "reason",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesReviewsDecideResponseSchema = {
  "properties": {
    "review": {
      "properties": {
        "confirmationCode": { "minLength": 1, "type": "string" },
        "decidedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "decidedBy": { "type": ["string", "null"] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "devicePrincipalId": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "reason": { "type": ["string", "null"] },
        "requestedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "reviewId": { "minLength": 1, "type": "string" },
        "state": {
          "enum": ["pending", "approved", "rejected", "expired", "revoked"],
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "reviewId",
        "deploymentId",
        "instanceId",
        "devicePrincipalId",
        "state",
        "confirmationCode",
        "requestedAt",
        "expiresAt",
        "decidedAt",
        "decidedBy",
        "reason",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["review"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesReviewsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "deploymentId": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "state": {
      "enum": ["pending", "approved", "rejected", "expired", "revoked"],
    },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesReviewsListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "confirmationCode": { "minLength": 1, "type": "string" },
          "decidedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "decidedBy": { "type": ["string", "null"] },
          "deploymentId": { "minLength": 1, "type": "string" },
          "devicePrincipalId": { "minLength": 1, "type": "string" },
          "expiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "instanceId": { "minLength": 1, "type": "string" },
          "reason": { "type": ["string", "null"] },
          "requestedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "reviewId": { "minLength": 1, "type": "string" },
          "state": {
            "enum": ["pending", "approved", "rejected", "expired", "revoked"],
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "reviewId",
          "deploymentId",
          "instanceId",
          "devicePrincipalId",
          "state",
          "confirmationCode",
          "requestedAt",
          "expiresAt",
          "decidedAt",
          "decidedBy",
          "reason",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesRevokeRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "devicePrincipalId": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["deploymentId", "devicePrincipalId", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesRevokeResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "enum": ["pending", "approved", "rejected", "revoked"],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": { "enum": ["active", "missing", "revoked"] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": ["string", "null"],
        },
        "identityPublicKey": { "type": ["string", "null"] },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["pending", "active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "administrativeApproval",
        "delegationRequired",
        "delegationState",
        "delegationExpiresAt",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "kickedSessionCount": { "minimum": 0, "type": "integer" },
  },
  "required": ["device", "kickedSessionCount"],
  "type": "object",
} as const;

export const AuthDevicesConnectInfoGetRequestSchema = {
  "properties": {
    "challengeDigest": {
      "pattern": "^[A-Za-z0-9_-]{43}$",
      "type": ["string", "null"],
    },
    "deploymentId": { "minLength": 1, "type": "string" },
    "deviceIdentityKeyId": {
      "pattern": "^[A-Za-z0-9_-]{43}$",
      "type": "string",
    },
    "instanceId": { "minLength": 1, "type": "string" },
    "issuedAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "newSessionNkey": { "minLength": 1, "type": "string" },
    "newSessionPublicKey": { "minLength": 1, "type": "string" },
    "participantDigest": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
    "participantId": { "minLength": 1, "type": "string" },
    "proof": {
      "properties": {
        "format": { "const": "trellis.session-proof.v1" },
        "signature": { "minLength": 1, "type": "string" },
      },
      "required": ["format", "signature"],
      "type": "object",
    },
    "requestId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "deploymentId",
    "instanceId",
    "deviceIdentityKeyId",
    "newSessionPublicKey",
    "newSessionNkey",
    "participantId",
    "participantDigest",
    "challengeDigest",
    "requestId",
    "issuedAt",
    "proof",
  ],
  "type": "object",
} as const;

export const AuthDevicesConnectInfoGetResponseSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "endpoints": {
      "properties": {
        "authMode": { "const": "session_nkey" },
        "authorityMode": { "const": "server_issued" },
        "maximumClockSkewMs": {
          "maximum": 300000,
          "minimum": 0,
          "type": "integer",
        },
        "native": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "websocket": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
      },
      "required": [
        "native",
        "websocket",
        "authMode",
        "authorityMode",
        "maximumClockSkewMs",
      ],
      "type": "object",
    },
    "instanceId": { "minLength": 1, "type": "string" },
    "participantId": { "type": ["string", "null"] },
  },
  "required": ["deploymentId", "instanceId", "participantId", "endpoints"],
  "type": "object",
} as const;

export const AuthDevicesDisableRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["instanceId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDevicesDisableResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "enum": ["pending", "approved", "rejected", "revoked"],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": { "enum": ["active", "missing", "revoked"] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": ["string", "null"],
        },
        "identityPublicKey": { "type": ["string", "null"] },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["pending", "active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "administrativeApproval",
        "delegationRequired",
        "delegationState",
        "delegationExpiresAt",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["device", "mutation"],
  "type": "object",
} as const;

export const AuthDevicesEnableRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["instanceId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDevicesEnableResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "enum": ["pending", "approved", "rejected", "revoked"],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": { "enum": ["active", "missing", "revoked"] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": ["string", "null"],
        },
        "identityPublicKey": { "type": ["string", "null"] },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["pending", "active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "administrativeApproval",
        "delegationRequired",
        "delegationState",
        "delegationExpiresAt",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["device", "mutation"],
  "type": "object",
} as const;

export const AuthDevicesListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "deploymentId": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "state": { "enum": ["pending", "active", "disabled", "revoked"] },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthDevicesListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "administrativeApproval": {
            "enum": ["pending", "approved", "rejected", "revoked"],
          },
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "delegationExpiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "delegationRequired": { "type": "boolean" },
          "delegationState": { "enum": ["active", "missing", "revoked"] },
          "deploymentId": { "minLength": 1, "type": "string" },
          "identityKeyId": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": ["string", "null"],
          },
          "identityPublicKey": { "type": ["string", "null"] },
          "instanceId": { "minLength": 1, "type": "string" },
          "participantId": { "type": ["string", "null"] },
          "principalId": { "minLength": 1, "type": "string" },
          "state": { "enum": ["pending", "active", "disabled", "revoked"] },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "instanceId",
          "deploymentId",
          "principalId",
          "identityPublicKey",
          "identityKeyId",
          "participantId",
          "state",
          "administrativeApproval",
          "delegationRequired",
          "delegationState",
          "delegationExpiresAt",
          "createdAt",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthDevicesProvisionRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "identityPublicKey": { "type": ["string", "null"] },
    "instanceId": { "type": ["string", "null"] },
    "participantId": { "type": ["string", "null"] },
  },
  "required": [
    "deploymentId",
    "instanceId",
    "identityPublicKey",
    "participantId",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthDevicesProvisionResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "enum": ["pending", "approved", "rejected", "revoked"],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": { "enum": ["active", "missing", "revoked"] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": ["string", "null"],
        },
        "identityPublicKey": { "type": ["string", "null"] },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["pending", "active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "administrativeApproval",
        "delegationRequired",
        "delegationState",
        "delegationExpiresAt",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "provisioningSecret": { "type": ["string", "null"] },
  },
  "required": ["device", "provisioningSecret"],
  "type": "object",
} as const;

export const AuthDevicesRemoveRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["instanceId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthDevicesRemoveResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "enum": ["pending", "approved", "rejected", "revoked"],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": { "enum": ["active", "missing", "revoked"] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": ["string", "null"],
        },
        "identityPublicKey": { "type": ["string", "null"] },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["pending", "active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "administrativeApproval",
        "delegationRequired",
        "delegationState",
        "delegationExpiresAt",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["device", "mutation"],
  "type": "object",
} as const;

export const AuthErrorDetailsSchema = {
  "properties": {
    "code": { "minLength": 1, "type": "string" },
    "field": { "type": ["string", "null"] },
    "message": { "minLength": 1, "type": "string" },
    "retryable": { "type": "boolean" },
  },
  "required": ["code", "message", "retryable", "field"],
  "type": "object",
} as const;

export const AuthIdentityAuthorityGetRequestSchema = {
  "properties": { "authorityId": { "minLength": 1, "type": "string" } },
  "required": ["authorityId"],
  "type": "object",
} as const;

export const AuthIdentityAuthorityGetResponseSchema = {
  "properties": {
    "authority": {
      "properties": {
        "acceptedNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "authorityId": { "minLength": 1, "type": "string" },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decision": {
          "anyOf": [{
            "properties": {
              "decidedAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "decidedBy": { "minLength": 1, "type": "string" },
              "reason": { "type": ["string", "null"] },
            },
            "required": ["decidedAt", "decidedBy", "reason"],
            "type": "object",
          }, { "type": "null" }],
        },
        "desiredCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "desiredGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "const": "identity" },
        "materialization": {
          "anyOf": [{
            "properties": {
              "authorityId": { "minLength": 1, "type": "string" },
              "authorityKind": { "enum": ["identity", "deployment"] },
              "authorityVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "effectiveCapabilities": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "effectiveGrantSet": {
                "properties": {
                  "format": { "const": "trellis.grant-set.v1" },
                  "permissions": {
                    "items": {
                      "properties": {
                        "action": {
                          "enum": [
                            "call",
                            "invoke",
                            "observe",
                            "cancel",
                            "control",
                            "publish",
                            "subscribe",
                            "read",
                            "write",
                            "delete",
                            "submit",
                            "process",
                            "consume",
                          ],
                        },
                        "target": {
                          "anyOf": [{
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "apiSurface" },
                              "name": { "minLength": 1, "type": "string" },
                              "surface": {
                                "enum": [
                                  "rpc",
                                  "operation",
                                  "event",
                                  "feed",
                                  "state",
                                ],
                              },
                            },
                            "required": ["kind", "api", "surface", "name"],
                            "type": "object",
                          }, {
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "operationSignal" },
                              "operation": { "minLength": 1, "type": "string" },
                              "signal": { "minLength": 1, "type": "string" },
                            },
                            "required": ["kind", "api", "operation", "signal"],
                            "type": "object",
                          }, {
                            "properties": {
                              "kind": { "const": "participantResource" },
                              "name": { "minLength": 1, "type": "string" },
                              "participant": {
                                "minLength": 1,
                                "type": "string",
                              },
                              "resource": {
                                "enum": [
                                  "state",
                                  "jobQueue",
                                  "eventConsumer",
                                  "kv",
                                  "store",
                                ],
                              },
                            },
                            "required": [
                              "kind",
                              "participant",
                              "resource",
                              "name",
                            ],
                            "type": "object",
                          }],
                        },
                      },
                      "required": ["target", "action"],
                      "type": "object",
                    },
                    "type": "array",
                  },
                },
                "required": ["format", "permissions"],
                "type": "object",
              },
              "error": { "type": ["string", "null"] },
              "expiresAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "materializationId": { "minLength": 1, "type": "string" },
              "materializationVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "participantArtifactDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "participantId": { "minLength": 1, "type": "string" },
              "participantKind": {
                "enum": ["service", "app", "device", "agent"],
              },
              "participantNeedsDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "reconciledAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "state": { "enum": ["available", "unavailable", "error"] },
              "subjectId": { "minLength": 1, "type": "string" },
            },
            "required": [
              "materializationId",
              "authorityKind",
              "authorityId",
              "authorityVersion",
              "materializationVersion",
              "subjectId",
              "participantId",
              "participantKind",
              "participantArtifactDigest",
              "participantNeedsDigest",
              "effectiveGrantSet",
              "effectiveCapabilities",
              "state",
              "reconciledAt",
              "error",
              "expiresAt",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "authorityId",
        "participantId",
        "participantArtifactDigest",
        "acceptedNeedsDigest",
        "desiredGrantSet",
        "desiredCapabilities",
        "state",
        "version",
        "createdAt",
        "updatedAt",
        "expiresAt",
        "decision",
        "materialization",
        "kind",
        "principalId",
      ],
      "type": "object",
    },
  },
  "required": ["authority"],
  "type": "object",
} as const;

export const AuthIdentityAuthorityListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "participantId": { "minLength": 1, "type": "string" },
    "principalId": { "minLength": 1, "type": "string" },
    "state": {
      "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
    },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthIdentityAuthorityListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "acceptedNeedsDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "authorityId": { "minLength": 1, "type": "string" },
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "decision": {
            "anyOf": [{
              "properties": {
                "decidedAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": "integer",
                },
                "decidedBy": { "minLength": 1, "type": "string" },
                "reason": { "type": ["string", "null"] },
              },
              "required": ["decidedAt", "decidedBy", "reason"],
              "type": "object",
            }, { "type": "null" }],
          },
          "desiredCapabilities": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "desiredGrantSet": {
            "properties": {
              "format": { "const": "trellis.grant-set.v1" },
              "permissions": {
                "items": {
                  "properties": {
                    "action": {
                      "enum": [
                        "call",
                        "invoke",
                        "observe",
                        "cancel",
                        "control",
                        "publish",
                        "subscribe",
                        "read",
                        "write",
                        "delete",
                        "submit",
                        "process",
                        "consume",
                      ],
                    },
                    "target": {
                      "anyOf": [{
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "apiSurface" },
                          "name": { "minLength": 1, "type": "string" },
                          "surface": {
                            "enum": [
                              "rpc",
                              "operation",
                              "event",
                              "feed",
                              "state",
                            ],
                          },
                        },
                        "required": ["kind", "api", "surface", "name"],
                        "type": "object",
                      }, {
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "operationSignal" },
                          "operation": { "minLength": 1, "type": "string" },
                          "signal": { "minLength": 1, "type": "string" },
                        },
                        "required": ["kind", "api", "operation", "signal"],
                        "type": "object",
                      }, {
                        "properties": {
                          "kind": { "const": "participantResource" },
                          "name": { "minLength": 1, "type": "string" },
                          "participant": { "minLength": 1, "type": "string" },
                          "resource": {
                            "enum": [
                              "state",
                              "jobQueue",
                              "eventConsumer",
                              "kv",
                              "store",
                            ],
                          },
                        },
                        "required": ["kind", "participant", "resource", "name"],
                        "type": "object",
                      }],
                    },
                  },
                  "required": ["target", "action"],
                  "type": "object",
                },
                "type": "array",
              },
            },
            "required": ["format", "permissions"],
            "type": "object",
          },
          "expiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "kind": { "const": "identity" },
          "materialization": {
            "anyOf": [{
              "properties": {
                "authorityId": { "minLength": 1, "type": "string" },
                "authorityKind": { "enum": ["identity", "deployment"] },
                "authorityVersion": {
                  "maximum": 9007199254740991,
                  "minimum": 1,
                  "type": "integer",
                },
                "effectiveCapabilities": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
                "effectiveGrantSet": {
                  "properties": {
                    "format": { "const": "trellis.grant-set.v1" },
                    "permissions": {
                      "items": {
                        "properties": {
                          "action": {
                            "enum": [
                              "call",
                              "invoke",
                              "observe",
                              "cancel",
                              "control",
                              "publish",
                              "subscribe",
                              "read",
                              "write",
                              "delete",
                              "submit",
                              "process",
                              "consume",
                            ],
                          },
                          "target": {
                            "anyOf": [{
                              "properties": {
                                "api": { "minLength": 1, "type": "string" },
                                "kind": { "const": "apiSurface" },
                                "name": { "minLength": 1, "type": "string" },
                                "surface": {
                                  "enum": [
                                    "rpc",
                                    "operation",
                                    "event",
                                    "feed",
                                    "state",
                                  ],
                                },
                              },
                              "required": ["kind", "api", "surface", "name"],
                              "type": "object",
                            }, {
                              "properties": {
                                "api": { "minLength": 1, "type": "string" },
                                "kind": { "const": "operationSignal" },
                                "operation": {
                                  "minLength": 1,
                                  "type": "string",
                                },
                                "signal": { "minLength": 1, "type": "string" },
                              },
                              "required": [
                                "kind",
                                "api",
                                "operation",
                                "signal",
                              ],
                              "type": "object",
                            }, {
                              "properties": {
                                "kind": { "const": "participantResource" },
                                "name": { "minLength": 1, "type": "string" },
                                "participant": {
                                  "minLength": 1,
                                  "type": "string",
                                },
                                "resource": {
                                  "enum": [
                                    "state",
                                    "jobQueue",
                                    "eventConsumer",
                                    "kv",
                                    "store",
                                  ],
                                },
                              },
                              "required": [
                                "kind",
                                "participant",
                                "resource",
                                "name",
                              ],
                              "type": "object",
                            }],
                          },
                        },
                        "required": ["target", "action"],
                        "type": "object",
                      },
                      "type": "array",
                    },
                  },
                  "required": ["format", "permissions"],
                  "type": "object",
                },
                "error": { "type": ["string", "null"] },
                "expiresAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": ["integer", "null"],
                },
                "materializationId": { "minLength": 1, "type": "string" },
                "materializationVersion": {
                  "maximum": 9007199254740991,
                  "minimum": 1,
                  "type": "integer",
                },
                "participantArtifactDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "participantId": { "minLength": 1, "type": "string" },
                "participantKind": {
                  "enum": ["service", "app", "device", "agent"],
                },
                "participantNeedsDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "reconciledAt": {
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": ["integer", "null"],
                },
                "state": { "enum": ["available", "unavailable", "error"] },
                "subjectId": { "minLength": 1, "type": "string" },
              },
              "required": [
                "materializationId",
                "authorityKind",
                "authorityId",
                "authorityVersion",
                "materializationVersion",
                "subjectId",
                "participantId",
                "participantKind",
                "participantArtifactDigest",
                "participantNeedsDigest",
                "effectiveGrantSet",
                "effectiveCapabilities",
                "state",
                "reconciledAt",
                "error",
                "expiresAt",
              ],
              "type": "object",
            }, { "type": "null" }],
          },
          "participantArtifactDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "participantId": { "minLength": 1, "type": "string" },
          "principalId": { "minLength": 1, "type": "string" },
          "state": {
            "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
          },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "authorityId",
          "participantId",
          "participantArtifactDigest",
          "acceptedNeedsDigest",
          "desiredGrantSet",
          "desiredCapabilities",
          "state",
          "version",
          "createdAt",
          "updatedAt",
          "expiresAt",
          "decision",
          "materialization",
          "kind",
          "principalId",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthIdentityAuthorityRevokeRequestSchema = {
  "properties": {
    "authorityId": { "minLength": 1, "type": "string" },
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["authorityId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthIdentityAuthorityRevokeResponseSchema = {
  "properties": {
    "authority": {
      "properties": {
        "acceptedNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "authorityId": { "minLength": 1, "type": "string" },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "decision": {
          "anyOf": [{
            "properties": {
              "decidedAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "decidedBy": { "minLength": 1, "type": "string" },
              "reason": { "type": ["string", "null"] },
            },
            "required": ["decidedAt", "decidedBy", "reason"],
            "type": "object",
          }, { "type": "null" }],
        },
        "desiredCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "desiredGrantSet": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "enum": [
                      "call",
                      "invoke",
                      "observe",
                      "cancel",
                      "control",
                      "publish",
                      "subscribe",
                      "read",
                      "write",
                      "delete",
                      "submit",
                      "process",
                      "consume",
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "enum": [
                            "rpc",
                            "operation",
                            "event",
                            "feed",
                            "state",
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "operationSignal" },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": { "const": "participantResource" },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "enum": [
                            "state",
                            "jobQueue",
                            "eventConsumer",
                            "kv",
                            "store",
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["target", "action"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "kind": { "const": "identity" },
        "materialization": {
          "anyOf": [{
            "properties": {
              "authorityId": { "minLength": 1, "type": "string" },
              "authorityKind": { "enum": ["identity", "deployment"] },
              "authorityVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "effectiveCapabilities": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "effectiveGrantSet": {
                "properties": {
                  "format": { "const": "trellis.grant-set.v1" },
                  "permissions": {
                    "items": {
                      "properties": {
                        "action": {
                          "enum": [
                            "call",
                            "invoke",
                            "observe",
                            "cancel",
                            "control",
                            "publish",
                            "subscribe",
                            "read",
                            "write",
                            "delete",
                            "submit",
                            "process",
                            "consume",
                          ],
                        },
                        "target": {
                          "anyOf": [{
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "apiSurface" },
                              "name": { "minLength": 1, "type": "string" },
                              "surface": {
                                "enum": [
                                  "rpc",
                                  "operation",
                                  "event",
                                  "feed",
                                  "state",
                                ],
                              },
                            },
                            "required": ["kind", "api", "surface", "name"],
                            "type": "object",
                          }, {
                            "properties": {
                              "api": { "minLength": 1, "type": "string" },
                              "kind": { "const": "operationSignal" },
                              "operation": { "minLength": 1, "type": "string" },
                              "signal": { "minLength": 1, "type": "string" },
                            },
                            "required": ["kind", "api", "operation", "signal"],
                            "type": "object",
                          }, {
                            "properties": {
                              "kind": { "const": "participantResource" },
                              "name": { "minLength": 1, "type": "string" },
                              "participant": {
                                "minLength": 1,
                                "type": "string",
                              },
                              "resource": {
                                "enum": [
                                  "state",
                                  "jobQueue",
                                  "eventConsumer",
                                  "kv",
                                  "store",
                                ],
                              },
                            },
                            "required": [
                              "kind",
                              "participant",
                              "resource",
                              "name",
                            ],
                            "type": "object",
                          }],
                        },
                      },
                      "required": ["target", "action"],
                      "type": "object",
                    },
                    "type": "array",
                  },
                },
                "required": ["format", "permissions"],
                "type": "object",
              },
              "error": { "type": ["string", "null"] },
              "expiresAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "materializationId": { "minLength": 1, "type": "string" },
              "materializationVersion": {
                "maximum": 9007199254740991,
                "minimum": 1,
                "type": "integer",
              },
              "participantArtifactDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "participantId": { "minLength": 1, "type": "string" },
              "participantKind": {
                "enum": ["service", "app", "device", "agent"],
              },
              "participantNeedsDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "reconciledAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": ["integer", "null"],
              },
              "state": { "enum": ["available", "unavailable", "error"] },
              "subjectId": { "minLength": 1, "type": "string" },
            },
            "required": [
              "materializationId",
              "authorityKind",
              "authorityId",
              "authorityVersion",
              "materializationVersion",
              "subjectId",
              "participantId",
              "participantKind",
              "participantArtifactDigest",
              "participantNeedsDigest",
              "effectiveGrantSet",
              "effectiveCapabilities",
              "state",
              "reconciledAt",
              "error",
              "expiresAt",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "enum": ["pending", "accepted", "rejected", "revoked", "stale"],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "authorityId",
        "participantId",
        "participantArtifactDigest",
        "acceptedNeedsDigest",
        "desiredGrantSet",
        "desiredCapabilities",
        "state",
        "version",
        "createdAt",
        "updatedAt",
        "expiresAt",
        "decision",
        "materialization",
        "kind",
        "principalId",
      ],
      "type": "object",
    },
  },
  "required": ["authority"],
  "type": "object",
} as const;

export const AuthPortalsGetRequestSchema = {
  "properties": { "portalId": { "minLength": 1, "type": "string" } },
  "required": ["portalId"],
  "type": "object",
} as const;

export const AuthPortalsGetResponseSchema = {
  "properties": {
    "portal": {
      "properties": {
        "builtIn": { "type": "boolean" },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "disabled": { "type": "boolean" },
        "displayName": { "minLength": 1, "type": "string" },
        "entryUrl": { "type": ["string", "null"] },
        "loginSettings": {
          "properties": {
            "federatedRegistration": { "type": "boolean" },
            "localLogin": { "type": "boolean" },
            "localRegistration": { "type": "boolean" },
            "providers": {
              "anyOf": [{
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              }, { "type": "null" }],
            },
          },
          "required": [
            "providers",
            "localLogin",
            "localRegistration",
            "federatedRegistration",
          ],
          "type": "object",
        },
        "portalId": { "minLength": 1, "type": "string" },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "portalId",
        "displayName",
        "entryUrl",
        "builtIn",
        "disabled",
        "loginSettings",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "routes": {
      "items": {
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "deploymentId": { "type": ["string", "null"] },
          "origin": { "type": ["string", "null"] },
          "participantId": { "type": ["string", "null"] },
          "portalId": { "minLength": 1, "type": "string" },
          "priority": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "routeId": { "minLength": 1, "type": "string" },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "routeId",
          "portalId",
          "participantId",
          "origin",
          "deploymentId",
          "priority",
          "createdAt",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
  },
  "required": ["portal", "routes"],
  "type": "object",
} as const;

export const AuthPortalsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "disabled": { "type": "boolean" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthPortalsListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "builtIn": { "type": "boolean" },
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "disabled": { "type": "boolean" },
          "displayName": { "minLength": 1, "type": "string" },
          "entryUrl": { "type": ["string", "null"] },
          "loginSettings": {
            "properties": {
              "federatedRegistration": { "type": "boolean" },
              "localLogin": { "type": "boolean" },
              "localRegistration": { "type": "boolean" },
              "providers": {
                "anyOf": [{
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                }, { "type": "null" }],
              },
            },
            "required": [
              "providers",
              "localLogin",
              "localRegistration",
              "federatedRegistration",
            ],
            "type": "object",
          },
          "portalId": { "minLength": 1, "type": "string" },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "portalId",
          "displayName",
          "entryUrl",
          "builtIn",
          "disabled",
          "loginSettings",
          "createdAt",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthPortalsLoginSettingsGetRequestSchema = {
  "properties": { "portalId": { "minLength": 1, "type": "string" } },
  "required": ["portalId"],
  "type": "object",
} as const;

export const AuthPortalsLoginSettingsGetResponseSchema = {
  "properties": {
    "portalId": { "minLength": 1, "type": "string" },
    "settings": {
      "properties": {
        "federatedRegistration": { "type": "boolean" },
        "localLogin": { "type": "boolean" },
        "localRegistration": { "type": "boolean" },
        "providers": {
          "anyOf": [{
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          }, { "type": "null" }],
        },
      },
      "required": [
        "providers",
        "localLogin",
        "localRegistration",
        "federatedRegistration",
      ],
      "type": "object",
    },
    "version": { "maximum": 9007199254740991, "minimum": 1, "type": "integer" },
  },
  "required": ["portalId", "settings", "version"],
  "type": "object",
} as const;

export const AuthPortalsLoginSettingsUpdateRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "portalId": { "minLength": 1, "type": "string" },
    "settings": {
      "properties": {
        "federatedRegistration": { "type": "boolean" },
        "localLogin": { "type": "boolean" },
        "localRegistration": { "type": "boolean" },
        "providers": {
          "anyOf": [{
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          }, { "type": "null" }],
        },
      },
      "required": [
        "providers",
        "localLogin",
        "localRegistration",
        "federatedRegistration",
      ],
      "type": "object",
    },
  },
  "required": ["portalId", "expectedVersion", "settings", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthPortalsLoginSettingsUpdateResponseSchema = {
  "properties": {
    "portalId": { "minLength": 1, "type": "string" },
    "settings": {
      "properties": {
        "federatedRegistration": { "type": "boolean" },
        "localLogin": { "type": "boolean" },
        "localRegistration": { "type": "boolean" },
        "providers": {
          "anyOf": [{
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          }, { "type": "null" }],
        },
      },
      "required": [
        "providers",
        "localLogin",
        "localRegistration",
        "federatedRegistration",
      ],
      "type": "object",
    },
    "version": { "maximum": 9007199254740991, "minimum": 1, "type": "integer" },
  },
  "required": ["portalId", "settings", "version"],
  "type": "object",
} as const;

export const AuthPortalsPutRequestSchema = {
  "properties": {
    "disabled": { "type": "boolean" },
    "displayName": { "minLength": 1, "type": "string" },
    "entryUrl": { "type": ["string", "null"] },
    "expectedVersion": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 1,
        "type": "integer",
      }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "loginSettings": {
      "properties": {
        "federatedRegistration": { "type": "boolean" },
        "localLogin": { "type": "boolean" },
        "localRegistration": { "type": "boolean" },
        "providers": {
          "anyOf": [{
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          }, { "type": "null" }],
        },
      },
      "required": [
        "providers",
        "localLogin",
        "localRegistration",
        "federatedRegistration",
      ],
      "type": "object",
    },
    "portalId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "portalId",
    "expectedVersion",
    "displayName",
    "entryUrl",
    "disabled",
    "loginSettings",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthPortalsPutResponseSchema = {
  "properties": {
    "portal": {
      "properties": {
        "builtIn": { "type": "boolean" },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "disabled": { "type": "boolean" },
        "displayName": { "minLength": 1, "type": "string" },
        "entryUrl": { "type": ["string", "null"] },
        "loginSettings": {
          "properties": {
            "federatedRegistration": { "type": "boolean" },
            "localLogin": { "type": "boolean" },
            "localRegistration": { "type": "boolean" },
            "providers": {
              "anyOf": [{
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              }, { "type": "null" }],
            },
          },
          "required": [
            "providers",
            "localLogin",
            "localRegistration",
            "federatedRegistration",
          ],
          "type": "object",
        },
        "portalId": { "minLength": 1, "type": "string" },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "portalId",
        "displayName",
        "entryUrl",
        "builtIn",
        "disabled",
        "loginSettings",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["portal"],
  "type": "object",
} as const;

export const AuthPortalsRemoveRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "portalId": { "minLength": 1, "type": "string" },
  },
  "required": ["portalId", "expectedVersion", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthPortalsRemoveResponseSchema = {
  "properties": { "removed": { "type": "boolean" } },
  "required": ["removed"],
  "type": "object",
} as const;

export const AuthPortalsRoutesPutRequestSchema = {
  "properties": {
    "deploymentId": { "type": ["string", "null"] },
    "expectedVersion": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 1,
        "type": "integer",
      }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "origin": { "type": ["string", "null"] },
    "participantId": { "type": ["string", "null"] },
    "portalId": { "minLength": 1, "type": "string" },
    "priority": { "minimum": 0, "type": "integer" },
    "routeId": { "type": ["string", "null"] },
  },
  "required": [
    "routeId",
    "portalId",
    "participantId",
    "origin",
    "deploymentId",
    "priority",
    "expectedVersion",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthPortalsRoutesPutResponseSchema = {
  "properties": {
    "route": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "type": ["string", "null"] },
        "origin": { "type": ["string", "null"] },
        "participantId": { "type": ["string", "null"] },
        "portalId": { "minLength": 1, "type": "string" },
        "priority": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "routeId": { "minLength": 1, "type": "string" },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "routeId",
        "portalId",
        "participantId",
        "origin",
        "deploymentId",
        "priority",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["route"],
  "type": "object",
} as const;

export const AuthPortalsRoutesRemoveRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "routeId": { "minLength": 1, "type": "string" },
  },
  "required": ["routeId", "expectedVersion", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthPortalsRoutesRemoveResponseSchema = {
  "properties": { "removed": { "type": "boolean" } },
  "required": ["removed"],
  "type": "object",
} as const;

export const AuthServiceInstancesDisableRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["instanceId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthServiceInstancesDisableResponseSchema = {
  "properties": {
    "instance": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "identityPublicKey": { "minLength": 1, "type": "string" },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["active", "disabled", "revoked", "stale"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["instance", "mutation"],
  "type": "object",
} as const;

export const AuthServiceInstancesEnableRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["instanceId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthServiceInstancesEnableResponseSchema = {
  "properties": {
    "instance": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "identityPublicKey": { "minLength": 1, "type": "string" },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["active", "disabled", "revoked", "stale"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["instance", "mutation"],
  "type": "object",
} as const;

export const AuthServiceInstancesListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "deploymentId": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "state": { "enum": ["active", "disabled", "revoked", "stale"] },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthServiceInstancesListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "deploymentId": { "minLength": 1, "type": "string" },
          "identityKeyId": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "identityPublicKey": { "minLength": 1, "type": "string" },
          "instanceId": { "minLength": 1, "type": "string" },
          "participantId": { "type": ["string", "null"] },
          "principalId": { "minLength": 1, "type": "string" },
          "state": { "enum": ["active", "disabled", "revoked", "stale"] },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "instanceId",
          "deploymentId",
          "principalId",
          "identityPublicKey",
          "identityKeyId",
          "participantId",
          "state",
          "createdAt",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthServiceInstancesProvisionRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "identityPublicKey": { "minLength": 1, "type": "string" },
    "instanceId": { "type": ["string", "null"] },
    "participantId": { "type": ["string", "null"] },
  },
  "required": [
    "deploymentId",
    "instanceId",
    "identityPublicKey",
    "participantId",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthServiceInstancesProvisionResponseSchema = {
  "properties": {
    "instance": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "identityPublicKey": { "minLength": 1, "type": "string" },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["active", "disabled", "revoked", "stale"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["instance"],
  "type": "object",
} as const;

export const AuthServiceInstancesRemoveRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
  },
  "required": ["instanceId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthServiceInstancesRemoveResponseSchema = {
  "properties": {
    "instance": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "identityPublicKey": { "minLength": 1, "type": "string" },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "state": { "enum": ["active", "disabled", "revoked", "stale"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "instanceId",
        "deploymentId",
        "principalId",
        "identityPublicKey",
        "identityKeyId",
        "participantId",
        "state",
        "createdAt",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "mutation": {
      "properties": {
        "changed": { "type": "boolean" },
        "resourceId": { "minLength": 1, "type": "string" },
        "state": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": ["resourceId", "state", "version", "changed"],
      "type": "object",
    },
  },
  "required": ["instance", "mutation"],
  "type": "object",
} as const;

export const AuthSessionsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "deploymentId": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "participantId": { "minLength": 1, "type": "string" },
    "principalId": { "minLength": 1, "type": "string" },
    "state": { "enum": ["active", "expired", "revoked"] },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthSessionsListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "expiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "inboxPrefix": { "minLength": 1, "type": "string" },
          "lastSeenAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "participantArtifactDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "participantId": { "minLength": 1, "type": "string" },
          "participantKind": { "enum": ["service", "app", "device", "agent"] },
          "participantNeedsDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "principalId": { "minLength": 1, "type": "string" },
          "principalKind": { "enum": ["user", "service", "device"] },
          "revokedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "sessionId": { "minLength": 1, "type": "string" },
          "sessionKeyId": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "sessionPublicKey": { "minLength": 1, "type": "string" },
          "state": { "enum": ["active", "expired", "revoked"] },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "sessionId",
          "principalId",
          "principalKind",
          "participantId",
          "participantKind",
          "participantArtifactDigest",
          "participantNeedsDigest",
          "sessionPublicKey",
          "sessionKeyId",
          "inboxPrefix",
          "state",
          "createdAt",
          "lastSeenAt",
          "expiresAt",
          "revokedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthSessionsLogoutRequestSchema = {
  "properties": {
    "issuedAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "proof": {
      "properties": {
        "format": { "const": "trellis.session-proof.v1" },
        "signature": { "minLength": 1, "type": "string" },
      },
      "required": ["format", "signature"],
      "type": "object",
    },
    "requestId": { "minLength": 1, "type": "string" },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": ["sessionId", "requestId", "issuedAt", "proof"],
  "type": "object",
} as const;

export const AuthSessionsLogoutResponseSchema = {
  "properties": {
    "kickedConnections": { "minimum": 0, "type": "integer" },
    "session": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "inboxPrefix": { "minLength": 1, "type": "string" },
        "lastSeenAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": { "enum": ["service", "app", "device", "agent"] },
        "participantNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "principalId": { "minLength": 1, "type": "string" },
        "principalKind": { "enum": ["user", "service", "device"] },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "sessionId": { "minLength": 1, "type": "string" },
        "sessionKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "sessionPublicKey": { "minLength": 1, "type": "string" },
        "state": { "enum": ["active", "expired", "revoked"] },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "sessionId",
        "principalId",
        "principalKind",
        "participantId",
        "participantKind",
        "participantArtifactDigest",
        "participantNeedsDigest",
        "sessionPublicKey",
        "sessionKeyId",
        "inboxPrefix",
        "state",
        "createdAt",
        "lastSeenAt",
        "expiresAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["session", "kickedConnections"],
  "type": "object",
} as const;

export const AuthSessionsMeRequestSchema = {
  "properties": {},
  "required": [],
  "type": "object",
} as const;

export const AuthSessionsMeResponseSchema = {
  "properties": {
    "deploymentId": { "type": ["string", "null"] },
    "instanceId": { "type": ["string", "null"] },
    "session": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "inboxPrefix": { "minLength": 1, "type": "string" },
        "lastSeenAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": { "enum": ["service", "app", "device", "agent"] },
        "participantNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "principalId": { "minLength": 1, "type": "string" },
        "principalKind": { "enum": ["user", "service", "device"] },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "sessionId": { "minLength": 1, "type": "string" },
        "sessionKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "sessionPublicKey": { "minLength": 1, "type": "string" },
        "state": { "enum": ["active", "expired", "revoked"] },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "sessionId",
        "principalId",
        "principalKind",
        "participantId",
        "participantKind",
        "participantArtifactDigest",
        "participantNeedsDigest",
        "sessionPublicKey",
        "sessionKeyId",
        "inboxPrefix",
        "state",
        "createdAt",
        "lastSeenAt",
        "expiresAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
    "user": {
      "anyOf": [{
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "disabledAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "email": { "type": ["string", "null"] },
          "image": { "type": ["string", "null"] },
          "name": { "type": ["string", "null"] },
          "principalId": { "minLength": 1, "type": "string" },
          "revokedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "state": { "enum": ["active", "disabled", "revoked"] },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "userId": { "minLength": 1, "type": "string" },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "userId",
          "principalId",
          "state",
          "name",
          "email",
          "image",
          "createdAt",
          "updatedAt",
          "disabledAt",
          "revokedAt",
          "version",
        ],
        "type": "object",
      }, { "type": "null" }],
    },
  },
  "required": ["session", "user", "deploymentId", "instanceId"],
  "type": "object",
} as const;

export const AuthSessionsRevokeRequestSchema = {
  "properties": {
    "expectedVersion": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 1,
        "type": "integer",
      }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": ["sessionId", "expectedVersion", "reason", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthSessionsRevokeResponseSchema = {
  "properties": {
    "kickedConnections": { "minimum": 0, "type": "integer" },
    "session": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "inboxPrefix": { "minLength": 1, "type": "string" },
        "lastSeenAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "participantArtifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": { "enum": ["service", "app", "device", "agent"] },
        "participantNeedsDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "principalId": { "minLength": 1, "type": "string" },
        "principalKind": { "enum": ["user", "service", "device"] },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "sessionId": { "minLength": 1, "type": "string" },
        "sessionKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "sessionPublicKey": { "minLength": 1, "type": "string" },
        "state": { "enum": ["active", "expired", "revoked"] },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "sessionId",
        "principalId",
        "principalKind",
        "participantId",
        "participantKind",
        "participantArtifactDigest",
        "participantNeedsDigest",
        "sessionPublicKey",
        "sessionKeyId",
        "inboxPrefix",
        "state",
        "createdAt",
        "lastSeenAt",
        "expiresAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["session", "kickedConnections"],
  "type": "object",
} as const;

export const AuthSessionsRevokedEventSchema = {
  "properties": {
    "eventId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "participantId": { "minLength": 1, "type": "string" },
    "principalId": { "minLength": 1, "type": "string" },
    "reason": { "type": ["string", "null"] },
    "revokedBy": { "type": ["string", "null"] },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "eventId",
    "occurredAt",
    "sessionId",
    "principalId",
    "participantId",
    "reason",
    "revokedBy",
  ],
  "type": "object",
} as const;

export const AuthUserIdentitiesListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "providerId": { "minLength": 1, "type": "string" },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthUserIdentitiesListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "lastSeenAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "observedEmail": { "type": ["string", "null"] },
          "observedName": { "type": ["string", "null"] },
          "principalId": { "minLength": 1, "type": "string" },
          "providerId": { "minLength": 1, "type": "string" },
          "subject": { "minLength": 1, "type": "string" },
          "username": { "type": ["string", "null"] },
        },
        "required": [
          "providerId",
          "subject",
          "principalId",
          "username",
          "observedName",
          "observedEmail",
          "createdAt",
          "lastSeenAt",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthUserIdentitiesUnlinkRequestSchema = {
  "properties": {
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "providerId": { "minLength": 1, "type": "string" },
    "subject": { "minLength": 1, "type": "string" },
  },
  "required": ["providerId", "subject", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthUserIdentitiesUnlinkResponseSchema = {
  "properties": { "unlinked": { "type": "boolean" } },
  "required": ["unlinked"],
  "type": "object",
} as const;

export const AuthUsersCreateRequestSchema = {
  "properties": {
    "email": { "type": ["string", "null"] },
    "idempotencyKey": { "maxLength": 256, "minLength": 1, "type": "string" },
    "image": { "type": ["string", "null"] },
    "name": { "type": ["string", "null"] },
  },
  "required": ["name", "email", "image", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthUsersCreateResponseSchema = {
  "properties": {
    "user": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "disabledAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "email": { "type": ["string", "null"] },
        "image": { "type": ["string", "null"] },
        "name": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "state": { "enum": ["active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "userId": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "userId",
        "principalId",
        "state",
        "name",
        "email",
        "image",
        "createdAt",
        "updatedAt",
        "disabledAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["user"],
  "type": "object",
} as const;

export const AuthUsersGetRequestSchema = {
  "properties": { "userId": { "minLength": 1, "type": "string" } },
  "required": ["userId"],
  "type": "object",
} as const;

export const AuthUsersGetResponseSchema = {
  "properties": {
    "user": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "disabledAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "email": { "type": ["string", "null"] },
        "image": { "type": ["string", "null"] },
        "name": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "state": { "enum": ["active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "userId": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "userId",
        "principalId",
        "state",
        "name",
        "email",
        "image",
        "createdAt",
        "updatedAt",
        "disabledAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["user"],
  "type": "object",
} as const;

export const AuthUsersIdentityLinkCreateRequestSchema = {
  "properties": {
    "allowedProviders": {
      "items": { "minLength": 1, "type": "string" },
      "type": "array",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "returnTarget": { "type": ["string", "null"] },
  },
  "required": ["allowedProviders", "returnTarget", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthUsersIdentityLinkCreateResponseSchema = {
  "properties": {
    "flow": {
      "properties": {
        "allowedProviders": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "completionUrl": { "minLength": 1, "type": "string" },
        "consumedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "flowId": { "minLength": 1, "type": "string" },
        "kind": { "const": "identity_link" },
        "returnTarget": { "type": ["string", "null"] },
        "targetPrincipalId": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "flowId",
        "kind",
        "targetPrincipalId",
        "allowedProviders",
        "returnTarget",
        "createdAt",
        "expiresAt",
        "consumedAt",
        "version",
        "completionUrl",
      ],
      "type": "object",
    },
  },
  "required": ["flow"],
  "type": "object",
} as const;

export const AuthUsersListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "state": { "enum": ["active", "disabled", "revoked"] },
  },
  "required": [],
  "type": "object",
} as const;

export const AuthUsersListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "disabledAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "email": { "type": ["string", "null"] },
          "image": { "type": ["string", "null"] },
          "name": { "type": ["string", "null"] },
          "principalId": { "minLength": 1, "type": "string" },
          "revokedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "state": { "enum": ["active", "disabled", "revoked"] },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "userId": { "minLength": 1, "type": "string" },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "userId",
          "principalId",
          "state",
          "name",
          "email",
          "image",
          "createdAt",
          "updatedAt",
          "disabledAt",
          "revokedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "type": ["string", "null"] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthUsersPasswordChangeRequestSchema = {
  "properties": {
    "currentPassword": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "newPassword": { "minLength": 1, "type": "string" },
  },
  "required": ["currentPassword", "newPassword", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthUsersPasswordChangeResponseSchema = {
  "properties": {
    "changedAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "revokedSessionCount": { "minimum": 0, "type": "integer" },
  },
  "required": ["changedAt", "revokedSessionCount"],
  "type": "object",
} as const;

export const AuthUsersPasswordResetCreateRequestSchema = {
  "properties": {
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "returnTarget": { "type": ["string", "null"] },
    "userId": { "minLength": 1, "type": "string" },
  },
  "required": ["userId", "returnTarget", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthUsersPasswordResetCreateResponseSchema = {
  "properties": {
    "flow": {
      "properties": {
        "allowedProviders": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "completionUrl": { "minLength": 1, "type": "string" },
        "consumedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "flowId": { "minLength": 1, "type": "string" },
        "kind": { "const": "password_reset" },
        "returnTarget": { "type": ["string", "null"] },
        "targetPrincipalId": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "flowId",
        "kind",
        "targetPrincipalId",
        "allowedProviders",
        "returnTarget",
        "createdAt",
        "expiresAt",
        "consumedAt",
        "version",
        "completionUrl",
      ],
      "type": "object",
    },
  },
  "required": ["flow"],
  "type": "object",
} as const;

export const AuthUsersResolveRequestSchema = {
  "properties": {
    "selector": {
      "anyOf": [{
        "properties": {
          "kind": { "const": "user" },
          "userId": { "minLength": 1, "type": "string" },
        },
        "required": ["kind", "userId"],
        "type": "object",
      }, {
        "properties": {
          "kind": { "const": "provider" },
          "providerId": { "minLength": 1, "type": "string" },
          "providerSubject": { "minLength": 1, "type": "string" },
        },
        "required": ["kind", "providerId", "providerSubject"],
        "type": "object",
      }],
    },
  },
  "required": ["selector"],
  "type": "object",
} as const;

export const AuthUsersResolveResponseSchema = {
  "properties": {
    "user": {
      "anyOf": [{
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "disabledAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "email": { "type": ["string", "null"] },
          "image": { "type": ["string", "null"] },
          "name": { "type": ["string", "null"] },
          "principalId": { "minLength": 1, "type": "string" },
          "revokedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": ["integer", "null"],
          },
          "state": { "enum": ["active", "disabled", "revoked"] },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "userId": { "minLength": 1, "type": "string" },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "userId",
          "principalId",
          "state",
          "name",
          "email",
          "image",
          "createdAt",
          "updatedAt",
          "disabledAt",
          "revokedAt",
          "version",
        ],
        "type": "object",
      }, { "type": "null" }],
    },
  },
  "required": ["user"],
  "type": "object",
} as const;

export const AuthUsersUpdateRequestSchema = {
  "properties": {
    "email": { "type": ["string", "null"] },
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "image": { "type": ["string", "null"] },
    "name": { "type": ["string", "null"] },
    "state": { "enum": ["active", "disabled"] },
    "userId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "userId",
    "expectedVersion",
    "name",
    "email",
    "image",
    "state",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthUsersUpdateResponseSchema = {
  "properties": {
    "user": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "disabledAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "email": { "type": ["string", "null"] },
        "image": { "type": ["string", "null"] },
        "name": { "type": ["string", "null"] },
        "principalId": { "minLength": 1, "type": "string" },
        "revokedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": ["integer", "null"],
        },
        "state": { "enum": ["active", "disabled", "revoked"] },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "userId": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "userId",
        "principalId",
        "state",
        "name",
        "email",
        "image",
        "createdAt",
        "updatedAt",
        "disabledAt",
        "revokedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["user"],
  "type": "object",
} as const;
