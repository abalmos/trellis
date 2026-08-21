from __future__ import annotations

from collections import defaultdict
from pathlib import Path
import json
import re

ROOT = Path("rust/crates/trellis/tests/integration")
FAMILIES = {
    "rpc": "rpc",
    "operations": "operations",
    "events": "events",
    "feeds": "feed",
}
PLATFORM_APIS = {
    "Auth": "trellis.auth@v1",
    "EventLog": "trellis.eventlog@v1",
    "Health": "trellis.health@v1",
    "Jobs": "trellis.jobs@v1",
    "State": "trellis.state@v1",
    "Trellis": "trellis.core@v1",
}


def api_namespace(api_id: str) -> str:
    marker = api_id.rfind("@v")
    if marker <= 0:
        raise RuntimeError(f"invalid API id {api_id!r}")
    lineage = api_id[:marker]
    major = api_id[marker + 2 :]
    if not major.isdigit() or major.startswith("0"):
        raise RuntimeError(f"invalid API id version {api_id!r}")
    return f"api.{lineage}.v{major}"


def add(mapping: dict[str, set[str]], old: str, new: str) -> None:
    if old != new:
        mapping[old].add(new)


def add_api(mapping: dict[str, set[str]], api: dict[str, object]) -> None:
    api_id = api.get("id")
    if api.get("format") != "trellis.api.v1" or not isinstance(api_id, str):
        return
    namespace = api_namespace(api_id)
    for section, family in FAMILIES.items():
        definitions = api.get(section)
        if not isinstance(definitions, dict):
            continue
        for name, definition in definitions.items():
            if not isinstance(name, str) or not isinstance(definition, dict):
                continue
            version = definition.get("version")
            if not isinstance(version, str):
                continue
            old = f"{family}.{version}.{name}"
            new = f"{family}.{version}.{namespace}.{name}"
            add(mapping, old, new)
            if section == "events":
                params = definition.get("params")
                count = len(params) if isinstance(params, list) else 0
                if count:
                    add(mapping, old + ".*" * count, new + ".*" * count)


def raw_json_values(text: str):
    # Integration API fixtures consistently use Rust raw strings. Support any
    # number of # delimiters so this stays robust if JSON content changes.
    pattern = re.compile(r'r(?P<h>#+)"(?P<body>[\s\S]*?)"(?P=h)')
    for match in pattern.finditer(text):
        body = match.group("body")
        if '"format"' not in body or "trellis.api.v1" not in body:
            continue
        try:
            value = json.loads(body)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict):
            yield value


def builder_apis(text: str):
    # Builder-defined integration APIs are intentionally simple. Read only the
    # canonical authoring chain up to .build(); anything more exotic remains a
    # hard residual for the validation gate instead of being guessed.
    pattern = re.compile(
        r'ContractBuilder::authoring\(\s*"(?P<id>[^"]+)"(?P<body>[\s\S]*?)\.build\(\)',
    )
    for match in pattern.finditer(text):
        api_id = match.group("id")
        body = match.group("body")
        api: dict[str, object] = {"format": "trellis.api.v1", "id": api_id}
        for method, section in (
            ("rpc", "rpc"),
            ("operation", "operations"),
            ("event", "events"),
            ("feed", "feeds"),
        ):
            definitions: dict[str, object] = {}
            action = re.compile(
                rf'\.{method}\(\s*"(?P<name>[^"]+)"\s*,[\s\S]*?contracts::{method}\(\s*"(?P<version>v[1-9][0-9]*)"',
            )
            for action_match in action.finditer(body):
                definitions[action_match.group("name")] = {
                    "version": action_match.group("version")
                }
            if definitions:
                api[section] = definitions
        yield api


def platform_mappings(mapping: dict[str, set[str]], text: str) -> None:
    literal = re.compile(
        r'(?P<family>rpc|operations|events|feed)\.(?P<version>v[1-9][0-9]*)\.(?P<name>[A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z0-9_*><-]+)*)'
    )
    for match in literal.finditer(text):
        name = match.group("name")
        root = name.split(".", 1)[0]
        api_id = PLATFORM_APIS.get(root)
        if api_id is None:
            continue
        old = match.group(0)
        new = (
            f"{match.group('family')}.{match.group('version')}."
            f"{api_namespace(api_id)}.{name}"
        )
        add(mapping, old, new)


def replace_unambiguous(text: str, mapping: dict[str, set[str]], path: Path) -> str:
    for old, candidates in sorted(mapping.items(), key=lambda item: -len(item[0])):
        if old not in text:
            continue
        if len(candidates) != 1:
            raise RuntimeError(
                f"ambiguous old subject {old!r} in {path}: {sorted(candidates)}"
            )
        new = next(iter(candidates))
        text = text.replace(f'"{old}"', f'"{new}"')
    return text


def qualified(subject: str) -> bool:
    parts = subject.split(".", 2)
    if len(parts) != 3:
        return False
    remainder = parts[2]
    return remainder.startswith("api.") and re.search(
        r'\.v[1-9][0-9]*\.[A-Za-z]', remainder
    ) is not None


for path in sorted(ROOT.glob("*.rs")):
    text = path.read_text()
    mapping: dict[str, set[str]] = defaultdict(set)
    for api in raw_json_values(text):
        add_api(mapping, api)
    for api in builder_apis(text):
        add_api(mapping, api)
    platform_mappings(mapping, text)
    path.write_text(replace_unambiguous(text, mapping, path))

# Manual descriptor constants must now already be canonical. Generated SDKs are
# handled by regeneration; this gate is specifically for ad-hoc integration
# descriptors and expectations that cannot be regenerated.
residual = re.compile(
    r'const\s+(?:SUBJECT|SUBSCRIBE_SUBJECT)\s*:\s*&\'static\s+str\s*=\s*"(?P<subject>(?:rpc|operations|events|feed)\.v[1-9][0-9]*\.[^"]+)"'
)
errors: list[str] = []
for path in sorted(ROOT.glob("*.rs")):
    text = path.read_text()
    for match in residual.finditer(text):
        subject = match.group("subject")
        if not qualified(subject):
            errors.append(f"{path}: {subject}")
if errors:
    raise RuntimeError(
        "manual integration descriptor subjects remain unqualified:\n  "
        + "\n  ".join(errors)
    )
