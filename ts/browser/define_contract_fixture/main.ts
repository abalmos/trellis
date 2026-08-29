import { defineAppContract } from "../../packages/trellis/contract_support/mod.ts";

const contract = defineAppContract(() => ({
  id: "trellis.test.browser-contract@v1",
  apiId: "trellis.test.browser-contract@v1",
  displayName: "Browser contract",
  description: "Defines a contract in a browser runtime.",
}));

Object.assign(globalThis, {
  contractIdentity: {
    id: contract.CONTRACT_ID,
    participantId: contract.PARTICIPANT.id,
    digest: contract.CONTRACT_DIGEST,
  },
});
