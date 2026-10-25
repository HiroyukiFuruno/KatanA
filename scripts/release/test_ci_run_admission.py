"""Require actual three-platform job success before reusing a push CI run."""

import copy
import importlib
from pathlib import Path
import unittest


SHA = "a" * 40
REPO = "HiroyukiFuruno/KatanA"
NAMES = tuple(f"Test and Build ({os})" for os in
              ("macos-latest", "windows-latest", "ubuntu-latest"))


def run_record():
    return {"id": 42, "run_attempt": 2, "event": "push", "head_sha": SHA,
            "head_branch": "master", "path": ".github/workflows/test-and-build.yml",
            "status": "completed", "conclusion": "success",
            "repository": {"full_name": REPO}}


def job_records():
    return [{"id": index, "run_id": 42, "run_attempt": 2, "head_sha": SHA,
             "name": name, "status": "completed", "conclusion": "success"}
            for index, name in enumerate(NAMES, 1)]


class CiRunAdmissionTests(unittest.TestCase):
    def test_existing_post_merge_caller_verifies_jobs_before_accepting_run(self):
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/build-and-release.yml").read_text()
        self.assertIn('scripts/release/verify-version-bump-ci.sh "${{ github.event.pull_request.merge_commit_sha }}"', workflow)
        gate = (root / "scripts/release/verify-version-bump-ci.sh").read_text()
        start = gate.index('if [ "$STATUS" = "completed" ] && [ "$CONCLUSION" = "success" ]')
        accepted = gate.index('success "CI passed', start)
        self.assertIn('python3 scripts/release/verify-ci-run.py', gate[start:accepted])
        self.assertIn('--json status,conclusion,url,headSha,databaseId', gate)
        verifier = (root / "scripts/release/verify-ci-run.py").read_text()
        self.assertIn('/jobs?filter=all&per_page=100', verifier)
        self.assertNotIn('/attempts/', verifier)

    def verify(self, run=None, jobs=None, after=None, pages=None):
        run = run_record() if run is None else run
        jobs = job_records() if jobs is None else jobs
        pages = [{"total_count": len(jobs), "jobs": jobs}] if pages is None else pages
        module = importlib.import_module("ci_run_admission")
        return module.verify_run(run, pages, copy.deepcopy(run) if after is None else after,
                                 SHA, 42, REPO)

    def test_three_platforms_allow_pr_only_jobs_to_be_skipped(self):
        jobs = job_records()
        jobs.append(jobs[0] | {"id": 4, "name": "Lint", "conclusion": "skipped"})
        self.assertEqual(self.verify(jobs=jobs), (42, 2))

    def test_partial_rerun_preserves_latest_success_for_each_platform(self):
        jobs = job_records()
        jobs[0]["run_attempt"] = jobs[2]["run_attempt"] = 1
        jobs.append(jobs[1] | {"id": 4, "run_attempt": 1, "conclusion": "failure"})
        self.assertEqual(self.verify(jobs=jobs), (42, 2))
        self.assertEqual(self.verify(jobs=list(reversed(jobs))), (42, 2))

    def test_latest_unsuccessful_job_cannot_reuse_older_success(self):
        for conclusion in ("failure", "skipped", "cancelled"):
            jobs = job_records()
            jobs.append(jobs[1] | {"id": 4, "run_attempt": 1})
            jobs[1]["conclusion"] = conclusion
            with self.subTest(conclusion=conclusion), self.assertRaises(ValueError):
                self.verify(jobs=jobs)
            with self.subTest(conclusion=conclusion, order="reverse"), self.assertRaises(ValueError):
                self.verify(jobs=list(reversed(jobs)))

    def test_old_jobs_alone_cannot_describe_a_new_attempt(self):
        jobs = [job | {"run_attempt": 1} for job in job_records()]
        with self.assertRaises(ValueError):
            self.verify(jobs=jobs)

    def test_successful_run_without_all_three_jobs_is_rejected(self):
        for index in range(3):
            with self.subTest(index=index), self.assertRaises(ValueError):
                self.verify(jobs=job_records()[:index] + job_records()[index + 1:])

    def test_each_required_job_must_have_executed_successfully(self):
        for conclusion in ("skipped", "neutral", "cancelled", "failure", None):
            jobs = job_records()
            jobs[1]["conclusion"] = conclusion
            with self.subTest(conclusion=conclusion), self.assertRaises(ValueError):
                self.verify(jobs=jobs)
        jobs[1]["conclusion"], jobs[1]["status"] = "success", "in_progress"
        with self.assertRaises(ValueError):
            self.verify(jobs=jobs)

    def test_source_workflow_event_branch_and_repository_are_bound(self):
        for key, value in (("head_sha", "b" * 40), ("event", "pull_request"),
                           ("path", ".github/workflows/other.yml"),
                           ("head_branch", "release/v0.22.42"),
                           ("repository", {"full_name": "other/repo"}),
                           ("id", 43), ("run_attempt", True),
                           ("conclusion", "skipped"), ("status", "in_progress")):
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.verify(run=run_record() | {key: value})

    def test_jobs_cannot_come_from_other_source_run_or_future_attempt(self):
        for key, value in (("run_id", 43), ("run_attempt", 3), ("run_attempt", True),
                           ("head_sha", "b" * 40), ("id", True)):
            jobs = job_records()
            jobs[1][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.verify(jobs=jobs)

    def test_rerun_started_during_observation_is_rejected(self):
        for change in ({"run_attempt": 3}, {"status": "in_progress"},
                       {"conclusion": "failure"}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.verify(after=run_record() | change)

    def test_pages_must_be_complete_and_duplicate_free(self):
        jobs = job_records()
        self.assertEqual(self.verify(pages=[{"total_count": 3, "jobs": jobs[:1]},
                                            {"total_count": 3, "jobs": jobs[1:]}]), (42, 2))
        for pages in ([], [{"total_count": 4, "jobs": jobs}],
                      [{"total_count": True, "jobs": jobs}],
                      [{"total_count": 3, "jobs": jobs}, {"total_count": 0, "jobs": []}],
                      [{"total_count": 4, "jobs": jobs + [jobs[0]]}]):
            with self.subTest(pages=pages), self.assertRaises(ValueError):
                self.verify(pages=pages)

    def test_duplicate_or_unexpected_test_matrix_leg_is_rejected(self):
        for name in (NAMES[0], "Test and Build (other-os)"):
            jobs = job_records()
            jobs.append(jobs[0] | {"id": 4, "name": name})
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.verify(jobs=jobs)


if __name__ == "__main__":
    unittest.main()
