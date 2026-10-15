"""List only --test executables emitted by the observed Cargo build stream."""

from pathlib import Path

from native_test_inventory import validate_ignored_subset
from native_test_inventory_collection import collect_inventory, InventoryExecutionError
from native_test_inventory_record import InventoryConditions
from native_test_run_identity import file_digest

MAX_LISTING_BYTES = 16 * 1024 * 1024


def _record(record) -> dict:
    return {"argv": list(record.argv), "raw": record.raw.decode("utf-8"),
            "raw_sha256": record.raw_sha256,
            "cases": [{"name": case.name, "kind": case.kind} for case in record.inventory.cases],
            "footer": record.inventory.footer_counts}


def list_artifacts(artifacts: list[dict], root: Path, kind: str) -> tuple[list, list]:
    records, reasons, remaining = [], [], MAX_LISTING_BYTES
    for artifact in artifacts:
        if remaining <= 0:
            reasons.append("native listing aggregate byte limit reached")
            break
        executable = Path(artifact["executable"])
        try:
            before = file_digest(executable)
            all_cases = collect_inventory((str(executable), "--list"), root, kind, InventoryConditions())
            ignored = collect_inventory((str(executable), "--list", "--ignored"), root, kind,
                                        InventoryConditions())
            validate_ignored_subset(all_cases.inventory, ignored.inventory)
            if before != file_digest(executable):
                raise ValueError("native executable changed during listing")
            remaining -= len(all_cases.raw) + len(ignored.raw)
            if remaining < 0:
                raise ValueError("native listing aggregate byte limit reached")
            records.append({"artifact": artifact, "binary_sha256_at_listing": before,
                            "all": _record(all_cases), "ignored": _record(ignored)})
        except (OSError, ValueError, InventoryExecutionError) as error:
            reasons.append(f"native listing unavailable: {error}")
    return records, sorted(set(reasons))
