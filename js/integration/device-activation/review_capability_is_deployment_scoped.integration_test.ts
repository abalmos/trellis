import { assert, assertEquals } from "@std/assert";
import {
  type ClientAuthContinuation,
  type ClientAuthRequiredContext,
  defineAppContract,
  TrellisClient,
} from "@qlever-llc/trellis";
import * as trellisAuth from "@qlever-llc/trellis/sdk/auth";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { caseScopedContractId, caseScopedName } from "../_support/names.ts";
import { createDeviceActivationFixture } from "./_fixture.ts";

const CASE_ID =
  "device-activation.review-capability-is-deployment-scoped" as const;
const fixture = createDeviceActivationFixture(CASE_ID);
const reviewerContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.device-activation-reviewer",
    CASE_ID,
  ),
  displayName: "Trellis Integration Device Activation Scoped Reviewer",
  description: "Scoped reviewer for device activation integration coverage.",
  uses: [
    trellisAuth.AuthDeviceUserAuthoritiesReviewsDecide,
    trellisAuth.AuthDeviceUserAuthoritiesReviewsList,
  ],
}));
const reviewerUsername = caseScopedName(
  "device-activation-scoped-reviewer",
  CASE_ID,
);
const reviewerPassword =
  `trellis-integration-${CASE_ID}-reviewer-password-2026`;

liveTrellisTest({
  name:
    "device-activation.review-capability-is-deployment-scoped limits review RPCs to scoped deployment",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const { admin, deploymentId: ownDeploymentId } = await fixture
      .setupDeviceDeployment(runtime, { reviewMode: "required" });
    const { admin: otherAdmin, deploymentId: otherDeploymentId } = await fixture
      .setupDeviceDeployment(runtime, { reviewMode: "required" });
    const ownDevice = await fixture.setupProvisionedDevice(
      admin,
      ownDeploymentId,
    );
    const otherDevice = await fixture.setupProvisionedDevice(
      otherAdmin,
      otherDeploymentId,
    );
    const ownActivation = await fixture.setupActivationRequest(
      runtime,
      ownDevice.identity,
    );
    const otherActivation = await fixture.setupActivationRequest(
      runtime,
      otherDevice.identity,
    );

    const ownResolve = await admin.authDeviceUserAuthoritiesResolve({
      flowId: ownActivation.flowId,
    }).start().orThrow();
    const otherResolve = await admin.authDeviceUserAuthoritiesResolve({
      flowId: otherActivation.flowId,
    }).start().orThrow();
    const waitForReview = async (
      deploymentId: string,
      instanceId: string,
      publicIdentityKey: string,
    ) => {
      return await runtime.waitFor(async () => {
        const reviews = await admin.authDeviceUserAuthoritiesReviewsList({
          deploymentId,
          instanceId,
          state: "pending",
          limit: 20,
        }).orThrow();
        return reviews.entries.find((entry) =>
          entry.deploymentId === deploymentId &&
          entry.instanceId === instanceId &&
          entry.publicIdentityKey === publicIdentityKey
        );
      }, { timeoutMs: 10_000, intervalMs: 25 });
    };
    const ownReview = await waitForReview(
      ownDeploymentId,
      ownDevice.provisioned.instance.instanceId,
      ownDevice.identity.publicIdentityKey,
    );
    const otherReview = await waitForReview(
      otherDeploymentId,
      otherDevice.provisioned.instance.instanceId,
      otherDevice.identity.publicIdentityKey,
    );

    const adminReviews = await admin.authDeviceUserAuthoritiesReviewsList({
      state: "pending",
      limit: 20,
    }).orThrow();
    assert(
      adminReviews.entries.some((entry) =>
        entry.reviewId === ownReview.reviewId
      ),
      "admin should see own deployment review",
    );
    assert(
      adminReviews.entries.some((entry) =>
        entry.reviewId === otherReview.reviewId
      ),
      "admin should see other deployment review",
    );

    const scopedCapability = `trellis.auth::device.review.${ownDeploymentId}`;
    const reviewerUser = await admin.authUsersCreate({
      username: reviewerUsername,
      name: "Device Activation Scoped Reviewer",
      email: `${reviewerUsername}@example.test`,
      active: true,
      capabilities: [scopedCapability],
      capabilityGroups: [],
    }).orThrow();
    const reset = await admin.authUsersPasswordResetCreate({
      userId: reviewerUser.user.userId,
    }).orThrow();
    await completeLocalPasswordAccountFlow({
      trellisUrl: runtime.trellisUrl,
      flowId: reset.flowId,
      username: reviewerUsername,
      password: reviewerPassword,
    });
    const reviewerKey = await runtime.registerClient({
      name: caseScopedName("device-activation-scoped-reviewer", CASE_ID),
      contract: reviewerContract,
    });
    const reviewer = await TrellisClient.connect({
      trellisUrl: runtime.trellisUrl,
      name: caseScopedName("device-activation-scoped-reviewer", CASE_ID),
      contract: reviewerContract,
      auth: {
        mode: "session_key",
        sessionKeySeed: reviewerKey.seed,
        redirectTo: `${runtime.trellisUrl}/_trellis/test/scoped-reviewer`,
      },
      onAuthRequired: async (ctx) =>
        await completeLocalLoginFlow({
          trellisUrl: runtime.trellisUrl,
          username: reviewerUsername,
          password: reviewerPassword,
          ctx,
        }),
    }).orThrow();
    try {
      const scopedReviews = await reviewer.authDeviceUserAuthoritiesReviewsList(
        { state: "pending", limit: 20 },
      )
        .orThrow();
      assert(
        scopedReviews.entries.some((entry) =>
          entry.reviewId === ownReview.reviewId
        ),
        "scoped reviewer should see own deployment review",
      );
      assert(
        scopedReviews.entries.every((entry) =>
          entry.deploymentId !== otherDeploymentId
        ),
        "scoped reviewer should not see other deployment reviews",
      );

      const denied = await reviewer.authDeviceUserAuthoritiesReviewsDecide({
        reviewId: otherReview.reviewId,
        decision: "approve",
      });
      assert(
        denied.isErr(),
        "scoped reviewer must not decide other deployment",
      );

      const decided = await reviewer.authDeviceUserAuthoritiesReviewsDecide({
        reviewId: ownReview.reviewId,
        decision: "approve",
      }).orThrow();
      assertEquals(decided.review.state, "approved");
      assertEquals(decided.review.deploymentId, ownDeploymentId);

      const terminal = await ownResolve.wait().orThrow();
      assertEquals(terminal.state, "completed");
    } finally {
      await reviewer.connection.close().catch(() => undefined);
    }
    void otherResolve;
  },
});

