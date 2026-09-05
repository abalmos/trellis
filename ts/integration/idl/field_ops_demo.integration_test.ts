import { Result } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { RetryJobError } from "@qlever-llc/trellis/jobs";
import { assertEquals } from "@std/assert";

import { participant as appParticipant } from "../../../demos/app/.trellis/ts/participants/demo-app/mod.ts";
import { participant as serviceParticipant } from "../../../demos/ts/service/.trellis/ts/participants/demo-service/mod.ts";
import { participant as adminParticipant } from "../../../web/.trellis/ts/participants/app-console/mod.ts";
import { withTrellisRuntime } from "../_support/runtime.ts";

Deno.test("generated TypeScript consumers complete public workflows", async (t) => {
  await withTrellisRuntime(async (runtime) => {
    const site = {
      siteId: "site-idl",
      siteName: "IDL Site",
      openInspections: 1,
      overdueInspections: 0,
      latestStatus: "ready",
      lastReportAt: "2026-09-03T00:00:00Z",
    };
    const identity = await runtime.registerService({
      name: "generated-consumer-service",
      contract: serviceParticipant,
    });
    const service = await TrellisService.connect({
      authorizationContextEphemeral: true,
      trellisUrl: runtime.trellisUrl,
      participant: serviceParticipant,
      name: "generated-consumer-service",
      identity,
      telemetry: false,
      runtime: {},
    }).orThrow();
    let serviceExit: Promise<unknown> | undefined;
    try {
      let attempts = 0;
      service.jobs.refreshSiteSummary.handle(async ({ job }) => {
        attempts += 1;
        if (attempts === 1) return Result.err(new RetryJobError());
        return Result.ok({
          refreshId: job.ref.id,
          site,
          status: "completed",
        });
      });
      await service.handleSitesList(() =>
        Result.ok({
          entries: [site],
          count: 1,
          offset: 0,
          limit: 1,
        })
      );
      await service.handleSitesRefresh(async ({ input, op, client }) => {
        await op.started().orThrow();
        await op.progress({ stage: "queued", message: "Queued refresh" })
          .orThrow();
        const job = await client.jobs.refreshSiteSummary.create({
          siteId: input.siteId,
        }).orThrow();
        const completed = await job.wait().orThrow();
        if (completed.state !== "completed" || !completed.result) {
          throw new Error(`refresh job ended as ${completed.state}`);
        }
        const output = { refreshId: job.id, site, status: "completed" };
        const terminal = await op.complete(output).orThrow();
        await client.publishSitesRefreshed({
          refreshId: output.refreshId,
          site: output.site,
          refreshedAt: "2026-09-04T00:00:00Z",
        }).orThrow();
        return terminal;
      });
      await service.handleEvidenceDownload(async ({ input, context }) => {
        const transfer = await service.createTransfer({
          direction: "receive",
          store: "uploads",
          key: input.key,
          sessionKey: context.sessionKey,
          permission: context.permission,
          inboxPrefix: context.inboxPrefix,
          requiredCapabilities: context.requiredCapabilities,
          expiresInMs: 60_000,
        }).orThrow();
        return Result.ok({ transfer });
      });
      await service.handleAuditFeed(async ({ emit }) => {
        await emit({
          name: "Sites.Refreshed",
          event: {
            refreshId: "feed-refresh",
            refreshedAt: "2026-09-04T00:00:00Z",
            site,
          },
        }).orThrow();
      });
      serviceExit = service.wait().then(
        () => undefined,
        (error: unknown) => error,
      );
      const client = await runtime.connectClient({
        name: "generated-consumer-app",
        contract: appParticipant,
      });
      const response = await client.sitesList({ limit: 1 }).orThrow();
      assertEquals(response.entries[0]?.siteId, "site-idl");

      await t.step("job retries and completes", async () => {
        const job = await service.jobs.refreshSiteSummary.create({
          siteId: site.siteId,
        }).orThrow();
        const completed = await job.wait().orThrow();
        assertEquals(completed.state, "completed");
        assertEquals(attempts, 2);
      });

      const receivedEvent = Promise.withResolvers<string>();
      const eventSubscription = new AbortController();
      await service.onSitesRefreshed(({ event }) => {
        receivedEvent.resolve(event.site.siteId);
        return Result.ok(undefined);
      }, { signal: eventSubscription.signal }).orThrow();
      const operation = await client.sitesRefresh({ siteId: site.siteId })
        .start().orThrow();
      await t.step("operation completes after job retry", async () => {
        const terminal = await operation.wait().orThrow();
        assertEquals(terminal.state, "completed");
        assertEquals(terminal.output?.site, site);
      });
      await t.step("service receives the published event", async () => {
        assertEquals(await receivedEvent.promise, site.siteId);
      });
      assertEquals(attempts, 3);
      eventSubscription.abort();

      const put = await client.state.workspaceContext.put("workspace-1", {
        siteId: site.siteId,
        note: "generated state",
        updatedBy: "integration-test",
        updatedAt: "2026-09-04T00:00:00Z",
      });
      assertEquals(put.isOk(), true, JSON.stringify(put));
      const stored = await client.state.workspaceContext.get("workspace-1")
        .orThrow();
      if (!("found" in stored) || !stored.found) {
        throw new Error("state entry missing");
      }
      assertEquals(stored.entry.value.siteId, site.siteId);
      await t.step("generated download preserves transfer bytes", async () => {
        const bytes = new Uint8Array(131_073).map((_, index) => index % 251);
        const store = await service.store.uploads.open().orThrow();
        await store.put("integration-evidence", bytes).orThrow();
        const download = await client.evidenceDownload({
          key: "integration-evidence",
        }).orThrow();
        const metadata = Object.fromEntries(
          Object.entries(download.transfer.info.metadata).map(
            ([key, value]) => {
              if (typeof value !== "string") {
                throw new Error("invalid transfer metadata");
              }
              return [key, value];
            },
          ),
        );
        const received = await client.transfer({
          ...download.transfer,
          info: { ...download.transfer.info, metadata },
        }).bytes().orThrow();
        assertEquals(received, bytes);
      });
      await t.step("generated feed delivers its typed event", async () => {
        const abort = new AbortController();
        try {
          const feed = await client.auditFeed({}, { signal: abort.signal })
            .orThrow();
          const first = await feed[Symbol.asyncIterator]().next();
          assertEquals(first.value?.name, "Sites.Refreshed");
        } finally {
          abort.abort();
        }
      });
      await t.step(
        "revoked session is denied further RPC access",
        async (step) => {
          const admin = await runtime.connectClient({
            name: "workflow-admin",
            contract: adminParticipant,
          });
          const sessions = await admin.authSessionsList({
            participantId: appParticipant.id,
            state: "active",
          }).orThrow();
          assertEquals(sessions.entries.length, 1);
          const session = sessions.entries[0];
          const revoked = await admin.authSessionsRevoke({
            sessionId: session.sessionId,
            expectedVersion: session.version,
            idempotencyKey: crypto.randomUUID(),
            reason: "public workflow regression",
          }).orThrow();
          assertEquals(revoked.session.state, "revoked");
          const current = await admin.authSessionsList({
            participantId: appParticipant.id,
            state: "active",
          }).orThrow();
          assertEquals(current.entries.length, 0);
          await step.step("revoked caller cannot dispatch", async () => {
            assertEquals(
              (await client.sitesList({ limit: 1 }, { timeout: 1000 })).isErr(),
              true,
            );
          });
        },
      );
    } finally {
      await service.stop();
      const error = await serviceExit;
      if (error !== undefined) throw error;
    }
  });
});
