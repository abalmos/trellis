import { assertEquals, assertThrows } from "@std/assert";
import {
  selectTypeScriptCases,
  validateLanguageSelectors,
} from "./live_runner.ts";

Deno.test("live runner rejects conflicting language selectors", () => {
  assertThrows(
    () => validateLanguageSelectors(true, true),
    Error,
    "mutually exclusive",
  );
  validateLanguageSelectors(true, false);
  validateLanguageSelectors(false, true);
});

Deno.test("live runner rejects empty TypeScript parent filters", () => {
  const cases = [
    { id: "rpc.one", completion: { typescript: "implemented" } },
    { id: "rpc.pending", completion: { typescript: "planned" } },
  ];
  assertEquals(selectTypeScriptCases(cases, "rpc.one"), [cases[0]]);
  assertThrows(
    () => selectTypeScriptCases(cases, "rpc.unknown"),
    Error,
    "does not name an implemented TypeScript case",
  );
  assertThrows(
    () => selectTypeScriptCases(cases, undefined, "feeds."),
    Error,
    "selects no implemented TypeScript cases",
  );
});
