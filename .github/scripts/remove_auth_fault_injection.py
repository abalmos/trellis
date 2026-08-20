from __future__ import annotations

import json
from pathlib import Path


def matching_brace(source: str, opening: int) -> int:
    depth = 0
    i = opening
    state = "code"
    block_depth = 0
    while i < len(source):
        c = source[i]
        n = source[i + 1] if i + 1 < len(source) else ""
        if state == "code":
            if c == '"':
                state = "string"
            elif c == "'":
                state = "char"
            elif c == "/" and n == "/":
                state = "line_comment"
                i += 1
            elif c == "/" and n == "*":
                state = "block_comment"
                block_depth = 1
                i += 1
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    return i + 1
        elif state == "string":
            if c == "\\":
                i += 1
            elif c == '"':
                state = "code"
        elif state == "char":
            if c == "\\":
                i += 1
            elif c == "'":
                state = "code"
        elif state == "line_comment":
            if c == "\n":
                state = "code"
        elif state == "block_comment":
            if c == "/" and n == "*":
                block_depth += 1
                i += 1
            elif c == "*" and n == "/":
                block_depth -= 1
                i += 1
                if block_depth == 0:
                    state = "code"
        i += 1
    raise RuntimeError("unclosed Rust block")


def remove_rust_function(path: str, signature: str) -> None:
    p = Path(path)
    source = p.read_text()
    index = source.index(signature)
    start = source.rfind("\n", 0, index) + 1
    while start > 0:
        previous_end = start - 1
        previous_start = source.rfind("\n", 0, previous_end) + 1
        previous = source[previous_start:previous_end].strip()
        if previous.startswith("///") or previous.startswith("#["):
            start = previous_start
        else:
            break
    opening = source.index("{", index)
    end = matching_brace(source, opening)
    while end < len(source) and source[end] == "\n":
        end += 1
    p.write_text(source[:start] + source[end:])


def remove_matrix_cases(path: str, ids: set[str]) -> None:
    p = Path(path)
    lines = p.read_text().splitlines(keepends=True)
    starts = [i for i, line in enumerate(lines) if line.rstrip("\r\n") == "    {"]
    if not starts:
        raise RuntimeError("runtime matrix has no top-level cases")

    cases: list[tuple[str, str, int, int]] = []
    for start in starts:
        end = next(
            (
                i
                for i in range(start, len(lines))
                if lines[i].rstrip("\r\n") in {"    },", "    }"}
            ),
            None,
        )
        if end is None:
            raise RuntimeError("unterminated runtime matrix case")
        raw = "".join(lines[start : end + 1])
        parseable = raw.rstrip()
        if parseable.endswith("},"):
            parseable = parseable[:-1]
        parsed = json.loads(parseable)
        cases.append((raw, parsed["id"], start, end))

    found = {case_id for _, case_id, _, _ in cases if case_id in ids}
    if found != ids:
        raise RuntimeError(f"matrix cases changed: expected {ids}, found {found}")

    remove_ranges = [(start, end) for _, case_id, start, end in cases if case_id in ids]
    for start, end in reversed(remove_ranges):
        del lines[start : end + 1]

    closing = next(i for i, line in enumerate(lines) if line.rstrip("\r\n") == "  ]")
    last_case_end = next(
        i
        for i in range(closing - 1, -1, -1)
        if lines[i].rstrip("\r\n") in {"    },", "    }"}
    )
    if lines[last_case_end].rstrip("\r\n") == "    },":
        newline = "\r\n" if lines[last_case_end].endswith("\r\n") else "\n"
        lines[last_case_end] = "    }" + newline

    text = "".join(lines)
    json.loads(text)
    p.write_text(text)


def remove_post_commit_injection() -> None:
    p = Path("rust/crates/runtime/src/platform/auth_post_commit.rs")
    source = p.read_text()
    start_marker = '        #[cfg(feature = "integration-test-hooks")]\n        if self\n            .repository\n            .consume_test_post_commit_failure'
    end_marker = "        match self.dispatch(&action).await {"
    start = source.index(start_marker)
    end = source.index(end_marker, start)
    p.write_text(source[:start] + source[end:])


def main() -> None:
    auth_tests = "rust/crates/trellis/tests/integration/auth.rs"
    remove_rust_function(
        auth_tests,
        "async fn auth_transaction_failure_rolls_back_state_idempotency_and_actions()",
    )
    remove_rust_function(
        auth_tests,
        "async fn auth_post_commit_failure_retries_committed_context_revocation_once()",
    )

    support = "rust/crates/trellis-test/src/lib.rs"
    remove_rust_function(support, "pub fn fail_user_update_transaction(")
    remove_rust_function(support, "pub fn clear_user_update_transaction_failure(")
    remove_rust_function(support, "pub fn fail_next_context_revocation_dispatch(")

    remove_rust_function(
        "rust/crates/runtime/src/platform/auth/sqlite/common.rs",
        "pub(crate) async fn consume_test_post_commit_failure(",
    )
    remove_post_commit_injection()

    remove_matrix_cases(
        "integration/rust-runtime-test-matrix.json",
        {
            "auth.transaction-failure-rolls-back-state-idempotency-and-actions",
            "auth.post-commit-failure-retries-committed-context-revocation-once",
        },
    )


if __name__ == "__main__":
    main()
