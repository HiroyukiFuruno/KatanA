"""Exercise real archive bytes and identity metadata for generated bump evidence."""
import hashlib
import io
import json
import unittest
import zipfile

from manual_bump_artifact import manual_steps_succeeded, validate_artifact, validate_commit_metadata
from postpublication_run_admission import _is_release_pull_request
from postpublication_source_binding import bind_workflow_run


def archive(files):
    result = io.BytesIO()
    with zipfile.ZipFile(result, "w") as package:
        for name, data in files:
            package.writestr(name, data)
    return result.getvalue()


class ArtifactTest(unittest.TestCase):
    def setUp(self):
        self.run = {"id": 10, "run_attempt": 2, "head_sha": "a" * 40}
        self.data = archive([("transition.json", json.dumps({"source": "b" * 40}))])
        self.artifact = {"name": "manual-bump-transition-2", "id": 20, "expired": False,
                         "digest": "sha256:" + hashlib.sha256(self.data).hexdigest(),
                         "workflow_run": {"id": 10, "head_sha": "a" * 40}}

    def test_matching_real_archive_is_accepted(self):
        self.assertEqual(validate_artifact(self.artifact, self.run, self.data)[0]["source"], "b" * 40)

    def test_wrong_run_attempt_digest_and_extra_files_are_rejected(self):
        for artifact in (self.artifact | {"name": "manual-bump-transition-1"},
                         self.artifact | {"digest": "sha256:" + "0" * 64},
                         self.artifact | {"workflow_run": {"id": 11, "head_sha": "a" * 40}}):
            with self.assertRaises(ValueError):
                validate_artifact(artifact, self.run, self.data)
        altered = archive([("transition.json", "{}"), ("extra", "bad")])
        artifact = self.artifact | {"digest": "sha256:" + hashlib.sha256(altered).hexdigest()}
        with self.assertRaises(ValueError):
            validate_artifact(artifact, self.run, altered)
        duplicate = archive([("transition.json", '{"source":"one","source":"two"}')])
        artifact = self.artifact | {"digest": "sha256:" + hashlib.sha256(duplicate).hexdigest()}
        with self.assertRaises(ValueError):
            validate_artifact(artifact, self.run, duplicate)

    def test_every_real_preflight_step_is_required(self):
        steps = [{"name": name, "status": "completed", "conclusion": "success"}
                 for name in ("Bump version and push", "Record manual bump transition", "Upload manual bump transition")]
        manual_steps_succeeded([{"name": "Preflight Check", "steps": steps}])
        for index in range(len(steps)):
            invalid = [step.copy() for step in steps]
            invalid[index]["conclusion"] = "skipped"
            with self.subTest(index=index), self.assertRaises(ValueError):
                manual_steps_succeeded([{"name": "Preflight Check", "steps": invalid}])

    def test_commit_metadata_requires_verified_exact_parent(self):
        record = {"sha": "b" * 40, "parents": [{"sha": "a" * 40}],
                  "commit": {"verification": {"verified": True}}}
        validate_commit_metadata([record], ["b" * 40], "a" * 40)
        for bad in (record | {"sha": "c" * 40}, record | {"parents": [{"sha": "d" * 40}]},
                    record | {"commit": {"verification": {"verified": False}}}):
            with self.assertRaises(ValueError):
                validate_commit_metadata([bad], ["b" * 40], "a" * 40)

    def test_nonrelease_pr_is_not_misclassified_as_release_binding(self):
        source = "b" * 40
        pr = {"merge_commit_sha": source, "head": {"ref": "dependabot/github_actions/example"}}
        self.assertFalse(_is_release_pull_request(pr, source))
        run = {"id": 123, "run_attempt": 1, "head_sha": source, "event": "workflow_dispatch",
               "head_branch": "master", "path": ".github/workflows/build-and-release.yml",
               "status": "completed", "conclusion": "success",
               "repository": {"full_name": "HiroyukiFuruno/KatanA"},
               "head_repository": {"full_name": "HiroyukiFuruno/KatanA"}}
        self.assertEqual(bind_workflow_run(run, source, "v0.1.1").kind, "direct_master_dispatch")
        self.assertTrue(_is_release_pull_request(pr | {"head": {"ref": "release/v0.1.1"}}, source))
        self.assertTrue(_is_release_pull_request(pr | {"head": None}, source))


if __name__ == "__main__":
    unittest.main()
