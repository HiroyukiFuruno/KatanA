from __future__ import annotations

import copy
import tempfile
import subprocess
import sys
import unittest
from pathlib import Path

from postpublication_run_admission import (
    REQUIRED_JOBS,
    OPTIONAL_JOBS,
    _candidate_runs,
    _newest_candidate,
    _write_new_output,
    verify_run,
)
from postpublication_source_binding import bind_workflow_run

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path(__file__).with_name("postpublication_run_admission.py")
SOURCE = "8c89a1544e9ac693d5ba388792a843baa55c9363"


def run_record() -> dict:
    return {"id": 40672472894, "run_attempt": 1, "head_sha": SOURCE,
            "path": ".github/workflows/build-and-release.yml", "event": "workflow_dispatch",
            "status": "completed", "conclusion": "failure",
            "repository": {"full_name": "HiroyukiFuruno/KatanA"},
            "head_repository": {"full_name": "HiroyukiFuruno/KatanA"},
            "head_branch": "master",
            "created_at": "2026-10-28T00:00:00Z"}


def jobs_record(run_id: int = 40672472894) -> list[dict]:
    return [{"id": index, "run_id": run_id, "run_attempt": 1, "head_sha": SOURCE,
             "name": name, "status": "completed", "conclusion": "success"}
            for index, name in enumerate(sorted(REQUIRED_JOBS), 1)]


