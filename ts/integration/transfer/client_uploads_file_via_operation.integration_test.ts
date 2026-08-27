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
    await fixture.withTransferFixture(
      runtime,
      async ({ runtime: rt, service }) => {
        const client = await rt.connectClient({
          name: fixture.clientName,
          contract: fixture.clientContract,
        });
        const store = await service.store.uploads.open().orThrow();
        const expectFailedUpload = async (
          key: string,
          body: AsyncIterable<Uint8Array>,
        ) => {
          const remoteFailure = new Promise<void>((resolve) => {
            reportTransferError = resolve;
          });
          const upload = await client.filesUpload({ key }).transfer(body)
            .start()
            .orThrow();
          assertEquals((await upload.wait()).isErr(), true);
          await Promise.race([
            remoteFailure,
            new Promise<never>((_, reject) =>
              setTimeout(
                () =>
                  reject(new Error("remote upload failure was not observed")),
                2_000,
              )
            ),
          ]);
        };

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
          const collisionUpload = await client.filesUpload({ key }).transfer(
            body,
          )
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

        const sourceError = new Error("upload source failed");
        const failBeforeData = {
          async *[Symbol.asyncIterator]() {
            // Let the Object Store consumer block in queue.next() before failure.
            await new Promise((resolve) => setTimeout(resolve, 100));
            throw sourceError;
          },
        };
        const emptyKey = `${fixture.uploadKey}.source-error-empty`;
        await expectFailedUpload(emptyKey, failBeforeData);
        assertEquals((await store.get(emptyKey)).isErr(), true);

        const failAfterPrefix = {
          async *[Symbol.asyncIterator]() {
            yield new Uint8Array([1, 2, 3]);
            // Exercise failure while the consumer waits for the next frame.
            await new Promise((resolve) => setTimeout(resolve, 100));
            throw sourceError;
          },
        };
        const prefixKey = `${fixture.uploadKey}.source-error-prefix`;
        await expectFailedUpload(prefixKey, failAfterPrefix);
        assertEquals((await store.get(prefixKey)).isErr(), true);

        const replacementKey = `${fixture.uploadKey}.source-error-replacement`;
        const previous = new TextEncoder().encode("previous complete object");
        await store.put(replacementKey, previous).orThrow();
        await expectFailedUpload(replacementKey, failAfterPrefix);
        assertEquals(
          await (await store.get(replacementKey).orThrow()).bytes().orThrow(),
          previous,
        );

        const oversizedKey = `${fixture.uploadKey}.oversized-prefix`;
        const oversized = {
          async *[Symbol.asyncIterator]() {
            yield new Uint8Array(262_144);
            yield new Uint8Array(1048576);
          },
        };
        await expectFailedUpload(oversizedKey, oversized);
        assertEquals((await store.get(oversizedKey)).isErr(), true);
      },
    );
  },
});
