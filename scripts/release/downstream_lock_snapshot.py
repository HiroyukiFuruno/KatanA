"""Cargo.lock dependency snapshots and semantic change classification."""

from __future__ import annotations

from downstream_dependency_records import (
    DependencyChange, DependencySnapshotError, LockPackage, load, table, text,
)


def _lock_package(value: object, index: int,
                  internal_names: frozenset[str]) -> LockPackage | None:
    record = table(value, f"lock.package[{index}]")
    name = text(record.get("name"), f"lock.package[{index}].name")
    version = text(record.get("version"), f"lock.package[{index}].version")
    source = text(record.get("source"), f"lock.package[{index}].source", optional=True)
    checksum = text(record.get("checksum"), f"lock.package[{index}].checksum", optional=True)
    dependencies = record.get("dependencies", [])
    if not isinstance(dependencies, list) or any(not isinstance(item, str) for item in dependencies):
        raise DependencySnapshotError(f"lock.package[{index}].dependencies must be a string array")
    if not name.startswith("katana-") or (name in internal_names and source is None):
        return None
    return LockPackage(name, version, source, checksum, tuple(sorted(dependencies)))


def lock_snapshot(content: str | bytes,
                  internal_names: frozenset[str]) -> tuple[LockPackage, ...]:
    document = load(content, "lockfile")
    packages = document.get("package")
    if not isinstance(packages, list):
        raise DependencySnapshotError("lock.package must be an array")
    snapshots = [_lock_package(value, index, internal_names)
                 for index, value in enumerate(packages)]
    return tuple(sorted((package for package in snapshots if package is not None),
                        key=lambda package: (package.name, package.version,
                                             package.source or "", package.checksum or "",
                                             package.dependencies)))


def classify_lock_changes(old: str | bytes, new: str | bytes,
                          internal_names: frozenset[str]) -> tuple[DependencyChange, ...]:
    before = lock_snapshot(old, internal_names)
    after = lock_snapshot(new, internal_names)
    old_map = {(item.name, item.version, item.source): item for item in before}
    new_map = {(item.name, item.version, item.source): item for item in after}
    if len(old_map) != len(before) or len(new_map) != len(after):
        raise DependencySnapshotError("duplicate lock package identity")
    old_unmatched = [item for key, item in old_map.items() if key not in new_map]
    new_unmatched = [item for key, item in new_map.items() if key not in old_map]
    result = []
    common_keys = set(old_map) & set(new_map)
    for key in sorted(common_keys, key=lambda item: (item[0], item[1],
                                                      item[2] is not None,
                                                      item[2] or "")):
        if old_map[key] != new_map[key]:
            result.append(DependencyChange("changed", old_map[key].name,
                                           old_map[key], new_map[key]))
    for name in sorted({item.name for item in old_unmatched + new_unmatched}):
        old_items = [item for item in old_unmatched if item.name == name]
        new_items = [item for item in new_unmatched if item.name == name]
        if len(old_items) == len(new_items) == 1:
            result.append(DependencyChange("changed", name, old_items[0], new_items[0]))
        else:
            result.extend(DependencyChange("removed", name, item, None) for item in old_items)
            result.extend(DependencyChange("added", name, None, item) for item in new_items)
    return tuple(result)
