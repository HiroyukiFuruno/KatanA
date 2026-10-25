"""Fresh isolated Terms observation and native acceptance boundary."""

import csv
import hashlib
import io
import json
from pathlib import Path
import re
import time

from docker_ui_document_fixture import open_fixture as open_native_fixture


TERMS_VERSION = "0.22.42"
_TOKEN = r"(?<![0-9A-Za-z]){value}(?![0-9A-Za-z])"
_MODAL_HEADING = re.compile(r"(?im)^\s*terms\s+of\s+use\s*$")
_VERSION_LINE = re.compile(r"(?im)^\s*version\s*:\s*0\.22\.42(?:\s|$)")


def validate_fresh_config(settings: dict, document_path: str) -> None:
    if not isinstance(settings, dict) or set(settings) != {"workspace"}:
        raise ValueError("fresh config must be isolated and Terms-free")
    workspace = settings["workspace"]
    if not isinstance(workspace, dict):
        raise ValueError("fresh config must be isolated and Terms-free")
    if set(workspace) != {
        "last_workspace", "open_tabs", "active_tab_idx", "restore_session",
        "open_workspace_in_tabs",
    }:
        raise ValueError("fresh config has unexpected workspace state")
    if workspace.get("open_tabs") != [document_path] or type(workspace.get("active_tab_idx")) is not int or workspace.get("active_tab_idx") != 0:
        raise ValueError("fresh config must activate only the requested fixture")
    if workspace.get("restore_session") is not True or workspace.get("open_workspace_in_tabs") is not True or workspace.get("last_workspace") != str(Path(document_path).parent):
        raise ValueError("fresh config workspace does not match the requested fixture")


def validate_runtime_config(settings: dict, document_path: str) -> None:
    if not isinstance(settings, dict) or settings.get("terms_accepted_version") is not None:
        raise ValueError("runtime config must not contain accepted Terms")
    workspace = settings.get("workspace")
    if not isinstance(workspace, dict) or workspace.get("last_workspace") != str(Path(document_path).parent):
        raise ValueError("runtime config workspace is not isolated")
    tabs = workspace.get("open_tabs")
    if tabs not in ([], [document_path]):
        raise ValueError("runtime config contains an unexpected fixture")
    if tabs == [] and workspace.get("active_tab_idx") is not None:
        raise ValueError("runtime config has an invalid empty-tab selection")
    if tabs == [document_path] and workspace.get("active_tab_idx") != 0:
        raise ValueError("runtime config has an invalid fixture selection")


