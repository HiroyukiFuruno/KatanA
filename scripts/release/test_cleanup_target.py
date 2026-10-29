"""Pure PR identity selection; not live publication or branch deletion proof."""

import copy
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

import cleanup_target

from cleanup_target import resolve_target, select_target
from release_cleanup_audit import CleanupRefused

REPOSITORY = "HiroyukiFuruno/KatanA"


class CleanupTargetTests(unittest.TestCase):
    def record(self):
        return {"number": 346, "state": "closed", "merged_at": "2026-10-15T00:00:00Z",
                "merge_commit_sha": "b" * 40,
                "head": {"ref": "release/v0.22.42", "sha": "a" * 40,
                         "repo": {"full_name": REPOSITORY}},
                "base": {"ref": "master", "repo": {"full_name": REPOSITORY}}}

    def test_branch_tip_is_not_replaced_with_merge_or_publication_sha(self):
        result = select_target([self.record()], REPOSITORY, "master", "0.22.42")
        self.assertEqual(result["expected"], "a" * 40)
        self.assertEqual(result["merge_commit"], "b" * 40)
        self.assertEqual(result["branch"], "release/v0.22.42")

    def test_missing_or_ambiguous_target_is_refused(self):
        for values in ([], [self.record(), self.record()]):
            with self.subTest(values=values), self.assertRaises(CleanupRefused):
                select_target(values, REPOSITORY, "master", "0.22.42")

    def test_other_version_base_open_and_unmerged_prs_are_not_selected(self):
        cases = [("head", "ref", "release/v0.22.41"), ("base", "ref", "other")]
        for side, key, value in cases:
            record = self.record()
            record[side][key] = value
            with self.assertRaises(CleanupRefused):
                select_target([record], REPOSITORY, "master", "0.22.42")
        for key, value in (("state", "open"), ("merged_at", None)):
            record = self.record() | {key: value}
            with self.assertRaises(CleanupRefused):
                select_target([record], REPOSITORY, "master", "0.22.42")

    def test_fork_or_foreign_base_is_refused(self):
        for side in ("head", "base"):
            record = self.record()
            record[side]["repo"] = {"full_name": "other/KatanA"}
            with self.assertRaises(CleanupRefused):
                select_target([record], REPOSITORY, "master", "0.22.42")

    def test_noncanonical_commit_and_boolean_number_are_refused(self):
        for key, value in (("number", True), ("merge_commit_sha", "b" * 12)):
            with self.assertRaises(CleanupRefused):
                select_target([self.record() | {key: value}], REPOSITORY, "master", "0.22.42")
        record = copy.deepcopy(self.record())
        record["head"]["sha"] = "a" * 12
        with self.assertRaises(CleanupRefused):
            select_target([record], REPOSITORY, "master", "0.22.42")


    def publication_record(self):
        record = self.record()
        record.update(number=391, merge_commit_sha="0d139094a39887a7a5657ecdd63f53a6db012fde")
        record["head"].update(ref="release/v0.22.43-dependency-refresh",
                              sha="95ce239cab09b5c9991af2c7daec507395f42b72")
        return record

    def repository_record(self):
        return {"full_name": REPOSITORY, "private": False,
                "visibility": "public", "default_branch": "master"}

    def test_explicit_pr391_queries_only_that_pr_and_preserves_both_identities(self):
        record = self.publication_record()
        with patch.object(cleanup_target, "_gh_json", side_effect=[self.repository_record(), record]) as lookup:
            with patch.object(cleanup_target, "_associated", side_effect=AssertionError("unexpected associated query")):
                result = resolve_target(REPOSITORY, "0.22.43", record["merge_commit_sha"], 391)
        self.assertEqual([call.args[0] for call in lookup.call_args_list],
                         [f"/repos/{REPOSITORY}", f"/repos/{REPOSITORY}/pulls/391"])
        self.assertEqual(result, {"branch": record["head"]["ref"], "expected": record["head"]["sha"],
                                  "pull_request": 391, "merge_commit": record["merge_commit_sha"]})

    def test_explicit_pr_response_number_must_match(self):
        record = self.publication_record() | {"number": 389}
        with patch.object(cleanup_target, "_gh_json", side_effect=[self.repository_record(), record]):
            with self.assertRaisesRegex(CleanupRefused, "requested number"):
                resolve_target(REPOSITORY, "0.22.43", record["merge_commit_sha"], 391)

    def test_associated_lookup_retains_unique_selection_and_rejects_two_same_version_prs(self):
        record = self.publication_record()
        earlier = self.record()
        earlier["head"]["ref"] = "release/v0.22.43"
        for records in ([record], [earlier, record]):
            with patch.object(cleanup_target, "_gh_json", return_value=self.repository_record()):
                with patch.object(cleanup_target, "_associated", return_value=records) as lookup:
                    if len(records) == 1:
                        result = resolve_target(REPOSITORY, "0.22.43", record["merge_commit_sha"], None)
                        self.assertEqual(result["pull_request"], 391)
                    else:
                        with self.assertRaises(CleanupRefused):
                            resolve_target(REPOSITORY, "0.22.43", record["merge_commit_sha"], None)
                    self.assertEqual(lookup.call_args.args[:2], (REPOSITORY, record["merge_commit_sha"]))

    def test_wrong_version_prefix_and_invalid_suffix_are_not_selected(self):
        for branch in ("release/v0.22.44-dependency-refresh", "release/v0.22.430-refresh",
                       "release/v0.22.43refresh", "release/v0.22.43-", "release/v0.22.43-_bad",
                       "release/v0.22.43-refresh/task", "release/v0.22.43-refresh\n", "other/v0.22.43"):
            record = self.publication_record()
            record["head"]["ref"] = branch
            with self.subTest(branch=branch), self.assertRaises(CleanupRefused):
                select_target([record], REPOSITORY, "master", "0.22.43")
        with self.assertRaises(CleanupRefused):
            select_target([self.publication_record()], REPOSITORY, "master", "0.22.43-dependency-refresh")

    def test_branch_grammar_matches_the_existing_release_version_parser(self):
        script = Path(__file__).with_name("extract-release-branch-version.sh")
        branches = ("release/v0.22.43", "release/v0.22.43-dependency-refresh",
                    "release/v01.2.3-A_0.x-y", "release/v0.22.43-", "release/v0.22.43-_x",
                    "release/v0.22.43-x/y", "release/v0.22.43-x\n", "release/v0.22.43x", "release/fixture")
        for branch in branches:
            result = subprocess.run(["bash", str(script), branch], capture_output=True, check=False, timeout=5)
            expected = result.stdout.decode().strip() if result.returncode == 0 else None
            with self.subTest(branch=branch):
                self.assertEqual(cleanup_target.release_branch_version(branch), expected)

    def test_associated_candidate_limit_is_retained(self):
        with self.assertRaisesRegex(CleanupRefused, "candidate limit"):
            select_target([self.publication_record()] * (cleanup_target.MAX_ASSOCIATED_PRS + 1),
                          REPOSITORY, "master", "0.22.43")

if __name__ == "__main__":
    unittest.main()
