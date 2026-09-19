import { liveVerifyServerProof } from "./protocol.ts";

/** Pinned peer identity from a verified authorization context. */
export type PinnedPeerIdentity = {
  connectionId: string;
  sessionKey: string;
  principalId: string;
  participantId: string;
  deploymentId?: string;
  instanceId?: string;
};

export type LiveGuardRequirement =
  | { kind: "observer"; permission: string }
  | { kind: "local-provider" }
  | { kind: "peer-provider"; expected: PinnedPeerIdentity };

/** Retained authorization coverage for one live session. */
export class LiveAuthorityGuard {
  readonly digest: string;
  readonly requirement: LiveGuardRequirement;

  constructor(digest: string, requirement: LiveGuardRequirement) {
    this.digest = digest;
    this.requirement = requirement;
  }

  verifyProviderFrame(
    proof: string,
    contextDigest: string,
    subject: string,
    body: Uint8Array,
    sessionKey: string,
  ): boolean {
    if (this.requirement.kind === "peer-provider") {
      if (sessionKey !== this.requirement.expected.sessionKey) return false;
    }
    try {
      liveVerifyServerProof(proof, contextDigest, subject, body, sessionKey);
      return true;
    } catch {
      return false;
    }
  }
}
