/**
 * O09 component proof at the production operation-observe authorization
 * boundary.
 *
 * Observation and control authority is scoped to the exact creating principal
 * and participant. A foreign principal, or the same principal on a different
 * participant, is denied without the operation being disclosed.
 */

import { assertEquals } from "@std/assert";

import { operationObserveAuthorized } from "./core.ts";

const OPERATION = {
  creatorPrincipalId: "principal-a",
  creatorParticipantId: "runtime-trellis.OperationCaller",
};

Deno.test("O09 the creating principal and participant are authorized", () => {
  assertEquals(
    operationObserveAuthorized(OPERATION, {
      principalId: "principal-a",
      participantId: "runtime-trellis.OperationCaller",
    }),
    true,
  );
});

Deno.test("O09 a foreign principal is denied", () => {
  assertEquals(
    operationObserveAuthorized(OPERATION, {
      principalId: "principal-b",
      participantId: "runtime-trellis.OperationCaller",
    }),
    false,
  );
});

Deno.test("O09 the same principal on a different participant is denied", () => {
  assertEquals(
    operationObserveAuthorized(OPERATION, {
      principalId: "principal-a",
      participantId: "runtime-trellis.OperationProvider",
    }),
    false,
  );
});

Deno.test("O09 internal callers without a verified caller are authorized", () => {
  assertEquals(operationObserveAuthorized(OPERATION, undefined), true);
});
