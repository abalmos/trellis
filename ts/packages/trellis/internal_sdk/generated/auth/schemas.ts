// Generated from trellis.auth@v1
export const AuthCapabilitiesListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "sourceApi": { "minLength": 1, "type": "string" },
  },
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
                  "anyOf": [
                    { "const": "call", "type": "string" },
                    { "const": "invoke", "type": "string" },
                    { "const": "observe", "type": "string" },
                    { "const": "cancel", "type": "string" },
                    { "const": "control", "type": "string" },
                    { "const": "publish", "type": "string" },
                    { "const": "subscribe", "type": "string" },
                    { "const": "read", "type": "string" },
                    { "const": "write", "type": "string" },
                    { "const": "delete", "type": "string" },
                    { "const": "submit", "type": "string" },
                    { "const": "process", "type": "string" },
                    { "const": "consume", "type": "string" },
                  ],
                },
                "target": {
                  "anyOf": [{
                    "properties": {
                      "api": { "minLength": 1, "type": "string" },
                      "kind": { "const": "apiSurface", "type": "string" },
                      "name": { "minLength": 1, "type": "string" },
                      "surface": {
                        "anyOf": [
                          { "const": "rpc", "type": "string" },
                          { "const": "operation", "type": "string" },
                          { "const": "event", "type": "string" },
                          { "const": "feed", "type": "string" },
                          { "const": "state", "type": "string" },
                        ],
                      },
                    },
                    "required": ["api", "kind", "name", "surface"],
                    "type": "object",
                  }, {
                    "properties": {
                      "api": { "minLength": 1, "type": "string" },
                      "kind": { "const": "operationSignal", "type": "string" },
                      "operation": { "minLength": 1, "type": "string" },
                      "signal": { "minLength": 1, "type": "string" },
                    },
                    "required": ["api", "kind", "operation", "signal"],
                    "type": "object",
                  }, {
                    "properties": {
                      "kind": {
                        "const": "participantResource",
                        "type": "string",
                      },
                      "name": { "minLength": 1, "type": "string" },
                      "participant": { "minLength": 1, "type": "string" },
                      "resource": {
                        "anyOf": [
                          { "const": "state", "type": "string" },
                          { "const": "jobQueue", "type": "string" },
                          { "const": "eventConsumer", "type": "string" },
                          { "const": "kv", "type": "string" },
                          { "const": "store", "type": "string" },
                        ],
                      },
                    },
                    "required": ["kind", "name", "participant", "resource"],
                    "type": "object",
                  }],
                },
              },
              "required": ["action", "target"],
              "type": "object",
            },
            "type": "array",
          },
          "capability": { "minLength": 1, "type": "string" },
          "description": { "minLength": 1, "type": "string" },
          "displayName": { "minLength": 1, "type": "string" },
          "sourceApi": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        },
        "required": [
          "allows",
          "capability",
          "description",
          "displayName",
          "sourceApi",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthCapabilityGroupsDeleteRequestSchema = {
  "properties": {
    "expectedVersion": { "minimum": 0, "type": "integer" },
    "groupKey": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
  },
  "required": ["expectedVersion", "groupKey", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthCapabilityGroupsDeleteResponseSchema = {
  "properties": { "success": { "type": "boolean" } },
  "required": ["success"],
  "type": "object",
} as const;

export const AuthCapabilityGroupsGetRequestSchema = {
  "properties": { "groupKey": { "minLength": 1, "type": "string" } },
  "required": ["groupKey"],
  "type": "object",
} as const;

export const AuthCapabilityGroupsGetResponseSchema = {
  "properties": {
    "group": {
      "properties": {
        "capabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "createdAt": { "minimum": 0, "type": "integer" },
        "description": { "type": "string" },
        "displayName": { "minLength": 1, "type": "string" },
        "groupKey": { "minLength": 1, "type": "string" },
        "includedGroups": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "updatedAt": { "minimum": 0, "type": "integer" },
        "version": { "minimum": 0, "type": "integer" },
      },
      "required": [
        "capabilities",
        "createdAt",
        "description",
        "displayName",
        "groupKey",
        "includedGroups",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["group"],
  "type": "object",
} as const;

export const AuthCapabilityGroupsListRequestSchema = {
  "properties": {
    "limit": { "maximum": 500, "minimum": 0, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
  },
  "required": ["limit"],
  "type": "object",
} as const;

export const AuthCapabilityGroupsListResponseSchema = {
  "properties": {
    "count": { "minimum": 0, "type": "integer" },
    "entries": {
      "items": {
        "properties": {
          "capabilities": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "createdAt": { "minimum": 0, "type": "integer" },
          "description": { "type": "string" },
          "displayName": { "minLength": 1, "type": "string" },
          "groupKey": { "minLength": 1, "type": "string" },
          "includedGroups": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "updatedAt": { "minimum": 0, "type": "integer" },
          "version": { "minimum": 0, "type": "integer" },
        },
        "required": [
          "capabilities",
          "createdAt",
          "description",
          "displayName",
          "groupKey",
          "includedGroups",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "limit": { "minimum": 0, "type": "integer" },
    "nextOffset": { "minimum": 0, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
  },
  "required": ["count", "entries", "limit", "offset"],
  "type": "object",
} as const;

export const AuthCapabilityGroupsPutRequestSchema = {
  "properties": {
    "capabilities": {
      "items": { "minLength": 1, "type": "string" },
      "type": "array",
    },
    "description": { "minLength": 1, "type": "string" },
    "displayName": { "minLength": 1, "type": "string" },
    "expectedVersion": {
      "anyOf": [{ "minimum": 0, "type": "integer" }, { "type": "null" }],
    },
    "groupKey": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "includedGroups": {
      "items": { "minLength": 1, "type": "string" },
      "type": "array",
    },
  },
  "required": [
    "capabilities",
    "description",
    "displayName",
    "expectedVersion",
    "groupKey",
    "idempotencyKey",
    "includedGroups",
  ],
  "type": "object",
} as const;

export const AuthCapabilityGroupsPutResponseSchema = {
  "properties": {
    "group": {
      "properties": {
        "capabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "createdAt": { "minimum": 0, "type": "integer" },
        "description": { "type": "string" },
        "displayName": { "minLength": 1, "type": "string" },
        "groupKey": { "minLength": 1, "type": "string" },
        "includedGroups": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "updatedAt": { "minimum": 0, "type": "integer" },
        "version": { "minimum": 0, "type": "integer" },
      },
      "required": [
        "capabilities",
        "createdAt",
        "description",
        "displayName",
        "groupKey",
        "includedGroups",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["group"],
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "connectionId",
    "eventId",
    "occurredAt",
    "participantId",
    "principalId",
    "reason",
    "sessionId",
  ],
  "type": "object",
} as const;

export const AuthConnectionsKickRequestSchema = {
  "properties": {
    "connectionId": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["connectionId", "idempotencyKey", "reason"],
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "connectionId",
    "eventId",
    "occurredAt",
    "participantId",
    "principalId",
    "reason",
    "sessionId",
  ],
  "type": "object",
} as const;

export const AuthConnectionsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "sessionId": { "minLength": 1, "type": "string" },
  },
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
          "contextDigest": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "deploymentId": {
            "anyOf": [{ "minLength": 1, "type": "string" }, { "type": "null" }],
          },
          "instanceId": {
            "anyOf": [{ "minLength": 1, "type": "string" }, { "type": "null" }],
          },
          "lastSeenAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "loginSessionId": {
            "anyOf": [{ "minLength": 1, "type": "string" }, { "type": "null" }],
          },
          "participantId": { "minLength": 1, "type": "string" },
          "principalId": { "minLength": 1, "type": "string" },
          "remoteAddress": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "runtimeConnectionId": { "minLength": 1, "type": "string" },
          "serverId": { "minLength": 1, "type": "string" },
          "userNkey": { "minLength": 1, "type": "string" },
        },
        "required": [
          "clientId",
          "connectedAt",
          "connectionId",
          "lastSeenAt",
          "remoteAddress",
          "serverId",
          "runtimeConnectionId",
          "loginSessionId",
          "contextDigest",
          "principalId",
          "participantId",
          "deploymentId",
          "instanceId",
          "userNkey",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
    "clientId",
    "connectionId",
    "eventId",
    "occurredAt",
    "participantId",
    "principalId",
    "serverId",
    "sessionId",
  ],
  "type": "object",
} as const;

export const AuthDeploymentsApplyRequestSchema = {
  "properties": {
    "apiArtifacts": {
      "items": { "properties": {}, "type": "object" },
      "type": "array",
    },
    "deploymentId": { "minLength": 1, "type": "string" },
    "expectedRevision": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "optionalCapabilities": {
      "items": { "minLength": 1, "type": "string" },
      "type": "array",
    },
    "participantArtifact": { "properties": {}, "type": "object" },
  },
  "required": [
    "deploymentId",
    "participantArtifact",
    "apiArtifacts",
    "expectedRevision",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthDeploymentsApplyResponseSchema = {
  "properties": {
    "binding": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "grants": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1", "type": "string" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "anyOf": [
                      { "const": "call", "type": "string" },
                      { "const": "invoke", "type": "string" },
                      { "const": "observe", "type": "string" },
                      { "const": "cancel", "type": "string" },
                      { "const": "control", "type": "string" },
                      { "const": "publish", "type": "string" },
                      { "const": "subscribe", "type": "string" },
                      { "const": "read", "type": "string" },
                      { "const": "write", "type": "string" },
                      { "const": "delete", "type": "string" },
                      { "const": "submit", "type": "string" },
                      { "const": "process", "type": "string" },
                      { "const": "consume", "type": "string" },
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface", "type": "string" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "anyOf": [
                            { "const": "rpc", "type": "string" },
                            { "const": "operation", "type": "string" },
                            { "const": "event", "type": "string" },
                            { "const": "feed", "type": "string" },
                            { "const": "state", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": {
                          "const": "operationSignal",
                          "type": "string",
                        },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": {
                          "const": "participantResource",
                          "type": "string",
                        },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "anyOf": [
                            { "const": "state", "type": "string" },
                            { "const": "jobQueue", "type": "string" },
                            { "const": "eventConsumer", "type": "string" },
                            { "const": "kv", "type": "string" },
                            { "const": "store", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["action", "target"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "installedRevision": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
        "ownerId": { "minLength": 1, "type": "string" },
        "ownerKind": {
          "anyOf": [{ "const": "deployment", "type": "string" }, {
            "const": "user",
            "type": "string",
          }],
        },
        "participantId": { "minLength": 1, "type": "string" },
        "platformPrivileges": {
          "items": { "const": "trellis.auth::admin", "type": "string" },
          "type": "array",
        },
        "provenance": {
          "anyOf": [{
            "properties": {
              "effectivePolicyDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "portalId": { "minLength": 1, "type": "string" },
              "providerId": { "minLength": 1, "type": "string" },
              "roles": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
            },
            "required": [
              "portalId",
              "providerId",
              "roles",
              "effectivePolicyDigest",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "revision": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "revoked",
            "type": "string",
          }],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
      },
      "required": [
        "ownerKind",
        "ownerId",
        "participantId",
        "installedRevision",
        "grants",
        "platformPrivileges",
        "revision",
        "state",
        "expiresAt",
        "provenance",
        "createdAt",
        "updatedAt",
      ],
      "type": "object",
    },
    "deployment": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "disabledAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "portalId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "reviewMode": {
          "anyOf": [
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
          ],
        },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
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
        "createdAt",
        "deploymentId",
        "disabledAt",
        "displayName",
        "expiresAt",
        "kind",
        "participantId",
        "portalId",
        "requiresDeviceDelegation",
        "reviewMode",
        "revokedAt",
        "state",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["deployment", "binding"],
  "type": "object",
} as const;

export const AuthDeploymentsCreateRequestSchema = {
  "properties": {
    "displayName": { "minLength": 1, "type": "string" },
    "expiresAt": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 0,
        "type": "integer",
      }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "kind": {
      "anyOf": [{ "const": "service", "type": "string" }, {
        "const": "device",
        "type": "string",
      }],
    },
    "participantId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "portalId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "requiresDeviceDelegation": { "type": "boolean" },
    "reviewMode": {
      "anyOf": [
        { "const": "none", "type": "string" },
        { "const": "required", "type": "string" },
        { "type": "null" },
        { "const": "none", "type": "string" },
        { "const": "required", "type": "string" },
        { "type": "null" },
      ],
    },
  },
  "required": [
    "displayName",
    "expiresAt",
    "idempotencyKey",
    "kind",
    "participantId",
    "portalId",
    "requiresDeviceDelegation",
    "reviewMode",
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "portalId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "reviewMode": {
          "anyOf": [
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
          ],
        },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
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
        "createdAt",
        "deploymentId",
        "disabledAt",
        "displayName",
        "expiresAt",
        "kind",
        "participantId",
        "portalId",
        "requiresDeviceDelegation",
        "reviewMode",
        "revokedAt",
        "state",
        "updatedAt",
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["deploymentId", "expectedVersion", "idempotencyKey", "reason"],
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "portalId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "reviewMode": {
          "anyOf": [
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
          ],
        },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
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
        "createdAt",
        "deploymentId",
        "disabledAt",
        "displayName",
        "expiresAt",
        "kind",
        "participantId",
        "portalId",
        "requiresDeviceDelegation",
        "reviewMode",
        "revokedAt",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["deploymentId", "expectedVersion", "idempotencyKey", "reason"],
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "portalId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "reviewMode": {
          "anyOf": [
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
          ],
        },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
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
        "createdAt",
        "deploymentId",
        "disabledAt",
        "displayName",
        "expiresAt",
        "kind",
        "participantId",
        "portalId",
        "requiresDeviceDelegation",
        "reviewMode",
        "revokedAt",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
      "type": "object",
    },
  },
  "required": ["deployment", "mutation"],
  "type": "object",
} as const;

export const AuthDeploymentsGetRequestSchema = {
  "properties": { "deploymentId": { "minLength": 1, "type": "string" } },
  "required": ["deploymentId"],
  "type": "object",
} as const;

export const AuthDeploymentsGetResponseSchema = {
  "properties": {
    "binding": {
      "anyOf": [{
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "expiresAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "grants": {
            "properties": {
              "format": { "const": "trellis.grant-set.v1", "type": "string" },
              "permissions": {
                "items": {
                  "properties": {
                    "action": {
                      "anyOf": [
                        { "const": "call", "type": "string" },
                        { "const": "invoke", "type": "string" },
                        { "const": "observe", "type": "string" },
                        { "const": "cancel", "type": "string" },
                        { "const": "control", "type": "string" },
                        { "const": "publish", "type": "string" },
                        { "const": "subscribe", "type": "string" },
                        { "const": "read", "type": "string" },
                        { "const": "write", "type": "string" },
                        { "const": "delete", "type": "string" },
                        { "const": "submit", "type": "string" },
                        { "const": "process", "type": "string" },
                        { "const": "consume", "type": "string" },
                      ],
                    },
                    "target": {
                      "anyOf": [{
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "apiSurface", "type": "string" },
                          "name": { "minLength": 1, "type": "string" },
                          "surface": {
                            "anyOf": [
                              { "const": "rpc", "type": "string" },
                              { "const": "operation", "type": "string" },
                              { "const": "event", "type": "string" },
                              { "const": "feed", "type": "string" },
                              { "const": "state", "type": "string" },
                            ],
                          },
                        },
                        "required": ["kind", "api", "surface", "name"],
                        "type": "object",
                      }, {
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": {
                            "const": "operationSignal",
                            "type": "string",
                          },
                          "operation": { "minLength": 1, "type": "string" },
                          "signal": { "minLength": 1, "type": "string" },
                        },
                        "required": ["kind", "api", "operation", "signal"],
                        "type": "object",
                      }, {
                        "properties": {
                          "kind": {
                            "const": "participantResource",
                            "type": "string",
                          },
                          "name": { "minLength": 1, "type": "string" },
                          "participant": { "minLength": 1, "type": "string" },
                          "resource": {
                            "anyOf": [
                              { "const": "state", "type": "string" },
                              { "const": "jobQueue", "type": "string" },
                              { "const": "eventConsumer", "type": "string" },
                              { "const": "kv", "type": "string" },
                              { "const": "store", "type": "string" },
                            ],
                          },
                        },
                        "required": ["kind", "participant", "resource", "name"],
                        "type": "object",
                      }],
                    },
                  },
                  "required": ["action", "target"],
                  "type": "object",
                },
                "type": "array",
              },
            },
            "required": ["format", "permissions"],
            "type": "object",
          },
          "installedRevision": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
          "ownerId": { "minLength": 1, "type": "string" },
          "ownerKind": {
            "anyOf": [{ "const": "deployment", "type": "string" }, {
              "const": "user",
              "type": "string",
            }],
          },
          "participantId": { "minLength": 1, "type": "string" },
          "platformPrivileges": {
            "items": { "const": "trellis.auth::admin", "type": "string" },
            "type": "array",
          },
          "provenance": {
            "anyOf": [{
              "properties": {
                "effectivePolicyDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "portalId": { "minLength": 1, "type": "string" },
                "providerId": { "minLength": 1, "type": "string" },
                "roles": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
              },
              "required": [
                "portalId",
                "providerId",
                "roles",
                "effectivePolicyDigest",
              ],
              "type": "object",
            }, { "type": "null" }],
          },
          "revision": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "revoked",
              "type": "string",
            }],
          },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
        },
        "required": [
          "ownerKind",
          "ownerId",
          "participantId",
          "installedRevision",
          "grants",
          "platformPrivileges",
          "revision",
          "state",
          "expiresAt",
          "provenance",
          "createdAt",
          "updatedAt",
        ],
        "type": "object",
      }, { "type": "null" }],
    },
    "deployment": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "disabledAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "portalId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "reviewMode": {
          "anyOf": [
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
          ],
        },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
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
        "createdAt",
        "deploymentId",
        "disabledAt",
        "displayName",
        "expiresAt",
        "kind",
        "participantId",
        "portalId",
        "requiresDeviceDelegation",
        "reviewMode",
        "revokedAt",
        "state",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "resources": {
      "items": {
        "properties": {
          "bindingId": { "minLength": 1, "type": "string" },
          "error": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "localName": { "minLength": 1, "type": "string" },
          "materializedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "ownerParticipantId": { "minLength": 1, "type": "string" },
          "providerIdentity": {
            "anyOf": [{
              "properties": {
                "bucket": { "minLength": 1, "type": "string" },
                "kind": { "const": "kv", "type": "string" },
              },
              "required": ["kind", "bucket"],
              "type": "object",
            }, {
              "properties": {
                "bucket": { "minLength": 1, "type": "string" },
                "kind": { "const": "store", "type": "string" },
              },
              "required": ["kind", "bucket"],
              "type": "object",
            }, {
              "properties": {
                "bucket": { "minLength": 1, "type": "string" },
                "kind": { "const": "state", "type": "string" },
              },
              "required": ["kind", "bucket"],
              "type": "object",
            }, {
              "properties": {
                "consumer": { "minLength": 1, "type": "string" },
                "kind": { "const": "jobQueue", "type": "string" },
                "namespace": { "minLength": 1, "type": "string" },
                "publishPrefix": { "minLength": 1, "type": "string" },
                "updatesPrefix": {
                  "anyOf": [{ "minLength": 1, "type": "string" }, {
                    "type": "null",
                  }],
                },
                "workStream": { "minLength": 1, "type": "string" },
                "workSubject": { "minLength": 1, "type": "string" },
              },
              "required": [
                "kind",
                "namespace",
                "workStream",
                "publishPrefix",
                "updatesPrefix",
                "workSubject",
                "consumer",
              ],
              "type": "object",
            }, {
              "properties": {
                "consumer": { "minLength": 1, "type": "string" },
                "filterSubjects": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
                "kind": { "const": "eventConsumer", "type": "string" },
                "stream": { "minLength": 1, "type": "string" },
              },
              "required": ["kind", "stream", "consumer", "filterSubjects"],
              "type": "object",
            }],
          },
          "resourceKind": { "minLength": 1, "type": "string" },
          "state": {
            "anyOf": [{ "const": "available", "type": "string" }, {
              "const": "unavailable",
              "type": "string",
            }, { "const": "stale", "type": "string" }],
          },
        },
        "required": [
          "resourceKind",
          "localName",
          "bindingId",
          "ownerParticipantId",
          "providerIdentity",
          "state",
          "materializedAt",
          "error",
        ],
        "type": "object",
      },
      "type": "array",
    },
  },
  "required": ["deployment", "binding", "resources"],
  "type": "object",
} as const;

