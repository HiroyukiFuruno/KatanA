#!/usr/bin/env python3
"""Unit tests for the document-host observation watcher CLI."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
import ast
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
WATCH_PATH = ROOT / "scripts" / "release" / "run-document-host-observation-watch.py"
SPEC = importlib.util.spec_from_file_location("document_host_observation_watch", WATCH_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("watcher module could not be loaded")
WATCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(WATCH)


class MarkerCapture(io.StringIO):
    def __init__(self, fsync_count):
        super().__init__()
        self.fsync_count = fsync_count
        self.fsyncs_at_markers = []

    def write(self, value):
        if value.startswith(("HOST_WATCH_READY ", "HOST_WATCH_DONE ")):
            self.fsyncs_at_markers.append(self.fsync_count())
        return super().write(value)


class IsolatedWatchTmp:
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="watch-unit-")
        self.private_tmp = Path(self.temporary.name) / "pr405-resolution"
        self.private_tmp.mkdir()
        self.tmp_patch = patch.object(WATCH, "TMP", self.private_tmp)
        self.tmp_patch.start()

    def tearDown(self):
        self.tmp_patch.stop()
        self.temporary.cleanup()


class WatchStatusTests(IsolatedWatchTmp, unittest.TestCase):
    def setUp(self):
        super().setUp()
        self.directory = self.private_tmp / "case"
        self.directory.mkdir()

    def status(self):
        return WATCH.Status(self.directory / "status.json", self.directory / "watch.jsonl",
                            "open", self.directory / "run", "qa-cycle01-office")

    @staticmethod
    def parse_marker(output, prefix):
        lines = [line for line in output.splitlines() if line.startswith(prefix)]
        if len(lines) != 1:
            raise AssertionError(f"expected one {prefix!r} marker, got {len(lines)}")
        return json.loads(lines[0][len(prefix):])

    def test_ready_marker_follows_status_and_log_fsync(self):
        count = 0

        def fsync(_fd):
            nonlocal count
            count += 1

        output = MarkerCapture(lambda: count)
        with patch.object(WATCH.os, "fsync", side_effect=fsync), contextlib.redirect_stdout(output):
            status = self.status()
            try:
                status.write("watching_open", True, pid=123, ready_seq=9)
            finally:
                status.close()

        row = self.parse_marker(output.getvalue(), "HOST_WATCH_READY ")
        self.assertGreaterEqual(output.fsyncs_at_markers[0], 4)
        self.assertEqual(row, json.loads((self.directory / "status.json").read_text(encoding="utf-8")))
        self.assertTrue(row["watcher_ready"])
        self.assertNotIn("HOST_WATCH_DONE ", output.getvalue())

    def test_done_marker_follows_finished_status_and_log_fsync(self):
        count = 0

        def fsync(_fd):
            nonlocal count
            count += 1

        output = MarkerCapture(lambda: count)
        with patch.object(WATCH.os, "fsync", side_effect=fsync), contextlib.redirect_stdout(output):
            status = self.status()
            try:
                status.write("finished", False, result_path="result.json", close_seq=11)
            finally:
                status.close()

        row = self.parse_marker(output.getvalue(), "HOST_WATCH_DONE ")
        self.assertGreaterEqual(output.fsyncs_at_markers[0], 4)
        self.assertEqual(row, json.loads((self.directory / "status.json").read_text(encoding="utf-8")))
        self.assertEqual(row["state"], "finished")

    def test_not_ready_states_emit_no_marker(self):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = self.status()
            try:
                status.write("watching_open", False, pid=123)
                status.write("waiting_post_closed_sample", False, pid=123)
            finally:
                status.close()
        self.assertEqual(output.getvalue(), "")

    def test_repeated_ready_state_emits_only_once(self):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = self.status()
            try:
                status.write("watching_open", True, pid=123)
                status.write("watching_open", True, pid=123)
            finally:
                status.close()
        self.assertEqual(output.getvalue().count("HOST_WATCH_READY "), 1)

    def test_status_fsync_failure_emits_no_ready_marker(self):
        count = 0

        def fsync(_fd):
            nonlocal count
            count += 1
            if count == 3:
                raise OSError("status fsync failed")

        output = io.StringIO()
        with patch.object(WATCH.os, "fsync", side_effect=fsync), contextlib.redirect_stdout(output):
            status = self.status()
            try:
                with self.assertRaisesRegex(OSError, "status fsync failed"):
                    status.write("watching_open", True, pid=123)
            finally:
                status.close()
        self.assertNotIn("HOST_WATCH_READY ", output.getvalue())

    def test_log_fsync_failure_emits_no_ready_marker(self):
        count = 0

        def fsync(_fd):
            nonlocal count
            count += 1
            if count == 4:
                raise OSError("log fsync failed")

        output = io.StringIO()
        with patch.object(WATCH.os, "fsync", side_effect=fsync), contextlib.redirect_stdout(output):
            status = self.status()
            try:
                with self.assertRaisesRegex(OSError, "log fsync failed"):
                    status.write("watching_open", True, pid=123)
            finally:
                status.close()
        self.assertNotIn("HOST_WATCH_READY ", output.getvalue())

    def test_status_replace_failure_emits_no_ready_marker(self):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = self.status()
            try:
                with patch.object(WATCH.os, "replace", side_effect=OSError("replace failed")):
                    with self.assertRaisesRegex(OSError, "replace failed"):
                        status.write("watching_open", True, pid=123)
            finally:
                status.close()
        self.assertNotIn("HOST_WATCH_READY ", output.getvalue())

    def test_foreign_status_owner_is_rejected_before_ready_marker(self):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = self.status()
            try:
                path = self.directory / "status.json"
                row = json.loads(path.read_text(encoding="utf-8"))
                row["watch_id"] = "foreign-owner"
                path.write_text(json.dumps(row), encoding="utf-8")
                with self.assertRaisesRegex(WATCH.WatchError, "ownership changed"):
                    status.write("watching_open", True, pid=123)
            finally:
                status.close()
        self.assertNotIn("HOST_WATCH_READY ", output.getvalue())

    def test_existing_status_file_is_never_overwritten(self):
        path = self.directory / "status.json"
        path.write_text("keep", encoding="utf-8")
        with self.assertRaisesRegex(WATCH.WatchError, "output already exists"):
            WATCH.Status(path, self.directory / "watch.jsonl", "open", self.directory / "run", "qa")
        self.assertEqual(path.read_text(encoding="utf-8"), "keep")

    def test_status_file_outside_private_tmp_is_rejected(self):
        with self.assertRaisesRegex(WATCH.WatchError, "must be under tmp/pr405-resolution"):
            WATCH.safe_new(ROOT / "scripts" / "release" / "unused-status.json", "status file")

    def test_symlink_status_file_is_rejected(self):
        target = self.directory / "target.json"
        target.write_text("keep", encoding="utf-8")
        link = self.directory / "status-link.json"
        try:
            link.symlink_to(target)
        except OSError as error:
            if sys.platform == "win32" and getattr(error, "winerror", None) == 1314:
                self.skipTest("Windows symlink privilege is unavailable in this runner")
            raise
        with self.assertRaisesRegex(WATCH.WatchError, "output already exists"):
            WATCH.safe_new(link, "status file")
        self.assertEqual(target.read_text(encoding="utf-8"), "keep")


class RecorderBoundaryTests(IsolatedWatchTmp, unittest.TestCase):
    def test_open_outputs_refuse_any_existing_recorder_file(self):
        with tempfile.TemporaryDirectory(prefix="watch-run-", dir=WATCH.TMP) as name:
            run = Path(name)
            label = "qa-cycle01-office"
            WATCH.ensure_run_outputs(run, label, "open")
            (run / (label + ".ready.json")).write_text("keep", encoding="utf-8")
            with self.assertRaisesRegex(WATCH.WatchError, "refusing overwrite"):
                WATCH.ensure_run_outputs(run, label, "open")
            self.assertEqual((run / (label + ".ready.json")).read_text(encoding="utf-8"), "keep")

    def test_close_requires_existing_start_ready_and_open_receipt(self):
        with tempfile.TemporaryDirectory(prefix="watch-run-", dir=WATCH.TMP) as name:
            run = Path(name)
            label = "qa-cycle01-office"
            with self.assertRaisesRegex(WATCH.WatchError, "required prior evidence missing"):
                WATCH.ensure_run_outputs(run, label, "close")
            for suffix in (".start.json", ".ready.json", "-open.receipt.json"):
                (run / (label + suffix)).write_text("{}", encoding="utf-8")
            WATCH.ensure_run_outputs(run, label, "close")

    def test_original_first_frame_budgets_remain_15_and_60_seconds(self):
        class Recorder:
            duration = 15_000

            @staticmethod
            def duration_ms(_start, _end):
                return Recorder.duration

        start = {"observedAt": "2026-11-03T00:00:00Z"}
        ready = {"observedAt": "2026-11-03T00:00:15Z"}
        WATCH.ensure_ready_budget(Recorder(), start, ready, 15)
        Recorder.duration = 15_001
        with self.assertRaisesRegex(WATCH.WatchError, "original 15s budget"):
            WATCH.ensure_ready_budget(Recorder(), start, ready, 15)
        Recorder.duration = 60_000
        ready = {"observedAt": "2026-11-03T00:01:00Z"}
        WATCH.ensure_ready_budget(Recorder(), start, ready, 60)
        Recorder.duration = 60_001
        with self.assertRaisesRegex(WATCH.WatchError, "original 60s budget"):
            WATCH.ensure_ready_budget(Recorder(), start, ready, 60)
        self.assertEqual(WATCH.output_names("qa-cycle01-office"), (
            "qa-cycle01-office-start.receipt.json", "qa-cycle01-office.start.json",
            "qa-cycle01-office-open.receipt.json", "qa-cycle01-office.ready.json",
            "qa-cycle01-office-close-start.receipt.json", "qa-cycle01-office.close-start.json",
            "qa-cycle01-office-closed.receipt.json", "qa-cycle01-office.closed.json",
            "qa-cycle01-office.result.json"))

    def test_close_deadline_and_post_close_sample_remain_distinct(self):
        tree = ast.parse(WATCH_PATH.read_text(encoding="utf-8"))
        functions = {node.name: node for node in tree.body if isinstance(node, ast.FunctionDef)}
        open_stage = functions["open_stage"]
        close_stage = functions["close_stage"]
        open_budget = next(node for node in ast.walk(open_stage)
                           if isinstance(node, ast.Assign)
                           and any(isinstance(target, ast.Name) and target.id == "budget"
                                   for target in node.targets))
        self.assertEqual(ast.dump(open_budget.value, include_attributes=False),
                         "IfExp(test=Compare(left=Attribute(value=Name(id='args', ctx=Load()), "
                         "attr='kind', ctx=Load()), ops=[Eq()], comparators=[Constant(value='html')]), "
                         "body=Constant(value=60), orelse=Constant(value=15))")
        deadline = next(node for node in ast.walk(close_stage)
                        if isinstance(node, ast.Assign)
                        and any(isinstance(target, ast.Name) and target.id == "deadline"
                                for target in node.targets))
        self.assertIsInstance(deadline.value, ast.BinOp)
        self.assertIsInstance(deadline.value.left, ast.Call)
        self.assertIsInstance(deadline.value.left.func, ast.Attribute)
        self.assertEqual(deadline.value.left.func.attr, "timestamp")
        self.assertIsInstance(deadline.value.right, ast.Constant)
        self.assertEqual(deadline.value.right.value, 5)
        self.assertTrue(any(isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                            and node.func.id == "stable_close_sample"
                            for node in ast.walk(close_stage)))
        self.assertTrue(any(isinstance(node, ast.Attribute) and node.attr == "sample_timeout_seconds"
                            for node in ast.walk(close_stage)))


if __name__ == "__main__":
    unittest.main()
