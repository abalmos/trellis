from pathlib import Path

p = Path("ts/packages/trellis/auth/authorization/provider_cache.ts")
text = p.read_text()
text = text.replace(
    "const result = verifier.verifyRequest(this.#requestInput(request));",
    "const result = verifier.verifyRequest(this.#requestInput(entry, request));",
)
text = text.replace(
    "this.#eventInput(event, revokedAt ?? null),",
    "this.#eventInput(entry, event, revokedAt ?? null),",
)
text = text.replace(
    "  #requestInput(\n    request: AuthorizationProviderRequest,",
    "  #requestInput(\n    entry: ProviderContextEntry,\n    request: AuthorizationProviderRequest,",
)
text = text.replace(
    "      policy: this.#policyForDigest(request.contextDigest, this.#now()),",
    "      policy: this.#policyFor(entry, this.#now()),",
)
text = text.replace(
    "  #eventInput(\n    event: AuthorizationProviderEvent,",
    "  #eventInput(\n    entry: ProviderContextEntry,\n    event: AuthorizationProviderEvent,",
)
text = text.replace(
    "      policy: this.#policyForDigest(event.contextDigest, this.#now(), true),",
    "      policy: this.#policyFor(entry, this.#now(), true),",
)
start = text.find("  #policyForDigest(\n")
end = text.find("  #policyFor(\n", start)
if start < 0 or end < 0:
    raise RuntimeError("generated policyForDigest block not found")
text = text[:start] + text[end:]
p.write_text(text)