export const AuthDeploymentsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "kind": {
      "anyOf": [{ "const": "service", "type": "string" }, {
        "const": "device",
        "type": "string",
      }],
    },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "state": {
      "anyOf": [{ "const": "active", "type": "string" }, {
        "const": "disabled",
        "type": "string",
      }, { "const": "revoked", "type": "string" }],
    },
  },
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
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "displayName": { "minLength": 1, "type": "string" },
          "expiresAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "kind": {
            "anyOf": [{ "const": "service", "type": "string" }, {
              "const": "device",
              "type": "string",
            }],
          },
          "participantId": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "portalId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "requiresDeviceDelegation": { "type": "boolean" },
          "reviewMode": {
            "anyOf": [
              { "const": "none", "type": "string" },
              { "const": "required", "type": "string" },
              { "type": "null" },
              { "const": "none", "type": "string" },
              { "const": "required", "type": "string" },
              { "type": "null" },
            ],
          },
          "revokedAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "disabled",
              "type": "string",
            }, { "const": "revoked", "type": "string" }],
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
          "createdAt",
          "deploymentId",
          "disabledAt",
          "displayName",
          "expiresAt",
          "kind",
          "participantId",
          "portalId",
          "requiresDeviceDelegation",
          "reviewMode",
          "revokedAt",
          "state",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["deploymentId", "expectedVersion", "idempotencyKey", "reason"],
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "displayName": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "kind": {
          "anyOf": [{ "const": "service", "type": "string" }, {
            "const": "device",
            "type": "string",
          }],
        },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "portalId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "requiresDeviceDelegation": { "type": "boolean" },
        "reviewMode": {
          "anyOf": [
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
            { "const": "none", "type": "string" },
            { "const": "required", "type": "string" },
            { "type": "null" },
          ],
        },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
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
        "createdAt",
        "deploymentId",
        "disabledAt",
        "displayName",
        "expiresAt",
        "kind",
        "participantId",
        "portalId",
        "requiresDeviceDelegation",
        "reviewMode",
        "revokedAt",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
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
    "approvedBy",
    "deploymentId",
    "eventId",
    "instanceId",
    "occurredAt",
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
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "device": {
            "properties": {
              "administrativeApproval": {
                "anyOf": [
                  { "const": "pending", "type": "string" },
                  { "const": "approved", "type": "string" },
                  { "const": "rejected", "type": "string" },
                  { "const": "revoked", "type": "string" },
                ],
              },
              "createdAt": {
                "maximum": 9007199254740991,
                "minimum": 0,
                "type": "integer",
              },
              "delegationExpiresAt": {
                "anyOf": [{
                  "maximum": 9007199254740991,
                  "minimum": 0,
                  "type": "integer",
                }, { "type": "null" }],
              },
              "delegationRequired": { "type": "boolean" },
              "delegationState": {
                "anyOf": [{ "const": "active", "type": "string" }, {
                  "const": "missing",
                  "type": "string",
                }, { "const": "revoked", "type": "string" }],
              },
              "deploymentId": { "minLength": 1, "type": "string" },
              "identityKeyId": {
                "anyOf": [{
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                }, { "type": "null" }],
              },
              "identityPublicKey": {
                "anyOf": [{ "type": "string" }, { "type": "null" }],
              },
              "instanceId": { "minLength": 1, "type": "string" },
              "participantId": {
                "anyOf": [{ "type": "string" }, { "type": "null" }],
              },
              "principalId": { "minLength": 1, "type": "string" },
              "state": {
                "anyOf": [
                  { "const": "pending", "type": "string" },
                  { "const": "active", "type": "string" },
                  { "const": "disabled", "type": "string" },
                  { "const": "revoked", "type": "string" },
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
              "administrativeApproval",
              "createdAt",
              "delegationExpiresAt",
              "delegationRequired",
              "delegationState",
              "deploymentId",
              "identityKeyId",
              "identityPublicKey",
              "instanceId",
              "participantId",
              "principalId",
              "state",
              "updatedAt",
              "version",
            ],
            "type": "object",
          },
        },
        "required": ["device"],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
    "deploymentId",
    "eventId",
    "instanceId",
    "occurredAt",
    "userPrincipalId",
  ],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesResolveProgressSchema = {
  "properties": {
    "retryAfterMs": { "minimum": 0, "type": "integer" },
    "state": {
      "anyOf": [{ "const": "waiting", "type": "string" }, {
        "const": "review_pending",
        "type": "string",
      }, { "const": "delegation_pending", "type": "string" }],
    },
  },
  "required": ["retryAfterMs", "state"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesResolveRequestSchema = {
  "properties": {
    "confirmationCode": { "minLength": 1, "type": "string" },
    "flowId": { "minLength": 1, "type": "string" },
  },
  "required": ["confirmationCode", "flowId"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesResolveResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "approved", "type": "string" },
            { "const": "rejected", "type": "string" },
            { "const": "revoked", "type": "string" },
          ],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "missing",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "anyOf": [{ "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" }, {
            "type": "null",
          }],
        },
        "identityPublicKey": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
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
        "administrativeApproval",
        "createdAt",
        "delegationExpiresAt",
        "delegationRequired",
        "delegationState",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "review": {
      "properties": {
        "activatedByUserPrincipalId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "decidedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "decidedBy": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "devicePrincipalId": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "requestedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "reviewId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "approved", "type": "string" },
            { "const": "rejected", "type": "string" },
            { "const": "expired", "type": "string" },
            { "const": "revoked", "type": "string" },
          ],
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "activatedByUserPrincipalId",
        "decidedAt",
        "decidedBy",
        "deploymentId",
        "devicePrincipalId",
        "expiresAt",
        "instanceId",
        "reason",
        "requestedAt",
        "reviewId",
        "state",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["device", "review"],
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
  "required": ["deploymentId", "eventId", "instanceId", "occurredAt", "state"],
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
    "deploymentId",
    "eventId",
    "instanceId",
    "occurredAt",
    "reviewId",
  ],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesReviewsDecideRequestSchema = {
  "properties": {
    "decision": {
      "anyOf": [{ "const": "approve", "type": "string" }, {
        "const": "reject",
        "type": "string",
      }],
    },
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "reviewId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "decision",
    "expectedVersion",
    "idempotencyKey",
    "reason",
    "reviewId",
  ],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesReviewsDecideResponseSchema = {
  "properties": {
    "review": {
      "properties": {
        "activatedByUserPrincipalId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "confirmationCode": { "minLength": 1, "type": "string" },
        "decidedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "decidedBy": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "deploymentId": { "minLength": 1, "type": "string" },
        "devicePrincipalId": { "minLength": 1, "type": "string" },
        "expiresAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "requestedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "reviewId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "approved", "type": "string" },
            { "const": "rejected", "type": "string" },
            { "const": "expired", "type": "string" },
            { "const": "revoked", "type": "string" },
          ],
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "activatedByUserPrincipalId",
        "confirmationCode",
        "decidedAt",
        "decidedBy",
        "deploymentId",
        "devicePrincipalId",
        "expiresAt",
        "instanceId",
        "reason",
        "requestedAt",
        "reviewId",
        "state",
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
      "anyOf": [
        { "const": "pending", "type": "string" },
        { "const": "approved", "type": "string" },
        { "const": "rejected", "type": "string" },
        { "const": "expired", "type": "string" },
        { "const": "revoked", "type": "string" },
      ],
    },
  },
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesReviewsListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "activatedByUserPrincipalId": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "confirmationCode": { "minLength": 1, "type": "string" },
          "decidedAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "decidedBy": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "deploymentId": { "minLength": 1, "type": "string" },
          "devicePrincipalId": { "minLength": 1, "type": "string" },
          "expiresAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "instanceId": { "minLength": 1, "type": "string" },
          "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "requestedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "reviewId": { "minLength": 1, "type": "string" },
          "state": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "approved", "type": "string" },
              { "const": "rejected", "type": "string" },
              { "const": "expired", "type": "string" },
              { "const": "revoked", "type": "string" },
            ],
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "activatedByUserPrincipalId",
          "confirmationCode",
          "decidedAt",
          "decidedBy",
          "deploymentId",
          "devicePrincipalId",
          "expiresAt",
          "instanceId",
          "reason",
          "requestedAt",
          "reviewId",
          "state",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesRevokeRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "devicePrincipalId": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["deploymentId", "devicePrincipalId", "idempotencyKey", "reason"],
  "type": "object",
} as const;

