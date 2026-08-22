from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(
            f"{path}: expected one occurrence, found {count}: {old[:120]!r}"
        )
    p.write_text(text.replace(old, new, 1))


# Client participant needs are contextual protocol state, not caller-authored
# identity. Resolve them inside the client runtime boundary and keep the public
# participant binding limited to intrinsic identity.
p = Path("ts/packages/trellis/client_connect.ts")
text = p.read_text()
old = '''    participant: {
      id: string;
      artifactDigest: string;
      needsDigest: string;
    };'''
new = '''    participant: {
      id: string;
      artifactDigest: string;
    };'''
if text.count(old) != 1:
    raise RuntimeError("client participant type anchor changed")
text = text.replace(old, new, 1)

old = '''  if (
    args.participant.id !== args.contract.CONTRACT_ID ||
    args.participant.artifactDigest !== args.contract.CONTRACT_DIGEST ||
    args.participant.needsDigest !== presentation.participantNeedsDigest
  ) {
    throw new Error("Client participant identity does not match its contract");
  }
  const presentation = resolveNativeProtocolPresentation(args.contract);
'''
new = '''  const presentation = resolveNativeProtocolPresentation(args.contract);
  if (
    args.participant.id !== args.contract.CONTRACT_ID ||
    args.participant.artifactDigest !== args.contract.CONTRACT_DIGEST
  ) {
    throw new Error("Client participant identity does not match its contract");
  }
'''
if text.count(old) != 1:
    raise RuntimeError("client contextual presentation anchor changed")
text = text.replace(old, new, 1)

old = "    participantNeedsDigest: args.participant.needsDigest,\n"
if text.count(old) != 1:
    raise RuntimeError("client auth request needs digest anchor changed")
text = text.replace(
    old,
    "    participantNeedsDigest: presentation.participantNeedsDigest,\n",
    1,
)
p.write_text(text)

# The intrinsic participant digest is an authoring primitive. Expose that
# existing helper on the contracts surface so the test-admin native artifact
# does not need the contextual WASM resolver just to establish its identity.
replace_once(
    "ts/packages/trellis/contract_support/mod.ts",
    'export * from "./features.ts";\n',
    'export * from "./features.ts";\nexport { participantDigest } from "./protocol_artifacts.ts";\n',
)

# Svelte app owners likewise carry only intrinsic participant identity. The
# Trellis client owns contextual resolution when the provider connects.
replace_once(
    "ts/packages/trellis-svelte/src/context.svelte.ts",
    '''  participant: {
    id: string;
    artifactDigest: string;
    needsDigest: string;
  };''',
    '''  participant: {
    id: string;
    artifactDigest: string;
  };''',
)

p = Path("ts/packages/trellis-svelte/src/context.api_check.ts")
text = p.read_text()
old = "    needsDigest: testContract.PARTICIPANT_NEEDS_DIGEST,\n"
if text.count(old) != 2:
    raise RuntimeError("svelte API check eager digest anchors changed")
text = text.replace(old, "")
p.write_text(text)

replace_once(
    "ts/apps/console/src/lib/trellis-context.svelte.ts",
    "    needsDigest: contract.PARTICIPANT_NEEDS_DIGEST,\n",
    "",
)

# Public typing fixtures describe the same intrinsic-only client binding.
replace_once(
    "ts/packages/trellis/tests/connect_public_typing_test.ts",
    '      needsDigest: "needs-digest",\n',
    "",
)

# Test runtime clients receive contextual needs from contract approval, but the
# public client connection must resolve and verify that state itself.
replace_once(
    "ts/packages/trellis-test/src/runtime.ts",
    '''      participant: {
        id: key.participantId,
        artifactDigest: key.participantArtifactDigest,
        needsDigest: key.participantNeedsDigest,
      },''',
    '''      participant: {
        id: key.participantId,
        artifactDigest: key.participantArtifactDigest,
      },''',
)

# A test contract descriptor is intrinsic contract identity, not contextual
# authority state.
replace_once(
    "ts/packages/trellis-test/src/types.ts",
    "  readonly PARTICIPANT: Readonly<Record<string, unknown>>;\n  readonly PARTICIPANT_NEEDS_DIGEST: string;\n",
    "  readonly PARTICIPANT: Readonly<Record<string, unknown>>;\n",
)

# The built-in test-admin contract is backed by a checked-in native participant
# artifact. Compute its intrinsic digest in TypeScript; contextual needs are
# resolved by TrellisClient when the admin actually connects.
p = Path("ts/packages/trellis-test/src/admin/methods.ts")
text = p.read_text()
old = '''import {
  CONTRACT_RUNTIME,
  resolveParticipantV1WasmSync,
} from "@qlever-llc/trellis/contracts";'''
new = '''import {
  CONTRACT_RUNTIME,
  participantDigest,
} from "@qlever-llc/trellis/contracts";'''
if text.count(old) != 1:
    raise RuntimeError("test admin protocol helper import anchor changed")
text = text.replace(old, new, 1)

old = '''const administrationResolution = resolveParticipantV1WasmSync({
  participant: administrationParticipant,
  apis: {
    [AUTH_API.id]: AUTH_API,
    [STATE_API.id]: STATE_API,
  },
});'''
new = '''const administrationParticipantDigest = participantDigest(
  administrationParticipant,
);'''
if text.count(old) != 1:
    raise RuntimeError("test admin contextual resolution anchor changed")
text = text.replace(old, new, 1)

old = '''    CONTRACT_ID: administrationParticipant.id,
    CONTRACT_DIGEST: administrationResolution.participantDigest,
    PARTICIPANT: administrationParticipant,
    PARTICIPANT_NEEDS_DIGEST: administrationResolution.participantNeedsDigest,
'''
new = '''    CONTRACT_ID: administrationParticipant.id,
    CONTRACT_DIGEST: administrationParticipantDigest,
    PARTICIPANT: administrationParticipant,
'''
if text.count(old) != 1:
    raise RuntimeError("test admin contract identity anchor changed")
text = text.replace(old, new, 1)

old = '''    | "CONTRACT_ID"
    | "CONTRACT_DIGEST"
    | "PARTICIPANT"
    | "PARTICIPANT_NEEDS_DIGEST"
'''
new = '''    | "CONTRACT_ID"
    | "CONTRACT_DIGEST"
    | "PARTICIPANT"
'''
if text.count(old) != 1:
    raise RuntimeError("test admin contract omit anchor changed")
text = text.replace(old, new, 1)

old = '''    readonly CONTRACT_ID: "trellis-platform-administration";
    readonly CONTRACT_DIGEST: string;
    readonly PARTICIPANT: typeof administrationParticipant;
    readonly PARTICIPANT_NEEDS_DIGEST: string;
'''
new = '''    readonly CONTRACT_ID: "trellis-platform-administration";
    readonly CONTRACT_DIGEST: string;
    readonly PARTICIPANT: typeof administrationParticipant;
'''
if text.count(old) != 1:
    raise RuntimeError("test admin contract type anchor changed")
text = text.replace(old, new, 1)

old = '''export const ADMIN_PARTICIPANT = {
  id: adminContract.CONTRACT_ID,
  artifactDigest: adminContract.CONTRACT_DIGEST,
  needsDigest: adminContract.PARTICIPANT_NEEDS_DIGEST,
} as const;'''
new = '''export const ADMIN_PARTICIPANT = {
  id: adminContract.CONTRACT_ID,
  artifactDigest: adminContract.CONTRACT_DIGEST,
} as const;'''
if text.count(old) != 1:
    raise RuntimeError("test admin participant identity anchor changed")
text = text.replace(old, new, 1)
p.write_text(text)
