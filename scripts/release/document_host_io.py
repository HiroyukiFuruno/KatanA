"""公開可能なhost証跡の型と参照を限定し、private原本の存在をCI成功へ読み替えない。"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import math
import re
from datetime import datetime
from pathlib import Path


class HostEvidenceError(ValueError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise HostEvidenceError(message)


def fields(value: object, expected: set[str], label: str) -> dict:
    require(isinstance(value, dict) and set(value) == expected, f"{label}: missing or unknown fields")
    return value


def integer(value: object, label: str, *, minimum: int = 0) -> int:
    require(type(value) is int and value >= minimum, f"{label}: invalid integer")
    return value


def number(value: object, label: str) -> float:
    require(type(value) in (int, float) and math.isfinite(value) and value >= 0, f"{label}: invalid number")
    return float(value)


def sha(value: object, label: str) -> str:
    require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None, f"{label}: invalid SHA256")
    return value


def moment(value: object, label: str) -> datetime:
    require(isinstance(value, str), f"{label}: timestamp required")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise HostEvidenceError(f"{label}: invalid timestamp") from error
    require(parsed.tzinfo is not None, f"{label}: timezone required")
    return parsed


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _pairs(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON field")
        result[key] = value
    return result


def decode(raw: bytes) -> object:
    try:
        return json.loads(raw, object_pairs_hook=_pairs,
                          parse_constant=lambda value: (_ for _ in ()).throw(HostEvidenceError("nonfinite JSON constant")))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise HostEvidenceError("invalid JSON evidence") from error


def read(path: Path, *, maximum: int = 64 * 1024 * 1024) -> bytes:
    require(not path.is_symlink() and path.is_file(), "regular evidence file required")
    require(0 < path.stat().st_size <= maximum, "empty or oversized evidence")
    with path.open("rb") as handle:
        raw = handle.read(maximum + 1)
    require(len(raw) <= maximum, "evidence grew beyond limit")
    return raw


def reference(root: Path, value: object, label: str) -> tuple[bytes, object]:
    ref = fields(value, {"path", "sha256"}, label)
    relative = ref["path"]
    require(isinstance(relative, str) and "\\" not in relative and ":" not in relative,
            f"{label}: repository relative public path required")
    path = Path(relative)
    require(not path.is_absolute() and path.parts and all(part not in {".", "..", "tmp"} for part in path.parts),
            f"{label}: private/traversing path forbidden")
    require(path.parts[0] == "openspec" and "evidence" in path.parts, f"{label}: public evidence path required")
    current = root
    for component in path.parts:
        current = current / component
        require(not current.is_symlink(), f"{label}: symlink reference forbidden")
    require(current.resolve().is_relative_to(root.resolve()), f"{label}: reference escapes root")
    raw = read(current)
    require(digest(raw) == sha(ref["sha256"], label), f"{label}: stale artifact digest")
    return raw, decode(raw)


def legacy():
    script = Path(__file__).with_name("check-document-fidelity-acceptance-evidence.py")
    spec = importlib.util.spec_from_file_location("_strict_document_evidence_source_helpers", script)
    require(spec is not None and spec.loader is not None, "strict source helper unavailable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_new(path: Path, value: object) -> dict:
    raw = (json.dumps(value, ensure_ascii=True, indent=2, allow_nan=False) + "\n").encode()
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as handle:
        handle.write(raw)
    return {"path": path.as_posix(), "sha256": digest(raw)}
