"""A receipt must bind successful, recent, same-source three-OS CI."""

import copy
from datetime import datetime, timedelta, timezone
from pathlib import Path
import unittest

from ci_run_receipt import make_receipt
from test_ci_run_admission import REPO, SHA, job_records, run_record

NOW = datetime(2026, 11, 2, 12, tzinfo=timezone.utc)


def full_jobs():
    jobs = job_records()
    for job in jobs:
        names = ["Run tests", "Test screenshot harness contracts", "Run multi-format headless acceptance"]
        if job["name"] == "Test and Build (macos-latest)":
            names += ["Install coverage tool", "Run coverage"]
        job["steps"] = [{"name": name, "status": "completed", "conclusion": "success"} for name in names]
    for index, platform in enumerate(("macos-latest", "windows-latest", "ubuntu-latest"), 4):
        names = ["Run linter"] + (["Check formatting"] if platform == "ubuntu-latest" else [])
        jobs.append(jobs[0] | {"id": index, "name": f"Lint ({platform})",
                                "steps": [{"name": name, "status": "completed", "conclusion": "success"} for name in names]})
    for index, name, names in ((7, "Dependency Supply Chain", ["Check advisories, licenses, and sources"]),
                               (8, "CodeQL Security Scan", ["Initialize CodeQL", "Auto-build", "Perform CodeQL Analysis"])):
        jobs.append(jobs[0] | {"id": index, "name": name,
                               "steps": [{"name": step, "status": "completed", "conclusion": "success"} for step in names]})
    return jobs


class ReceiptTests(unittest.TestCase):
    def make(self, run=None, jobs=None, after=None, *, now=NOW, tree="b" * 40):
        run = run or (run_record() | {"run_started_at": NOW.isoformat()})
        jobs = full_jobs() if jobs is None else jobs
        pages = [{"total_count": len(jobs), "jobs": jobs}]
        return make_receipt(run, pages, copy.deepcopy(run) if after is None else after,
                            SHA, 42, REPO, tree, now=now)

    def test_receipt_is_bound_and_does_not_allow_generic_reuse(self):
        receipt = self.make()
        self.assertEqual(receipt["source_commit"], SHA)
        self.assertEqual(receipt["source_tree"], "b" * 40)
        self.assertEqual(len(receipt["jobs"]), 8)
        self.assertEqual(len(receipt["snapshot_sha256"]), 64)
        self.assertFalse(receipt["generic_reuse_authorized"])

    def test_dispatch_requires_trusted_actor_and_master(self):
        run = run_record() | {"run_started_at": NOW.isoformat(), "event": "workflow_dispatch",
                              "actor": {"login": "github-actions[bot]"}}
        self.assertEqual(self.make(run=run)["event"], "workflow_dispatch")
        for change in ({"actor": {"login": "untrusted"}}, {"actor": None},
                       {"head_branch": "other"}, {"head_sha": "c" * 40}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.make(run=run | change)

    def test_expired_future_missing_or_altered_evidence_is_rejected(self):
        for start in ((NOW - timedelta(days=8)).isoformat(),
                      (NOW + timedelta(minutes=6)).isoformat(), "invalid", None):
            with self.subTest(start=start), self.assertRaises(ValueError):
                self.make(run=run_record() | {"run_started_at": start})
        for key, value in (("head_sha", "c" * 40), ("path", "wrong"),
                           ("event", "pull_request"), ("conclusion", "failure")):
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.make(run=run_record() | {"run_started_at": NOW.isoformat(), key: value})
        jobs = full_jobs()
        jobs[0]["conclusion"] = "skipped"
        with self.assertRaises(ValueError):
            self.make(jobs=jobs)
        with self.assertRaises(ValueError):
            self.make(tree="not-a-tree")
        run = run_record() | {"run_started_at": NOW.isoformat()}
        with self.assertRaises(ValueError):
            self.make(run=run, after=run | {"run_started_at": (NOW - timedelta(hours=1)).isoformat()})

    def test_each_full_job_and_required_test_step_must_succeed(self):
        for index in range(8):
            jobs = full_jobs()
            jobs[index]["conclusion"] = "skipped"
            with self.subTest(job=jobs[index]["name"]), self.assertRaises(ValueError):
                self.make(jobs=jobs)
        for job_index, step_name in ((0, "Run coverage"), (0, "Test screenshot harness contracts"),
                                     (1, "Run multi-format headless acceptance"),
                                     (2, "Test screenshot harness contracts"), (3, "Run linter"),
                                     (6, "Check advisories, licenses, and sources"),
                                     (7, "Perform CodeQL Analysis")):
            jobs = full_jobs()
            step = next(step for step in jobs[job_index]["steps"] if step["name"] == step_name)
            step["conclusion"] = "skipped"
            with self.subTest(job=jobs[job_index]["name"], step=step_name), self.assertRaises(ValueError):
                self.make(jobs=jobs)

    def test_manual_gate_is_after_bump_and_mandatory(self):
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/build-and-release.yml").read_text()
        step = workflow.split("      - name: Verify manual release source CI", 1)[1].split(
            "      - name: Preserve manual source CI receipt", 1)[0]
        self.assertIn("if: github.event_name == 'workflow_dispatch'", step)
        self.assertIn('git rev-parse HEAD', step)
        self.assertIn('--required "$RUNNER_TEMP/manual-ci-receipt.json"', step)
        self.assertLess(workflow.index("      - name: Bump version and push"),
                        workflow.index("      - name: Verify manual release source CI"))
        gate = (root / "scripts/release/verify-version-bump-ci.sh").read_text()
        self.assertIn('[[ "$REQUIRED" != "--required" ]]', gate)
        self.assertIn('--dispatch-if-missing', gate)
        self.assertIn('gh workflow run test-and-build.yml --ref master', gate)
        self.assertIn('Master changed during CI verification', gate)


if __name__ == "__main__":
    unittest.main()
