import matrix from "../../../integration/client-test-matrix.json" with {
  type: "json",
};

export type JsIntegrationCase = {
  readonly id: string;
  readonly file: string;
  readonly testName: string;
  readonly runtime: "live-trellis";
};

/** TypeScript cases implemented by the client interoperability matrix. */
export const jsIntegrationCases: readonly JsIntegrationCase[] = matrix.cases
  .filter((entry) => entry.completion.typescript === "implemented")
  .map((entry) => ({
    id: entry.id,
    file: entry.implementations.typescript.file,
    testName: entry.implementations.typescript.testName,
    runtime: "live-trellis",
  }));

/** Returns all TypeScript cases for one fixture. */
export function jsCasesForFixture(
  fixture: string,
): readonly JsIntegrationCase[] {
  return jsIntegrationCases.filter((entry) =>
    entry.id.startsWith(`${fixture}.`)
  );
}

/** Returns the TypeScript implementation for a matrix case ID. */
export function jsCaseById(id: string): JsIntegrationCase | undefined {
  return jsIntegrationCases.find((entry) => entry.id === id);
}
