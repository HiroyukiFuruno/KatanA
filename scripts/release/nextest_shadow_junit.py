"""Keep one phase-owned Nextest JUnit receipt without trusting replay output."""

import hashlib
from pathlib import Path
import re
import xml.etree.ElementTree as ElementTree

from nextest_shadow_junit_io import read_report, write_phase_config

UUID_RE = re.compile(r"[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}")
DECLARATION_RE = re.compile(r"^\s*<\?xml[^>]*encoding=[\"']([^\"']+)[\"']", re.I)


def record_run_uuid(records: list[dict]) -> str:
    """Bind the report UUID to the one record store emitted by a phase."""
    if not isinstance(records, list) or len(records) != 1:
        raise ValueError("shadow JUnit requires one Nextest record")
    path = records[0].get("path") if isinstance(records[0], dict) else None
    parts = Path(path).parts if isinstance(path, str) and not Path(path).is_absolute() else ()
    if len(parts) < 4 or parts[-4:-2] != ("records", "runs") or parts[-1] != "store.zip":
        raise ValueError("shadow JUnit record path is invalid")
    run_uuid = parts[-2]
    if not UUID_RE.fullmatch(run_uuid):
        raise ValueError("shadow JUnit record UUID is invalid")
    return run_uuid


def inspect_report(path: Path, run_uuid: str) -> dict:
    """Read a bounded official JUnit XML report and reject ambiguous outcomes."""
    if not UUID_RE.fullmatch(run_uuid):
        raise ValueError("shadow JUnit expected UUID is invalid")
    content = read_report(Path(path))
    try:
        text = content.decode("utf-8-sig")
    except UnicodeDecodeError as error:
        raise ValueError("shadow JUnit XML must be UTF-8") from error
    declaration = DECLARATION_RE.search(text)
    if (declaration and declaration.group(1).upper() != "UTF-8"
            or "<!doctype" in text.lower()):
        raise ValueError("shadow JUnit DTD is forbidden")
    try:
        root = ElementTree.fromstring(text)
    except ElementTree.ParseError as error:
        raise ValueError("shadow JUnit XML is invalid") from error
    result = _report_result(root, run_uuid)
    result.update({"path": str(path), "sha256": hashlib.sha256(content).hexdigest()})
    return result


def _report_result(root: ElementTree.Element, run_uuid: str) -> dict:
    if root.tag != "testsuites" or root.get("uuid") != run_uuid:
        raise ValueError("shadow JUnit report UUID is not bound to the recorded run")
    cases, counts = _suite_cases(root)
    _validate_root_counts(root, cases, counts)
    return {"run_uuid": run_uuid, "test_count": len(cases), "testcases": cases, **counts}


def _suite_cases(root: ElementTree.Element) -> tuple[list[dict], dict]:
    cases, identities, suites = [], set(), set()
    counts = {"skipped_count": 0, "failure_count": 0, "error_count": 0, "retry_count": 0}
    elements = list(root)
    if not elements or any(suite.tag != "testsuite" for suite in elements):
        raise ValueError("shadow JUnit suites are invalid")
    for suite in elements:
        name = suite.get("name", "")
        if not name or name in suites:
            raise ValueError("shadow JUnit suite identity is invalid")
        suites.add(name)
        suite_cases = list(suite)
        if any(case.tag != "testcase" for case in suite_cases):
            raise ValueError("shadow JUnit suite contents are invalid")
        outcomes = [_case_result(name, case, identities) for case in suite_cases]
        _validate_suite_counts(suite, outcomes)
        cases.extend(outcomes)
        for outcome in outcomes:
            key = f"{outcome['status']}_count"
            if key in counts:
                counts[key] += 1
    return cases, counts


def _case_result(suite: str, case: ElementTree.Element, identities: set) -> dict:
    identity = suite, case.get("classname", ""), case.get("name", "")
    if not all(identity) or identity in identities:
        raise ValueError("shadow JUnit testcase identity is invalid")
    identities.add(identity)
    tags = [child.tag for child in case]
    if any(tag not in {"skipped", "failure", "error", "rerunFailure", "rerunError",
                       "flakyFailure", "flakyError", "system-out", "system-err"} for tag in tags):
        raise ValueError("shadow JUnit testcase outcome is invalid")
    status = [tag for tag in tags if tag in {"skipped", "failure", "error"}]
    retries = sum(tag.startswith(("rerun", "flaky")) for tag in tags)
    if len(status) > 1 or retries:
        raise ValueError("shadow JUnit testcase outcome violates retries = 0")
    return {"suite": suite, "classname": identity[1], "name": identity[2],
            "status": status[0] if status else "passed", "retry_count": retries}


def _validate_suite_counts(suite: ElementTree.Element, cases: list[dict]) -> None:
    counts = {status: sum(case["status"] == status for case in cases)
              for status in ("skipped", "failure", "error")}
    if (_count(suite, "tests") != len(cases) or _count(suite, "skipped") != counts["skipped"]
            or _count(suite, "failures") != counts["failure"]
            or _count(suite, "errors") != counts["error"]):
        raise ValueError("shadow JUnit suite count is invalid")


def _validate_root_counts(root: ElementTree.Element, cases: list[dict], counts: dict) -> None:
    if _count(root, "tests") != len(cases):
        raise ValueError("shadow JUnit total count is invalid")
    if _count(root, "failures") != counts["failure_count"]:
        raise ValueError("shadow JUnit failure count is invalid")
    if _count(root, "errors") != counts["error_count"]:
        raise ValueError("shadow JUnit error count is invalid")
    if _count(root, "skipped") != counts["skipped_count"]:
        raise ValueError("shadow JUnit skipped count is invalid")


def _count(element: ElementTree.Element, name: str) -> int:
    value = element.get(name)
    if value is None or not value.isascii() or not value.isdigit():
        raise ValueError("shadow JUnit count is invalid")
    return int(value)
