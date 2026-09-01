import { deepEqual, equal } from "node:assert/strict";

import administrationParticipant from "../../../../../rust/crates/trellis/artifacts/trellis.admin.participant.json" with {
  type: "json",
};
import { nativeProtocolPresentation } from "../../../../packages/trellis/contract_support/protocol_artifacts.ts";
import contract from "../../contract.ts";
import {
  buildAdminAccountLoginUrl,
  consumeAdminAccountToken,
} from "./admin_account.ts";

declare const Deno: {
  test(name: string, fn: () => void): void;
};

Deno.test("administrator setup preserves the Console browser flow", () => {
  equal(
    buildAdminAccountLoginUrl(
      "http://localhost:3000/login?flowId=flow_console",
      "secret-token",
      new URL("http://localhost:3000/console/profile"),
    ),
    "http://localhost:3000/login/admin/bootstrap?flowId=secret-token&browserFlowId=flow_console",
  );
  equal(
    buildAdminAccountLoginUrl(
      "/login?flowId=flow_console",
      "secret-token",
      new URL("http://localhost:3000/console/profile"),
    ),
    "http://localhost:3000/login/admin/bootstrap?flowId=secret-token&browserFlowId=flow_console",
  );
  equal(
    buildAdminAccountLoginUrl(
      "/login",
      "secret-token",
      new URL("http://localhost:3000/console/profile"),
    ),
    null,
  );
});

Deno.test("administrator token is removed before Console records its return URL", () => {
  const url = new URL(
    "http://localhost:3000/console/profile?adminAccountToken=secret&tab=account",
  );
  equal(consumeAdminAccountToken(url), "secret");
  equal(
    url.toString(),
    "http://localhost:3000/console/profile?tab=account",
  );
});

Deno.test("Console is the canonical platform administration participant", () => {
  deepEqual(
    nativeProtocolPresentation(contract).participant,
    administrationParticipant,
  );
});