async function completeLocalPasswordAccountFlow(args: {
  trellisUrl: string;
  flowId: string;
  username: string;
  password: string;
}): Promise<void> {
  const response = await fetch(
    `${args.trellisUrl}/auth/account-flow/${
      encodeURIComponent(args.flowId)
    }/local-password`,
    {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        username: args.username,
        password: args.password,
      }),
    },
  );
  const body = await response.text();
  assertEquals(response.status, 200, body);
}

async function completeLocalLoginFlow(args: {
  trellisUrl: string;
  username: string;
  password: string;
  ctx: ClientAuthRequiredContext;
}): Promise<ClientAuthContinuation> {
  const flowId = flowIdFromUrl(args.ctx.loginUrl);
  const response = await fetch(`${args.trellisUrl}/auth/login/local`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      flowId,
      username: args.username,
      password: args.password,
    }),
  });
  const body = await response.text();
  assertEquals(response.status, 200, body);
  const state = await fetchJson(
    `${args.trellisUrl}/auth/flow/${encodeURIComponent(flowId)}`,
  );
  assert(isRecord(state), "expected portal flow state response object");
  if (state.status === "approval_required") {
    const approved = await fetchJson(
      `${args.trellisUrl}/auth/flow/${encodeURIComponent(flowId)}/approval`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ approved: true }),
      },
    );
    assert(isRecord(approved), "expected portal approval response object");
    assertEquals(approved.status, "redirect", JSON.stringify(approved));
  } else {
    assertEquals(state.status, "redirect", JSON.stringify(state));
  }
  return { status: "bound", flowId };
}

async function fetchJson(url: string, init?: RequestInit): Promise<unknown> {
  const response = await fetch(url, init);
  const body = await response.text();
  assertEquals(response.status, 200, body);
  return JSON.parse(body);
}

function flowIdFromUrl(url: string): string {
  const flowId = new URL(url).searchParams.get("flowId");
  if (!flowId) throw new Error(`Trellis auth URL is missing flowId: ${url}`);
  return flowId;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
