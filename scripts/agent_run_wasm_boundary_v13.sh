#!/usr/bin/env bash
set -euo pipefail

# Keep the previously audited v3.13 validator exact, then patch only the
# packaging-phase regression exposed by attempt 3. This file is scratch-only.
BASE_VALIDATOR_COMMIT="578feac189833cfd336d9c9bb9063f28cc58ac2e"
BASE_URL="https://raw.githubusercontent.com/abalmos/trellis/${BASE_VALIDATOR_COMMIT}/scripts/agent_run_wasm_boundary_v13.sh"
curl --fail --silent --show-error --location "$BASE_URL" -o /tmp/agent-run-wasm-boundary-v13-base.sh

test -s /tmp/agent-run-wasm-boundary-v13-base.sh

python3 - <<'PY'
from pathlib import Path

path = Path("/tmp/agent-run-wasm-boundary-v13-base.sh")
text = path.read_text()


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"{label}: expected one occurrence, found {count}: {old[:120]!r}"
        )
    text = text.replace(old, new, 1)


replace_once(
    '''git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3_vector_regen.py > /tmp/vector-regen.py
python3 -m py_compile \\
''',
    '''git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3_vector_regen.py > /tmp/vector-regen.py
git show origin/"$TRANSFORM_BRANCH":scripts/agent_packaging_phase_fix.py > /tmp/package-phase.py
python3 -m py_compile \\
''',
    "load packaging transform",
)
replace_once(
    '''  /tmp/wasm-boundary-ci.py \\
  /tmp/proof-v1.py \\
  /tmp/vector-regen.py
''',
    '''  /tmp/wasm-boundary-ci.py \\
  /tmp/proof-v1.py \\
  /tmp/vector-regen.py \\
  /tmp/package-phase.py
''',
    "compile packaging transform",
)
replace_once(
    '''git commit -m 'Refresh generated participant baselines'

# Prove clean generation order: source resolution and participant emission occur
''',
    '''git commit -m 'Refresh generated participant baselines'

# Keep build-dependent package assertions out of ordinary source tests and make
# the existing packaging phase an explicit Check responsibility.
python3 /tmp/package-phase.py
deno fmt -c ts/deno.json \\
  ts/packages/result/tests/package_identity_test.ts \\
  ts/tools/package_build/result_npm_test.ts
git diff --check
git add -- \\
  .github/workflows/check.yml \\
  ts/packages/result/tests/package_identity_test.ts \\
  ts/tools/package_build/result_npm_test.ts
git commit -m 'Keep package-build tests in packaging phase'

# Prove clean generation order: source resolution and participant emission occur
''',
    "commit packaging cleanup",
)
replace_once(
    '''deno task -c ts/deno.json test:prepared
deno task -c ts/packages/trellis/deno.json build:npm
deno test -A -c ts/deno.json ts/packages/trellis/tests/npm_artifact_smoke_test.ts
git diff --exit-code
''',
    '''deno task -c ts/deno.json test:prepared
deno task -c ts/deno.json test:prepared:packaging
git diff --exit-code
''',
    "validate packaging phase",
)
replace_once(
    "git log --oneline -4\n",
    "git log --oneline -5\n",
    "final log depth",
)

path.write_text(text)
PY

bash /tmp/agent-run-wasm-boundary-v13-base.sh
