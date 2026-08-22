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


# Once the test-admin participant digest is intrinsic TS work, the full API
# artifacts are no longer loaded eagerly by this module; only their digests are
# needed to refresh the checked-in participant references.
replace_once(
    "ts/packages/trellis-test/src/admin/methods.ts",
    '''import {
  API as AUTH_API,
  API_DIGEST as AUTH_API_DIGEST,
} from "@qlever-llc/trellis/sdk/auth/api";''',
    '''import { API_DIGEST as AUTH_API_DIGEST } from "@qlever-llc/trellis/sdk/auth/api";''',
)
replace_once(
    "ts/packages/trellis-test/src/admin/methods.ts",
    '''import {
  API as STATE_API,
  API_DIGEST as STATE_API_DIGEST,
} from "@qlever-llc/trellis/sdk/state/api";''',
    '''import { API_DIGEST as STATE_API_DIGEST } from "@qlever-llc/trellis/sdk/state/api";''',
)
