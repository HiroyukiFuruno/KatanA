"""原本HTMLのhost受入条件を実行前に照合する。"""

import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

SOURCE_SHA256 = "c02d2d7a2420e4e15e3d98a044a310c67bc75fa858c95c4867b9c3f5d7aca012"


def summarize_log(log: str) -> dict:
    frame = re.search(r"HTML browser first frame ready in ([0-9]+\.[0-9]+)s", log)
    elapsed = float(frame[1]) if frame else None
    typed_failure = bool(re.search(r"HTML browser did not produce an initial frame|opening URL .* failed", log))
    snapshot = re.search(r'^[ \t]*runtime snapshot "closed_idle": (.+)$', log, re.MULTILINE)
    resources = ("previews", "html_surfaces", "document_surfaces", "office_workers",
                 "frames", "textures", "cache_entries")
    closed_idle = bool(snapshot) and all(re.search(rf"\b{name}: 0\b", snapshot[1]) for name in resources)
    return {
        "first_frame_seconds": elapsed,
        "typed_failure_observed": typed_failure,
        "closed_idle_observed": closed_idle,
        "successful_frame_and_close": elapsed is not None and elapsed <= 60 and closed_idle and not typed_failure,
    }


def _validate_close(steps: list[dict]) -> None:
    closes = [(index, step["action"]["close_active_document"])
              for index, step in enumerate(steps)
              if step["type"] == "action" and "close_active_document" in step.get("action", {})]
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
    urls = [step["action"]["open_url"]["url"] for step in payload["steps"]
            if step["type"] == "action" and "open_url" in step.get("action", {})]
    if len(urls) != 1:
        raise ValueError("request must navigate to the supplied HTML #s15")
    url = urlsplit(urls[0])
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
        if sys.argv[1] == "--parse-log":
            evidence = summarize_log(Path(sys.argv[2]).read_text(encoding="utf-8"))
        else:
            evidence = validate(Path(sys.argv[1]), Path(sys.argv[2]), SOURCE_SHA256)
    except (OSError, ValueError, KeyError, TypeError, IndexError) as error:
        sys.exit(f"HTML host acceptance preflight failed: {error}")
    print(json.dumps(evidence, ensure_ascii=False, indent=2))
    if sys.argv[1] == "--parse-log" and not evidence["successful_frame_and_close"]:
        sys.exit("HTML host acceptance requires a successful frame and zero-resource close")
