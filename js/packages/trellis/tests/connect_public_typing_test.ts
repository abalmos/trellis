import { assertEquals } from "@std/assert";
import { Type } from "typebox";

import { TrellisClient } from "../index.ts";
import { defineAppContract, defineServiceContract } from "../contract.ts";
import { JobsQuery } from "../sdk/jobs.ts";
import { AuthSessionsMe } from "../sdk/auth.ts";

const selectionContract = defineServiceContract(
  {
    schemas: {
      Empty: Type.Object({}),
      Selected: Type.Object({ selected: Type.Boolean() }),
    },
  },
  (ref) => ({
    id: "trellis.connect-typing-selection@v1",
    displayName: "Connect Typing Selection Service",
    description: "Expose multiple RPCs for selected-action typing tests.",
    rpc: {
      "Selection.Selected": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("Selected"),
      },
      "Selection.Hidden": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("Selected"),
      },
    },
  }),
);

const appContract = defineAppContract(() => ({
  id: "trellis.connect-typing-app@v1",
  displayName: "Connect Typing App",
  description: "Typecheck the flat caller runtime.",
  uses: [AuthSessionsMe, JobsQuery, selectionContract.SelectionSelected],
}));

if (false) {
  const connected = await TrellisClient.connect({
    trellisUrl: "http://127.0.0.1:3000",
    contract: appContract,
    participant: {
      id: "trellis.connect-typing-app@v1",
      artifactDigest: "participant-digest",
      needsDigest: "needs-digest",
    },
  }).orThrow();

  const me = await connected.authSessionsMe({}).orThrow();
  const selected = await connected.selectionSelected({}).orThrow();
  const jobs = await connected.jobsQuery({ limit: 8 }).orThrow();
  me.session.participantKind;
  selected.selected;
  jobs.entries;

  // @ts-expect-error unselected actions are absent from the caller runtime
  connected.selectionHidden({});
  // @ts-expect-error unselected Jobs actions are absent from the caller runtime
  connected.jobsCancel({ id: "job" });
  // @ts-expect-error the raw string request escape hatch is private
  connected.request("Selection.Selected", {});
}

Deno.test("caller contracts expose direct selected descriptors", () => {
  const uses = appContract.PARTICIPANT.uses;
  if (uses === null || typeof uses !== "object" || Array.isArray(uses)) {
    throw new Error("participant uses are missing");
  }
  const required = uses.required;
  if (
    required === null || typeof required !== "object" ||
    Array.isArray(required)
  ) {
    throw new Error("participant required uses are missing");
  }
  const selection = required["trellis.connect-typing-selection@v1"];
  if (
    selection === null || typeof selection !== "object" ||
    Array.isArray(selection)
  ) {
    throw new Error("participant selection is missing");
  }
  const rpc = selection.rpc;
  if (rpc === null || typeof rpc !== "object" || Array.isArray(rpc)) {
    throw new Error("participant selection RPC is missing");
  }
  assertEquals(rpc.call, ["Selection.Selected"]);
  assertEquals(
    selectionContract.SelectionSelected.connectedName,
    "selectionSelected",
  );
  assertEquals(JobsQuery.connectedName, "jobsQuery");
});
