# Trellis Contract Helpers

TypeScript contract authoring helpers and runtime metadata types behind
`@qlever-llc/trellis/contracts`.

Provides kind-specific contract authoring helpers such as
`defineServiceContract(...)` and `defineAppContract(...)` for authoring
contracts in TypeScript and consuming generated SDK metadata. Repo-local
`trellis install` workflows, or `cargo xtask install` in this repository, build
canonical API artifacts and consumer-local SDKs from contract source.

See
[Trellis TypeScript Contract Authoring Design](../../../design/contracts/trellis-typescript-contract-authoring.md).
