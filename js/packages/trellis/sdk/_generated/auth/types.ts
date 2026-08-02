// Generated from ./generated/apis/trellis.auth@v1.json
import type { SerializableErrorData } from "../../../contracts.ts";
import { TrellisError } from "../../../errors/index.ts";
import { AuthErrorDetailsSchema } from "./schemas.ts";

export type AuthCapabilitiesListInput = {
  cursor?: string;
  limit?: number;
  sourceApi?: string;
};
export type AuthCapabilitiesListOutput = {
  entries: Array<
    {
      allows: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
      capability: string;
      description: string;
      displayName: string;
      sourceApi: string | null;
    }
  >;
  nextCursor: string | null;
};

export type AuthConnectionsKickInput = {
  connectionId: string;
  idempotencyKey: string;
  reason: string | null;
};
export type AuthConnectionsKickOutput = {
  connectionId: string;
  kicked: boolean;
};

export type AuthConnectionsListInput = {
  cursor?: string;
  limit?: number;
  sessionId?: string;
};
export type AuthConnectionsListOutput = {
  entries: Array<
    {
      clientId: string;
      connectedAt: number;
      connectionId: string;
      lastSeenAt: number;
      remoteAddress: string | null;
      serverId: string;
      sessionId: string;
      userNkey: string;
    }
  >;
  nextCursor: string | null;
};

