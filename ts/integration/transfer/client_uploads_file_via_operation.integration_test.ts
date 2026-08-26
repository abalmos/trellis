import { assertEquals } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createTransferFixture } from "./_fixture.ts";

const CASE_ID = "transfer.client-uploads-file-via-operation" as const;
const storedBodies = new Map<string, Uint8Array>();
let reportTransferError: (() => void) | undefined;
const fixture = createTransferFixture(CASE_ID, {
  onStored: ({ key, body }) => {
    storedBodies.set(key, body);
  },
  onTransferError: () => reportTransferError?.(),
});

liveTrellisTest({
  name:
    "transfer.client-uploads-file-via-operation uploads bytes through a transfer operation",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await fixture.withTransferFixture(runtime, async ({ runtime: rt }) => {
      const client = await rt.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });

      const uploadBytes = new Uint8Array(3 * 262_144 + 17).map((_, i) =>
        i % 251
      );
      const upload = await client.filesUpload({
        key: fixture.uploadKey,
        contentType: "text/plain",
      }).transfer(uploadBytes).start().orThrow();
      const completed = await upload.wait().orThrow();

      assertEquals(completed.transferred.size, uploadBytes.length);
      assertEquals(completed.transferred.key, fixture.uploadKey);
      assertEquals(completed.transferred.contentType, "text/plain");
      assertEquals(completed.terminal.state, "completed");
      assertEquals(completed.terminal.output, {
        key: fixture.uploadKey,
        size: uploadBytes.length,
        contentType: "text/plain",
      });

      const control = new TextEncoder().encode('{"action":"cancel"}');
      const normal = new Uint8Array([0, 1, 2, 255]);
      const collisionCases = [
        [control],
        [control, normal],
        [normal, control, normal],
        [normal, control],
      ];
      for (const [index, chunks] of collisionCases.entries()) {
        const key = `${fixture.uploadKey}.control-data-${index}`;
        const body = {
          async *[Symbol.asyncIterator]() {
            yield* chunks;
          },
        };
        const collisionUpload = await client.filesUpload({ key }).transfer(body)
          .start()
          .orThrow();
        await collisionUpload.wait().orThrow();
        const expected = new Uint8Array(
          chunks.reduce((size, chunk) => size + chunk.length, 0),
        );
        let offset = 0;
        for (const chunk of chunks) {
          expected.set(chunk, offset);
          offset += chunk.length;
        }
        assertEquals(storedBodies.get(key), expected);
      }

      const remoteAbort = new Promise<void>((resolve) => {
        reportTransferError = resolve;
      });
      const sourceError = new Error("upload source failed");
      const failedSource = {
        async *[Symbol.asyncIterator]() {
          yield new Uint8Array([1, 2, 3]);
          throw sourceError;
        },
      };
      const failedUpload = await client.filesUpload({
        key: `${fixture.uploadKey}.source-error`,
      }).transfer(failedSource).start().orThrow();
      assertEquals((await failedUpload.wait()).isErr(), true);
      await Promise.race([
        remoteAbort,
        new Promise<never>((_, reject) =>
          setTimeout(
            () => reject(new Error("remote upload abort was not observed")),
            2_000,
          )
        ),
      ]);
    });
  },
});
