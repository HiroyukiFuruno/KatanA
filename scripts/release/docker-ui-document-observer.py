"""Shared timing and resource-receipt checks for the packaged document driver."""

import time
from pathlib import Path


class ReceiptVerifier:
    def __init__(self, path: Path, run_id: str, pid: int, parser):
        self.path = path / "latest.json"
        self.run_id = run_id
        self.pid = pid
        self.parser = parser
        self.seq = 0

    def baseline(self) -> int:
        if not self.path.exists():
            return 0
        receipt = self.parser.read_resource_receipt(self.path, self.run_id, self.pid)
        self.seq = receipt["seq"]
        return self.seq

    def wait(self, label: str, deadline: float, predicate, require_new: bool = True):
        while time.monotonic() < deadline:
            try:
                receipt = self.parser.read_resource_receipt(self.path, self.run_id, self.pid)
            except ValueError as error:
                if "missing" not in str(error):
                    raise
                receipt = None
            if receipt is not None:
                sequence = receipt["seq"]
                if sequence < self.seq:
                    raise AssertionError("resource receipt sequence rolled back")
                is_new = sequence > self.seq
                self.seq = max(self.seq, sequence)
                if predicate(receipt) and (is_new or not require_new):
                    return receipt
            if time.monotonic() >= deadline:
                break
            time.sleep(0.1)
        raise AssertionError(f"Timed out waiting for {label} resource receipt")


def observe_terms(owner, name: str):
    text = owner.capture(name)
    if "terms" in text.lower():
        owner.report["terms_state"] = "blocked_terms"
        owner.record("blocked_terms", reason="Terms modal appeared during packaged observation")
        owner.close_window()
        raise owner.terms_blocked_type("Terms were not accepted through the packaged UI")


def wait_for_debug(owner, label: str, offset: int, name: str, deadline: float,
                   guard_terms: bool = True, **fields: str):
    next_terms_probe = time.monotonic()
    while time.monotonic() < deadline:
        if guard_terms and time.monotonic() >= next_terms_probe:
            observe_terms(owner, f"terms-guard-{name}")
            next_terms_probe = time.monotonic() + 1
        if time.monotonic() >= deadline:
            break
        lines = owner.debug_lines()
        index = owner.parser.first_after(lines, offset, name, **fields)
        if index is not None and time.monotonic() < deadline:
            return index, lines[index]
        time.sleep(owner.base.POLL_SECONDS)
    raise AssertionError(f"Timed out waiting for {label}")


def require_before_deadline(label: str, deadline: float):
    if time.monotonic() >= deadline:
        raise AssertionError(f"{label} exceeded its observation deadline")


def wait_for_heartbeat(owner, frame: int, deadline: float):
    while time.monotonic() < deadline:
        if owner.frame() > frame:
            return owner.frame()
        time.sleep(owner.base.POLL_SECONDS)
    raise AssertionError("Timed out waiting for UI completion after document frame")


def capture_nonblank(owner, name: str):
    owner.capture(name)
    observation = owner.report["steps"][-1]
    assert observation["pixel_spread"] > 1, "native window capture was blank"
    return owner.last_capture
