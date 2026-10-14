"""Public facade for pure downstream dependency classification."""

from downstream_dependency_records import (
    DependencyChange, DependencyDeclaration, DependencySnapshotError, DependencySource, LockPackage,
)
from downstream_lock_snapshot import classify_lock_changes, lock_snapshot
from downstream_manifest_snapshot import classify_manifest_changes, manifest_snapshot

__all__ = [
    "DependencyChange", "DependencyDeclaration", "DependencySnapshotError", "DependencySource", "LockPackage",
    "classify_lock_changes", "classify_manifest_changes", "lock_snapshot", "manifest_snapshot",
]