class TermsController:
    def __init__(self, acceptance, document, prepared_settings, accept_terms):
        self.acceptance = acceptance
        self.document = document
        self.accept_terms = accept_terms
        if accept_terms and prepared_settings is not None:
            raise ValueError("native Terms acceptance requires the fresh isolated config")

    @staticmethod
    def visible_version(text: str) -> bool:
        return _VERSION_LINE.search(text) is not None

    @classmethod
    def is_terms_modal(cls, text: str) -> bool:
        return (
            _MODAL_HEADING.search(text) is not None
            and cls.visible_version(text)
            and re.search(_TOKEN.format(value="accept"), text, re.IGNORECASE) is not None
            and re.search(_TOKEN.format(value="decline"), text, re.IGNORECASE) is not None
        )

    @staticmethod
    def accept_button_position(tsv: str) -> tuple[int, int]:
        rows = list(csv.DictReader(io.StringIO(tsv), delimiter="\t"))
        matches = [row for row in rows if row.get("text", "").strip().casefold() == "accept"]
        if len(matches) != 1:
            raise AssertionError("fresh Terms modal must expose exactly one native Accept button")
        row = matches[0]
        return (int(row["left"]) + int(row["width"]) // 2,
                int(row["top"]) + int(row["height"]) // 2)

    def _runtime_settings(self, settings_path: Path) -> tuple[dict, str]:
        snapshot_path = self.acceptance.root / "fresh-config.prelaunch.json"
        if settings_path.is_symlink() or snapshot_path.is_symlink() or not snapshot_path.is_file():
            raise ValueError("fresh prelaunch settings snapshot is missing or unsafe")
        snapshot = json.loads(snapshot_path.read_text(encoding="utf-8"))
        validate_fresh_config(snapshot, str(self.document))
        if settings_path.parent != self.acceptance.root / "config":
            raise ValueError("runtime config is outside the isolated evidence root")
        settings = json.loads(settings_path.read_text(encoding="utf-8"))
        validate_runtime_config(settings, str(self.document))
        return settings, hashlib.sha256(snapshot_path.read_bytes()).hexdigest()

    def _click_native_accept(self, probe: Path) -> tuple[int, int]:
        text = self.acceptance.base.command("tesseract", str(probe), "stdout", "--psm", "6")
        if not self.visible_version(text):
            raise AssertionError("fresh Terms modal does not show fixed version 0.22.42")
        tsv = self.acceptance.base.command("tesseract", str(probe), "stdout", "--psm", "6", "tsv")
        x, y = self.accept_button_position(tsv)
        self.acceptance.base.command(
            "xdotool", "mousemove", "--window", self.acceptance.window_id,
            str(x), str(y), "click", "1",
        )
        return x, y

    def _wait_terms_saved(self, settings_path: Path, deadline: float) -> tuple[dict, str]:
        while time.monotonic() < deadline:
            after = json.loads(settings_path.read_text(encoding="utf-8"))
            version = after.get("terms_accepted_version")
            if version == TERMS_VERSION:
                return after, hashlib.sha256(settings_path.read_bytes()).hexdigest()
            if version is not None:
                raise AssertionError(f"native Terms persisted unexpected version: {version!r}")
            time.sleep(self.acceptance.base.POLL_SECONDS)
        raise AssertionError("native Terms acceptance was not persisted before deadline")

    def open_fixture(self, deadline: float) -> None:
        if not self.accept_terms:
            return
        while time.monotonic() < deadline:
            text = self.acceptance.capture("terms-dismissed")
            if not self.is_terms_modal(text) and "opened workspace" in text.casefold():
                break
            time.sleep(self.acceptance.base.POLL_SECONDS)
        else:
            raise AssertionError("fresh workspace was not positively observed after Terms")
        if hasattr(self, "_pending_acceptance"):
            after, after_sha256, before_sha256, prelaunch_sha256 = self._pending_acceptance
            self.acceptance.report["terms_accepted"] = True
            self.acceptance.record(
                "terms-accepted-native", terms_version_before=None,
                terms_version_after=after["terms_accepted_version"],
                settings_before_sha256=before_sha256,
                settings_after_sha256=after_sha256,
                prelaunch_settings_sha256=prelaunch_sha256,
                scope="fresh-config-only",
            )
            del self._pending_acceptance
        open_native_fixture(self, deadline)

    def _accept_natively(self, deadline: float) -> None:
        settings_path = self.acceptance.root / "config" / "settings.json"
        _, prelaunch_sha256 = self._runtime_settings(settings_path)
        before_sha256 = hashlib.sha256(settings_path.read_bytes()).hexdigest()
        probe = self.acceptance.last_capture
        assert probe is not None, "Terms probe capture is missing"
        x, y = self._click_native_accept(probe)
        self.acceptance.record(
            "terms-accept-native-click", button="Accept", x=x, y=y,
            scope="fresh-config-only", terms_version=TERMS_VERSION,
            prepared_config=False, existing_profile=False, native_ui=True,
        )
        after, after_sha256 = self._wait_terms_saved(settings_path, deadline)
        self._pending_acceptance = (after, after_sha256, before_sha256, prelaunch_sha256)

    def observe(self, text: str, deadline: float) -> None:
        if self.is_terms_modal(text):
            if self.accept_terms:
                self._accept_natively(deadline)
            else:
                self.acceptance.report["terms_state"] = "blocked_terms"
                self.acceptance.record("blocked_terms", reason="existing Terms modal was visible")
                self.acceptance.close_window()
                raise self.acceptance.terms_blocked_type(
                    "Terms were not accepted through the packaged UI"
                )
        elif self.accept_terms:
            raise AssertionError("--accept-terms requires a visible fresh Terms modal")
        self.acceptance.report["terms_state"] = (
            "accepted_natively" if self.accept_terms else "modal_not_observed"
        )
