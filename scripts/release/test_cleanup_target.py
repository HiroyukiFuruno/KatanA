"""Pure PR identity selection; not live publication or branch deletion proof."""

import copy
import unittest

from cleanup_target import select_target
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


if __name__ == "__main__":
    unittest.main()
