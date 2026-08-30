import { canonicalizeJson } from "./canonical.ts";
import {
  type NativeProtocolContract,
  type NativeProtocolPresentation,
  nativeProtocolPresentation,
} from "./protocol_artifacts.ts";

export type ResolvedNativeProtocolPresentation = NativeProtocolPresentation & {
  participantDigest: string;
  participantNeedsDigest: string;
};

/** Resolve and validate one defined contract against its exact API evidence. */
export async function resolveNativeProtocolPresentation(
  contract: NativeProtocolContract,
): Promise<ResolvedNativeProtocolPresentation> {
  const { resolveParticipantV1Wasm } = await import(
    "../auth/protocol_resolver_wasm.ts"
  );
  const intrinsic = nativeProtocolPresentation(contract);
  const apis = Object.fromEntries(
    [intrinsic.api, ...intrinsic.referencedApis].map((api) => [
      String(api.id),
      api,
    ]),
  );
  const ownedApiId = String(intrinsic.api.id);
  const resolved = await resolveParticipantV1Wasm({
    participant: intrinsic.participant,
    apis,
  });
  const resolvedApi = resolved.apiArtifacts[ownedApiId];
  if (
    !resolvedApi ||
    canonicalizeJson(resolvedApi) !== canonicalizeJson(intrinsic.api)
  ) {
    throw new Error(
      "Resolved owned API does not match the defined contract API",
    );
  }
  if (resolved.apiDigests[ownedApiId] !== contract.API_DIGEST) {
    throw new Error("Defined contract API digest does not match resolution");
  }
  if (resolved.participantDigest !== contract.CONTRACT_DIGEST) {
    throw new Error(
      "Defined contract participant digest does not match resolution",
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