class PostpublicationRunAdmissionTests(unittest.TestCase):
    def test_pr_run_jobs_bind_to_verified_head_H_while_manifests_bind_to_product_M(self) -> None:
        head = "9ea1d1331885ed889ee2c87608c61ef1f447314a"
        run = run_record() | {
            "head_sha": head,
            "event": "pull_request",
            "head_branch": "release/v0.22.42",
        }
        jobs = [job | {"head_sha": head} for job in jobs_record()]
        result = verify_run(run, [{"jobs": jobs}], SOURCE, run_head_sha=head)
        self.assertEqual(result["run"]["head_sha"], head)
        self.assertEqual(result["jobs"][0]["head_sha"], head)

    def test_admits_required_successes_even_when_cleanup_failed_in_overall_run(self) -> None:
        result = verify_run(run_record(), [{"jobs": jobs_record()}], SOURCE)
        self.assertEqual(result["run"]["conclusion"], "failure")
        self.assertEqual(result["optional_job_results"][
            "Audit and clean published release branches"]["conclusion"], None)

    def test_records_optional_cleanup_failure_without_treating_it_as_cleanup_success(self) -> None:
        jobs = jobs_record() + [{"id": 99, "run_id": 40672472894, "run_attempt": 1,
                                 "head_sha": SOURCE,
                                 "name": "Audit and clean published release branches",
                                 "status": "completed", "conclusion": "failure"}]
        result = verify_run(run_record(), [{"jobs": jobs}], SOURCE)
        self.assertEqual(result["optional_job_results"][
            "Audit and clean published release branches"]["conclusion"], "failure")

    def test_rejects_failure_skipped_cancelled_and_running_for_every_required_job(self) -> None:
        for name in REQUIRED_JOBS:
            for status, conclusion in (("completed", "failure"), ("completed", "skipped"),
                                       ("completed", "cancelled"), ("in_progress", None)):
                with self.subTest(name=name, status=status, conclusion=conclusion):
                    jobs = jobs_record()
                    next(job for job in jobs if job["name"] == name).update(
                        status=status, conclusion=conclusion)
                    with self.assertRaises(ValueError):
                        verify_run(run_record(), [{"jobs": jobs}], SOURCE)

    def test_rejects_missing_duplicate_and_unknown_jobs(self) -> None:
        for name in REQUIRED_JOBS:
            with self.subTest(missing=name):
                jobs = [job for job in jobs_record() if job["name"] != name]
                with self.assertRaisesRegex(ValueError, "missing"):
                    verify_run(run_record(), [{"jobs": jobs}], SOURCE)
            with self.subTest(duplicate=name):
                jobs = jobs_record()
                duplicate = next(job for job in jobs if job["name"] == name)
                jobs.append(copy.deepcopy(duplicate) | {"id": 100})
                with self.assertRaisesRegex(ValueError, "duplicate job name"):
                    verify_run(run_record(), [{"jobs": jobs}], SOURCE)
        jobs = jobs_record() + [jobs_record()[0] | {"id": 101, "name": "Cleanup installer"}]
        with self.assertRaisesRegex(ValueError, "unknown job"):
            verify_run(run_record(), [{"jobs": jobs}], SOURCE)

    def test_rejects_stale_attempt_or_mixed_run_job(self) -> None:
        jobs = jobs_record()
        jobs[0]["run_attempt"] = 2
        with self.assertRaisesRegex(ValueError, "different run"):
            verify_run(run_record(), [{"jobs": jobs}], SOURCE)

    def test_rejects_job_head_sha_mismatch_for_each_required_job(self) -> None:
        for name in REQUIRED_JOBS:
            with self.subTest(name=name):
                jobs = jobs_record()
                next(job for job in jobs if job["name"] == name)["head_sha"] = "a" * 40
                with self.assertRaisesRegex(ValueError, "source"):
                    verify_run(run_record(), [{"jobs": jobs}], SOURCE)
        jobs = jobs_record()
        jobs[0]["run_attempt"] = 0
        with self.assertRaises(ValueError):
            verify_run(run_record(), [{"jobs": jobs}], SOURCE)

    def test_rejects_invalid_run_identity_and_source_binding(self) -> None:
        for key, value in (("id", True), ("run_attempt", 0), ("head_sha", "a" * 40),
                           ("path", "other.yml"), ("event", "push"),
                           ("status", "in_progress"), ("conclusion", "skipped"),
                           ("repository", {"full_name": "other/repo"})):
            with self.subTest(key=key):
                run = run_record()
                run[key] = value
                with self.assertRaises(ValueError):
                    verify_run(run, [{"jobs": jobs_record()}], SOURCE)

    def test_cli_rejects_bad_repository_and_source(self) -> None:
        for args in (("--repository", "other/repo", "--source", SOURCE,
                      "--version", "v0.22.42", "--output", "/tmp/out.json"),
                     ("--repository", "HiroyukiFuruno/KatanA", "--source", "bad",
                      "--version", "v0.22.42", "--output", "/tmp/out.json"),
                     ("--repository", "HiroyukiFuruno/KatanA", "--source", SOURCE,
                      "--version", "0.22.42", "--output", "/tmp/out.json")):
            result = subprocess.run([sys.executable, "-B", str(SCRIPT), *args],
                                    cwd=ROOT, capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 2)

    def test_rejects_optional_job_without_known_terminal_completion(self) -> None:
        optional = next(iter(OPTIONAL_JOBS))
        for status, conclusion in (("in_progress", None), ("completed", "unknown"),
                                   ("completed", None)):
            with self.subTest(status=status, conclusion=conclusion):
                jobs = jobs_record() + [{"id": 99, "run_id": 40672472894,
                                         "run_attempt": 1, "head_sha": SOURCE,
                                         "name": optional, "status": status,
                                         "conclusion": conclusion}]
                with self.assertRaisesRegex(ValueError, "optional job"):
                    verify_run(run_record(), [{"jobs": jobs}], SOURCE)

    def test_rejects_existing_output_file_without_overwriting(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "evidence.json"
            output.write_text("keep", encoding="utf-8")
            with self.assertRaises(FileExistsError):
                _write_new_output(output, {"run": run_record()})
            self.assertEqual(output.read_text(encoding="utf-8"), "keep")

    def test_rejects_output_symlink_without_touching_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "target.json"
            output = Path(directory) / "evidence.json"
            target.write_text("keep", encoding="utf-8")
            output.symlink_to(target)
            with self.assertRaises(FileExistsError):
                _write_new_output(output, {"run": run_record()})
            self.assertEqual(target.read_text(encoding="utf-8"), "keep")

    def test_rejects_invalid_candidate_selection_metadata(self) -> None:
        run = run_record()
        for key, value in (("created_at", 123), ("id", True), ("run_attempt", "1"),
                           ("created_at", "not-a-time")):
            with self.subTest(key=key):
                candidate = run | {key: value}
                with self.assertRaises(ValueError):
                    _candidate_runs([{"workflow_runs": [candidate]}], SOURCE)

    def test_selects_newest_matching_run_and_does_not_fall_back_by_validation(self) -> None:
        older = run_record() | {"id": 10, "created_at": "2026-10-27T00:00:00Z"}
        newest = run_record() | {"id": 11, "created_at": "2026-10-28T00:00:00Z",
                                "conclusion": "failure"}
        candidates = _candidate_runs([{"workflow_runs": [older, newest]}], SOURCE)
        selected, _ = _newest_candidate([
            (run, bind_workflow_run(run, SOURCE, "v0.22.42")) for run in candidates
        ])
        self.assertEqual(selected["id"], 11)
        jobs = jobs_record(run_id=11)
        jobs[0]["conclusion"] = "failure"
        with self.assertRaises(ValueError):
            verify_run(selected, [{"jobs": jobs}], SOURCE)


if __name__ == "__main__":
    unittest.main()