export const AuthDeviceUserAuthoritiesRevokeResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "approved", "type": "string" },
            { "const": "rejected", "type": "string" },
            { "const": "revoked", "type": "string" },
          ],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "missing",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "anyOf": [{ "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" }, {
            "type": "null",
          }],
        },
        "identityPublicKey": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
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
        "administrativeApproval",
        "createdAt",
        "delegationExpiresAt",
        "delegationRequired",
        "delegationState",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
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

export const AuthDevicesDisableRequestSchema = {
  "properties": {
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "instanceId": { "minLength": 1, "type": "string" },
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["expectedVersion", "idempotencyKey", "instanceId", "reason"],
  "type": "object",
} as const;

export const AuthDevicesDisableResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "approved", "type": "string" },
            { "const": "rejected", "type": "string" },
            { "const": "revoked", "type": "string" },
          ],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "missing",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "anyOf": [{ "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" }, {
            "type": "null",
          }],
        },
        "identityPublicKey": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
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
        "administrativeApproval",
        "createdAt",
        "delegationExpiresAt",
        "delegationRequired",
        "delegationState",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["expectedVersion", "idempotencyKey", "instanceId", "reason"],
  "type": "object",
} as const;

export const AuthDevicesEnableResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "approved", "type": "string" },
            { "const": "rejected", "type": "string" },
            { "const": "revoked", "type": "string" },
          ],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "missing",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "anyOf": [{ "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" }, {
            "type": "null",
          }],
        },
        "identityPublicKey": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
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
        "administrativeApproval",
        "createdAt",
        "delegationExpiresAt",
        "delegationRequired",
        "delegationState",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
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
    "state": {
      "anyOf": [
        { "const": "pending", "type": "string" },
        { "const": "active", "type": "string" },
        { "const": "disabled", "type": "string" },
        { "const": "revoked", "type": "string" },
      ],
    },
  },
  "type": "object",
} as const;

