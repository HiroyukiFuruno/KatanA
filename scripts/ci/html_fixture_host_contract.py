"""原本HTMLのhost受入条件を実行前に照合する。"""

import hashlib
import json
import re
import sys
import time
from pathlib import Path
from urllib.parse import unquote, urlsplit

SOURCE_SHA256 = "c02d2d7a2420e4e15e3d98a044a310c67bc75fa858c95c4867b9c3f5d7aca012"


def remaining_nanoseconds(started: int, observed: int, budget_seconds: int) -> int:
    if started < 0 or observed < started or budget_seconds <= 0:
        raise ValueError("invalid monotonic operation clock")
    return budget_seconds * 1_000_000_000 - (observed - started)


def poll_seconds(remaining_ns: int) -> float:
    return max(0.0, min(1.0, remaining_ns / 1_000_000_000))


def operations_within_deadlines(payload: dict, log: str) -> bool:
    completions = re.findall(r"^completed step ([0-9]+)/([0-9]+) elapsed_ns=([0-9]+)$", log, re.MULTILINE)
    steps = payload["steps"]
    if len(completions) != len(steps):
        return False
    for expected_index, (index, total, elapsed_ns) in enumerate(completions, 1):
        if int(index) != expected_index or int(total) != len(steps):
            return False
        _, _, budget = observe_operation(payload, f"step {index}/{total}: {steps[expected_index - 1]['type']}")
        if int(elapsed_ns) > budget * 1_000_000_000:
            return False
    return True


def observe_operation(payload: dict, log: str) -> tuple[int, str, int]:
    markers = re.findall(r"^step ([0-9]+)/([0-9]+): ([a-z_]+)$", log, re.MULTILINE)
    if not markers:
        return (0, "startup", 60)
    index, total, label = markers[-1]
    index = int(index)
    if int(total) != len(payload["steps"]) or not 1 <= index <= len(payload["steps"]):
        raise ValueError("invalid HTML operation marker index")
    step = payload["steps"][index - 1]
    if step["type"] != label:
        raise ValueError("HTML operation marker does not match the request")
    budget = 60
    if label == "open_file":
        budget = int(step["max_first_frame_seconds"])
    elif label == "action" and "open_url" in step["action"]:
        budget = int(step["action"]["open_url"]["timeout_seconds"])
    elif label == "action" and "close_active_document" in step["action"]:
        budget = int(step["action"]["close_active_document"]["wait_seconds"])
    elif label == "action" and "close_all_documents" in step["action"]:
        budget = int(step["action"]["close_all_documents"]["wait_seconds"])
    return (index, label, budget)


def summarize_log(log: str, payload: dict | None = None) -> dict:
    frame = re.search(r"HTML browser first frame ready in ([0-9]+\.[0-9]+)s", log)
    elapsed = float(frame[1]) if frame else None
    typed_failure = bool(re.search(r"HTML browser did not produce an initial frame|opening URL .* failed", log))
    snapshot = re.search(r'^[ \t]*runtime snapshot "closed_idle": (.+)$', log, re.MULTILINE)
    resources = ("previews", "html_surfaces", "document_surfaces", "office_workers",
                 "frames", "textures", "cache_entries")
    closed_idle = bool(snapshot) and all(re.search(rf"\b{name}: 0\b", snapshot[1]) for name in resources)
    within_deadlines = payload is not None and operations_within_deadlines(payload, log)
    return {
        "first_frame_seconds": elapsed,
        "typed_failure_observed": typed_failure,
        "closed_idle_observed": closed_idle,
        "operation_deadlines_verified": within_deadlines,
        "successful_frame_and_close": elapsed is not None and elapsed <= 60 and closed_idle and within_deadlines and not typed_failure,
    }


def _validate_close(steps: list[dict]) -> None:
    closes = [(index, step["action"]["close_all_documents"])
              for index, step in enumerate(steps)
              if step["type"] == "action" and "close_all_documents" in step.get("action", {})]
    navigations = [index for index, step in enumerate(steps)
                   if step["type"] == "action" and "open_url" in step.get("action", {})]
    if len(closes) != 1 or closes[0][1] != {"wait_seconds": 5.0} or closes[0][0] <= max(navigations):
        raise ValueError("request requires a bounded 5-second resource close after navigation")
    snapshots = [index for index, step in enumerate(steps)
                 if step["type"] == "record_runtime_snapshot" and step.get("name") == "closed_idle"]
    if len(snapshots) != 1 or snapshots[0] <= closes[0][0] or steps[-1]["type"] != "quit":
        raise ValueError("request requires a closed idle snapshot before quit")


