"""Pure inventory records and condition-aware comparisons."""

from dataclasses import dataclass
import hashlib
import re

from native_test_inventory import InventoryFormatError, InventoryCase, TestInventory, parse_inventory


OID_RE = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")


@dataclass(frozen=True)
class InventoryConditions:
    source_commit: str | None = None
    os_identity: str | None = None
    target: str | None = None
    toolchain: str | None = None
    features: tuple[str, ...] = ()
    rustflags: str | None = None
    jobs: int | None = None
    test_threads: int | None = None
    worker_identity: str | None = None
    binary_sha256: str | None = None
    measured: bool = False

    def complete(self) -> bool:
        identities = (self.source_commit, self.os_identity, self.target,
                      self.toolchain, self.worker_identity)
        if not all(isinstance(value, str) and bool(value.strip()) for value in identities):
            return False
        if (not isinstance(self.source_commit, str) or not isinstance(self.binary_sha256, str)
                or OID_RE.fullmatch(self.source_commit) is None
                or SHA256_RE.fullmatch(self.binary_sha256) is None):
            return False
        if not isinstance(self.rustflags, str) or not isinstance(self.features, tuple):
            return False
        if any(not isinstance(feature, str) or not feature.strip() for feature in self.features):
            return False
        return (type(self.jobs) is int and self.jobs > 0
                and type(self.test_threads) is int and self.test_threads > 0)

    def equivalent(self, other: "InventoryConditions") -> bool:
        return (type(self.measured) is bool and type(other.measured) is bool
                and self.measured and other.measured and self.complete()
                and other.complete() and self == other)


@dataclass(frozen=True)
class InventoryRecord:
    kind: str
    raw: bytes
    raw_sha256: str
    argv: tuple[str, ...]
    inventory: TestInventory
    conditions: InventoryConditions


@dataclass(frozen=True)
class InventoryComparison:
    added: tuple[InventoryCase, ...]
    removed: tuple[InventoryCase, ...]
    kind_changed: tuple[tuple[str, str, str], ...]
    footer_changed: bool
    raw_changed: bool
    argv_changed: bool
    run_kind_changed: bool
    raw_integrity: bool
    conditions_equivalent: bool

    @property
    def equivalent(self) -> bool:
        return (not (self.added or self.removed or self.kind_changed or self.footer_changed
                     or self.raw_changed or self.argv_changed or self.run_kind_changed)
                and self.raw_integrity and self.conditions_equivalent)


def compare_inventory(left: InventoryRecord, right: InventoryRecord) -> InventoryComparison:
    left_by_name = {case.name: case for case in left.inventory.cases}
    right_by_name = {case.name: case for case in right.inventory.cases}
    added = tuple(right_by_name[name] for name in sorted(set(right_by_name) - set(left_by_name)))
    removed = tuple(left_by_name[name] for name in sorted(set(left_by_name) - set(right_by_name)))
    changed = tuple((name, left_by_name[name].kind, right_by_name[name].kind)
                    for name in sorted(set(left_by_name) & set(right_by_name))
                    if left_by_name[name].kind != right_by_name[name].kind)
    footer_changed = (left.inventory.footer_counts is None or right.inventory.footer_counts is None
                      or left.inventory.footer_counts != right.inventory.footer_counts)
    integrity = _raw_matches(left) and _raw_matches(right)
    return InventoryComparison(added, removed, changed, footer_changed,
                               left.raw_sha256 != right.raw_sha256,
                               left.argv != right.argv, left.kind != right.kind,
                               integrity, left.conditions.equivalent(right.conditions))


def _raw_matches(record: InventoryRecord) -> bool:
    if hashlib.sha256(record.raw).hexdigest() != record.raw_sha256:
        return False
    try:
        return parse_inventory(record.raw.decode("utf-8")) == record.inventory
    except (UnicodeError, InventoryFormatError):
        return False