export const AuthDevicesListResponseSchema = {
  "properties": {
    "entries": {
      "items": {
        "properties": {
          "administrativeApproval": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "approved", "type": "string" },
              { "const": "rejected", "type": "string" },
              { "const": "revoked", "type": "string" },
            ],
          },
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "delegationExpiresAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "delegationRequired": { "type": "boolean" },
          "delegationState": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "missing",
              "type": "string",
            }, { "const": "revoked", "type": "string" }],
          },
          "deploymentId": { "minLength": 1, "type": "string" },
          "identityKeyId": {
            "anyOf": [{ "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" }, {
              "type": "null",
            }],
          },
          "identityPublicKey": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "instanceId": { "minLength": 1, "type": "string" },
          "participantId": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "principalId": { "minLength": 1, "type": "string" },
          "state": {
            "anyOf": [
              { "const": "pending", "type": "string" },
              { "const": "active", "type": "string" },
              { "const": "disabled", "type": "string" },
              { "const": "revoked", "type": "string" },
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
          "administrativeApproval",
          "createdAt",
          "delegationExpiresAt",
          "delegationRequired",
          "delegationState",
          "deploymentId",
          "identityKeyId",
          "identityPublicKey",
          "instanceId",
          "participantId",
          "principalId",
          "state",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthDevicesProvisionRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "identityPublicKey": {
      "anyOf": [{ "type": "string" }, { "type": "null" }],
    },
    "instanceId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "participantId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": [
    "deploymentId",
    "idempotencyKey",
    "identityPublicKey",
    "instanceId",
    "participantId",
  ],
  "type": "object",
} as const;

export const AuthDevicesProvisionResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "approved", "type": "string" },
            { "const": "rejected", "type": "string" },
            { "const": "revoked", "type": "string" },
          ],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "missing",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "anyOf": [{ "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" }, {
            "type": "null",
          }],
        },
        "identityPublicKey": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
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
        "administrativeApproval",
        "createdAt",
        "delegationExpiresAt",
        "delegationRequired",
        "delegationState",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
    "provisioningSecret": {
      "anyOf": [{ "type": "string" }, { "type": "null" }],
    },
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["expectedVersion", "idempotencyKey", "instanceId", "reason"],
  "type": "object",
} as const;

export const AuthDevicesRemoveResponseSchema = {
  "properties": {
    "device": {
      "properties": {
        "administrativeApproval": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "approved", "type": "string" },
            { "const": "rejected", "type": "string" },
            { "const": "revoked", "type": "string" },
          ],
        },
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "delegationExpiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "delegationRequired": { "type": "boolean" },
        "delegationState": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "missing",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
        "deploymentId": { "minLength": 1, "type": "string" },
        "identityKeyId": {
          "anyOf": [{ "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" }, {
            "type": "null",
          }],
        },
        "identityPublicKey": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "instanceId": { "minLength": 1, "type": "string" },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "pending", "type": "string" },
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
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
        "administrativeApproval",
        "createdAt",
        "delegationExpiresAt",
        "delegationRequired",
        "delegationState",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
      "type": "object",
    },
  },
  "required": ["device", "mutation"],
  "type": "object",
} as const;

export const AuthErrorDetailsSchema = {
  "properties": {
    "code": { "minLength": 1, "type": "string" },
    "field": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "message": { "minLength": 1, "type": "string" },
    "retryable": { "type": "boolean" },
  },
  "required": ["code", "field", "message", "retryable"],
  "type": "object",
} as const;

export const AuthGrantsChangedEventSchema = {
  "properties": {
    "binding": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "grants": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1", "type": "string" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "anyOf": [
                      { "const": "call", "type": "string" },
                      { "const": "invoke", "type": "string" },
                      { "const": "observe", "type": "string" },
                      { "const": "cancel", "type": "string" },
                      { "const": "control", "type": "string" },
                      { "const": "publish", "type": "string" },
                      { "const": "subscribe", "type": "string" },
                      { "const": "read", "type": "string" },
                      { "const": "write", "type": "string" },
                      { "const": "delete", "type": "string" },
                      { "const": "submit", "type": "string" },
                      { "const": "process", "type": "string" },
                      { "const": "consume", "type": "string" },
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface", "type": "string" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "anyOf": [
                            { "const": "rpc", "type": "string" },
                            { "const": "operation", "type": "string" },
                            { "const": "event", "type": "string" },
                            { "const": "feed", "type": "string" },
                            { "const": "state", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": {
                          "const": "operationSignal",
                          "type": "string",
                        },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": {
                          "const": "participantResource",
                          "type": "string",
                        },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "anyOf": [
                            { "const": "state", "type": "string" },
                            { "const": "jobQueue", "type": "string" },
                            { "const": "eventConsumer", "type": "string" },
                            { "const": "kv", "type": "string" },
                            { "const": "store", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["action", "target"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "installedRevision": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
        "ownerId": { "minLength": 1, "type": "string" },
        "ownerKind": {
          "anyOf": [{ "const": "deployment", "type": "string" }, {
            "const": "user",
            "type": "string",
          }],
        },
        "participantId": { "minLength": 1, "type": "string" },
        "platformPrivileges": {
          "items": { "const": "trellis.auth::admin", "type": "string" },
          "type": "array",
        },
        "provenance": {
          "anyOf": [{
            "properties": {
              "effectivePolicyDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "portalId": { "minLength": 1, "type": "string" },
              "providerId": { "minLength": 1, "type": "string" },
              "roles": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
            },
            "required": [
              "portalId",
              "providerId",
              "roles",
              "effectivePolicyDigest",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "revision": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "revoked",
            "type": "string",
          }],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
      },
      "required": [
        "ownerKind",
        "ownerId",
        "participantId",
        "installedRevision",
        "grants",
        "platformPrivileges",
        "revision",
        "state",
        "expiresAt",
        "provenance",
        "createdAt",
        "updatedAt",
      ],
      "type": "object",
    },
    "eventId": { "minLength": 1, "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
  },
  "required": ["eventId", "occurredAt", "binding"],
  "type": "object",
} as const;

export const AuthGrantsGetRequestSchema = {
  "properties": {
    "ownerId": { "minLength": 1, "type": "string" },
    "ownerKind": {
      "anyOf": [{ "const": "deployment", "type": "string" }, {
        "const": "user",
        "type": "string",
      }],
    },
    "participantId": { "minLength": 1, "type": "string" },
  },
  "required": ["ownerKind", "ownerId", "participantId"],
  "type": "object",
} as const;

export const AuthGrantsGetResponseSchema = {
  "properties": {
    "binding": {
      "anyOf": [{
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "expiresAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "grants": {
            "properties": {
              "format": { "const": "trellis.grant-set.v1", "type": "string" },
              "permissions": {
                "items": {
                  "properties": {
                    "action": {
                      "anyOf": [
                        { "const": "call", "type": "string" },
                        { "const": "invoke", "type": "string" },
                        { "const": "observe", "type": "string" },
                        { "const": "cancel", "type": "string" },
                        { "const": "control", "type": "string" },
                        { "const": "publish", "type": "string" },
                        { "const": "subscribe", "type": "string" },
                        { "const": "read", "type": "string" },
                        { "const": "write", "type": "string" },
                        { "const": "delete", "type": "string" },
                        { "const": "submit", "type": "string" },
                        { "const": "process", "type": "string" },
                        { "const": "consume", "type": "string" },
                      ],
                    },
                    "target": {
                      "anyOf": [{
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "apiSurface", "type": "string" },
                          "name": { "minLength": 1, "type": "string" },
                          "surface": {
                            "anyOf": [
                              { "const": "rpc", "type": "string" },
                              { "const": "operation", "type": "string" },
                              { "const": "event", "type": "string" },
                              { "const": "feed", "type": "string" },
                              { "const": "state", "type": "string" },
                            ],
                          },
                        },
                        "required": ["kind", "api", "surface", "name"],
                        "type": "object",
                      }, {
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": {
                            "const": "operationSignal",
                            "type": "string",
                          },
                          "operation": { "minLength": 1, "type": "string" },
                          "signal": { "minLength": 1, "type": "string" },
                        },
                        "required": ["kind", "api", "operation", "signal"],
                        "type": "object",
                      }, {
                        "properties": {
                          "kind": {
                            "const": "participantResource",
                            "type": "string",
                          },
                          "name": { "minLength": 1, "type": "string" },
                          "participant": { "minLength": 1, "type": "string" },
                          "resource": {
                            "anyOf": [
                              { "const": "state", "type": "string" },
                              { "const": "jobQueue", "type": "string" },
                              { "const": "eventConsumer", "type": "string" },
                              { "const": "kv", "type": "string" },
                              { "const": "store", "type": "string" },
                            ],
                          },
                        },
                        "required": ["kind", "participant", "resource", "name"],
                        "type": "object",
                      }],
                    },
                  },
                  "required": ["action", "target"],
                  "type": "object",
                },
                "type": "array",
              },
            },
            "required": ["format", "permissions"],
            "type": "object",
          },
          "installedRevision": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
          "ownerId": { "minLength": 1, "type": "string" },
          "ownerKind": {
            "anyOf": [{ "const": "deployment", "type": "string" }, {
              "const": "user",
              "type": "string",
            }],
          },
          "participantId": { "minLength": 1, "type": "string" },
          "platformPrivileges": {
            "items": { "const": "trellis.auth::admin", "type": "string" },
            "type": "array",
          },
          "provenance": {
            "anyOf": [{
              "properties": {
                "effectivePolicyDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "portalId": { "minLength": 1, "type": "string" },
                "providerId": { "minLength": 1, "type": "string" },
                "roles": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
              },
              "required": [
                "portalId",
                "providerId",
                "roles",
                "effectivePolicyDigest",
              ],
              "type": "object",
            }, { "type": "null" }],
          },
          "revision": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "revoked",
              "type": "string",
            }],
          },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
        },
        "required": [
          "ownerKind",
          "ownerId",
          "participantId",
          "installedRevision",
          "grants",
          "platformPrivileges",
          "revision",
          "state",
          "expiresAt",
          "provenance",
          "createdAt",
          "updatedAt",
        ],
        "type": "object",
      }, { "type": "null" }],
    },
  },
  "required": ["binding"],
  "type": "object",
} as const;

export const AuthGrantsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 500, "minimum": 1, "type": "integer" },
    "ownerId": { "minLength": 1, "type": "string" },
    "ownerKind": {
      "anyOf": [{ "const": "deployment", "type": "string" }, {
        "const": "user",
        "type": "string",
      }],
    },
    "participantId": { "minLength": 1, "type": "string" },
    "state": {
      "anyOf": [{ "const": "active", "type": "string" }, {
        "const": "revoked",
        "type": "string",
      }],
    },
  },
  "type": "object",
} as const;