export type AuthDeploymentAuthorityAcceptMigrationInput = {
  expectedBaseAuthorityVersion: number | null;
  idempotencyKey: string;
  proposalId: string;
  reason: string | null;
};
export type AuthDeploymentAuthorityAcceptMigrationOutput = {
  authority: {
    acceptedNeedsDigest: string;
    authorityId: string;
    createdAt: number;
    decision:
      | { decidedAt: number; decidedBy: string; reason: string | null }
      | null;
    deploymentId: string;
    desiredCapabilities: Array<string>;
    desiredGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    expiresAt: number | null;
    kind: "deployment";
    materialization: {
      authorityId: string;
      authorityKind: "identity" | "deployment";
      authorityVersion: number;
      effectiveCapabilities: Array<string>;
      effectiveGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      error: string | null;
      expiresAt: number | null;
      materializationId: string;
      materializationVersion: number;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "app" | "device" | "agent";
      participantNeedsDigest: string;
      reconciledAt: number | null;
      state: "available" | "unavailable" | "error";
      subjectId: string;
    } | null;
    participantArtifactDigest: string;
    participantId: string;
    participantKind: "service" | "device";
    state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
  proposal: {
    authorityKind: "identity" | "deployment";
    baseAuthorityVersion: number | null;
    classification: "initial" | "update" | "migration";
    createdAt: number;
    decisionAt: number | null;
    decisionBy: string | null;
    decisionReason: string | null;
    expiresAt: number | null;
    participantArtifactDigest: string;
    participantId: string;
    participantNeedsDigest: string;
    proposalId: string;
    proposedCapabilities: Array<string>;
    proposedGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    reasons: Array<string>;
    state: "pending" | "accepted" | "rejected" | "superseded" | "expired";
    subjectId: string;
  };
};

export type AuthDeploymentAuthorityAcceptUpdateInput = {
  expectedBaseAuthorityVersion: number | null;
  idempotencyKey: string;
  proposalId: string;
  reason: string | null;
};
export type AuthDeploymentAuthorityAcceptUpdateOutput = {
  authority: {
    acceptedNeedsDigest: string;
    authorityId: string;
    createdAt: number;
    decision:
      | { decidedAt: number; decidedBy: string; reason: string | null }
      | null;
    deploymentId: string;
    desiredCapabilities: Array<string>;
    desiredGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    expiresAt: number | null;
    kind: "deployment";
    materialization: {
      authorityId: string;
      authorityKind: "identity" | "deployment";
      authorityVersion: number;
      effectiveCapabilities: Array<string>;
      effectiveGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      error: string | null;
      expiresAt: number | null;
      materializationId: string;
      materializationVersion: number;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "app" | "device" | "agent";
      participantNeedsDigest: string;
      reconciledAt: number | null;
      state: "available" | "unavailable" | "error";
      subjectId: string;
    } | null;
    participantArtifactDigest: string;
    participantId: string;
    participantKind: "service" | "device";
    state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
  proposal: {
    authorityKind: "identity" | "deployment";
    baseAuthorityVersion: number | null;
    classification: "initial" | "update" | "migration";
    createdAt: number;
    decisionAt: number | null;
    decisionBy: string | null;
    decisionReason: string | null;
    expiresAt: number | null;
    participantArtifactDigest: string;
    participantId: string;
    participantNeedsDigest: string;
    proposalId: string;
    proposedCapabilities: Array<string>;
    proposedGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    reasons: Array<string>;
    state: "pending" | "accepted" | "rejected" | "superseded" | "expired";
    subjectId: string;
  };
};

export type AuthDeploymentAuthorityGetInput = { authorityId: string };
export type AuthDeploymentAuthorityGetOutput = {
  authority: {
    acceptedNeedsDigest: string;
    authorityId: string;
    createdAt: number;
    decision:
      | { decidedAt: number; decidedBy: string; reason: string | null }
      | null;
    deploymentId: string;
    desiredCapabilities: Array<string>;
    desiredGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    expiresAt: number | null;
    kind: "deployment";
    materialization: {
      authorityId: string;
      authorityKind: "identity" | "deployment";
      authorityVersion: number;
      effectiveCapabilities: Array<string>;
      effectiveGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      error: string | null;
      expiresAt: number | null;
      materializationId: string;
      materializationVersion: number;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "app" | "device" | "agent";
      participantNeedsDigest: string;
      reconciledAt: number | null;
      state: "available" | "unavailable" | "error";
      subjectId: string;
    } | null;
    participantArtifactDigest: string;
    participantId: string;
    participantKind: "service" | "device";
    state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
};

export type AuthDeploymentAuthorityListInput = {
  cursor?: string;
  deploymentId?: string;
  limit?: number;
  participantId?: string;
  state?: "pending" | "accepted" | "rejected" | "revoked" | "stale";
};
export type AuthDeploymentAuthorityListOutput = {
  entries: Array<
    {
      acceptedNeedsDigest: string;
      authorityId: string;
      createdAt: number;
      decision:
        | { decidedAt: number; decidedBy: string; reason: string | null }
        | null;
      deploymentId: string;
      desiredCapabilities: Array<string>;
      desiredGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      expiresAt: number | null;
      kind: "deployment";
      materialization: {
        authorityId: string;
        authorityKind: "identity" | "deployment";
        authorityVersion: number;
        effectiveCapabilities: Array<string>;
        effectiveGrantSet: {
          format: "trellis.grant-set.v1";
          permissions: Array<
            {
              action:
                | "call"
                | "invoke"
                | "observe"
                | "cancel"
                | "control"
                | "publish"
                | "subscribe"
                | "read"
                | "write"
                | "delete"
                | "submit"
                | "process"
                | "consume";
              target: {
                api: string;
                kind: "apiSurface";
                name: string;
                surface: "rpc" | "operation" | "event" | "feed" | "state";
              } | {
                api: string;
                kind: "operationSignal";
                operation: string;
                signal: string;
              } | {
                kind: "participantResource";
                name: string;
                participant: string;
                resource:
                  | "state"
                  | "jobQueue"
                  | "eventConsumer"
                  | "kv"
                  | "store";
              };
            }
          >;
        };
        error: string | null;
        expiresAt: number | null;
        materializationId: string;
        materializationVersion: number;
        participantArtifactDigest: string;
        participantId: string;
        participantKind: "service" | "app" | "device" | "agent";
        participantNeedsDigest: string;
        reconciledAt: number | null;
        state: "available" | "unavailable" | "error";
        subjectId: string;
      } | null;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "device";
      state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
      updatedAt: number;
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthDeploymentAuthorityPlanInput = {
  deploymentId: string;
  expiresAt: number | null;
  idempotencyKey: string;
  participantArtifact: {};
  referencedApiArtifacts: Array<{}>;
};
export type AuthDeploymentAuthorityPlanOutput = {
  proposal: {
    authorityKind: "identity" | "deployment";
    baseAuthorityVersion: number | null;
    classification: "initial" | "update" | "migration";
    createdAt: number;
    decisionAt: number | null;
    decisionBy: string | null;
    decisionReason: string | null;
    expiresAt: number | null;
    participantArtifactDigest: string;
    participantId: string;
    participantNeedsDigest: string;
    proposalId: string;
    proposedCapabilities: Array<string>;
    proposedGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    reasons: Array<string>;
    state: "pending" | "accepted" | "rejected" | "superseded" | "expired";
    subjectId: string;
  };
};

export type AuthDeploymentAuthorityPlansGetInput = { proposalId: string };
export type AuthDeploymentAuthorityPlansGetOutput = {
  proposal: {
    authorityKind: "identity" | "deployment";
    baseAuthorityVersion: number | null;
    classification: "initial" | "update" | "migration";
    createdAt: number;
    decisionAt: number | null;
    decisionBy: string | null;
    decisionReason: string | null;
    expiresAt: number | null;
    participantArtifactDigest: string;
    participantId: string;
    participantNeedsDigest: string;
    proposalId: string;
    proposedCapabilities: Array<string>;
    proposedGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    reasons: Array<string>;
    state: "pending" | "accepted" | "rejected" | "superseded" | "expired";
    subjectId: string;
  };
};

export type AuthDeploymentAuthorityPlansListInput = {
  cursor?: string;
  deploymentId?: string;
  limit?: number;
  state?: "pending" | "accepted" | "rejected" | "superseded" | "expired";
};
export type AuthDeploymentAuthorityPlansListOutput = {
  entries: Array<
    {
      authorityKind: "identity" | "deployment";
      baseAuthorityVersion: number | null;
      classification: "initial" | "update" | "migration";
      createdAt: number;
      decisionAt: number | null;
      decisionBy: string | null;
      decisionReason: string | null;
      expiresAt: number | null;
      participantArtifactDigest: string;
      participantId: string;
      participantNeedsDigest: string;
      proposalId: string;
      proposedCapabilities: Array<string>;
      proposedGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      reasons: Array<string>;
      state: "pending" | "accepted" | "rejected" | "superseded" | "expired";
      subjectId: string;
    }
  >;
  nextCursor: string | null;
};

export type AuthDeploymentAuthorityReconcileInput = {
  authorityId: string;
  expectedVersion: number | null;
  idempotencyKey: string;
};
export type AuthDeploymentAuthorityReconcileOutput = {
  authority: {
    acceptedNeedsDigest: string;
    authorityId: string;
    createdAt: number;
    decision:
      | { decidedAt: number; decidedBy: string; reason: string | null }
      | null;
    deploymentId: string;
    desiredCapabilities: Array<string>;
    desiredGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    expiresAt: number | null;
    kind: "deployment";
    materialization: {
      authorityId: string;
      authorityKind: "identity" | "deployment";
      authorityVersion: number;
      effectiveCapabilities: Array<string>;
      effectiveGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      error: string | null;
      expiresAt: number | null;
      materializationId: string;
      materializationVersion: number;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "app" | "device" | "agent";
      participantNeedsDigest: string;
      reconciledAt: number | null;
      state: "available" | "unavailable" | "error";
      subjectId: string;
    } | null;
    participantArtifactDigest: string;
    participantId: string;
    participantKind: "service" | "device";
    state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
};

export type AuthDeploymentAuthorityRejectInput = {
  idempotencyKey: string;
  proposalId: string;
  reason: string | null;
};
export type AuthDeploymentAuthorityRejectOutput = {
  proposal: {
    authorityKind: "identity" | "deployment";
    baseAuthorityVersion: number | null;
    classification: "initial" | "update" | "migration";
    createdAt: number;
    decisionAt: number | null;
    decisionBy: string | null;
    decisionReason: string | null;
    expiresAt: number | null;
    participantArtifactDigest: string;
    participantId: string;
    participantNeedsDigest: string;
    proposalId: string;
    proposedCapabilities: Array<string>;
    proposedGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    reasons: Array<string>;
    state: "pending" | "accepted" | "rejected" | "superseded" | "expired";
    subjectId: string;
  };
};

export type AuthDeploymentsCreateInput = {
  displayName: string;
  expiresAt: number | null;
  idempotencyKey: string;
  kind: "service" | "device";
  participantId: string | null;
  portalId: string | null;
  requiresDeviceDelegation: boolean;
};
export type AuthDeploymentsCreateOutput = {
  deployment: {
    createdAt: number;
    deploymentId: string;
    disabledAt: number | null;
    displayName: string;
    expiresAt: number | null;
    kind: "service" | "device";
    participantId: string | null;
    portalId: string | null;
    requiresDeviceDelegation: boolean;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
};

export type AuthDeploymentsDisableInput = {
  deploymentId: string;
  expectedVersion: number;
  idempotencyKey: string;
  reason: string | null;
};
export type AuthDeploymentsDisableOutput = {
  deployment: {
    createdAt: number;
    deploymentId: string;
    disabledAt: number | null;
    displayName: string;
    expiresAt: number | null;
    kind: "service" | "device";
    participantId: string | null;
    portalId: string | null;
    requiresDeviceDelegation: boolean;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthDeploymentsEnableInput = {
  deploymentId: string;
  expectedVersion: number;
  idempotencyKey: string;
  reason: string | null;
};
export type AuthDeploymentsEnableOutput = {
  deployment: {
    createdAt: number;
    deploymentId: string;
    disabledAt: number | null;
    displayName: string;
    expiresAt: number | null;
    kind: "service" | "device";
    participantId: string | null;
    portalId: string | null;
    requiresDeviceDelegation: boolean;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthDeploymentsListInput = {
  cursor?: string;
  kind?: "service" | "device";
  limit?: number;
  state?: "active" | "disabled" | "revoked";
};
export type AuthDeploymentsListOutput = {
  entries: Array<
    {
      createdAt: number;
      deploymentId: string;
      disabledAt: number | null;
      displayName: string;
      expiresAt: number | null;
      kind: "service" | "device";
      participantId: string | null;
      portalId: string | null;
      requiresDeviceDelegation: boolean;
      revokedAt: number | null;
      state: "active" | "disabled" | "revoked";
      updatedAt: number;
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthDeploymentsRemoveInput = {
  deploymentId: string;
  expectedVersion: number;
  idempotencyKey: string;
  reason: string | null;
};
export type AuthDeploymentsRemoveOutput = {
  deployment: {
    createdAt: number;
    deploymentId: string;
    disabledAt: number | null;
    displayName: string;
    expiresAt: number | null;
    kind: "service" | "device";
    participantId: string | null;
    portalId: string | null;
    requiresDeviceDelegation: boolean;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthDeviceUserAuthoritiesListInput = {
  cursor?: string;
  deploymentId?: string;
  limit?: number;
  principalId?: string;
};
export type AuthDeviceUserAuthoritiesListOutput = {
  entries: Array<
    {
      authority: {
        acceptedNeedsDigest: string;
        authorityId: string;
        createdAt: number;
        decision: {
          decidedAt: number;
          decidedBy: string;
          reason: string | null;
        } | null;
        desiredCapabilities: Array<string>;
        desiredGrantSet: {
          format: "trellis.grant-set.v1";
          permissions: Array<
            {
              action:
                | "call"
                | "invoke"
                | "observe"
                | "cancel"
                | "control"
                | "publish"
                | "subscribe"
                | "read"
                | "write"
                | "delete"
                | "submit"
                | "process"
                | "consume";
              target: {
                api: string;
                kind: "apiSurface";
                name: string;
                surface: "rpc" | "operation" | "event" | "feed" | "state";
              } | {
                api: string;
                kind: "operationSignal";
                operation: string;
                signal: string;
              } | {
                kind: "participantResource";
                name: string;
                participant: string;
                resource:
                  | "state"
                  | "jobQueue"
                  | "eventConsumer"
                  | "kv"
                  | "store";
              };
            }
          >;
        };
        expiresAt: number | null;
        kind: "identity";
        materialization: {
          authorityId: string;
          authorityKind: "identity" | "deployment";
          authorityVersion: number;
          effectiveCapabilities: Array<string>;
          effectiveGrantSet: {
            format: "trellis.grant-set.v1";
            permissions: Array<
              {
                action:
                  | "call"
                  | "invoke"
                  | "observe"
                  | "cancel"
                  | "control"
                  | "publish"
                  | "subscribe"
                  | "read"
                  | "write"
                  | "delete"
                  | "submit"
                  | "process"
                  | "consume";
                target: {
                  api: string;
                  kind: "apiSurface";
                  name: string;
                  surface: "rpc" | "operation" | "event" | "feed" | "state";
                } | {
                  api: string;
                  kind: "operationSignal";
                  operation: string;
                  signal: string;
                } | {
                  kind: "participantResource";
                  name: string;
                  participant: string;
                  resource:
                    | "state"
                    | "jobQueue"
                    | "eventConsumer"
                    | "kv"
                    | "store";
                };
              }
            >;
          };
          error: string | null;
          expiresAt: number | null;
          materializationId: string;
          materializationVersion: number;
          participantArtifactDigest: string;
          participantId: string;
          participantKind: "service" | "app" | "device" | "agent";
          participantNeedsDigest: string;
          reconciledAt: number | null;
          state: "available" | "unavailable" | "error";
          subjectId: string;
        } | null;
        participantArtifactDigest: string;
        participantId: string;
        principalId: string;
        state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
        updatedAt: number;
        version: number;
      } | null;
      device: {
        administrativeApproval: "pending" | "approved" | "rejected" | "revoked";
        createdAt: number;
        delegationExpiresAt: number | null;
        delegationRequired: boolean;
        delegationState: "active" | "missing" | "revoked";
        deploymentId: string;
        identityKeyId: string | null;
        identityPublicKey: string | null;
        instanceId: string;
        participantId: string | null;
        principalId: string;
        state: "pending" | "active" | "disabled" | "revoked";
        updatedAt: number;
        version: number;
      };
    }
  >;
  nextCursor: string | null;
};

export type AuthDeviceUserAuthoritiesReviewsDecideInput = {
  decision: "approve" | "reject";
  expectedVersion: number;
  idempotencyKey: string;
  reason: string | null;
  reviewId: string;
};
export type AuthDeviceUserAuthoritiesReviewsDecideOutput = {
  review: {
    confirmationCode: string;
    decidedAt: number | null;
    decidedBy: string | null;
    deploymentId: string;
    devicePrincipalId: string;
    expiresAt: number;
    instanceId: string;
    reason: string | null;
    requestedAt: number;
    reviewId: string;
    state: "pending" | "approved" | "rejected" | "expired" | "revoked";
    version: number;
  };
};

export type AuthDeviceUserAuthoritiesReviewsListInput = {
  cursor?: string;
  deploymentId?: string;
  limit?: number;
  state?: "pending" | "approved" | "rejected" | "expired" | "revoked";
};
export type AuthDeviceUserAuthoritiesReviewsListOutput = {
  entries: Array<
    {
      confirmationCode: string;
      decidedAt: number | null;
      decidedBy: string | null;
      deploymentId: string;
      devicePrincipalId: string;
      expiresAt: number;
      instanceId: string;
      reason: string | null;
      requestedAt: number;
      reviewId: string;
      state: "pending" | "approved" | "rejected" | "expired" | "revoked";
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthDeviceUserAuthoritiesRevokeInput = {
  deploymentId: string;
  devicePrincipalId: string;
  idempotencyKey: string;
  reason: string | null;
};
export type AuthDeviceUserAuthoritiesRevokeOutput = {
  device: {
    administrativeApproval: "pending" | "approved" | "rejected" | "revoked";
    createdAt: number;
    delegationExpiresAt: number | null;
    delegationRequired: boolean;
    delegationState: "active" | "missing" | "revoked";
    deploymentId: string;
    identityKeyId: string | null;
    identityPublicKey: string | null;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "pending" | "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  kickedSessionCount: number;
};

export type AuthDevicesConnectInfoGetInput = {
  challengeDigest: string | null;
  deploymentId: string;
  deviceIdentityKeyId: string;
  instanceId: string;
  issuedAt: number;
  newSessionNkey: string;
  newSessionPublicKey: string;
  participantDigest: string;
  participantId: string;
  proof: { format: "trellis.session-proof.v1"; signature: string };
  requestId: string;
};
export type AuthDevicesConnectInfoGetOutput = {
  deploymentId: string;
  endpoints: {
    authMode: "session_nkey";
    authorityMode: "server_issued";
    maximumClockSkewMs: number;
    native: Array<string>;
    websocket: Array<string>;
  };
  instanceId: string;
  participantId: string | null;
};

export type AuthDevicesDisableInput = {
  expectedVersion: number;
  idempotencyKey: string;
  instanceId: string;
  reason: string | null;
};
export type AuthDevicesDisableOutput = {
  device: {
    administrativeApproval: "pending" | "approved" | "rejected" | "revoked";
    createdAt: number;
    delegationExpiresAt: number | null;
    delegationRequired: boolean;
    delegationState: "active" | "missing" | "revoked";
    deploymentId: string;
    identityKeyId: string | null;
    identityPublicKey: string | null;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "pending" | "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthDevicesEnableInput = {
  expectedVersion: number;
  idempotencyKey: string;
  instanceId: string;
  reason: string | null;
};
export type AuthDevicesEnableOutput = {
  device: {
    administrativeApproval: "pending" | "approved" | "rejected" | "revoked";
    createdAt: number;
    delegationExpiresAt: number | null;
    delegationRequired: boolean;
    delegationState: "active" | "missing" | "revoked";
    deploymentId: string;
    identityKeyId: string | null;
    identityPublicKey: string | null;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "pending" | "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthDevicesListInput = {
  cursor?: string;
  deploymentId?: string;
  limit?: number;
  state?: "pending" | "active" | "disabled" | "revoked";
};
export type AuthDevicesListOutput = {
  entries: Array<
    {
      administrativeApproval: "pending" | "approved" | "rejected" | "revoked";
      createdAt: number;
      delegationExpiresAt: number | null;
      delegationRequired: boolean;
      delegationState: "active" | "missing" | "revoked";
      deploymentId: string;
      identityKeyId: string | null;
      identityPublicKey: string | null;
      instanceId: string;
      participantId: string | null;
      principalId: string;
      state: "pending" | "active" | "disabled" | "revoked";
      updatedAt: number;
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthDevicesProvisionInput = {
  deploymentId: string;
  idempotencyKey: string;
  identityPublicKey: string | null;
  instanceId: string | null;
  participantId: string | null;
};
export type AuthDevicesProvisionOutput = {
  device: {
    administrativeApproval: "pending" | "approved" | "rejected" | "revoked";
    createdAt: number;
    delegationExpiresAt: number | null;
    delegationRequired: boolean;
    delegationState: "active" | "missing" | "revoked";
    deploymentId: string;
    identityKeyId: string | null;
    identityPublicKey: string | null;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "pending" | "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  provisioningSecret: string | null;
};

export type AuthDevicesRemoveInput = {
  expectedVersion: number;
  idempotencyKey: string;
  instanceId: string;
  reason: string | null;
};
export type AuthDevicesRemoveOutput = {
  device: {
    administrativeApproval: "pending" | "approved" | "rejected" | "revoked";
    createdAt: number;
    delegationExpiresAt: number | null;
    delegationRequired: boolean;
    delegationState: "active" | "missing" | "revoked";
    deploymentId: string;
    identityKeyId: string | null;
    identityPublicKey: string | null;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "pending" | "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthIdentityAuthorityGetInput = { authorityId: string };
export type AuthIdentityAuthorityGetOutput = {
  authority: {
    acceptedNeedsDigest: string;
    authorityId: string;
    createdAt: number;
    decision:
      | { decidedAt: number; decidedBy: string; reason: string | null }
      | null;
    desiredCapabilities: Array<string>;
    desiredGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    expiresAt: number | null;
    kind: "identity";
    materialization: {
      authorityId: string;
      authorityKind: "identity" | "deployment";
      authorityVersion: number;
      effectiveCapabilities: Array<string>;
      effectiveGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      error: string | null;
      expiresAt: number | null;
      materializationId: string;
      materializationVersion: number;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "app" | "device" | "agent";
      participantNeedsDigest: string;
      reconciledAt: number | null;
      state: "available" | "unavailable" | "error";
      subjectId: string;
    } | null;
    participantArtifactDigest: string;
    participantId: string;
    principalId: string;
    state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
};

export type AuthIdentityAuthorityListInput = {
  cursor?: string;
  limit?: number;
  participantId?: string;
  principalId?: string;
  state?: "pending" | "accepted" | "rejected" | "revoked" | "stale";
};
export type AuthIdentityAuthorityListOutput = {
  entries: Array<
    {
      acceptedNeedsDigest: string;
      authorityId: string;
      createdAt: number;
      decision:
        | { decidedAt: number; decidedBy: string; reason: string | null }
        | null;
      desiredCapabilities: Array<string>;
      desiredGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      expiresAt: number | null;
      kind: "identity";
      materialization: {
        authorityId: string;
        authorityKind: "identity" | "deployment";
        authorityVersion: number;
        effectiveCapabilities: Array<string>;
        effectiveGrantSet: {
          format: "trellis.grant-set.v1";
          permissions: Array<
            {
              action:
                | "call"
                | "invoke"
                | "observe"
                | "cancel"
                | "control"
                | "publish"
                | "subscribe"
                | "read"
                | "write"
                | "delete"
                | "submit"
                | "process"
                | "consume";
              target: {
                api: string;
                kind: "apiSurface";
                name: string;
                surface: "rpc" | "operation" | "event" | "feed" | "state";
              } | {
                api: string;
                kind: "operationSignal";
                operation: string;
                signal: string;
              } | {
                kind: "participantResource";
                name: string;
                participant: string;
                resource:
                  | "state"
                  | "jobQueue"
                  | "eventConsumer"
                  | "kv"
                  | "store";
              };
            }
          >;
        };
        error: string | null;
        expiresAt: number | null;
        materializationId: string;
        materializationVersion: number;
        participantArtifactDigest: string;
        participantId: string;
        participantKind: "service" | "app" | "device" | "agent";
        participantNeedsDigest: string;
        reconciledAt: number | null;
        state: "available" | "unavailable" | "error";
        subjectId: string;
      } | null;
      participantArtifactDigest: string;
      participantId: string;
      principalId: string;
      state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
      updatedAt: number;
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthIdentityAuthorityRevokeInput = {
  authorityId: string;
  expectedVersion: number;
  idempotencyKey: string;
  reason: string | null;
};
export type AuthIdentityAuthorityRevokeOutput = {
  authority: {
    acceptedNeedsDigest: string;
    authorityId: string;
    createdAt: number;
    decision:
      | { decidedAt: number; decidedBy: string; reason: string | null }
      | null;
    desiredCapabilities: Array<string>;
    desiredGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    expiresAt: number | null;
    kind: "identity";
    materialization: {
      authorityId: string;
      authorityKind: "identity" | "deployment";
      authorityVersion: number;
      effectiveCapabilities: Array<string>;
      effectiveGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      error: string | null;
      expiresAt: number | null;
      materializationId: string;
      materializationVersion: number;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "app" | "device" | "agent";
      participantNeedsDigest: string;
      reconciledAt: number | null;
      state: "available" | "unavailable" | "error";
      subjectId: string;
    } | null;
    participantArtifactDigest: string;
    participantId: string;
    principalId: string;
    state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
};

export type AuthPortalsGetInput = { portalId: string };
export type AuthPortalsGetOutput = {
  portal: {
    builtIn: boolean;
    createdAt: number;
    disabled: boolean;
    displayName: string;
    entryUrl: string | null;
    loginSettings: {
      federatedRegistration: boolean;
      localLogin: boolean;
      localRegistration: boolean;
      providers: Array<string> | null;
    };
    portalId: string;
    updatedAt: number;
    version: number;
  };
  routes: Array<
    {
      createdAt: number;
      deploymentId: string | null;
      origin: string | null;
      participantId: string | null;
      portalId: string;
      priority: number;
      routeId: string;
      updatedAt: number;
      version: number;
    }
  >;
};

export type AuthPortalsListInput = {
  cursor?: string;
  disabled?: boolean;
  limit?: number;
};
export type AuthPortalsListOutput = {
  entries: Array<
    {
      builtIn: boolean;
      createdAt: number;
      disabled: boolean;
      displayName: string;
      entryUrl: string | null;
      loginSettings: {
        federatedRegistration: boolean;
        localLogin: boolean;
        localRegistration: boolean;
        providers: Array<string> | null;
      };
      portalId: string;
      updatedAt: number;
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthPortalsLoginSettingsGetInput = { portalId: string };
export type AuthPortalsLoginSettingsGetOutput = {
  portalId: string;
  settings: {
    federatedRegistration: boolean;
    localLogin: boolean;
    localRegistration: boolean;
    providers: Array<string> | null;
  };
  version: number;
};

export type AuthPortalsLoginSettingsUpdateInput = {
  expectedVersion: number;
  idempotencyKey: string;
  portalId: string;
  settings: {
    federatedRegistration: boolean;
    localLogin: boolean;
    localRegistration: boolean;
    providers: Array<string> | null;
  };
};
export type AuthPortalsLoginSettingsUpdateOutput = {
  portalId: string;
  settings: {
    federatedRegistration: boolean;
    localLogin: boolean;
    localRegistration: boolean;
    providers: Array<string> | null;
  };
  version: number;
};

export type AuthPortalsPutInput = {
  disabled: boolean;
  displayName: string;
  entryUrl: string | null;
  expectedVersion: number | null;
  idempotencyKey: string;
  loginSettings: {
    federatedRegistration: boolean;
    localLogin: boolean;
    localRegistration: boolean;
    providers: Array<string> | null;
  };
  portalId: string;
};
export type AuthPortalsPutOutput = {
  portal: {
    builtIn: boolean;
    createdAt: number;
    disabled: boolean;
    displayName: string;
    entryUrl: string | null;
    loginSettings: {
      federatedRegistration: boolean;
      localLogin: boolean;
      localRegistration: boolean;
      providers: Array<string> | null;
    };
    portalId: string;
    updatedAt: number;
    version: number;
  };
};

export type AuthPortalsRemoveInput = {
  expectedVersion: number;
  idempotencyKey: string;
  portalId: string;
};
export type AuthPortalsRemoveOutput = { removed: boolean };

export type AuthPortalsRoutesPutInput = {
  deploymentId: string | null;
  expectedVersion: number | null;
  idempotencyKey: string;
  origin: string | null;
  participantId: string | null;
  portalId: string;
  priority: number;
  routeId: string | null;
};
export type AuthPortalsRoutesPutOutput = {
  route: {
    createdAt: number;
    deploymentId: string | null;
    origin: string | null;
    participantId: string | null;
    portalId: string;
    priority: number;
    routeId: string;
    updatedAt: number;
    version: number;
  };
};

export type AuthPortalsRoutesRemoveInput = {
  expectedVersion: number;
  idempotencyKey: string;
  routeId: string;
};
export type AuthPortalsRoutesRemoveOutput = { removed: boolean };

export type AuthServiceInstancesDisableInput = {
  expectedVersion: number;
  idempotencyKey: string;
  instanceId: string;
  reason: string | null;
};
export type AuthServiceInstancesDisableOutput = {
  instance: {
    createdAt: number;
    deploymentId: string;
    identityKeyId: string;
    identityPublicKey: string;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "active" | "disabled" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthServiceInstancesEnableInput = {
  expectedVersion: number;
  idempotencyKey: string;
  instanceId: string;
  reason: string | null;
};
export type AuthServiceInstancesEnableOutput = {
  instance: {
    createdAt: number;
    deploymentId: string;
    identityKeyId: string;
    identityPublicKey: string;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "active" | "disabled" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthServiceInstancesListInput = {
  cursor?: string;
  deploymentId?: string;
  limit?: number;
  state?: "active" | "disabled" | "revoked" | "stale";
};
export type AuthServiceInstancesListOutput = {
  entries: Array<
    {
      createdAt: number;
      deploymentId: string;
      identityKeyId: string;
      identityPublicKey: string;
      instanceId: string;
      participantId: string | null;
      principalId: string;
      state: "active" | "disabled" | "revoked" | "stale";
      updatedAt: number;
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthServiceInstancesProvisionInput = {
  deploymentId: string;
  idempotencyKey: string;
  identityPublicKey: string;
  instanceId: string | null;
  participantId: string | null;
};
export type AuthServiceInstancesProvisionOutput = {
  instance: {
    createdAt: number;
    deploymentId: string;
    identityKeyId: string;
    identityPublicKey: string;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "active" | "disabled" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
};

export type AuthServiceInstancesRemoveInput = {
  expectedVersion: number;
  idempotencyKey: string;
  instanceId: string;
  reason: string | null;
};
export type AuthServiceInstancesRemoveOutput = {
  instance: {
    createdAt: number;
    deploymentId: string;
    identityKeyId: string;
    identityPublicKey: string;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "active" | "disabled" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  };
  mutation: {
    changed: boolean;
    resourceId: string;
    state: string;
    version: number;
  };
};

export type AuthSessionsListInput = {
  cursor?: string;
  deploymentId?: string;
  limit?: number;
  participantId?: string;
  principalId?: string;
  state?: "active" | "expired" | "revoked";
};
export type AuthSessionsListOutput = {
  entries: Array<
    {
      createdAt: number;
      expiresAt: number | null;
      inboxPrefix: string;
      lastSeenAt: number;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "app" | "device" | "agent";
      participantNeedsDigest: string;
      principalId: string;
      principalKind: "user" | "service" | "device";
      revokedAt: number | null;
      sessionId: string;
      sessionKeyId: string;
      sessionPublicKey: string;
      state: "active" | "expired" | "revoked";
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthSessionsLogoutInput = {};
export type AuthSessionsLogoutOutput = {
  kickedConnections: number;
  session: {
    createdAt: number;
    expiresAt: number | null;
    inboxPrefix: string;
    lastSeenAt: number;
    participantArtifactDigest: string;
    participantId: string;
    participantKind: "service" | "app" | "device" | "agent";
    participantNeedsDigest: string;
    principalId: string;
    principalKind: "user" | "service" | "device";
    revokedAt: number | null;
    sessionId: string;
    sessionKeyId: string;
    sessionPublicKey: string;
    state: "active" | "expired" | "revoked";
    version: number;
  };
};

export type AuthSessionsMeInput = {};
export type AuthSessionsMeOutput = {
  deploymentId: string | null;
  instanceId: string | null;
  session: {
    createdAt: number;
    expiresAt: number | null;
    inboxPrefix: string;
    lastSeenAt: number;
    participantArtifactDigest: string;
    participantId: string;
    participantKind: "service" | "app" | "device" | "agent";
    participantNeedsDigest: string;
    principalId: string;
    principalKind: "user" | "service" | "device";
    revokedAt: number | null;
    sessionId: string;
    sessionKeyId: string;
    sessionPublicKey: string;
    state: "active" | "expired" | "revoked";
    version: number;
  };
  user: {
    createdAt: number;
    disabledAt: number | null;
    email: string | null;
    image: string | null;
    name: string | null;
    principalId: string;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    userId: string;
    version: number;
  } | null;
};

export type AuthSessionsRevokeInput = {
  expectedVersion: number | null;
  idempotencyKey: string;
  reason: string | null;
  sessionId: string;
};
export type AuthSessionsRevokeOutput = {
  kickedConnections: number;
  session: {
    createdAt: number;
    expiresAt: number | null;
    inboxPrefix: string;
    lastSeenAt: number;
    participantArtifactDigest: string;
    participantId: string;
    participantKind: "service" | "app" | "device" | "agent";
    participantNeedsDigest: string;
    principalId: string;
    principalKind: "user" | "service" | "device";
    revokedAt: number | null;
    sessionId: string;
    sessionKeyId: string;
    sessionPublicKey: string;
    state: "active" | "expired" | "revoked";
    version: number;
  };
};

export type AuthUserIdentitiesListInput = {
  cursor?: string;
  limit?: number;
  providerId?: string;
};
export type AuthUserIdentitiesListOutput = {
  entries: Array<
    {
      createdAt: number;
      lastSeenAt: number;
      observedEmail: string | null;
      observedName: string | null;
      principalId: string;
      providerId: string;
      subject: string;
      username: string | null;
    }
  >;
  nextCursor: string | null;
};

export type AuthUserIdentitiesUnlinkInput = {
  idempotencyKey: string;
  providerId: string;
  subject: string;
};
export type AuthUserIdentitiesUnlinkOutput = { unlinked: boolean };

export type AuthUsersCreateInput = {
  email: string | null;
  idempotencyKey: string;
  image: string | null;
  name: string | null;
};
export type AuthUsersCreateOutput = {
  user: {
    createdAt: number;
    disabledAt: number | null;
    email: string | null;
    image: string | null;
    name: string | null;
    principalId: string;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    userId: string;
    version: number;
  };
};

export type AuthUsersGetInput = { userId: string };
export type AuthUsersGetOutput = {
  user: {
    createdAt: number;
    disabledAt: number | null;
    email: string | null;
    image: string | null;
    name: string | null;
    principalId: string;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    userId: string;
    version: number;
  };
};

export type AuthUsersIdentityLinkCreateInput = {
  allowedProviders: Array<string>;
  idempotencyKey: string;
  returnTarget: string | null;
};
export type AuthUsersIdentityLinkCreateOutput = {
  flow: {
    allowedProviders: Array<string>;
    completionUrl: string;
    consumedAt: number | null;
    createdAt: number;
    expiresAt: number;
    flowId: string;
    kind: "identity_link";
    returnTarget: string | null;
    targetPrincipalId: string;
    version: number;
  };
};

export type AuthUsersListInput = {
  cursor?: string;
  limit?: number;
  state?: "active" | "disabled" | "revoked";
};
export type AuthUsersListOutput = {
  entries: Array<
    {
      createdAt: number;
      disabledAt: number | null;
      email: string | null;
      image: string | null;
      name: string | null;
      principalId: string;
      revokedAt: number | null;
      state: "active" | "disabled" | "revoked";
      updatedAt: number;
      userId: string;
      version: number;
    }
  >;
  nextCursor: string | null;
};

export type AuthUsersPasswordChangeInput = {
  currentPassword: string;
  idempotencyKey: string;
  newPassword: string;
};
export type AuthUsersPasswordChangeOutput = {
  changedAt: number;
  revokedSessionCount: number;
};

export type AuthUsersPasswordResetCreateInput = {
  idempotencyKey: string;
  returnTarget: string | null;
  userId: string;
};
export type AuthUsersPasswordResetCreateOutput = {
  flow: {
    allowedProviders: Array<string>;
    completionUrl: string;
    consumedAt: number | null;
    createdAt: number;
    expiresAt: number;
    flowId: string;
    kind: "password_reset";
    returnTarget: string | null;
    targetPrincipalId: string;
    version: number;
  };
};

export type AuthUsersResolveInput = {
  selector: { kind: "user"; userId: string } | {
    kind: "provider";
    providerId: string;
    providerSubject: string;
  };
};
export type AuthUsersResolveOutput = {
  user: {
    createdAt: number;
    disabledAt: number | null;
    email: string | null;
    image: string | null;
    name: string | null;
    principalId: string;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    userId: string;
    version: number;
  } | null;
};

export type AuthUsersUpdateInput = {
  email: string | null;
  expectedVersion: number;
  idempotencyKey: string;
  image: string | null;
  name: string | null;
  state: "active" | "disabled";
  userId: string;
};
export type AuthUsersUpdateOutput = {
  user: {
    createdAt: number;
    disabledAt: number | null;
    email: string | null;
    image: string | null;
    name: string | null;
    principalId: string;
    revokedAt: number | null;
    state: "active" | "disabled" | "revoked";
    updatedAt: number;
    userId: string;
    version: number;
  };
};

export type AuthDeviceUserAuthoritiesResolveInput = { flowId: string };
export type AuthDeviceUserAuthoritiesResolveProgress = {
  retryAfterMs: number;
  state: "waiting" | "review_pending" | "delegation_pending";
};
export type AuthDeviceUserAuthoritiesResolveOutput = {
  authority: {
    acceptedNeedsDigest: string;
    authorityId: string;
    createdAt: number;
    decision:
      | { decidedAt: number; decidedBy: string; reason: string | null }
      | null;
    desiredCapabilities: Array<string>;
    desiredGrantSet: {
      format: "trellis.grant-set.v1";
      permissions: Array<
        {
          action:
            | "call"
            | "invoke"
            | "observe"
            | "cancel"
            | "control"
            | "publish"
            | "subscribe"
            | "read"
            | "write"
            | "delete"
            | "submit"
            | "process"
            | "consume";
          target: {
            api: string;
            kind: "apiSurface";
            name: string;
            surface: "rpc" | "operation" | "event" | "feed" | "state";
          } | {
            api: string;
            kind: "operationSignal";
            operation: string;
            signal: string;
          } | {
            kind: "participantResource";
            name: string;
            participant: string;
            resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
          };
        }
      >;
    };
    expiresAt: number | null;
    kind: "identity";
    materialization: {
      authorityId: string;
      authorityKind: "identity" | "deployment";
      authorityVersion: number;
      effectiveCapabilities: Array<string>;
      effectiveGrantSet: {
        format: "trellis.grant-set.v1";
        permissions: Array<
          {
            action:
              | "call"
              | "invoke"
              | "observe"
              | "cancel"
              | "control"
              | "publish"
              | "subscribe"
              | "read"
              | "write"
              | "delete"
              | "submit"
              | "process"
              | "consume";
            target: {
              api: string;
              kind: "apiSurface";
              name: string;
              surface: "rpc" | "operation" | "event" | "feed" | "state";
            } | {
              api: string;
              kind: "operationSignal";
              operation: string;
              signal: string;
            } | {
              kind: "participantResource";
              name: string;
              participant: string;
              resource: "state" | "jobQueue" | "eventConsumer" | "kv" | "store";
            };
          }
        >;
      };
      error: string | null;
      expiresAt: number | null;
      materializationId: string;
      materializationVersion: number;
      participantArtifactDigest: string;
      participantId: string;
      participantKind: "service" | "app" | "device" | "agent";
      participantNeedsDigest: string;
      reconciledAt: number | null;
      state: "available" | "unavailable" | "error";
      subjectId: string;
    } | null;
    participantArtifactDigest: string;
    participantId: string;
    principalId: string;
    state: "pending" | "accepted" | "rejected" | "revoked" | "stale";
    updatedAt: number;
    version: number;
  } | null;
  device: {
    administrativeApproval: "pending" | "approved" | "rejected" | "revoked";
    createdAt: number;
    delegationExpiresAt: number | null;
    delegationRequired: boolean;
    delegationState: "active" | "missing" | "revoked";
    deploymentId: string;
    identityKeyId: string | null;
    identityPublicKey: string | null;
    instanceId: string;
    participantId: string | null;
    principalId: string;
    state: "pending" | "active" | "disabled" | "revoked";
    updatedAt: number;
    version: number;
  };
  review: {
    confirmationCode: string;
    decidedAt: number | null;
    decidedBy: string | null;
    deploymentId: string;
    devicePrincipalId: string;
    expiresAt: number;
    instanceId: string;
    reason: string | null;
    requestedAt: number;
    reviewId: string;
    state: "pending" | "approved" | "rejected" | "expired" | "revoked";
    version: number;
  };
};

export type AuthConnectionsClosedEvent = {
  connectionId: string;
  eventId: string;
  occurredAt: number;
  participantId: string;
  principalId: string;
  reason: string | null;
  sessionId: string;
};

export type AuthConnectionsKickedEvent = {
  connectionId: string;
  eventId: string;
  occurredAt: number;
  participantId: string;
  principalId: string;
  reason: string | null;
  sessionId: string;
};

export type AuthConnectionsOpenedEvent = {
  clientId: string;
  connectionId: string;
  eventId: string;
  occurredAt: number;
  participantId: string;
  principalId: string;
  serverId: string;
  sessionId: string;
};

export type AuthDeviceUserAuthoritiesApprovedEvent = {
  approvedBy: string;
  deploymentId: string;
  eventId: string;
  instanceId: string;
  occurredAt: number;
};

export type AuthDeviceUserAuthoritiesRequestedEvent = {
  deploymentId: string;
  eventId: string;
  instanceId: string;
  occurredAt: number;
  userPrincipalId: string;
};

export type AuthDeviceUserAuthoritiesResolvedEvent = {
  deploymentId: string;
  eventId: string;
  instanceId: string;
  occurredAt: number;
  state: string;
};

export type AuthDeviceUserAuthoritiesReviewRequestedEvent = {
  deploymentId: string;
  eventId: string;
  instanceId: string;
  occurredAt: number;
  reviewId: string;
};

export type AuthSessionsRevokedEvent = {
  eventId: string;
  occurredAt: number;
  participantId: string;
  principalId: string;
  reason: string | null;
  revokedBy: string | null;
  sessionId: string;
};

export type AuthErrorData =
  & SerializableErrorData
  & ({
    code: string;
    field: string | null;
    message: string;
    retryable: boolean;
  });
export class AuthError extends TrellisError<AuthErrorData> {
  static readonly schema = AuthErrorDetailsSchema;
  override readonly name = "AuthError" as const;
  readonly data: AuthErrorData;
  constructor(data: AuthErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }
  static fromSerializable(data: AuthErrorData): AuthError {
    return new AuthError(data);
  }
  override toSerializable(): AuthErrorData {
    return this.data;
  }
}

export type UnexpectedErrorData =
  & SerializableErrorData
  & ({
    code: string;
    field: string | null;
    message: string;
    retryable: boolean;
  });
export class UnexpectedError extends TrellisError<UnexpectedErrorData> {
  static readonly schema = AuthErrorDetailsSchema;
  override readonly name = "UnexpectedError" as const;
  readonly data: UnexpectedErrorData;
  constructor(data: UnexpectedErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }
  static fromSerializable(data: UnexpectedErrorData): UnexpectedError {
    return new UnexpectedError(data);
  }
  override toSerializable(): UnexpectedErrorData {
    return this.data;
  }
}

export type ValidationErrorData =
  & SerializableErrorData
  & ({
    code: string;
    field: string | null;
    message: string;
    retryable: boolean;
  });
export class ValidationError extends TrellisError<ValidationErrorData> {
  static readonly schema = AuthErrorDetailsSchema;
  override readonly name = "ValidationError" as const;
  readonly data: ValidationErrorData;
  constructor(data: ValidationErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }
  static fromSerializable(data: ValidationErrorData): ValidationError {
    return new ValidationError(data);
  }
  override toSerializable(): ValidationErrorData {
    return this.data;
  }
}
