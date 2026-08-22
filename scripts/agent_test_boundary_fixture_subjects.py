from pathlib import Path


def replace_exact(path: str, old: str, new: str, expected: int) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != expected:
        raise RuntimeError(f"{path}: expected {expected} occurrences of {old!r}, found {count}")
    file.write_text(text.replace(old, new))


# With runtime subject rewriting removed, fixture-authored surfaces must be
# globally distinct whenever their fixture processes may run concurrently.
# Keep this ordinary test data; do not recreate a runtime namespace resolver.
replace_exact(
    "rust/crates/trellis/tests/integration/authority_plan.rs",
    '"Value.Get"',
    '"AuthorityPlan.Value.Get"',
    9,
)
replace_exact(
    "rust/crates/trellis/tests/integration/authority_plan.rs",
    '"rpc.v1.Value.Get"',
    '"rpc.v1.AuthorityPlan.Value.Get"',
    2,
)

replace_exact(
    "rust/crates/trellis/tests/integration/prepared_events.rs",
    '"Entity.Changed"',
    '"Prepared.Entity.Changed"',
    5,
)
replace_exact(
    "rust/crates/trellis/tests/integration/prepared_events.rs",
    '"events.v1.Entity.Changed"',
    '"events.v1.Prepared.Entity.Changed"',
    1,
)