export const AuthGrantsListResponseSchema = {
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
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "grants": {
            "properties": {
              "format": { "const": "trellis.grant-set.v1", "type": "string" },
              "permissions": {
                "items": {
                  "properties": {
                    "action": {
                      "anyOf": [
                        { "const": "call", "type": "string" },
                        { "const": "invoke", "type": "string" },
                        { "const": "observe", "type": "string" },
                        { "const": "cancel", "type": "string" },
                        { "const": "control", "type": "string" },
                        { "const": "publish", "type": "string" },
                        { "const": "subscribe", "type": "string" },
                        { "const": "read", "type": "string" },
                        { "const": "write", "type": "string" },
                        { "const": "delete", "type": "string" },
                        { "const": "submit", "type": "string" },
                        { "const": "process", "type": "string" },
                        { "const": "consume", "type": "string" },
                      ],
                    },
                    "target": {
                      "anyOf": [{
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "apiSurface", "type": "string" },
                          "name": { "minLength": 1, "type": "string" },
                          "surface": {
                            "anyOf": [
                              { "const": "rpc", "type": "string" },
                              { "const": "operation", "type": "string" },
                              { "const": "event", "type": "string" },
                              { "const": "feed", "type": "string" },
                              { "const": "state", "type": "string" },
                            ],
                          },
                        },
                        "required": ["kind", "api", "surface", "name"],
                        "type": "object",
                      }, {
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": {
                            "const": "operationSignal",
                            "type": "string",
                          },
                          "operation": { "minLength": 1, "type": "string" },
                          "signal": { "minLength": 1, "type": "string" },
                        },
                        "required": ["kind", "api", "operation", "signal"],
                        "type": "object",
                      }, {
                        "properties": {
                          "kind": {
                            "const": "participantResource",
                            "type": "string",
                          },
                          "name": { "minLength": 1, "type": "string" },
                          "participant": { "minLength": 1, "type": "string" },
                          "resource": {
                            "anyOf": [
                              { "const": "state", "type": "string" },
                              { "const": "jobQueue", "type": "string" },
                              { "const": "eventConsumer", "type": "string" },
                              { "const": "kv", "type": "string" },
                              { "const": "store", "type": "string" },
                            ],
                          },
                        },
                        "required": ["kind", "participant", "resource", "name"],
                        "type": "object",
                      }],
                    },
                  },
                  "required": ["action", "target"],
                  "type": "object",
                },
                "type": "array",
              },
            },
            "required": ["format", "permissions"],
            "type": "object",
          },
          "installedRevision": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
          "ownerId": { "minLength": 1, "type": "string" },
          "ownerKind": {
            "anyOf": [{ "const": "deployment", "type": "string" }, {
              "const": "user",
              "type": "string",
            }],
          },
          "participantId": { "minLength": 1, "type": "string" },
          "platformPrivileges": {
            "items": { "const": "trellis.auth::admin", "type": "string" },
            "type": "array",
          },
          "provenance": {
            "anyOf": [{
              "properties": {
                "effectivePolicyDigest": {
                  "pattern": "^[A-Za-z0-9_-]{43}$",
                  "type": "string",
                },
                "portalId": { "minLength": 1, "type": "string" },
                "providerId": { "minLength": 1, "type": "string" },
                "roles": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
              },
              "required": [
                "portalId",
                "providerId",
                "roles",
                "effectivePolicyDigest",
              ],
              "type": "object",
            }, { "type": "null" }],
          },
          "revision": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "revoked",
              "type": "string",
            }],
          },
          "updatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
        },
        "required": [
          "ownerKind",
          "ownerId",
          "participantId",
          "installedRevision",
          "grants",
          "platformPrivileges",
          "revision",
          "state",
          "expiresAt",
          "provenance",
          "createdAt",
          "updatedAt",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthGrantsMutationResponseSchema = {
  "properties": {
    "binding": {
      "properties": {
        "createdAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "expiresAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "grants": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1", "type": "string" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "anyOf": [
                      { "const": "call", "type": "string" },
                      { "const": "invoke", "type": "string" },
                      { "const": "observe", "type": "string" },
                      { "const": "cancel", "type": "string" },
                      { "const": "control", "type": "string" },
                      { "const": "publish", "type": "string" },
                      { "const": "subscribe", "type": "string" },
                      { "const": "read", "type": "string" },
                      { "const": "write", "type": "string" },
                      { "const": "delete", "type": "string" },
                      { "const": "submit", "type": "string" },
                      { "const": "process", "type": "string" },
                      { "const": "consume", "type": "string" },
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface", "type": "string" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "anyOf": [
                            { "const": "rpc", "type": "string" },
                            { "const": "operation", "type": "string" },
                            { "const": "event", "type": "string" },
                            { "const": "feed", "type": "string" },
                            { "const": "state", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": {
                          "const": "operationSignal",
                          "type": "string",
                        },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": {
                          "const": "participantResource",
                          "type": "string",
                        },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "anyOf": [
                            { "const": "state", "type": "string" },
                            { "const": "jobQueue", "type": "string" },
                            { "const": "eventConsumer", "type": "string" },
                            { "const": "kv", "type": "string" },
                            { "const": "store", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["action", "target"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "installedRevision": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
        "ownerId": { "minLength": 1, "type": "string" },
        "ownerKind": {
          "anyOf": [{ "const": "deployment", "type": "string" }, {
            "const": "user",
            "type": "string",
          }],
        },
        "participantId": { "minLength": 1, "type": "string" },
        "platformPrivileges": {
          "items": { "const": "trellis.auth::admin", "type": "string" },
          "type": "array",
        },
        "provenance": {
          "anyOf": [{
            "properties": {
              "effectivePolicyDigest": {
                "pattern": "^[A-Za-z0-9_-]{43}$",
                "type": "string",
              },
              "portalId": { "minLength": 1, "type": "string" },
              "providerId": { "minLength": 1, "type": "string" },
              "roles": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
            },
            "required": [
              "portalId",
              "providerId",
              "roles",
              "effectivePolicyDigest",
            ],
            "type": "object",
          }, { "type": "null" }],
        },
        "revision": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "revoked",
            "type": "string",
          }],
        },
        "updatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
      },
      "required": [
        "ownerKind",
        "ownerId",
        "participantId",
        "installedRevision",
        "grants",
        "platformPrivileges",
        "revision",
        "state",
        "expiresAt",
        "provenance",
        "createdAt",
        "updatedAt",
      ],
      "type": "object",
    },
  },
  "required": ["binding"],
  "type": "object",
} as const;

