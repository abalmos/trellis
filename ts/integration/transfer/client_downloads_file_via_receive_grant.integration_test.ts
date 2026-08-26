import { assertEquals, assertRejects } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import {
  createTransferFixture,
  requireReceiveTransferGrant,
} from "./_fixture.ts";

const CASE_ID = "transfer.client-downloads-file-via-receive-grant" as const;
const fixture = createTransferFixture(CASE_ID);

liveTrellisTest({
  name:
    "transfer.client-downloads-file-via-receive-grant downloads bytes through a receive grant",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await fixture.withTransferFixture(runtime, async ({ runtime: rt }) => {
      const client = await rt.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });

      const grant = requireReceiveTransferGrant(
        await client.filesDownload({
          key: fixture.downloadKey,
        }).orThrow(),
      );
      assertEquals(grant.direction, "receive");
      assertEquals(grant.info.key, fixture.downloadKey);
      assertEquals(grant.info.contentType, "text/plain");

      const downloaded = await client.transfer(grant).bytes().orThrow();
      assertEquals(
        new TextDecoder().decode(downloaded),
        `download:${fixture.downloadKey}`,
      );

      const cancelledGrant = requireReceiveTransferGrant(
        await client.filesDownload({ key: `${fixture.downloadKey}.cancel` })
          .orThrow(),
      );
      const stream = await client.transfer(cancelledGrant).stream().orThrow();
      const reader = stream.getReader();
      assertEquals((await reader.read()).done, false);
      await reader.cancel();

      const staleStream = await client.transfer(cancelledGrant).stream()
        .orThrow();
      await assertRejects(
        () => staleStream.getReader().cancel(),
        Error,
        "Transfer cancel failed",
      );
    });
  },
});
