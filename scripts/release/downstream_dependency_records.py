"""Immutable records and strict primitive parsing for dependency snapshots."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
import tomllib


class DependencySnapshotError(ValueError):
    """A manifest or lockfile cannot satisfy the snapshot contract."""


@dataclass(frozen=True, order=True)
class DependencySource:
    kind: str
    locator: str
    selectors: tuple[tuple[str, str], ...] = ()


@dataclass(frozen=True, order=True)
class DependencyDeclaration:
    scope: str
    alias: str
    package: str
    version: str | None
    source: DependencySource | None
    features: tuple[str, ...]
    optional: bool | None
    default_features: bool | None


@dataclass(frozen=True)
class LockPackage:
    name: str
    version: str
    source: str | None
    checksum: str | None
    dependencies: tuple[str, ...]


@dataclass(frozen=True)
class DependencyChange:
    kind: str
    identity: str
    before: object | None
    after: object | None


def table(value: object, label: str) -> dict[str, object]:
    if not isinstance(value, dict) or any(not isinstance(key, str) for key in value):
        raise DependencySnapshotError(f"{label} must be a table")
    return value


def text(value: object, label: str, *, optional: bool = False) -> str | None:
    if value is None and optional:
        return None
    if not isinstance(value, str) or not value.strip():
        raise DependencySnapshotError(f"{label} must be nonempty text")
    return value


def features(value: object, label: str) -> tuple[str, ...]:
    if value is None:
        return ()
    if not isinstance(value, list) or any(not isinstance(item, str) or not item.strip() for item in value):
        raise DependencySnapshotError(f"{label} must be a string array")
    return tuple(sorted(set(value)))


def source(entry: dict[str, object], label: str) -> DependencySource:
    fields = [key for key in ("path", "git", "registry") if key in entry]
    if len(fields) > 1:
        raise DependencySnapshotError(f"{label} has multiple dependency sources")
    if not fields:
        return DependencySource("registry", "")
    kind = fields[0]
    value = text(entry[kind], f"{label}.{kind}")
    if kind == "path":
        return DependencySource("path", value)
    if kind == "registry":
        return DependencySource("registry", value)
    selector_names = [key for key in ("branch", "tag", "rev") if key in entry]
    if len(selector_names) > 1:
        raise DependencySnapshotError(f"{label} has multiple git selectors")
    selectors: tuple[tuple[str, str], ...] = ()
    if selector_names:
        selector_value = text(entry[selector_names[0]], f"{label}.{selector_names[0]}")
        selectors = ((selector_names[0], selector_value),)
    return DependencySource("git", value, selectors)


def load(content: str | bytes, label: str) -> dict[str, object]:
    if not isinstance(content, (str, bytes)):
        raise DependencySnapshotError(f"{label} must be text or bytes")
    try:
        value = tomllib.loads(content.decode("utf-8") if isinstance(content, bytes) else content)
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise DependencySnapshotError(f"{label} is malformed TOML") from error
    return table(value, label)


def changes(before: tuple[object, ...], after: tuple[object, ...],
            identity: Callable[[object], str]) -> tuple[DependencyChange, ...]:
    before_keys = [identity(value) for value in before]
    after_keys = [identity(value) for value in after]
    if len(set(before_keys)) != len(before_keys) or len(set(after_keys)) != len(after_keys):
        raise DependencySnapshotError("duplicate dependency snapshot identity")
    old = dict(zip(before_keys, before))
    new = dict(zip(after_keys, after))
    result = []
    for key in sorted(set(old) | set(new)):
        if key not in old:
            result.append(DependencyChange("added", key, None, new[key]))
        elif key not in new:
            result.append(DependencyChange("removed", key, old[key], None))
        elif old[key] != new[key]:
            result.append(DependencyChange("changed", key, old[key], new[key]))
    return tuple(result)
