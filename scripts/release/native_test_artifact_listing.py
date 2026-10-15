"""List only --test executables emitted by the observed Cargo build stream."""

import os
from pathlib import Path
import tempfile

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


def list_artifacts(artifacts: list[dict], root: Path, kind: str,
                   environment: dict[str, str] | None = None) -> tuple[list, list]:
    try:
        with tempfile.TemporaryDirectory(prefix="katana-inventory-profiles-") as directory:
            listing_environment = _listing_environment(Path(directory), root, environment)
            return _list_artifacts(artifacts, root, kind, listing_environment)
    except (OSError, ValueError) as error:
        return [], [f"native listing profile isolation unavailable: {error}"]


def _listing_environment(directory: Path, root: Path,
                         environment: dict[str, str] | None) -> dict[str, str]:
    result = dict(os.environ if environment is None else environment)
    report_roots = {root.resolve()}
    for key in ("CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "CARGO_LLVM_COV_TARGET_DIR",
                "CARGO_LLVM_COV_BUILD_DIR"):
        if value := result.get(key):
            report_roots.add((root / value).resolve())
    if value := result.get("LLVM_PROFILE_FILE"):
        report_roots.add((root / Path(value).parent).resolve())
    if any(directory.resolve().is_relative_to(path) for path in report_roots):
        raise ValueError("temporary profile directory overlaps a report or workspace root")
    # WHY: 一覧取得だけの計測値が実テストのcoverageへ混入するのを防ぐ。
    result["LLVM_PROFILE_FILE"] = str(directory / "listing-%p-%m.profraw")
    return result


def _list_artifacts(artifacts: list[dict], root: Path, kind: str,
                    environment: dict[str, str]) -> tuple[list, list]:
    records, reasons, remaining = [], [], MAX_LISTING_BYTES
    for artifact in artifacts:
        if remaining <= 0:
            reasons.append("native listing aggregate byte limit reached")
            break
        executable = Path(artifact["executable"])
        try:
            before = file_digest(executable)
            all_cases = collect_inventory((str(executable), "--list"), root, kind,
                                          InventoryConditions(), environment)
            ignored = collect_inventory((str(executable), "--list", "--ignored"), root, kind,
                                        InventoryConditions(), environment)
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
