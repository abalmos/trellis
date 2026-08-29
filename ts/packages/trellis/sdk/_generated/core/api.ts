// Generated from ./generated/protocol/apis/trellis.core@v1.json

export const API_ID = "trellis.core@v1" as const;
export const API_DIGEST =
  "5hQQ8APSyxcdMSl6BgaNNjskL-B4sWTabNslL35hT_8" as const;
export const API = {
  "capabilities": {
    "trellis.core::authority.read": {
      "allows": [{
        "action": "call",
        "target": {
          "api": "trellis.core@v1",
          "kind": "apiSurface",
          "name": "Trellis.Surface.Status",
          "surface": "rpc",
        },
      }],
    },
  },
  "consent": {
    "trellis.core::authority.read": {
      "consequence": "",
      "description": "Inspect native participant surface authority.",
      "title": "Read participant authority",
    },
  },
  "description":
    "Trellis runtime RPCs available to all connected participants.",
  "displayName": "Trellis Core",
  "docs": {
    "markdown":
      "Exposes runtime bindings and surface availability checks used by platform participants.",
    "summary": "Runtime authority and binding APIs.",
  },
  "errors": { "UnexpectedError": {}, "ValidationError": {} },
  "format": "trellis.api.v1",
  "id": "trellis.core@v1",
  "rpc": {
    "Trellis.Surface.Status": {
      "docs": {
        "markdown":
          "Reports capability and deployment authority status for a contract-owned surface.",
        "summary": "Inspect surface availability.",
      },
      "errors": ["UnexpectedError", "ValidationError"],
      "input": { "schema": "TrellisSurfaceStatusRequest" },
      "output": { "schema": "TrellisSurfaceStatusResponse" },
      "version": "v1",
    },
  },
  "schemas": {
    "TrellisSurfaceStatusRequest": {
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
    },
    "TrellisSurfaceStatusResponse": {
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
                "anyOf": [{
                  "const": "authority_unavailable",
                  "type": "string",
                }],
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
    },
  },
  "version": "1.0.0",
} as const;
