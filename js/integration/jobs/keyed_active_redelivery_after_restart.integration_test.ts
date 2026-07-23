import { assertEquals } from "@std/assert";
import { Result } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";

import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createJobsFixture } from "./_fixture.ts";

const CASE_ID = "jobs.keyed-active-redelivery-after-restart" as const;
const fixture = createJobsFixture(CASE_ID);

liveTrellisTest({
  name:
    "jobs.keyed-active-redelivery-after-restart reacquires a released key slot",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const serviceKey = await runtime.registerService({
      name: fixture.serviceName,
      contract: fixture.serviceContract,
    });
    const connect = () =>
      TrellisService.connect({
        trellisUrl: runtime.trellisUrl,
        contract: fixture.serviceContract,
        name: fixture.serviceName,
        identity: serviceKey,
        telemetry: false,
        server: { log: false },
      }).orThrow();
    let service = await connect();
    let serviceWait: Promise<void> | undefined;
    let replacementWait: Promise<void> | undefined;

    try {
      const firstStarted = Promise.withResolvers<void>();
      service.jobs.keyedProcessDocument.handle(async ({ job }) => {
        firstStarted.resolve();
        await new Promise<void>((resolve) => {
          job.signal.addEventListener("abort", () => resolve(), { once: true });
        });
        throw new Error("worker stopped during keyed job");
      });
      serviceWait = service.wait();
      await service.jobs.keyedProcessDocument.create({
        documentId: fixture.documentId,
        groupKey: "restart",
        sequence: 1,
      }).orThrow();
      await firstStarted.promise;

      await service.stop();
      await serviceWait;
      serviceWait = undefined;

      service = await connect();
      const processed = Promise.withResolvers<number>();
      service.jobs.keyedProcessDocument.handle(({ job }) => {
        processed.resolve(job.payload.sequence);
        return Promise.resolve(Result.ok({
          documentId: job.payload.documentId,
          groupKey: job.payload.groupKey,
          sequence: job.payload.sequence,
          processedBy: "replacement-worker",
          requestId: job.context.requestId,
          traceId: job.context.traceId,
        }));
      });
      replacementWait = service.wait();

      assertEquals(
        await Promise.race([
          processed.promise,
          new Promise<never>((_, reject) =>
            setTimeout(
              () => reject(new Error("keyed job was not redelivered")),
              15_000,
            )
          ),
        ]),
        1,
      );
    } finally {
      await service.stop().catch(() => undefined);
      if (serviceWait) await serviceWait;
      if (replacementWait) await replacementWait;
    }
  },
});