def _validate_request(source: Path, payload: dict) -> None:
    files = payload["fixture"]["workspace_files"]
    if len(files) != 1 or Path(files[0]["source"]).resolve(strict=True) != source:
        raise ValueError("request source does not match the supplied HTML")
    launches = [step for step in payload["steps"] if step["type"] == "launch"]
    if len(launches) != 1 or launches[0]["viewport"] != {"width": 1280, "height": 900}:
        raise ValueError("HTML acceptance requires a 1280x900 viewport")
    opens = [step for step in payload["steps"] if step["type"] == "open_file"]
    if len(opens) != 1 or opens[0]["file_name"] != files[0]["name"]:
        raise ValueError("request must open the supplied workspace file")
    if opens[0].get("wait_for_html_frame") is not True or opens[0].get("max_first_frame_seconds") != 60:
        raise ValueError("HTML acceptance requires the 60-second initial-frame contract")
    urls = [step["action"]["open_url"] for step in payload["steps"]
            if step["type"] == "action" and "open_url" in step.get("action", {})]
    if len(urls) != 1 or urls[0].get("timeout_seconds") != 60:
        raise ValueError("request must navigate to the supplied HTML #s15 within 60 seconds")
    url = urlsplit(urls[0]["url"])
    if url.scheme != "file" or url.netloc or url.fragment != "s15" or Path(unquote(url.path)).resolve(strict=True) != source:
        raise ValueError("request navigation does not match the supplied HTML #s15")
    _validate_close(payload["steps"])


def validate(source: Path, request: Path, expected_sha256: str) -> dict:
    source = source.resolve(strict=True)
    raw_source = source.read_bytes()
    actual_sha256 = hashlib.sha256(raw_source).hexdigest()
    if actual_sha256 != expected_sha256:
        raise ValueError("supplied HTML SHA-256 mismatch")
    raw_request = request.read_bytes()
    _validate_request(source, json.loads(raw_request))
    return {
        "mode": "in_process_host",
        "source_path": str(source),
        "source_bytes": len(raw_source),
        "source_sha256": actual_sha256,
        "request_sha256": hashlib.sha256(raw_request).hexdigest(),
        "viewport": {"width": 1280, "height": 900},
        "fragment": "s15",
    }


if __name__ == "__main__":
    try:
        if sys.argv[1] == "--monotonic-ns":
            print(time.monotonic_ns())
            sys.exit(0)
        elif sys.argv[1] == "--remaining-ns":
            print(remaining_nanoseconds(int(sys.argv[2]), time.monotonic_ns(), int(sys.argv[3])))
            sys.exit(0)
        elif sys.argv[1] == "--poll-sleep":
            remaining = remaining_nanoseconds(int(sys.argv[2]), time.monotonic_ns(), int(sys.argv[3]))
            time.sleep(poll_seconds(remaining))
            sys.exit(0)
        elif sys.argv[1] == "--observe-operation":
            operation = observe_operation(json.loads(Path(sys.argv[2]).read_text()), Path(sys.argv[3]).read_text())
            print("\t".join(str(value) for value in operation))
            sys.exit(0)
        elif sys.argv[1] == "--parse-log":
            evidence = summarize_log(Path(sys.argv[2]).read_text(encoding="utf-8"), json.loads(Path(sys.argv[3]).read_text()))
        else:
            evidence = validate(Path(sys.argv[1]), Path(sys.argv[2]), SOURCE_SHA256)
    except (OSError, ValueError, KeyError, TypeError, IndexError) as error:
        sys.exit(f"HTML host acceptance preflight failed: {error}")
    print(json.dumps(evidence, ensure_ascii=False, indent=2))
    if sys.argv[1] == "--parse-log" and not evidence["successful_frame_and_close"]:
        sys.exit("HTML host acceptance requires a successful frame and zero-resource close")
