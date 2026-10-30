"""v44候補契約の拒否回帰。fixtureの成功は実公開一致の証明ではない。"""
from dataclasses import FrozenInstanceError
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

import v043_postpublication_scope as policy

ROOT = Path(__file__).resolve().parents[2]
BRANCH = "release/v0.22.44-public-release-evidence"
VERSION = "0.22.44"
CHANGE = "v0-22-44-document-loading-input-regressions"
SOURCE = "20bf151f40165084c8748b4cca8c7ab54ff6bdf2"
TASKS = "- [x] 1.2 implemented\n- [x] 1.3 verified\n- [x] 5.5 CI\n- [ ] 5.6 archive\n"
SPEC = importlib.util.spec_from_file_location("v044_scope_cli", ROOT / "scripts/release/check-v043-postpublication-scope.py")
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


def contract():
    return policy.select_contract(BRANCH, VERSION, policy.REPOSITORY)


class ScopePolicy44Test(unittest.TestCase):
    def test_exact_immutable_contract_does_not_broaden_43(self):
        selected = contract()
        self.assertEqual(selected.source, SOURCE)
        self.assertEqual(selected.capabilities, ("multi-format-document-preview",))
        self.assertIn("scripts/release/test_v044_postpublication_scope.py", selected.controllers)
        with self.assertRaises(FrozenInstanceError):
            selected.source = "a" * 40
        for branch, version, repository in ((BRANCH + "-extra", VERSION, policy.REPOSITORY),
                (BRANCH, "0.22.45", policy.REPOSITORY), (BRANCH, VERSION, "other/repo"),
                (policy.BRANCH, VERSION, policy.REPOSITORY)):
            with self.subTest(branch=branch, version=version), self.assertRaises(ValueError):
                policy.select_contract(branch, version, repository)
        with self.assertRaises(ValueError):
            policy.validate_route(BRANCH, VERSION, policy.REPOSITORY)
        self.assertEqual(policy.select_contract(policy.BRANCH, policy.VERSION, policy.REPOSITORY).source, policy.SOURCE)

    def test_candidate_source_requires_actual_stable_tag_identity(self):
        selected = contract()
        release = {"tag_name": "v0.22.44", "draft": False, "prerelease": False,
                   "published_at": "synthetic-only"}
        policy.validate_publication(release, SOURCE, SOURCE, selected)
        for key, value in (("tag_name", policy.TAG), ("draft", True),
                           ("prerelease", True), ("published_at", None)):
            with self.subTest(key=key), self.assertRaises(ValueError):
                policy.validate_publication(release | {key: value}, SOURCE, SOURCE, selected)
        for api_source, local_source in ((policy.SOURCE, SOURCE), (SOURCE, "b" * 40)):
            with self.assertRaises(ValueError):
                policy.validate_publication(release, api_source, local_source, selected)

    def test_paths_and_controller_additions_are_exact(self):
        selected = contract()
        inventory = {selected.active + "tasks.md"}
        policy.validate_paths([*selected.controllers, *inventory,
            "openspec/specs/multi-format-document-preview/spec.md"], inventory, selected)
        for path in ("Cargo.toml", "Cargo.lock", "scripts/screenshot/Cargo.lock", "crates/runtime.rs",
                "scripts/release/cleanup_target.py", "scripts/release/sync-external.sh",
                "openspec/specs/docker-resource-lifecycle/spec.md", policy.ACTIVE + "tasks.md",
                policy.EVIDENCE + "publication.md", f"openspec/changes/archive/2026-11-01-{policy.CHANGE}/tasks.md",
                selected.active + "new.json", "evidence/releases/v0.22.44/../other.json"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                policy.validate_paths([path], inventory, selected)
        for path, anchor, addition in selected.routes:
            baseline = b"preserved v43\n" + anchor + b"strict preserved\n"
            changed = policy.expected_controller(path, baseline, selected)
            self.assertIn(addition, changed)
            policy.validate_controller(path, baseline, changed, selected)
            with self.assertRaises(ValueError):
                policy.validate_controller(path, baseline, changed + b"weaken\n", selected)


class ScopeGit44Test(unittest.TestCase):
    def setUp(self):
        self.contract = contract()
        self.temporary = tempfile.TemporaryDirectory(prefix="v044-scope-", dir=ROOT / "tmp")
        self.root = Path(self.temporary.name)
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        self.environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
        self.git("init", "--quiet", "--initial-branch=" + BRANCH)
        self.originals = {path: b"v43 preserved\n" + anchor for path, anchor, _addition in self.contract.routes}
        for path, content in self.originals.items():
            self.write(path, content)
        for name in ("check-openspec-task-phase.py", "check-openspec-task-completion.py"):
            self.write("scripts/release/" + name, (ROOT / "scripts/release" / name).read_bytes())
        self.write(self.contract.active + "tasks.md", TASKS.encode())
        self.write("Cargo.lock", b"immutable product lock\n")
        self.commit()
        self.published = self.git("rev-parse", "HEAD").strip().decode()
        for path, content in self.originals.items():
            self.write(path, policy.expected_controller(path, content, self.contract))
        self.commit()

    def tearDown(self):
        self.temporary.cleanup()

    def git(self, *arguments):
        return subprocess.run(["git", "-C", str(self.root), *arguments], check=True,
                              capture_output=True, env=self.environment).stdout

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)

    def commit(self):
        self.git("add", ".")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                 "commit", "--quiet", "-m", "範囲検査fixture")

    def verify_tasks(self):
        return CHECK.verify_tasks(self.root, self.published, self.contract)

    def test_active_pending_only_defers_5_6_and_43_defaults_are_preserved(self):
        CHECK.verify_committed_scope(self.root, self.published, self.contract)
        self.assertEqual(self.verify_tasks(), self.contract.active + "tasks.md")
        self.write(self.contract.active + "tasks.md", TASKS.replace("[x] 1.2", "[ ] 1.2").encode())
        self.commit()
        with self.assertRaises(subprocess.CalledProcessError):
            self.verify_tasks()

    def test_real_archive_is_strict_and_preserves_task_inventory(self):
        archive = f"openspec/changes/archive/2026-11-01-{CHANGE}"
        target = self.root / archive
        target.parent.mkdir(parents=True)
        (self.root / self.contract.active).rename(target)
        self.write(archive + "/tasks.md", TASKS.replace("[ ]", "[x]").encode())
        self.commit()
        CHECK.verify_committed_scope(self.root, self.published, self.contract)
        self.assertEqual(self.verify_tasks(), archive + "/tasks.md")
        for text in (TASKS, TASKS.replace("[ ]", "[x]") + "- [x] 9.9 extra\n"):
            self.write(archive + "/tasks.md", text.encode())
            self.commit()
            with self.assertRaises((ValueError, subprocess.CalledProcessError)):
                self.verify_tasks()

    def test_product_change_and_symlink_are_rejected(self):
        self.write("Cargo.lock", b"changed product lock\n")
        self.commit()
        with self.assertRaises(ValueError):
            CHECK.verify_committed_scope(self.root, self.published, self.contract)
        self.write("Cargo.lock", b"immutable product lock\n")
        evidence = self.root / self.contract.evidence / "publication.md"
        evidence.parent.mkdir(parents=True)
        evidence.symlink_to("../../Cargo.lock")
        self.commit()
        with self.assertRaises(ValueError):
            CHECK.verify_committed_scope(self.root, self.published, self.contract)

    def test_duplicate_missing_and_changed_task_inventories_are_rejected(self):
        archive = f"openspec/changes/archive/2026-11-01-{CHANGE}/tasks.md"
        self.write(archive, TASKS.replace("[ ]", "[x]").encode())
        self.commit()
        with self.assertRaises(ValueError):
            self.verify_tasks()
        for text in (b"", TASKS.replace("- [x] 1.2 implemented\n", "").encode()):
            policy.validate_task_inventory(TASKS, TASKS, self.contract)
            with self.assertRaises(ValueError):
                policy.validate_task_inventory(TASKS, text.decode(), self.contract)
        with self.assertRaises(ValueError):
            policy.select_tasks([], self.contract)

    def test_missing_committed_task_location_is_rejected(self):
        (self.root / self.contract.active / "tasks.md").rename(self.root / "held-task.txt")
        self.commit()
        with self.assertRaises(ValueError):
            self.verify_tasks()


if __name__ == "__main__":
    unittest.main()
