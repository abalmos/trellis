import { canonicalizeJson } from "./json.ts";
import {
  type GeneratedParticipantEvidence,
  type ParticipantPresentation,
  participantPresentation,
} from "./artifacts.ts";

export type ResolvedParticipantPresentation = ParticipantPresentation & {
  participantDigest: string;
  participantNeedsDigest: string;
};

/** Resolve and validate one generated participant against its exact API evidence. */
export async function resolveParticipantPresentation(
  participant: GeneratedParticipantEvidence,
): Promise<ResolvedParticipantPresentation> {
  const { resolveParticipantV1WasmSync } = await import(
    "../auth/protocol_wasm.ts"
  );
  const intrinsic = participantPresentation(participant);
  const apis = Object.fromEntries(
    [intrinsic.api, ...intrinsic.referencedApis].map((api) => [
      String(api.id),
      api,
    ]),
  );
  const ownedApiId = String(intrinsic.api.id);
  const resolved = resolveParticipantV1WasmSync({
    participant: intrinsic.participant,
    apis,
  });
  const resolvedApi = resolved.apiArtifacts[ownedApiId];
  if (
    !resolvedApi ||
    canonicalizeJson(resolvedApi) !== canonicalizeJson(intrinsic.api)
  ) {
    throw new Error(
      "Resolved owned API does not match the generated participant API",
    );
  }
  return {
    api: resolvedApi,
    participant: resolved.participant,
    referencedApis: Object.entries(resolved.apiArtifacts)
      .filter(([id]) => id !== ownedApiId)
      .map(([, referencedApi]) => referencedApi),
    participantDigest: resolved.participantDigest,
    participantNeedsDigest: resolved.participantNeedsDigest,
  };
}
