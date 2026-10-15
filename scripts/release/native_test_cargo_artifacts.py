"""Bounded, pure recording of Cargo's native-test compiler artifacts."""

import copy
import json


MAX_LINE_BYTES = 64 * 1024
MAX_ARTIFACTS = 4096
MAX_REASONS = 128


class CargoArtifactInventory:
    """Collect only test compiler-artifact records from an existing Cargo stream."""

    def __init__(self) -> None:
        self._artifacts: list[dict[str, object]] = []
        self._events: dict[str, dict[str, str]] = {}
        self._reasons: list[str] = []
        self._finished: bool | None = None

    def _reason(self, value: str) -> None:
        if value not in self._reasons and len(self._reasons) < MAX_REASONS:
            self._reasons.append(value)

    @staticmethod
    def _is_string_list(value: object, *, nonempty: bool = False) -> bool:
        return (isinstance(value, list) and bool(value) if nonempty else isinstance(value, list)) and all(
            isinstance(item, str) and bool(item) for item in value)

    def _valid_artifact(self, event: dict[str, object]) -> bool:
        profile = event.get("profile")
        if not isinstance(profile, dict) or type(profile.get("test")) is not bool:
            self._reason("invalid_compiler_artifact_profile")
            return False
        if profile["test"] is not True:
            return False
        target = event.get("target")
        if (not isinstance(event.get("package_id"), str) or not event["package_id"]
                or not isinstance(target, dict)
                or not isinstance(target.get("name"), str) or not target["name"]
                or not self._is_string_list(target.get("kind"), nonempty=True)
                or not isinstance(target.get("src_path"), str) or not target["src_path"]
                or not self._is_string_list(target.get("crate_types"), nonempty=True)
                or not self._is_string_list(event.get("features"))
                or not isinstance(event.get("executable"), str) or not event["executable"]
                or type(event.get("fresh")) is not bool
                or not isinstance(event.get("manifest_path"), str)
                or not event["manifest_path"]
                or any(not isinstance(item, str) or not item for item in event["features"])
                or ("manifest_path" in event
                    and not isinstance(event["manifest_path"], str))
                or ("doctest" in target and type(target["doctest"]) is not bool)):
            self._reason("invalid_compiler_artifact")
            return False
        return True

    @staticmethod
    def _kept_artifact(event: dict[str, object]) -> dict[str, object]:
        target = event["target"]
        kept_target = {name: target[name] for name in
                       ("name", "kind", "src_path", "crate_types")}
        if "doctest" in target:
            kept_target["doctest"] = target["doctest"]
        kept = {"package_id": event["package_id"], "target": kept_target,
                "features": event["features"], "executable": event["executable"],
                "fresh": event["fresh"], "manifest_path": event["manifest_path"]}
        return copy.deepcopy(kept)

    def _artifact(self, event: dict[str, object]) -> None:
        if not self._valid_artifact(event):
            return
        kept = self._kept_artifact(event)
        key = (kept["package_id"], kept["target"]["name"],
               tuple(kept["target"]["kind"]))
        identity = json.dumps(kept, sort_keys=True, separators=(",", ":"))
        previous = self._events.get(str(key))
        if previous is not None:
            if previous["identity"] == identity:
                return
            self._reason("conflicting_compiler_artifact")
            return
        if len(self._artifacts) >= MAX_ARTIFACTS:
            self._reason("artifact_limit_exceeded")
            return
        self._events[str(key)] = {"identity": identity}
        self._artifacts.append(kept)

    def _finished_event(self, event: dict[str, object]) -> None:
        success = event.get("success")
        if type(success) is not bool:
            self._reason("invalid_build_finished")
            return
        if self._finished is not None:
            return
        self._finished = success

    def consume(self, line: bytes) -> None:
        """Consume one stdout line; ordinary test output is intentionally ignored."""
        if self._finished is not None:
            return
        if not isinstance(line, bytes) or len(line) > MAX_LINE_BYTES:
            self._reason("line_too_long")
            return
        try:
            text = line.decode("utf-8").strip()
        except UnicodeDecodeError:
            self._reason("invalid_cargo_json")
            return
        if not text:
            return
        try:
            event = json.loads(text)
        except (json.JSONDecodeError, RecursionError):
            if text.startswith("{") or text.startswith("["):
                self._reason("invalid_cargo_json")
            return
        if not isinstance(event, dict):
            return
        reason = event.get("reason")
        if reason == "compiler-artifact":
            self._artifact(event)
        elif reason == "build-finished":
            self._finished_event(event)

    def result(self) -> dict[str, object]:
        reasons = list(self._reasons)
        if self._finished is None:
            reasons.append("missing_build_finished")
        elif self._finished is False:
            reasons.append("build_failed")
        return {"native_artifacts_complete": not reasons,
                "reasons": reasons, "artifacts": copy.deepcopy(self._artifacts)}
