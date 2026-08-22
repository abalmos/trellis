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
