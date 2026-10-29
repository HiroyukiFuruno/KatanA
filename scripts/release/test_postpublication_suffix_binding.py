"""Fixture-only suffix bindings; not live release admission evidence."""

from copy import deepcopy
from dataclasses import replace
from datetime import datetime, timezone
from pathlib import Path
import subprocess
import unittest

import cleanup_target
from postpublication_source_binding import (
    PreMergePullRequestRun, REPOSITORY, SourceBinding, WORKFLOW,
    bind_merged_pull_request, bind_workflow_run, select_merged_pull_request,
)

SOURCE = "0d139094a39887a7a5657ecdd63f53a6db012fde"
HEAD = "95ce239cab09b5c9991af2c7daec507395f42b72"
BRANCH = "release/v0.22.43-dependency-refresh"
VERSION = "v0.22.43"
MERGED_AT = "2026-10-29T13:26:21Z"


def pull_request_record():
    return {"number": 391, "state": "closed", "merged": True,
            "merged_at": MERGED_AT, "merge_commit_sha": SOURCE,
            "base": {"ref": "master", "repo": {"full_name": REPOSITORY}},
            "head": {"ref": BRANCH, "sha": HEAD, "repo": {"full_name": REPOSITORY}}}


def run_record():
    return {"id": 40881935779, "run_attempt": 1, "path": WORKFLOW,
            "repository": {"full_name": REPOSITORY},
            "head_repository": {"full_name": REPOSITORY}, "event": "pull_request",
            "head_branch": BRANCH, "head_sha": HEAD, "status": "completed",
            "conclusion": "failure", "created_at": "2026-10-29T13:26:22Z"}


def verified_binding():
    return SourceBinding("merged_pull_request", SOURCE, VERSION, HEAD, BRANCH,
                         "pull_request", 391, datetime(2026, 10, 29, 13, 26, 21, tzinfo=timezone.utc))


class PostpublicationSuffixBindingTests(unittest.TestCase):
    def test_suffix_pr_preserves_actual_branch_head_and_product_merge(self):
        binding = bind_merged_pull_request(pull_request_record(), SOURCE, VERSION)
        self.assertEqual(binding, verified_binding())
        self.assertNotEqual(binding.product_commit, binding.run_head_sha)
        self.assertEqual(bind_workflow_run(run_record(), SOURCE, VERSION, binding), binding)

    def test_verified_suffix_binding_reaches_run_guards(self):
        binding = verified_binding()
        self.assertEqual(bind_workflow_run(run_record(), SOURCE, VERSION, binding), binding)
        for mutation in ({"head_sha": SOURCE}, {"head_branch": "release/v0.22.43"},
                         {"run_attempt": 0}, {"id": True}, {"path": "other.yml"},
                         {"head_repository": {"full_name": "other/Repo"}},
                         {"repository": {"full_name": "other/Repo"}}):
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                bind_workflow_run(run_record() | mutation, SOURCE, VERSION, binding)
        with self.assertRaises(PreMergePullRequestRun):
            bind_workflow_run(run_record() | {"created_at": "2026-10-29T13:26:20Z"},
                              SOURCE, VERSION, binding)

    def test_rejects_wrong_version_prefix_and_malformed_suffix_at_both_bindings(self):
        for branch in ("release/v0.22.430-refresh", "release/v0.22.44-refresh",
                       "release/v0.22.43refresh", "release/v0.22.43-", "release/v0.22.43-_bad",
                       "release/v0.22.43-x/y", "release/v0.22.43-x\n", "other/v0.22.43", None):
            record = pull_request_record()
            record["head"]["ref"] = branch
            with self.subTest(branch=branch), self.assertRaises(ValueError):
                bind_merged_pull_request(record, SOURCE, VERSION)
            with self.subTest(branch=branch), self.assertRaises(ValueError):
                bind_workflow_run(run_record() | {"head_branch": branch}, SOURCE, VERSION,
                                  replace(verified_binding(), head_branch=branch))

    def test_suffix_pr_keeps_repository_merge_date_and_full_identity_guards(self):
        for field, value in (("number", True), ("state", "open"), ("merged", False),
                             ("merge_commit_sha", HEAD), ("merged_at", None),
                             ("merged_at", "2026-10-29T13:26:21")):
            with self.subTest(field=field), self.assertRaises(ValueError):
                bind_merged_pull_request(pull_request_record() | {field: value}, SOURCE, VERSION)
        for side in ("base", "head"):
            record = deepcopy(pull_request_record())
            record[side]["repo"]["full_name"] = "other/Repo"
            with self.subTest(side=side), self.assertRaises(ValueError):
                bind_merged_pull_request(record, SOURCE, VERSION)
        for side, field, value in (("base", "ref", "main"), ("head", "sha", HEAD[:12])):
            record = deepcopy(pull_request_record())
            record[side][field] = value
            with self.subTest(side=side, field=field), self.assertRaises(ValueError):
                bind_merged_pull_request(record, SOURCE, VERSION)
        for version in ("0.22.43", "v0.22.43-refresh", "v0.22.430", "v00.22.43"):
            with self.subTest(version=version), self.assertRaises(ValueError):
                bind_merged_pull_request(pull_request_record(), SOURCE, version)

    def test_ambiguous_same_version_bare_and_suffix_prs_remain_rejected(self):
        bare = deepcopy(pull_request_record())
        bare["number"] = 389
        bare["head"]["ref"] = "release/v0.22.43"
        with self.assertRaisesRegex(ValueError, "ambiguous"):
            select_merged_pull_request([bare, pull_request_record()], SOURCE, VERSION)

    def test_bash_grammar_remains_equal_through_legacy_cleanup_alias(self):
        parser = Path(__file__).with_name("extract-release-branch-version.sh")
        for branch in (BRANCH, "release/v0.22.43", "release/v01.2.3-A_0.x-y",
                       "release/v0.22.43-", "release/v0.22.43-_x", "release/v0.22.43-x/y",
                       "release/v0.22.43-x\n", "release/v0.22.43x", "release/fixture"):
            result = subprocess.run(["bash", str(parser), branch], capture_output=True,
                                    check=False, timeout=5)
            expected = result.stdout.decode().strip() if result.returncode == 0 else None
            with self.subTest(branch=branch):
                self.assertEqual(cleanup_target.release_branch_version(branch), expected)


if __name__ == "__main__":
    unittest.main()