export const AuthGrantsRevokeRequestSchema = {
  "properties": {
    "expectedRevision": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "ownerId": { "minLength": 1, "type": "string" },
    "ownerKind": {
      "anyOf": [{ "const": "deployment", "type": "string" }, {
        "const": "user",
        "type": "string",
      }],
    },
    "participantId": { "minLength": 1, "type": "string" },
    "reason": { "type": "string" },
  },
  "required": [
    "ownerKind",
    "ownerId",
    "participantId",
    "expectedRevision",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthGrantsSetRequestSchema = {
  "properties": {
    "expectedRevision": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "expiresAt": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 0,
        "type": "integer",
      }, { "type": "null" }],
    },
    "grants": {
      "properties": {
        "format": { "const": "trellis.grant-set.v1", "type": "string" },
        "permissions": {
          "items": {
            "properties": {
              "action": {
                "anyOf": [
                  { "const": "call", "type": "string" },
                  { "const": "invoke", "type": "string" },
                  { "const": "observe", "type": "string" },
                  { "const": "cancel", "type": "string" },
                  { "const": "control", "type": "string" },
                  { "const": "publish", "type": "string" },
                  { "const": "subscribe", "type": "string" },
                  { "const": "read", "type": "string" },
                  { "const": "write", "type": "string" },
                  { "const": "delete", "type": "string" },
                  { "const": "submit", "type": "string" },
                  { "const": "process", "type": "string" },
                  { "const": "consume", "type": "string" },
                ],
              },
              "target": {
                "anyOf": [{
                  "properties": {
                    "api": { "minLength": 1, "type": "string" },
                    "kind": { "const": "apiSurface", "type": "string" },
                    "name": { "minLength": 1, "type": "string" },
                    "surface": {
                      "anyOf": [
                        { "const": "rpc", "type": "string" },
                        { "const": "operation", "type": "string" },
                        { "const": "event", "type": "string" },
                        { "const": "feed", "type": "string" },
                        { "const": "state", "type": "string" },
                      ],
                    },
                  },
                  "required": ["kind", "api", "surface", "name"],
                  "type": "object",
                }, {
                  "properties": {
                    "api": { "minLength": 1, "type": "string" },
                    "kind": { "const": "operationSignal", "type": "string" },
                    "operation": { "minLength": 1, "type": "string" },
                    "signal": { "minLength": 1, "type": "string" },
                  },
                  "required": ["kind", "api", "operation", "signal"],
                  "type": "object",
                }, {
                  "properties": {
                    "kind": {
                      "const": "participantResource",
                      "type": "string",
                    },
                    "name": { "minLength": 1, "type": "string" },
                    "participant": { "minLength": 1, "type": "string" },
                    "resource": {
                      "anyOf": [
                        { "const": "state", "type": "string" },
                        { "const": "jobQueue", "type": "string" },
                        { "const": "eventConsumer", "type": "string" },
                        { "const": "kv", "type": "string" },
                        { "const": "store", "type": "string" },
                      ],
                    },
                  },
                  "required": ["kind", "participant", "resource", "name"],
                  "type": "object",
                }],
              },
            },
            "required": ["action", "target"],
            "type": "object",
          },
          "type": "array",
        },
      },
      "required": ["format", "permissions"],
      "type": "object",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "installedRevision": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "ownerId": { "minLength": 1, "type": "string" },
    "ownerKind": {
      "anyOf": [{ "const": "deployment", "type": "string" }, {
        "const": "user",
        "type": "string",
      }],
    },
    "participantId": { "minLength": 1, "type": "string" },
    "platformPrivileges": {
      "items": { "const": "trellis.auth::admin", "type": "string" },
      "type": "array",
    },
  },
  "required": [
    "ownerKind",
    "ownerId",
    "participantId",
    "installedRevision",
    "grants",
    "platformPrivileges",
    "expiresAt",
    "expectedRevision",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthIssuersRevokeRequestSchema = {
  "properties": {
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "keyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
    "reason": { "type": "string" },
  },
  "required": ["keyId", "idempotencyKey"],
  "type": "object",
} as const;

export const AuthIssuersRevokeResponseSchema = {
  "properties": {
    "keyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
    "state": { "const": "revoked", "type": "string" },
  },
  "required": ["keyId", "state"],
  "type": "object",
} as const;

export const AuthIssuersRevokedEventSchema = {
  "properties": {
    "eventId": { "minLength": 1, "type": "string" },
    "keyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
    "occurredAt": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "reason": { "type": "string" },
    "revokedBy": { "minLength": 1, "type": "string" },
  },
  "required": ["eventId", "occurredAt", "keyId", "revokedBy"],
  "type": "object",
} as const;

export const AuthParticipantsGetRequestSchema = {
  "properties": {
    "participantId": { "minLength": 1, "type": "string" },
    "revision": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
  },
  "required": ["participantId"],
  "type": "object",
} as const;

export const AuthParticipantsGetResponseSchema = {
  "properties": {
    "participant": {
      "properties": {
        "apiArtifacts": {
          "items": { "properties": {}, "type": "object" },
          "type": "array",
        },
        "artifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "installedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "optionalBundles": {
          "items": {
            "properties": {
              "apiId": { "minLength": 1, "type": "string" },
              "id": { "minLength": 1, "type": "string" },
              "permissions": {
                "items": {
                  "properties": {
                    "action": {
                      "anyOf": [
                        { "const": "call", "type": "string" },
                        { "const": "invoke", "type": "string" },
                        { "const": "observe", "type": "string" },
                        { "const": "cancel", "type": "string" },
                        { "const": "control", "type": "string" },
                        { "const": "publish", "type": "string" },
                        { "const": "subscribe", "type": "string" },
                        { "const": "read", "type": "string" },
                        { "const": "write", "type": "string" },
                        { "const": "delete", "type": "string" },
                        { "const": "submit", "type": "string" },
                        { "const": "process", "type": "string" },
                        { "const": "consume", "type": "string" },
                      ],
                    },
                    "target": {
                      "anyOf": [{
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": { "const": "apiSurface", "type": "string" },
                          "name": { "minLength": 1, "type": "string" },
                          "surface": {
                            "anyOf": [
                              { "const": "rpc", "type": "string" },
                              { "const": "operation", "type": "string" },
                              { "const": "event", "type": "string" },
                              { "const": "feed", "type": "string" },
                              { "const": "state", "type": "string" },
                            ],
                          },
                        },
                        "required": ["kind", "api", "surface", "name"],
                        "type": "object",
                      }, {
                        "properties": {
                          "api": { "minLength": 1, "type": "string" },
                          "kind": {
                            "const": "operationSignal",
                            "type": "string",
                          },
                          "operation": { "minLength": 1, "type": "string" },
                          "signal": { "minLength": 1, "type": "string" },
                        },
                        "required": ["kind", "api", "operation", "signal"],
                        "type": "object",
                      }, {
                        "properties": {
                          "kind": {
                            "const": "participantResource",
                            "type": "string",
                          },
                          "name": { "minLength": 1, "type": "string" },
                          "participant": { "minLength": 1, "type": "string" },
                          "resource": {
                            "anyOf": [
                              { "const": "state", "type": "string" },
                              { "const": "jobQueue", "type": "string" },
                              { "const": "eventConsumer", "type": "string" },
                              { "const": "kv", "type": "string" },
                              { "const": "store", "type": "string" },
                            ],
                          },
                        },
                        "required": ["kind", "participant", "resource", "name"],
                        "type": "object",
                      }],
                    },
                  },
                  "required": ["action", "target"],
                  "type": "object",
                },
                "type": "array",
              },
            },
            "required": ["id", "apiId", "permissions"],
            "type": "object",
          },
          "type": "array",
        },
        "participantArtifact": { "properties": {}, "type": "object" },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": {
          "anyOf": [
            { "const": "service", "type": "string" },
            { "const": "device", "type": "string" },
            { "const": "app", "type": "string" },
            { "const": "agent", "type": "string" },
          ],
        },
        "requiredGrants": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1", "type": "string" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "anyOf": [
                      { "const": "call", "type": "string" },
                      { "const": "invoke", "type": "string" },
                      { "const": "observe", "type": "string" },
                      { "const": "cancel", "type": "string" },
                      { "const": "control", "type": "string" },
                      { "const": "publish", "type": "string" },
                      { "const": "subscribe", "type": "string" },
                      { "const": "read", "type": "string" },
                      { "const": "write", "type": "string" },
                      { "const": "delete", "type": "string" },
                      { "const": "submit", "type": "string" },
                      { "const": "process", "type": "string" },
                      { "const": "consume", "type": "string" },
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface", "type": "string" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "anyOf": [
                            { "const": "rpc", "type": "string" },
                            { "const": "operation", "type": "string" },
                            { "const": "event", "type": "string" },
                            { "const": "feed", "type": "string" },
                            { "const": "state", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": {
                          "const": "operationSignal",
                          "type": "string",
                        },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": {
                          "const": "participantResource",
                          "type": "string",
                        },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "anyOf": [
                            { "const": "state", "type": "string" },
                            { "const": "jobQueue", "type": "string" },
                            { "const": "eventConsumer", "type": "string" },
                            { "const": "kv", "type": "string" },
                            { "const": "store", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["action", "target"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "revision": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "participantId",
        "participantKind",
        "revision",
        "artifactDigest",
        "installedAt",
        "participantArtifact",
        "apiArtifacts",
        "requiredGrants",
        "optionalBundles",
      ],
      "type": "object",
    },
  },
  "required": ["participant"],
  "type": "object",
} as const;

export const AuthParticipantsInstallRequestSchema = {
  "properties": {
    "apiArtifacts": {
      "items": { "properties": {}, "type": "object" },
      "type": "array",
    },
    "expectedRevision": {
      "maximum": 9007199254740991,
      "minimum": 0,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "participantArtifact": { "properties": {}, "type": "object" },
  },
  "required": [
    "participantArtifact",
    "apiArtifacts",
    "expectedRevision",
    "idempotencyKey",
  ],
  "type": "object",
} as const;

export const AuthParticipantsInstallResponseSchema = {
  "properties": {
    "participant": {
      "properties": {
        "artifactDigest": {
          "pattern": "^[A-Za-z0-9_-]{43}$",
          "type": "string",
        },
        "installedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": {
          "anyOf": [
            { "const": "service", "type": "string" },
            { "const": "device", "type": "string" },
            { "const": "app", "type": "string" },
            { "const": "agent", "type": "string" },
          ],
        },
        "revision": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "participantId",
        "participantKind",
        "revision",
        "artifactDigest",
        "installedAt",
      ],
      "type": "object",
    },
  },
  "required": ["participant"],
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
        "entryUrl": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
            "federatedRegistration",
            "localLogin",
            "localRegistration",
            "providers",
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
        "builtIn",
        "createdAt",
        "disabled",
        "displayName",
        "entryUrl",
        "loginSettings",
        "portalId",
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
          "deploymentId": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "origin": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "participantId": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
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
          "createdAt",
          "deploymentId",
          "origin",
          "participantId",
          "portalId",
          "priority",
          "routeId",
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

export const AuthPortalsGrantOverridesListRequestSchema = {
  "properties": {
    "limit": { "maximum": 500, "minimum": 0, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
    "participantId": { "minLength": 1, "type": "string" },
    "portalId": { "minLength": 1, "type": "string" },
  },
  "required": ["limit"],
  "type": "object",
} as const;

export const AuthPortalsGrantOverridesListResponseSchema = {
  "properties": {
    "count": { "minimum": 0, "type": "integer" },
    "entries": {
      "items": {
        "properties": {
          "capabilityGroupKeys": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "createdAt": { "minimum": 0, "type": "integer" },
          "directCapabilities": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "participantId": { "minLength": 1, "type": "string" },
          "portalId": { "minLength": 1, "type": "string" },
          "roleMappings": {
            "items": {
              "properties": {
                "capabilityGroupKeys": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
                "directCapabilities": {
                  "items": { "minLength": 1, "type": "string" },
                  "type": "array",
                },
                "providerId": { "minLength": 1, "type": "string" },
                "role": { "minLength": 1, "type": "string" },
              },
              "required": [
                "capabilityGroupKeys",
                "directCapabilities",
                "providerId",
                "role",
              ],
              "type": "object",
            },
            "type": "array",
          },
          "updatedAt": { "minimum": 0, "type": "integer" },
          "version": { "minimum": 0, "type": "integer" },
        },
        "required": [
          "capabilityGroupKeys",
          "createdAt",
          "directCapabilities",
          "participantId",
          "portalId",
          "roleMappings",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "limit": { "minimum": 0, "type": "integer" },
    "nextOffset": { "minimum": 0, "type": "integer" },
    "offset": { "minimum": 0, "type": "integer" },
  },
  "required": ["count", "entries", "limit", "offset"],
  "type": "object",
} as const;

export const AuthPortalsGrantOverridesPutRequestSchema = {
  "properties": {
    "capabilityGroupKeys": {
      "items": { "minLength": 1, "type": "string" },
      "type": "array",
    },
    "directCapabilities": {
      "items": { "minLength": 1, "type": "string" },
      "type": "array",
    },
    "expectedVersion": {
      "anyOf": [{ "minimum": 0, "type": "integer" }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "participantId": { "minLength": 1, "type": "string" },
    "portalId": { "minLength": 1, "type": "string" },
    "roleMappings": {
      "items": {
        "properties": {
          "capabilityGroupKeys": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "directCapabilities": {
            "items": { "minLength": 1, "type": "string" },
            "type": "array",
          },
          "providerId": { "minLength": 1, "type": "string" },
          "role": { "minLength": 1, "type": "string" },
        },
        "required": [
          "capabilityGroupKeys",
          "directCapabilities",
          "providerId",
          "role",
        ],
        "type": "object",
      },
      "type": "array",
    },
  },
  "required": [
    "capabilityGroupKeys",
    "directCapabilities",
    "expectedVersion",
    "idempotencyKey",
    "participantId",
    "portalId",
    "roleMappings",
  ],
  "type": "object",
} as const;

export const AuthPortalsGrantOverridesPutResponseSchema = {
  "properties": {
    "policy": {
      "properties": {
        "capabilityGroupKeys": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "createdAt": { "minimum": 0, "type": "integer" },
        "directCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "portalId": { "minLength": 1, "type": "string" },
        "roleMappings": {
          "items": {
            "properties": {
              "capabilityGroupKeys": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "directCapabilities": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "providerId": { "minLength": 1, "type": "string" },
              "role": { "minLength": 1, "type": "string" },
            },
            "required": [
              "capabilityGroupKeys",
              "directCapabilities",
              "providerId",
              "role",
            ],
            "type": "object",
          },
          "type": "array",
        },
        "updatedAt": { "minimum": 0, "type": "integer" },
        "version": { "minimum": 0, "type": "integer" },
      },
      "required": [
        "capabilityGroupKeys",
        "createdAt",
        "directCapabilities",
        "participantId",
        "portalId",
        "roleMappings",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["policy"],
  "type": "object",
} as const;

export const AuthPortalsGrantOverridesRemoveRequestSchema = {
  "properties": {
    "expectedVersion": { "minimum": 0, "type": "integer" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "participantId": { "minLength": 1, "type": "string" },
    "portalId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "expectedVersion",
    "idempotencyKey",
    "participantId",
    "portalId",
  ],
  "type": "object",
} as const;

export const AuthPortalsGrantOverridesRemoveResponseSchema = {
  "properties": {
    "removed": {
      "properties": {
        "capabilityGroupKeys": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "createdAt": { "minimum": 0, "type": "integer" },
        "directCapabilities": {
          "items": { "minLength": 1, "type": "string" },
          "type": "array",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "portalId": { "minLength": 1, "type": "string" },
        "roleMappings": {
          "items": {
            "properties": {
              "capabilityGroupKeys": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "directCapabilities": {
                "items": { "minLength": 1, "type": "string" },
                "type": "array",
              },
              "providerId": { "minLength": 1, "type": "string" },
              "role": { "minLength": 1, "type": "string" },
            },
            "required": [
              "capabilityGroupKeys",
              "directCapabilities",
              "providerId",
              "role",
            ],
            "type": "object",
          },
          "type": "array",
        },
        "updatedAt": { "minimum": 0, "type": "integer" },
        "version": { "minimum": 0, "type": "integer" },
      },
      "required": [
        "capabilityGroupKeys",
        "createdAt",
        "directCapabilities",
        "participantId",
        "portalId",
        "roleMappings",
        "updatedAt",
        "version",
      ],
      "type": "object",
    },
  },
  "type": "object",
} as const;

export const AuthPortalsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "disabled": { "type": "boolean" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
  },
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
          "entryUrl": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
              "federatedRegistration",
              "localLogin",
              "localRegistration",
              "providers",
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
          "builtIn",
          "createdAt",
          "disabled",
          "displayName",
          "entryUrl",
          "loginSettings",
          "portalId",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
        "federatedRegistration",
        "localLogin",
        "localRegistration",
        "providers",
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
        "federatedRegistration",
        "localLogin",
        "localRegistration",
        "providers",
      ],
      "type": "object",
    },
  },
  "required": ["expectedVersion", "idempotencyKey", "portalId", "settings"],
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
        "federatedRegistration",
        "localLogin",
        "localRegistration",
        "providers",
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
    "entryUrl": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
        "federatedRegistration",
        "localLogin",
        "localRegistration",
        "providers",
      ],
      "type": "object",
    },
    "portalId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "disabled",
    "displayName",
    "entryUrl",
    "expectedVersion",
    "idempotencyKey",
    "loginSettings",
    "portalId",
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
        "entryUrl": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
            "federatedRegistration",
            "localLogin",
            "localRegistration",
            "providers",
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
        "builtIn",
        "createdAt",
        "disabled",
        "displayName",
        "entryUrl",
        "loginSettings",
        "portalId",
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
  "required": ["expectedVersion", "idempotencyKey", "portalId"],
  "type": "object",
} as const;

export const AuthPortalsRemoveResponseSchema = {
  "properties": { "removed": { "type": "boolean" } },
  "required": ["removed"],
  "type": "object",
} as const;

export const AuthPortalsRoutesPutRequestSchema = {
  "properties": {
    "deploymentId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "expectedVersion": {
      "anyOf": [{
        "maximum": 9007199254740991,
        "minimum": 1,
        "type": "integer",
      }, { "type": "null" }],
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "origin": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "participantId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "portalId": { "minLength": 1, "type": "string" },
    "priority": { "minimum": 0, "type": "integer" },
    "routeId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": [
    "deploymentId",
    "expectedVersion",
    "idempotencyKey",
    "origin",
    "participantId",
    "portalId",
    "priority",
    "routeId",
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
        "deploymentId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "origin": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
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
        "createdAt",
        "deploymentId",
        "origin",
        "participantId",
        "portalId",
        "priority",
        "routeId",
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
  "required": ["expectedVersion", "idempotencyKey", "routeId"],
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["expectedVersion", "idempotencyKey", "instanceId", "reason"],
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
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
            { "const": "stale", "type": "string" },
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
        "createdAt",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["expectedVersion", "idempotencyKey", "instanceId", "reason"],
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
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
            { "const": "stale", "type": "string" },
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
        "createdAt",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
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
    "state": {
      "anyOf": [
        { "const": "active", "type": "string" },
        { "const": "disabled", "type": "string" },
        { "const": "revoked", "type": "string" },
        { "const": "stale", "type": "string" },
      ],
    },
  },
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
          "participantId": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "principalId": { "minLength": 1, "type": "string" },
          "state": {
            "anyOf": [
              { "const": "active", "type": "string" },
              { "const": "disabled", "type": "string" },
              { "const": "revoked", "type": "string" },
              { "const": "stale", "type": "string" },
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
          "createdAt",
          "deploymentId",
          "identityKeyId",
          "identityPublicKey",
          "instanceId",
          "participantId",
          "principalId",
          "state",
          "updatedAt",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthServiceInstancesProvisionRequestSchema = {
  "properties": {
    "deploymentId": { "minLength": 1, "type": "string" },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "identityPublicKey": { "minLength": 1, "type": "string" },
    "instanceId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "participantId": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": [
    "deploymentId",
    "idempotencyKey",
    "identityPublicKey",
    "instanceId",
    "participantId",
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
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
            { "const": "stale", "type": "string" },
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
        "createdAt",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["expectedVersion", "idempotencyKey", "instanceId", "reason"],
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
        "participantId": {
          "anyOf": [{ "type": "string" }, { "type": "null" }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [
            { "const": "active", "type": "string" },
            { "const": "disabled", "type": "string" },
            { "const": "revoked", "type": "string" },
            { "const": "stale", "type": "string" },
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
        "createdAt",
        "deploymentId",
        "identityKeyId",
        "identityPublicKey",
        "instanceId",
        "participantId",
        "principalId",
        "state",
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
      "required": ["changed", "resourceId", "state", "version"],
      "type": "object",
    },
  },
  "required": ["instance", "mutation"],
  "type": "object",
} as const;

export const AuthSessionsListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "participantId": { "minLength": 1, "type": "string" },
    "principalId": { "minLength": 1, "type": "string" },
    "state": {
      "anyOf": [{ "const": "active", "type": "string" }, {
        "const": "expired",
        "type": "string",
      }, { "const": "revoked", "type": "string" }],
    },
  },
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
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "lastAuthenticatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "participantId": { "minLength": 1, "type": "string" },
          "participantKind": {
            "anyOf": [{ "const": "app", "type": "string" }, {
              "const": "agent",
              "type": "string",
            }],
          },
          "principalId": { "minLength": 1, "type": "string" },
          "revokedAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "sessionId": { "minLength": 1, "type": "string" },
          "sessionKeyId": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "sessionPublicKey": { "minLength": 1, "type": "string" },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "expired",
              "type": "string",
            }, { "const": "revoked", "type": "string" }],
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "createdAt",
          "expiresAt",
          "lastAuthenticatedAt",
          "participantId",
          "participantKind",
          "principalId",
          "revokedAt",
          "sessionId",
          "sessionKeyId",
          "sessionPublicKey",
          "state",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["entries", "nextCursor"],
  "type": "object",
} as const;

export const AuthSessionsLogoutRequestSchema = {
  "properties": {},
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "lastAuthenticatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": {
          "anyOf": [{ "const": "app", "type": "string" }, {
            "const": "agent",
            "type": "string",
          }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "sessionId": { "minLength": 1, "type": "string" },
        "sessionKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "sessionPublicKey": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "expired",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "createdAt",
        "expiresAt",
        "lastAuthenticatedAt",
        "participantId",
        "participantKind",
        "principalId",
        "revokedAt",
        "sessionId",
        "sessionKeyId",
        "sessionPublicKey",
        "state",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["kickedConnections", "session"],
  "type": "object",
} as const;

export const AuthSessionsMeRequestSchema = {
  "properties": {},
  "type": "object",
} as const;

export const AuthSessionsMeResponseSchema = {
  "properties": {
    "connection": {
      "properties": {
        "connectionId": { "minLength": 1, "type": "string" },
        "deploymentId": { "minLength": 1, "type": "string" },
        "grants": {
          "properties": {
            "format": { "const": "trellis.grant-set.v1", "type": "string" },
            "permissions": {
              "items": {
                "properties": {
                  "action": {
                    "anyOf": [
                      { "const": "call", "type": "string" },
                      { "const": "invoke", "type": "string" },
                      { "const": "observe", "type": "string" },
                      { "const": "cancel", "type": "string" },
                      { "const": "control", "type": "string" },
                      { "const": "publish", "type": "string" },
                      { "const": "subscribe", "type": "string" },
                      { "const": "read", "type": "string" },
                      { "const": "write", "type": "string" },
                      { "const": "delete", "type": "string" },
                      { "const": "submit", "type": "string" },
                      { "const": "process", "type": "string" },
                      { "const": "consume", "type": "string" },
                    ],
                  },
                  "target": {
                    "anyOf": [{
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": { "const": "apiSurface", "type": "string" },
                        "name": { "minLength": 1, "type": "string" },
                        "surface": {
                          "anyOf": [
                            { "const": "rpc", "type": "string" },
                            { "const": "operation", "type": "string" },
                            { "const": "event", "type": "string" },
                            { "const": "feed", "type": "string" },
                            { "const": "state", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "api", "surface", "name"],
                      "type": "object",
                    }, {
                      "properties": {
                        "api": { "minLength": 1, "type": "string" },
                        "kind": {
                          "const": "operationSignal",
                          "type": "string",
                        },
                        "operation": { "minLength": 1, "type": "string" },
                        "signal": { "minLength": 1, "type": "string" },
                      },
                      "required": ["kind", "api", "operation", "signal"],
                      "type": "object",
                    }, {
                      "properties": {
                        "kind": {
                          "const": "participantResource",
                          "type": "string",
                        },
                        "name": { "minLength": 1, "type": "string" },
                        "participant": { "minLength": 1, "type": "string" },
                        "resource": {
                          "anyOf": [
                            { "const": "state", "type": "string" },
                            { "const": "jobQueue", "type": "string" },
                            { "const": "eventConsumer", "type": "string" },
                            { "const": "kv", "type": "string" },
                            { "const": "store", "type": "string" },
                          ],
                        },
                      },
                      "required": ["kind", "participant", "resource", "name"],
                      "type": "object",
                    }],
                  },
                },
                "required": ["action", "target"],
                "type": "object",
              },
              "type": "array",
            },
          },
          "required": ["format", "permissions"],
          "type": "object",
        },
        "identityKeyId": { "minLength": 1, "type": "string" },
        "inboxPrefix": { "minLength": 1, "type": "string" },
        "instanceId": { "minLength": 1, "type": "string" },
        "loginSessionId": { "minLength": 1, "type": "string" },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": {
          "anyOf": [
            { "const": "service", "type": "string" },
            { "const": "app", "type": "string" },
            { "const": "device", "type": "string" },
            { "const": "agent", "type": "string" },
          ],
        },
        "platformPrivileges": {
          "items": { "const": "trellis.auth::admin", "type": "string" },
          "type": "array",
        },
        "principalId": { "minLength": 1, "type": "string" },
        "principalKind": {
          "anyOf": [{ "const": "user", "type": "string" }, {
            "const": "service",
            "type": "string",
          }, { "const": "device", "type": "string" }],
        },
        "sessionKey": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
      },
      "required": [
        "connectionId",
        "sessionKey",
        "inboxPrefix",
        "participantId",
        "participantKind",
        "principalId",
        "principalKind",
        "grants",
        "platformPrivileges",
      ],
      "type": "object",
    },
    "session": {
      "anyOf": [{
        "properties": {
          "createdAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "expiresAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "lastAuthenticatedAt": {
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          },
          "participantId": { "minLength": 1, "type": "string" },
          "participantKind": {
            "anyOf": [{ "const": "app", "type": "string" }, {
              "const": "agent",
              "type": "string",
            }],
          },
          "principalId": { "minLength": 1, "type": "string" },
          "revokedAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "sessionId": { "minLength": 1, "type": "string" },
          "sessionKeyId": {
            "pattern": "^[A-Za-z0-9_-]{43}$",
            "type": "string",
          },
          "sessionPublicKey": { "minLength": 1, "type": "string" },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "expired",
              "type": "string",
            }, { "const": "revoked", "type": "string" }],
          },
          "version": {
            "maximum": 9007199254740991,
            "minimum": 1,
            "type": "integer",
          },
        },
        "required": [
          "createdAt",
          "expiresAt",
          "lastAuthenticatedAt",
          "participantId",
          "participantKind",
          "principalId",
          "revokedAt",
          "sessionId",
          "sessionKeyId",
          "sessionPublicKey",
          "state",
          "version",
        ],
        "type": "object",
      }, { "type": "null" }],
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
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "email": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "image": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "principalId": { "minLength": 1, "type": "string" },
          "revokedAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "disabled",
              "type": "string",
            }, { "const": "revoked", "type": "string" }],
          },
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
          "createdAt",
          "disabledAt",
          "email",
          "image",
          "name",
          "principalId",
          "revokedAt",
          "state",
          "updatedAt",
          "userId",
          "version",
        ],
        "type": "object",
      }, { "type": "null" }],
    },
  },
  "required": ["connection", "session", "user"],
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": ["expectedVersion", "idempotencyKey", "reason", "sessionId"],
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "lastAuthenticatedAt": {
          "maximum": 9007199254740991,
          "minimum": 0,
          "type": "integer",
        },
        "participantId": { "minLength": 1, "type": "string" },
        "participantKind": {
          "anyOf": [{ "const": "app", "type": "string" }, {
            "const": "agent",
            "type": "string",
          }],
        },
        "principalId": { "minLength": 1, "type": "string" },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "sessionId": { "minLength": 1, "type": "string" },
        "sessionKeyId": { "pattern": "^[A-Za-z0-9_-]{43}$", "type": "string" },
        "sessionPublicKey": { "minLength": 1, "type": "string" },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "expired",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "createdAt",
        "expiresAt",
        "lastAuthenticatedAt",
        "participantId",
        "participantKind",
        "principalId",
        "revokedAt",
        "sessionId",
        "sessionKeyId",
        "sessionPublicKey",
        "state",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["kickedConnections", "session"],
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
    "reason": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "revokedBy": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "sessionId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "eventId",
    "occurredAt",
    "participantId",
    "principalId",
    "reason",
    "revokedBy",
    "sessionId",
  ],
  "type": "object",
} as const;

export const AuthUserIdentitiesListRequestSchema = {
  "properties": {
    "cursor": { "minLength": 1, "type": "string" },
    "limit": { "maximum": 100, "minimum": 1, "type": "integer" },
    "providerId": { "minLength": 1, "type": "string" },
  },
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
          "observedEmail": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "observedName": {
            "anyOf": [{ "type": "string" }, { "type": "null" }],
          },
          "principalId": { "minLength": 1, "type": "string" },
          "providerId": { "minLength": 1, "type": "string" },
          "subject": { "minLength": 1, "type": "string" },
          "username": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        },
        "required": [
          "createdAt",
          "lastSeenAt",
          "observedEmail",
          "observedName",
          "principalId",
          "providerId",
          "subject",
          "username",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
  "required": ["idempotencyKey", "providerId", "subject"],
  "type": "object",
} as const;

export const AuthUserIdentitiesUnlinkResponseSchema = {
  "properties": { "unlinked": { "type": "boolean" } },
  "required": ["unlinked"],
  "type": "object",
} as const;

export const AuthUsersCreateRequestSchema = {
  "properties": {
    "email": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "idempotencyKey": { "maxLength": 256, "minLength": 1, "type": "string" },
    "image": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["email", "idempotencyKey", "image", "name"],
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "email": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "image": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "principalId": { "minLength": 1, "type": "string" },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
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
        "createdAt",
        "disabledAt",
        "email",
        "image",
        "name",
        "principalId",
        "revokedAt",
        "state",
        "updatedAt",
        "userId",
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "email": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "image": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "principalId": { "minLength": 1, "type": "string" },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
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
        "createdAt",
        "disabledAt",
        "email",
        "image",
        "name",
        "principalId",
        "revokedAt",
        "state",
        "updatedAt",
        "userId",
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
    "returnTarget": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
  },
  "required": ["allowedProviders", "idempotencyKey", "returnTarget"],
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
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
        "kind": { "const": "identity_link", "type": "string" },
        "returnTarget": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "targetPrincipalId": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "allowedProviders",
        "completionUrl",
        "consumedAt",
        "createdAt",
        "expiresAt",
        "flowId",
        "kind",
        "returnTarget",
        "targetPrincipalId",
        "version",
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
    "state": {
      "anyOf": [{ "const": "active", "type": "string" }, {
        "const": "disabled",
        "type": "string",
      }, { "const": "revoked", "type": "string" }],
    },
  },
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
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "email": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "image": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "principalId": { "minLength": 1, "type": "string" },
          "revokedAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "disabled",
              "type": "string",
            }, { "const": "revoked", "type": "string" }],
          },
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
          "createdAt",
          "disabledAt",
          "email",
          "image",
          "name",
          "principalId",
          "revokedAt",
          "state",
          "updatedAt",
          "userId",
          "version",
        ],
        "type": "object",
      },
      "type": "array",
    },
    "nextCursor": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
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
  "required": ["currentPassword", "idempotencyKey", "newPassword"],
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
    "returnTarget": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "userId": { "minLength": 1, "type": "string" },
  },
  "required": ["idempotencyKey", "returnTarget", "userId"],
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
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
        "kind": { "const": "password_reset", "type": "string" },
        "returnTarget": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "targetPrincipalId": { "minLength": 1, "type": "string" },
        "version": {
          "maximum": 9007199254740991,
          "minimum": 1,
          "type": "integer",
        },
      },
      "required": [
        "allowedProviders",
        "completionUrl",
        "consumedAt",
        "createdAt",
        "expiresAt",
        "flowId",
        "kind",
        "returnTarget",
        "targetPrincipalId",
        "version",
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
          "kind": { "const": "user", "type": "string" },
          "userId": { "minLength": 1, "type": "string" },
        },
        "required": ["kind", "userId"],
        "type": "object",
      }, {
        "properties": {
          "kind": { "const": "provider", "type": "string" },
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
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "email": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "image": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
          "principalId": { "minLength": 1, "type": "string" },
          "revokedAt": {
            "anyOf": [{
              "maximum": 9007199254740991,
              "minimum": 0,
              "type": "integer",
            }, { "type": "null" }],
          },
          "state": {
            "anyOf": [{ "const": "active", "type": "string" }, {
              "const": "disabled",
              "type": "string",
            }, { "const": "revoked", "type": "string" }],
          },
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
          "createdAt",
          "disabledAt",
          "email",
          "image",
          "name",
          "principalId",
          "revokedAt",
          "state",
          "updatedAt",
          "userId",
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
    "email": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "expectedVersion": {
      "maximum": 9007199254740991,
      "minimum": 1,
      "type": "integer",
    },
    "idempotencyKey": { "minLength": 1, "type": "string" },
    "image": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
    "state": {
      "anyOf": [{ "const": "active", "type": "string" }, {
        "const": "disabled",
        "type": "string",
      }],
    },
    "userId": { "minLength": 1, "type": "string" },
  },
  "required": [
    "email",
    "expectedVersion",
    "idempotencyKey",
    "image",
    "name",
    "state",
    "userId",
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
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "email": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "image": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "name": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
        "principalId": { "minLength": 1, "type": "string" },
        "revokedAt": {
          "anyOf": [{
            "maximum": 9007199254740991,
            "minimum": 0,
            "type": "integer",
          }, { "type": "null" }],
        },
        "state": {
          "anyOf": [{ "const": "active", "type": "string" }, {
            "const": "disabled",
            "type": "string",
          }, { "const": "revoked", "type": "string" }],
        },
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
        "createdAt",
        "disabledAt",
        "email",
        "image",
        "name",
        "principalId",
        "revokedAt",
        "state",
        "updatedAt",
        "userId",
        "version",
      ],
      "type": "object",
    },
  },
  "required": ["user"],
  "type": "object",
} as const;
