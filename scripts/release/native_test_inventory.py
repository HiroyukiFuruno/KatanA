"""Parse selection-only libtest inventory, never execution or reuse evidence."""

from dataclasses import dataclass
import re

ENTRY_RE = re.compile(r"(?P<name>.+): (?P<kind>test|benchmark)\Z")
FOOTER_RE = re.compile(r"(?P<tests>[0-9]+) tests?, (?P<benchmarks>[0-9]+) benchmarks?\Z")


class InventoryFormatError(ValueError):
    """The supplied inventory is not a consistent libtest listing."""


@dataclass(frozen=True)
class InventoryCase:
    name: str
    kind: str


@dataclass(frozen=True)
class TestInventory:
    cases: tuple[InventoryCase, ...]
    footer_counts: tuple[int, int] | None


def _footer(line: str) -> tuple[int, int] | None:
    match = FOOTER_RE.fullmatch(line)
    if match is None:
        return None
    try:
        return int(match["tests"]), int(match["benchmarks"])
    except ValueError as error:
        raise InventoryFormatError("footer counts are invalid") from error
def _entry(line: str, seen: set[str]) -> InventoryCase:
    match = ENTRY_RE.fullmatch(line)
    if match is None or not match["name"].strip():
        raise InventoryFormatError("unknown inventory line")
    name = match["name"]
    if name in seen:
        raise InventoryFormatError("duplicate or conflicting test name")
    seen.add(name)
    return InventoryCase(name, match["kind"])
def _check_counts(cases: list[InventoryCase], footer: tuple[int, int] | None,
                  require_footer: bool) -> None:
    if footer is None:
        if require_footer:
            raise InventoryFormatError("inventory footer is missing")
        return
    actual = tuple(sum(case.kind == kind for case in cases) for kind in ("test", "benchmark"))
    if actual != footer:
        raise InventoryFormatError("footer counts do not match entries")


def parse_inventory(raw: str, *, require_footer: bool = True) -> TestInventory:
    cases: list[InventoryCase] = []
    seen: set[str] = set()
    footer: tuple[int, int] | None = None
    for line in raw.splitlines():
        if not line.strip():
            continue
        if footer is not None:
            raise InventoryFormatError("content follows the inventory footer")
        counts = _footer(line)
        if counts is not None:
            footer = counts
        else:
            cases.append(_entry(line, seen))
    _check_counts(cases, footer, require_footer)
    return TestInventory(tuple(cases), footer)


def validate_ignored_subset(complete: TestInventory, ignored: TestInventory) -> None:
    all_cases = set(complete.cases)
    if not set(ignored.cases) <= all_cases:
        raise InventoryFormatError("ignored cases are not a subset")


def selected_cases(inventory: TestInventory, skips: tuple[str, ...]) -> tuple[InventoryCase, ...]:
    # skip条件と元一覧を別に保持し、選択件数を成功件数へすり替えない。
    return tuple(case for case in inventory.cases if not any(skip in case.name for skip in skips))
