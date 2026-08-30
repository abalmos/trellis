// Generated from ./ts/packages/trellis/.trellis/generated/protocol/apis/trellis.core@v1.json
export const TrellisSurfaceStatusRequestSchema = {
  "properties": {
    "action": {
      "anyOf": [
        { "const": "call", "type": "string" },
        { "const": "publish", "type": "string" },
        { "const": "subscribe", "type": "string" },
        { "const": "observe", "type": "string" },
      ],
    },
    "contractId": { "minLength": 1, "type": "string" },
    "kind": {
      "anyOf": [
        { "const": "rpc", "type": "string" },
        { "const": "operation", "type": "string" },
        { "const": "event", "type": "string" },
        { "const": "feed", "type": "string" },
      ],
    },
    "surface": { "minLength": 1, "type": "string" },
  },
  "required": ["contractId", "kind", "surface"],
  "type": "object",
} as const;

export const TrellisSurfaceStatusResponseSchema = {
  "properties": {
    "status": {
      "anyOf": [{
        "properties": {
          "liveImplementer": { "type": "boolean" },
          "runtime": {
            "anyOf": [{ "const": "live", "type": "string" }, {
              "const": "no_live_implementer",
              "type": "string",
            }, { "const": "disabled", "type": "string" }],
          },
          "state": { "const": "available", "type": "string" },
        },
        "required": ["state", "liveImplementer", "runtime"],
        "type": "object",
      }, {
        "properties": {
          "reason": {
            "anyOf": [{ "const": "authority_unavailable", "type": "string" }],
          },
          "state": { "const": "unavailable", "type": "string" },
        },
        "required": ["state", "reason"],
        "type": "object",
      }, {
        "properties": {
          "missingCapabilities": {
            "items": { "type": "string" },
            "type": "array",
          },
          "state": { "const": "unauthorized", "type": "string" },
        },
        "required": ["state", "missingCapabilities"],
        "type": "object",
      }, {
        "properties": {
          "contractId": { "minLength": 1, "type": "string" },
          "state": { "const": "unknown_contract", "type": "string" },
        },
        "required": ["state", "contractId"],
        "type": "object",
      }, {
        "properties": {
          "contractId": { "minLength": 1, "type": "string" },
          "kind": { "minLength": 1, "type": "string" },
          "state": { "const": "unknown_surface", "type": "string" },
          "surface": { "minLength": 1, "type": "string" },
        },
        "required": ["state", "contractId", "kind", "surface"],
        "type": "object",
      }],
    },
  },
  "required": ["status"],
  "type": "object",
} as const;
