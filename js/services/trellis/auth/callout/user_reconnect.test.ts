import { assertEquals } from "@std/assert";

import type { UserContractApprovalPlan } from "../approval/plan.ts";
import type { UserProjectionEntry, UserSession } from "../schemas.ts";
import { resolveUserReconnectSession } from "./user_reconnect.ts";

function createSession(overrides: Partial<UserSession> = {}): UserSession {
  return {
    type: "user",
    userId: "usr_123",
    identity: {
      identityId: "idn_github_123",
      provider: "github",
      subject: "123",
    },
    email: "user@example.com",
    name: "User",
    participantKind: "app",
    createdAt: new Date("2026-01-01T00:00:00.000Z"),
    lastAuth: new Date("2026-01-01T00:00:00.000Z"),
    contractDigest: "digest-approved",
    contractId: "trellis.console@v1",
    contractDisplayName: "Console",
    contractDescription: "Admin app",
    app: {
      contractId: "trellis.console@v1",
      origin: "https://app.example.com",
    },
    approvalSource: "stored_approval",
    delegatedCapabilities: ["audit"],
    delegatedPublishSubjects: ["trellis.console.audit.publish"],
    delegatedSubscribeSubjects: ["trellis.console.audit"],
    identityAuthorityNeeds: {
      contracts: [{ contractId: "trellis.audit@v1", required: true }],
      surfaces: [{
        contractId: "trellis.audit@v1",
        kind: "event",
        name: "Audit.Recorded",
        action: "subscribe",
        required: true,
      }],
      capabilities: ["audit"],
      resources: [],
    },
    ...overrides,
  };
}

function activeUser(
  overrides: Partial<UserProjectionEntry> = {},
): UserProjectionEntry {
  return {
    origin: "github",
    id: "123",
    name: "User",
    email: "user@example.com",
    active: true,
    capabilities: ["audit"],
    capabilityGroups: [],
    ...overrides,
  };
}

Deno.test("resolveUserReconnectSession preserves stored delegated subjects without contract replanning", async () => {
  const result = await resolveUserReconnectSession({
    session: createSession(),
    presentedContractDigest: "digest-approved",
    loadUserProjection: async () => activeUser(),
  });

  assertEquals(result.ok, true);
  if (!result.ok) return;
  assertEquals(result.session.contractDigest, "digest-approved");
  assertEquals(result.session.delegatedCapabilities, ["audit"]);
  assertEquals(result.session.delegatedPublishSubjects, [
    "trellis.console.audit.publish",
  ]);
  assertEquals(result.session.delegatedSubscribeSubjects, [
    "trellis.console.audit",
  ]);
});

Deno.test("resolveUserReconnectSession denies subjects outside stored delegated grants even if current contracts would allow them", async () => {
  const result = await resolveUserReconnectSession({
    session: createSession({
      delegatedPublishSubjects: ["trellis.console.audit.publish"],
    }),
    presentedContractDigest: "digest-approved",
    loadUserProjection: async () =>
      activeUser({ capabilities: ["audit", "catalog.read"] }),
  });

  assertEquals(result.ok, true);
  if (!result.ok) return;
  assertEquals(
    result.session.delegatedPublishSubjects.includes(
      "trellis.console.catalog.ready",
    ),
    false,
  );
});

Deno.test("resolveUserReconnectSession rejects changed digest and missing current capabilities", async () => {
  assertEquals(
    await resolveUserReconnectSession({
      session: createSession(),
      presentedContractDigest: "digest-current-contract",
      loadUserProjection: async () => activeUser(),
    }),
    { ok: false, reason: "contract_changed" },
  );

  assertEquals(
    await resolveUserReconnectSession({
      session: createSession(),
      presentedContractDigest: "digest-approved",
      loadUserProjection: async () => activeUser({ capabilities: [] }),
    }),
    { ok: false, reason: "insufficient_permissions" },
  );
});

