"""Validate tag, merged pull-request, and workflow-run source bindings."""

from copy import deepcopy
import unittest

from postpublication_source_binding import (
    REPOSITORY,
    PreMergePullRequestRun,
    bind_merged_pull_request,
    bind_workflow_run,
    select_merged_pull_request,
    verify_tag_source,
)

SOURCE = "26e62e15e721ef9bb1084244c99c9aaa0f9a8619"
DIRECT_SOURCE = "8c89a1544e9ac693d5ba388792a843baa55c9363"
HEAD = "9ea1d1331885ed889ee2c87608c61ef1f447314a"
VERSION = "v0.22.42"
WORKFLOW = ".github/workflows/build-and-release.yml"


def pull_request_record() -> dict:
    return {
        "number": 346,
        "state": "closed",
        "merged": True,
        "merged_at": "2026-10-25T11:57:01Z",
        "merge_commit_sha": SOURCE,
        "base": {
            "ref": "master",
            "sha": "0561692fadef215cf8bec9752d4072a7cdff5960",
            "repo": {"full_name": REPOSITORY},
        },
        "head": {
            "ref": "release/v0.22.42",
            "sha": HEAD,
            "repo": {"full_name": REPOSITORY},
        },
    }


def pull_request_run() -> dict:
    return {
        "id": 40190998638,
        "run_attempt": 1,
        "path": WORKFLOW,
        "repository": {"full_name": REPOSITORY},
        "head_repository": {"full_name": REPOSITORY},
        "event": "pull_request",
        "head_branch": "release/v0.22.42",
        "head_sha": HEAD,
        "status": "completed",
        "conclusion": "cancelled",
        "created_at": "2026-10-25T12:08:11Z",
    }


def direct_run() -> dict:
    return {
        "id": 40672472894,
        "run_attempt": 1,
        "path": WORKFLOW,
        "repository": {"full_name": REPOSITORY},
        "head_repository": {"full_name": REPOSITORY},
        "event": "workflow_dispatch",
        "head_branch": "master",
        "head_sha": DIRECT_SOURCE,
        "status": "completed",
        "conclusion": "failure",
    }


class PostpublicationSourceBindingTests(unittest.TestCase):
    def test_tag_target_must_equal_actual_product_source(self) -> None:
        self.assertEqual(verify_tag_source({"sha": SOURCE}, SOURCE), SOURCE)
        with self.assertRaisesRegex(ValueError, "differs"):
            verify_tag_source({"sha": HEAD}, SOURCE)

    def test_actual_merged_pull_request_binds_merge_M_to_head_H(self) -> None:
        binding = bind_merged_pull_request(pull_request_record(), SOURCE, VERSION)
        self.assertEqual(binding.product_commit, SOURCE)
        self.assertEqual(binding.tag, VERSION)
        self.assertEqual(binding.run_head_sha, HEAD)
        self.assertEqual(binding.pull_request_number, 346)
        run_binding = bind_workflow_run(pull_request_run(), SOURCE, VERSION, binding)
        self.assertEqual(run_binding.kind, "merged_pull_request")
        self.assertEqual(run_binding.product_commit, SOURCE)
        self.assertEqual(run_binding.tag, VERSION)
        self.assertEqual(run_binding.run_head_sha, HEAD)

    def test_direct_master_dispatch_remains_bound_to_product_commit(self) -> None:
        binding = bind_workflow_run(direct_run(), DIRECT_SOURCE, VERSION)
        self.assertEqual(binding.kind, "direct_master_dispatch")
        self.assertEqual(binding.product_commit, DIRECT_SOURCE)
        self.assertEqual(binding.run_head_sha, DIRECT_SOURCE)

    def test_rejects_wrong_tag_version_or_unmerged_or_mismatched_pull_request(self) -> None:
        for mutation in (
            {"merge_commit_sha": HEAD},
            {"merged": False},
            {"state": "open"},
            {"head": {"ref": "release/v0.22.43", "sha": HEAD,
                      "repo": {"full_name": REPOSITORY}}},
            {"base": {"ref": "main", "sha": SOURCE,
                      "repo": {"full_name": REPOSITORY}}},
        ):
            with self.subTest(mutation=mutation):
                record = deepcopy(pull_request_record())
                record.update(mutation)
                with self.assertRaises(ValueError):
                    bind_merged_pull_request(record, SOURCE, VERSION)

    def test_rejects_foreign_pull_request_repositories_and_ambiguous_binding(self) -> None:
        for side in ("head", "base"):
            record = deepcopy(pull_request_record())
            record[side]["repo"]["full_name"] = "Other/Repo"
            with self.subTest(side=side), self.assertRaisesRegex(ValueError, "repository"):
                bind_merged_pull_request(record, SOURCE, VERSION)
        with self.assertRaisesRegex(ValueError, "ambiguous"):
            select_merged_pull_request(
                [pull_request_record(), pull_request_record() | {"number": 347}],
                SOURCE,
                VERSION,
            )

    def test_rejects_pr_run_before_merge_or_from_wrong_source_branch(self) -> None:
        binding = bind_merged_pull_request(pull_request_record(), SOURCE, VERSION)
        with self.assertRaises(PreMergePullRequestRun):
            bind_workflow_run(
                pull_request_run() | {"created_at": "2026-10-25T11:56:59Z"},
                SOURCE,
                VERSION,
                binding,
            )
        for mutation in ({"head_sha": SOURCE}, {"head_branch": "release/v0.22.43"},
                         {"head_repository": {"full_name": "Other/Repo"}}):
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                bind_workflow_run(pull_request_run() | mutation, SOURCE, VERSION, binding)

    def test_rejects_direct_run_on_non_master_or_wrong_source(self) -> None:
        for mutation in ({"head_branch": "release/v0.22.42"}, {"head_sha": HEAD},
                         {"event": "pull_request"}):
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                bind_workflow_run(direct_run() | mutation, DIRECT_SOURCE, VERSION)


if __name__ == "__main__":
    unittest.main()
