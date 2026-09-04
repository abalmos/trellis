import { isJsonValue, type JsonValue } from "./json.ts";
import type { ParticipantRuntime } from "./participant.ts";
import { PARTICIPANT_RUNTIME } from "./participant.ts";

type JsonObject = Record<string, JsonValue>;

/** Canonical protocol artifacts presented during runtime bootstrap. */
export type ParticipantPresentation = {
  api: JsonObject;
  participant: JsonObject;
  referencedApis: readonly JsonObject[];
};

/** Generated participant evidence consumed by Trellis runtimes. */
export type GeneratedParticipantEvidence = {
  readonly id: string;
  readonly digest: string;
  readonly artifact: Readonly<Record<string, unknown>>;
  readonly api: Readonly<Record<string, unknown>>;
  readonly apiDigest: string;
  readonly referencedApis: readonly Readonly<Record<string, unknown>>[];
  readonly [PARTICIPANT_RUNTIME]: ParticipantRuntime;
};

function checkedObject(value: Readonly<Record<string, unknown>>): JsonObject {
  if (!Object.values(value).every(isJsonValue)) {
    throw new Error(
      "Generated participant evidence must contain only JSON values",
    );
  }
  return value as JsonObject;
}

/** Reads the canonical evidence embedded in a generated participant module. */
export function participantPresentation(
  participant: GeneratedParticipantEvidence,
): ParticipantPresentation {
  const api = checkedObject(participant.api);
  const artifact = checkedObject(participant.artifact);
  const implementsApi = artifact.implements &&
      typeof artifact.implements === "object" &&
      !Array.isArray(artifact.implements) &&
      "self" in artifact.implements &&
      artifact.implements.self &&
      typeof artifact.implements.self === "object" &&
      !Array.isArray(artifact.implements.self)
    ? artifact.implements.self.api
    : undefined;
  if (api.id !== implementsApi || artifact.id !== participant.id) {
    throw new Error(
      "Generated participant identity does not match its artifacts",
    );
  }
  return {
    api,
    participant: artifact,
    referencedApis: participant.referencedApis.map(checkedObject),
  };
}
