#!/usr/bin/env python3
"""Target-scoped Docker-volume manager for KatanA Issue #376.

No prune operation exists. Unknown or unlabelled volumes are never enumerated
as deletion candidates. CI caches remain persistent unless individually named.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timedelta, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

OWNER_LABEL = "io.katana.owner"
PURPOSE_LABEL = "io.katana.purpose"
RUN_LABEL = "io.katana.run-id"
OWNER = "HiroyukiFuruno/KatanA"
RETENTION_DAYS = 30
RUN_ID_RE = re.compile(r"docker-ui-functional\.[A-Za-z0-9][A-Za-z0-9._-]{0,107}\Z")


def docker(*args: str, check: bool = True) -> str:
    result = subprocess.run(["docker", *args], capture_output=True, text=True, check=False)
    if check and result.returncode:
        raise RuntimeError(result.stderr.strip() or f"docker {' '.join(args)} failed")
    return result.stdout


def inspect_volume(name: str) -> dict:
    raw = json.loads(docker("volume", "inspect", name))
    if len(raw) != 1:
        raise RuntimeError(f"expected one volume inspection for {name}")
    return raw[0]


def volume_refs(name: str) -> list[str]:
    # WHY: 作成済み・停止中のコンテナも参照として保護する。
    return [line for line in docker("ps", "-aq", "--filter", f"volume={name}").splitlines() if line]


def list_owned_volumes() -> list[str]:
    return [line for line in docker("volume", "ls", "-q", "--filter", f"label={OWNER_LABEL}={OWNER}").splitlines() if line]


def cache_preview(volume: dict, references: list[dict] | None) -> dict:
    labels = volume.get("Labels") or {}
    usage = volume.get("UsageData")
    size_bytes = usage.get("Size") if isinstance(usage, dict) else None
    engine_ref_count = usage.get("RefCount") if isinstance(usage, dict) else None
    if not isinstance(size_bytes, int) or isinstance(size_bytes, bool) or size_bytes < 0:
        size_bytes = None
    if not isinstance(engine_ref_count, int) or isinstance(engine_ref_count, bool) or engine_ref_count < 0:
        engine_ref_count = None
    return {
        "volume": volume.get("Name"),
        "purpose": labels.get(PURPOSE_LABEL),
        "status": "persistent",
        "reason": "explicit_target_required" if labels.get(PURPOSE_LABEL) == "ci-cache" else "shared_cache_never_deleted",
        "deletion": "explicit_unreferenced_target_only" if labels.get(PURPOSE_LABEL) == "ci-cache" else "not_supported",
        "size_bytes": size_bytes,
        "size_status": "available" if size_bytes is not None else "unknown",
        "engine_reference_count": engine_ref_count,
        "referenced_containers": references,
        "reference_status": (
            "unknown" if references is None else
            "partial" if any(item.get("state") == "unknown" for item in references) else
            "known"
        ),
    }


def cache_reference_details(name: str) -> list[dict] | None:
    try:
        container_ids = volume_refs(name)
    except (RuntimeError, OSError):
        return None
    details = []
    for container_id in container_ids:
        try:
            raw = json.loads(docker("inspect", container_id))
            state = raw[0].get("State", {}).get("Status") if len(raw) == 1 else None
        except (RuntimeError, json.JSONDecodeError, IndexError, AttributeError, TypeError):
            state = None
        details.append({"id": container_id, "state": state or "unknown"})
    return details


def evidence_tree_sha256(path: Path) -> str:
    if not path.is_dir():
        raise ValueError("exported evidence directory is missing")
    digest = hashlib.sha256()
    files = sorted(path.rglob("*"), key=lambda item: item.relative_to(path).as_posix())
    for item in files:
        if item.is_symlink():
            raise ValueError("evidence export contains a symlink")
        if not item.is_file():
            continue
        relative = item.relative_to(path).as_posix().encode("utf-8")
        digest.update(len(relative).to_bytes(4, "big"))
        digest.update(relative)
        file_digest = hashlib.sha256()
        with item.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                file_digest.update(chunk)
        digest.update(file_digest.digest())
    return digest.hexdigest()


def receipt_for(root: Path, run_id: str, x11_name: str, evidence_name: str) -> tuple[bool, str, datetime | None]:
    path = root / "tmp" / run_id / "evidence-export-receipt.json"
    try:
        receipt = json.loads(path.read_text(encoding="utf-8"))
        if receipt.get("schema_version") != 1 or receipt.get("owner") != OWNER:
            return False, "invalid_export_receipt", None
        if (receipt.get("run_id") != run_id or receipt.get("x11_volume") != x11_name or
                receipt.get("evidence_volume") != evidence_name):
            return False, "receipt_volume_mismatch", None
        evidence_dir = path.parent / "evidence"
        actual = evidence_tree_sha256(evidence_dir)
        if receipt.get("evidence_tree_sha256") != actual:
            return False, "export_hash_mismatch", None
        exported = datetime.fromisoformat(receipt["exported_at"].replace("Z", "+00:00"))
        if exported.tzinfo is None:
            return False, "receipt_timestamp_missing_timezone", None
        return True, "evidence_exported", exported.astimezone(timezone.utc)
    except FileNotFoundError:
        return False, "evidence_not_exported", None
    except (AttributeError, KeyError, OSError, TypeError, ValueError, json.JSONDecodeError):
        return False, "invalid_export_receipt", None


def inspect_ui_run(root: Path, run_id: str, now: datetime) -> dict:
    if not RUN_ID_RE.fullmatch(run_id):
        return {"run_id": run_id, "status": "retained", "reason": "invalid_run_id"}
    x11_name = f"katana-{run_id}-x11"
    evidence_name = f"katana-{run_id}-evidence"
    names = (x11_name, evidence_name)
    found: dict[str, dict] = {}
    for name in names:
        result = subprocess.run(["docker", "volume", "inspect", name], capture_output=True, text=True, check=False)
        if result.returncode == 0:
            parsed = json.loads(result.stdout)
            if len(parsed) == 1:
                found[name] = parsed[0]
    if set(found) != set(names):
        return {"run_id": run_id, "status": "retained", "reason": "volume_pair_incomplete"}
    expected = {x11_name: "ui-x11", evidence_name: "ui-evidence"}
    for name, purpose in expected.items():
        labels = found[name].get("Labels") or {}
        if labels.get(OWNER_LABEL) != OWNER or labels.get(PURPOSE_LABEL) != purpose or labels.get(RUN_LABEL) != run_id:
            return {"run_id": run_id, "status": "retained", "reason": "ownership_or_purpose_label_mismatch"}
    references = {name: volume_refs(name) for name in names}
    referenced = {name: refs for name, refs in references.items() if refs}
    if referenced:
        return {"run_id": run_id, "status": "retained", "reason": "container_reference", "references": referenced}
    exported, reason, exported_at = receipt_for(root, run_id, x11_name, evidence_name)
    if not exported:
        return {"run_id": run_id, "status": "retained", "reason": reason}
    if exported_at is None or exported_at > now - timedelta(days=RETENTION_DAYS):
        return {"run_id": run_id, "status": "retained", "reason": "within_evidence_retention"}
    return {"run_id": run_id, "status": "eligible", "reason": "exported_and_retention_elapsed",
            "volumes": list(names), "exported_at": exported_at.isoformat()}


def discover_runs(root: Path, names: list[str]) -> list[str]:
    candidates: set[str] = set()
    for name in names:
        if not name.startswith("katana-"):
            continue
        for suffix in ("-x11", "-evidence"):
            if name.endswith(suffix):
                candidates.add(name[len("katana-"):-len(suffix)])
    return sorted(candidates)


def cache_candidate(name: str) -> tuple[bool, str]:
    try:
        volume = inspect_volume(name)
    except RuntimeError:
        return False, "volume_missing"
    labels = volume.get("Labels") or {}
    if labels.get(OWNER_LABEL) != OWNER or labels.get(PURPOSE_LABEL) != "ci-cache":
        return False, "ownership_or_cache_label_mismatch"
    refs = volume_refs(name)
    if refs:
        return False, "container_reference" if len(refs) == 1 else "container_references"
    return True, "explicit_unreferenced_cache_target"


def print_json(value: object) -> None:
    print(json.dumps(value, ensure_ascii=False, indent=2))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2],
                        help=argparse.SUPPRESS)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("preview", help="preview managed UI-volume retention and references")
    record = commands.add_parser("record-export", help="record a successful evidence export for one owned UI run")
    record.add_argument("run_id")
    record.add_argument("--x11-volume", required=True)
    record.add_argument("--evidence-volume", required=True)
    clean_ui = commands.add_parser("delete-ui-run", help="delete one expired, exported UI run pair")
    clean_ui.add_argument("run_id")
    clean_cache = commands.add_parser("delete-cache", help="delete one named, unreferenced CI cache")
    clean_cache.add_argument("volume")
    args = parser.parse_args(argv)
    now = datetime.now(timezone.utc)
    if args.command == "record-export":
        run_id = args.run_id
        if (not RUN_ID_RE.fullmatch(run_id) or args.x11_volume != f"katana-{run_id}-x11" or
                args.evidence_volume != f"katana-{run_id}-evidence"):
            print_json({"status": "rejected", "reason": "run_volume_name_mismatch"})
            return 2
        for name, purpose in ((args.x11_volume, "ui-x11"), (args.evidence_volume, "ui-evidence")):
            labels = inspect_volume(name).get("Labels") or {}
            if labels.get(OWNER_LABEL) != OWNER or labels.get(PURPOSE_LABEL) != purpose or labels.get(RUN_LABEL) != run_id:
                print_json({"status": "rejected", "reason": "ownership_or_purpose_label_mismatch", "volume": name})
                return 2
        run_dir = args.root.resolve() / "tmp" / run_id
        evidence_hash = evidence_tree_sha256(run_dir / "evidence")
        receipt = {"schema_version": 1, "owner": OWNER, "run_id": run_id,
                   "x11_volume": args.x11_volume, "evidence_volume": args.evidence_volume,
                   "exported_at": now.isoformat(), "evidence_tree_sha256": evidence_hash}
        path = run_dir / "evidence-export-receipt.json"
        temporary = path.with_suffix(".json.tmp")
        temporary.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        os.replace(temporary, path)
        print_json({"status": "recorded", "receipt": str(path), "evidence_tree_sha256": evidence_hash})
        return 0
    if args.command == "preview":
        names = list_owned_volumes()
        runs = [inspect_ui_run(args.root.resolve(), run_id, now) for run_id in discover_runs(args.root, names)]
        caches = []
        for name in names:
            volume = inspect_volume(name)
            purpose = (volume.get("Labels") or {}).get(PURPOSE_LABEL)
            if purpose in {"ci-cache", "shared-ci-cache"}:
                caches.append(cache_preview(volume, cache_reference_details(name)))
        print_json({"retention_days": RETENTION_DAYS, "ui_runs": runs, "ci_caches": caches,
                    "unknown_volumes": "excluded_and_preserved"})
        return 0
    if args.command == "delete-ui-run":
        state = inspect_ui_run(args.root.resolve(), args.run_id, now)
        if state["status"] != "eligible":
            print_json(state)
            return 2
        for name in state["volumes"]:
            # WHY: 削除直前にも参照を再確認し、検証の開始との競合を防ぐ。
            refs = volume_refs(name)
            if refs:
                print_json({"run_id": args.run_id, "status": "retained", "reason": "container_reference",
                            "volume": name, "references": refs})
                return 2
            docker("volume", "rm", name)
        print_json({"run_id": args.run_id, "status": "deleted", "volumes": state["volumes"]})
        return 0
    if args.command == "delete-cache":
        allowed, reason = cache_candidate(args.volume)
        if not allowed:
            print_json({"volume": args.volume, "status": "retained", "reason": reason})
            return 2
        # WHY: 対象を一つに限定し、名前の再入力でも選択間違いを防ぐ。
        typed = input(f"Delete only CI cache {args.volume!r}? Type the exact volume name: ")
        if typed != args.volume:
            print_json({"volume": args.volume, "status": "retained", "reason": "confirmation_mismatch"})
            return 2
        allowed, reason = cache_candidate(args.volume)
        if not allowed:
            print_json({"volume": args.volume, "status": "retained", "reason": reason})
            return 2
        docker("volume", "rm", args.volume)
        print_json({"volume": args.volume, "status": "deleted"})
        return 0
    return 2


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (RuntimeError, OSError, json.JSONDecodeError) as error:
        print(f"docker-volume-lifecycle: {error}", file=sys.stderr)
        raise SystemExit(1)