Deno.test("resolveUserReconnectSession narrows stale optional delegated capabilities", async () => {
  const approvalPlan: UserContractApprovalPlan = {
    digest: "digest-approved",
    contract: {
      format: "trellis.contract.v1",
      id: "krishi.krishi-ui@v1",
      displayName: "Krishi",
      description: "Krishi UI",
      kind: "app",
    },
    approval: {
      contractDigest: "digest-approved",
      contractId: "krishi.krishi-ui@v1",
      displayName: "Krishi",
      description: "Krishi UI",
      participantKind: "app",
      capabilities: {
        "krishi.sherpa::devices.admin.read": {
          displayName: "Sherpa admin read",
          description: "Read Sherpa admin data.",
        },
        "krishi.workspace::read": {
          displayName: "Workspace read",
          description: "Read workspace data.",
        },
      },
    },
    requiredCapabilities: ["krishi.workspace::read"],
    publishSubjects: [
      "rpc.v1.Sherpa.Admin.Devices.List",
      "rpc.v1.Workspace.Me",
    ],
    subscribeSubjects: [],
    publishSubjectGrants: [{
      subject: "rpc.v1.Workspace.Me",
      capabilities: ["krishi.workspace::read"],
    }, {
      subject: "rpc.v1.Sherpa.Admin.Devices.List",
      capabilities: ["krishi.sherpa::devices.admin.read"],
    }],
    subscribeSubjectGrants: [],
  };
  const result = await resolveUserReconnectSession({
    session: createSession({
      contractId: "krishi.krishi-ui@v1",
      delegatedCapabilities: [
        "krishi.sherpa::devices.admin.read",
        "krishi.workspace::read",
      ],
      delegatedPublishSubjects: [
        "rpc.v1.Sherpa.Admin.Devices.List",
        "rpc.v1.Workspace.Me",
      ],
      identityAuthorityNeeds: {
        contracts: [],
        surfaces: [],
        capabilities: [{
          capability: "krishi.workspace::read",
          required: true,
        }],
        resources: [],
      },
    }),
    presentedContractDigest: "digest-approved",
    loadUserProjection: async () =>
      activeUser({ capabilities: ["krishi.workspace::read"] }),
    approvalPlan,
  });

  assertEquals(result.ok, true);
  if (!result.ok) return;
  assertEquals(result.session.delegatedCapabilities, [
    "krishi.workspace::read",
  ]);
  assertEquals(result.session.delegatedPublishSubjects, [
    "rpc.v1.Workspace.Me",
  ]);
});

Deno.test("resolveUserReconnectSession checks approval-plan required capabilities before stored authority extras", async () => {
  const approvalPlan: UserContractApprovalPlan = {
    digest: "digest-approved",
    contract: {
      format: "trellis.contract.v1",
      id: "example.console@v1",
      displayName: "Example Console",
      description: "Browser app",
      kind: "app",
    },
    approval: {
      contractDigest: "digest-approved",
      contractId: "example.console@v1",
      displayName: "Example Console",
      description: "Browser app",
      participantKind: "app",
      capabilities: {
        "jobs:cancel": {
          displayName: "Cancel jobs",
          description: "Cancel jobs.",
        },
        "jobs:read": {
          displayName: "Read jobs",
          description: "Read jobs.",
        },
      },
    },
    requiredCapabilities: ["jobs:cancel", "jobs:read"],
    publishSubjects: [
      "operations.v1.example.Jobs.Run",
      "operations.v1.example.Jobs.Run.control",
    ],
    subscribeSubjects: [],
    publishSubjectGrants: [{
      subject: "operations.v1.example.Jobs.Run",
      capabilities: ["jobs:read"],
      required: true,
    }, {
      subject: "operations.v1.example.Jobs.Run.control",
      capabilities: ["jobs:cancel"],
      required: true,
    }],
    subscribeSubjectGrants: [],
  };
  const result = await resolveUserReconnectSession({
    session: createSession({
      contractId: "example.console@v1",
      delegatedCapabilities: ["jobs:cancel", "jobs:read"],
      delegatedPublishSubjects: [
        "operations.v1.example.Jobs.Run",
        "operations.v1.example.Jobs.Run.control",
      ],
      identityAuthorityNeeds: {
        contracts: [],
        surfaces: [],
        capabilities: [
          { capability: "jobs:cancel", required: true },
          { capability: "jobs:read", required: true },
          { capability: "jobs:signal", required: true },
        ],
        resources: [],
      },
    }),
    presentedContractDigest: "digest-approved",
    loadUserProjection: async () =>
      activeUser({ capabilities: ["jobs:cancel", "jobs:read"] }),
    approvalPlan,
  });

  assertEquals(result.ok, true);
  if (!result.ok) return;
  assertEquals(result.session.delegatedCapabilities, [
    "jobs:cancel",
    "jobs:read",
  ]);
});

Deno.test("resolveUserReconnectSession returns user state failures", async () => {
  assertEquals(
    await resolveUserReconnectSession({
      session: createSession(),
      presentedContractDigest: "digest-approved",
      loadUserProjection: async () => activeUser({ active: false }),
    }),
    { ok: false, reason: "user_inactive" },
  );

  assertEquals(
    await resolveUserReconnectSession({
      session: createSession(),
      presentedContractDigest: "digest-approved",
      loadUserProjection: async () => null,
    }),
    { ok: false, reason: "user_not_found" },
  );
});
