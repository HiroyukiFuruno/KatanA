"""Parse and validate the explicit Issue evidence block without live effects."""

from __future__ import annotations

import json
import re

from downstream_evidence_fields import EvidenceSchemaError, validate_evidence_document


MARKER = "<!-- katana-dependency-evidence:v1 -->"
MAX_ISSUE_BODY_CHARACTERS = 1_048_576
BLOCK_RE = re.compile(r"\A\s*```json[ \t]*\n(?P<payload>.*?)\n```[ \t]*(?:\n|\Z)", re.DOTALL)


def _duplicate_key(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise EvidenceSchemaError("duplicate JSON key")
        result[key] = value
    return result


def _json_payload(text: str) -> object:
    try:
        return json.loads(text, object_pairs_hook=_duplicate_key)
    except (json.JSONDecodeError, EvidenceSchemaError, RecursionError) as error:
        raise EvidenceSchemaError("evidence JSON is invalid") from error


def parse_issue_evidence(body: str, repository: str) -> dict[str, object] | None:
    """Return a validated record, or None for an unclassified Issue body."""
    if len(body) > MAX_ISSUE_BODY_CHARACTERS:
        raise EvidenceSchemaError("Issue body exceeds the size limit")
    body = body.replace("\r\n", "\n").replace("\r", "\n")
    count = body.count(MARKER)
    if count == 0:
        return None
    if count != 1:
        raise EvidenceSchemaError("duplicate evidence marker")
    match = BLOCK_RE.match(body.split(MARKER, maxsplit=1)[1])
    if match is None:
        raise EvidenceSchemaError("evidence marker is not followed by one json block")
    tail = body.split(MARKER, maxsplit=1)[1][match.end():]
    if "```json" in tail:
        raise EvidenceSchemaError("duplicate evidence block")
    document = _json_payload(match.group("payload"))
    validate_evidence_document(document, repository)
    return document
