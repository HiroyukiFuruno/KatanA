#!/usr/bin/env python3
"""Pure parser for the external packaged-document Docker driver."""

import argparse
from dataclasses import dataclass
import json
from pathlib import Path
import re

from docker_ui_document_terms import validate_fresh_config


PREFIX = "[KATANA_DEBUG] event="
NAME = re.compile(r"[a-z_]+")
GENERATION = re.compile(r"[0-9]+")


@dataclass(frozen=True)
class DebugEvent:
    name: str
    fields: dict[str, str]


def parse(line: str) -> DebugEvent | None:
    if not line.startswith(PREFIX):
        return None
    remainder = line[len(PREFIX):]
    name, separator, details = remainder.partition(" ")
    if not NAME.fullmatch(name):
        return None
    fields: dict[str, str] = {}
    if separator:
        for token in details.split():
            key, equals, value = token.partition("=")
            if not equals or not key or not value or key in fields:
                return None
            fields[key] = value
    return DebugEvent(name, fields)


def matches(line: str, name: str, **expected: str) -> bool:
    event = parse(line)
    return event is not None and event.name == name and all(
        event.fields.get(key) == value for key, value in expected.items()
    )


def generation(line: str) -> str | None:
    event = parse(line)
    value = event.fields.get("generation") if event is not None else None
    return value if value is not None and GENERATION.fullmatch(value) else None


def first_after(lines: list[str], offset: int, name: str, **expected: str) -> int | None:
    for index, line in enumerate(lines[offset:], start=offset):
        if matches(line, name, **expected):
            return index
    return None


def close_sequence(lines: list[str], offset: int, generation: str, uri: str) -> tuple[int, int] | None:
    dropped = first_after(lines, offset, "document_surface_drop", generation=generation, uri=uri)
    if dropped is None:
        return None
    session = first_after(lines, dropped + 1, "document_session_closed", generation=generation, uri=uri)
    return None if session is None else (dropped, session)


def requires_texture(fmt: str, surface: str) -> bool:
    """Only page surfaces use painter_page and emit document_texture_updated."""
    return fmt != "xlsx" and surface == "Page"


def validate_prepared_config(settings: dict, document_path: str) -> str:
    """Require one active fixture tab without manufacturing Terms acceptance."""
    terms = terms_version(settings)
    workspace = settings.get("workspace")
    if not isinstance(workspace, dict):
        raise ValueError("prepared config has no workspace state")
    if workspace.get("open_tabs") != [document_path]:
        raise ValueError("prepared config must restore only the requested fixture")
    if workspace.get("active_tab_idx") != 0:
        raise ValueError("prepared config must make the requested fixture active")
    if workspace.get("restore_session") is not True:
        raise ValueError("prepared config must restore its existing session")
    if workspace.get("last_workspace") != str(Path(document_path).parent):
        raise ValueError("prepared config workspace does not match the requested fixture")
    return terms


def terms_version(settings: dict) -> str:
    terms = settings.get("terms_accepted_version")
    if not isinstance(terms, str) or not terms:
        raise ValueError("prepared config has no human-accepted Terms version")
    return terms


def write_fresh_config(settings_path: Path, workspace_path: Path, document_path: str) -> None:
    """Create the isolated workspace shape without manufacturing Terms acceptance."""
    workspace = str(Path(document_path).parent)
    settings = {
        "workspace": {
            "last_workspace": workspace,
            "open_tabs": [document_path],
            "active_tab_idx": 0,
            "restore_session": True,
            "open_workspace_in_tabs": True,
        },
    }
    workspace_state = {
        "persisted": [workspace],
        "histories": [workspace],
        "open_workspace_tabs": [workspace],
        "active_workspace": workspace,
    }
    settings_path.write_text(json.dumps(settings, indent=2) + "\n", encoding="utf-8")
    workspace_path.write_text(json.dumps(workspace_state, indent=2) + "\n", encoding="utf-8")


def read_resource_receipt(path: Path, run_id: str, pid: int) -> dict:
    if not path.is_file():
        raise ValueError(f"resource receipt is missing: {path}")
    try:
        receipt = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"resource receipt is unreadable: {error}") from error
    if not isinstance(receipt, dict):
        raise ValueError("resource receipt is not an object")
    if receipt.get("schema") != "katana-completed-frame-resource-receipt-v1":
        raise ValueError("resource receipt schema mismatch")
    if receipt.get("run_id") != run_id or receipt.get("pid") != pid:
        raise ValueError("resource receipt run or pid mismatch")
    sequence = receipt.get("seq")
    if type(sequence) is not int or sequence < 1:
        raise ValueError("resource receipt sequence is invalid")
    return receipt


def receipt_has_document(receipt: dict, uri: str, generation: int) -> bool:
    surfaces = receipt.get("document_surfaces")
    if not isinstance(surfaces, list):
        raise ValueError("resource receipt document surfaces are missing")
    return any(
        isinstance(surface, dict)
        and surface.get("uri") == uri
        and surface.get("generation") == generation
        for surface in surfaces
    )


def receipt_resources_are_zero(receipt: dict) -> bool:
    host = receipt.get("host")
    kdv = receipt.get("kdv")
    surfaces = receipt.get("document_surfaces")
    if not isinstance(host, dict) or not isinstance(kdv, dict) or surfaces != []:
        return False
    counts = [host.get(key) for key in (
        "previews", "html_surfaces", "document_surfaces", "frames", "textures", "cache_entries",
    )]
    kdv_counts = [kdv.get(key) for key in (
        "live_document_sessions", "live_spreadsheet_workers", "live_worker_workspaces",
        "retained_artifact_bytes", "cached_page_count", "cached_page_bytes",
        "cached_spreadsheet_cell_count", "cached_spreadsheet_cell_bytes",
    )]
    return type(receipt.get("document_workers_live")) is int and receipt.get("document_workers_live") == 0 and all(
        type(value) is int and value == 0 for value in counts + kdv_counts
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--validate-prepared-config", type=Path)
    group.add_argument("--validate-fresh-config", type=Path)
    group.add_argument("--read-terms-version", type=Path)
    group.add_argument("--write-fresh-config", nargs=2, type=Path, metavar=("SETTINGS", "WORKSPACE"))
    parser.add_argument("--document", type=str)
    options = parser.parse_args()
    if options.write_fresh_config is not None:
        if options.document is None:
            parser.error("--document is required with --write-fresh-config")
        write_fresh_config(*options.write_fresh_config, options.document)
    elif options.validate_fresh_config is not None:
        if options.document is None:
            parser.error("--document is required with --validate-fresh-config")
        settings = json.loads(options.validate_fresh_config.read_text(encoding="utf-8"))
        validate_fresh_config(settings, options.document)
    elif options.validate_prepared_config is not None:
        if options.document is None:
            parser.error("--document is required with --validate-prepared-config")
        settings = json.loads(options.validate_prepared_config.read_text(encoding="utf-8"))
        print(validate_prepared_config(settings, options.document))
    else:
        settings = json.loads(options.read_terms_version.read_text(encoding="utf-8"))
        print(terms_version(settings))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
