"""Real temporary Git/tree/archive contracts; no GitHub or product mutation."""
import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

import v043_postpublication_scope as policy

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("v043_scope_cli", ROOT / "scripts/release/check-v043-postpublication-scope.py")
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)
TASKS = "- [x] 1.1 implemented\n- [x] 8.4 merged\n- [ ] 8.5 publication\n- [ ] 8.6 archive\n"


class ScopeGitTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="v043-scope-", dir=ROOT / "tmp")
        self.root = Path(self.temporary.name)
        self.environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        self.environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
        self.git("init", "--quiet", "--initial-branch=" + policy.BRANCH)
        self.originals = {name: b"v42 preserved\n" + anchor + b"strict preserved\n"
                          for name, (anchor, _insert) in policy.ROUTE_EDITS.items()}
        for name, content in self.originals.items():
            self.write(name, content)
        for name in ("check-openspec-task-phase.py", "check-openspec-task-completion.py"):
            self.write("scripts/release/" + name, (ROOT / "scripts/release" / name).read_bytes())
        self.write(policy.ACTIVE + "tasks.md", TASKS.encode())
        self.write("Cargo.lock", b"immutable product lock\n")
        self.commit()
        self.published = self.git("rev-parse", "HEAD").strip().decode()
        for name, content in self.originals.items():
            self.write(name, policy.expected_controller(name, content))
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

    def commit(self, stage=True):
        if stage:
            self.git("add", ".")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                 "commit", "--quiet", "-m", "範囲検査fixture")

    def test_committed_active_scope_reuses_existing_phase_checker(self):
        CHECK.verify_committed_scope(self.root, self.published)
        self.assertEqual(CHECK.verify_tasks(self.root, self.published), policy.ACTIVE + "tasks.md")
        self.write(policy.ACTIVE + "tasks.md", TASKS.replace("[x] 8.4", "[ ] 8.4").encode())
        self.commit()
        with self.assertRaises(subprocess.CalledProcessError):
            CHECK.verify_tasks(self.root, self.published)

    def test_real_archive_move_is_explicitly_checked_even_with_empty_active_glob(self):
        archive = f"openspec/changes/archive/2026-11-01-{policy.CHANGE}"
        target = self.root / archive
        target.parent.mkdir(parents=True)
        (self.root / policy.ACTIVE).rename(target)
        self.write(archive + "/tasks.md", TASKS.replace("[ ]", "[x]").encode())
        self.commit()
        CHECK.verify_committed_scope(self.root, self.published)
        self.assertEqual(CHECK.verify_tasks(self.root, self.published), archive + "/tasks.md")
        self.write(archive + "/tasks.md", TASKS.replace("[ ] 8.5", "[x] 8.5").encode())
        self.commit()
        with self.assertRaises(subprocess.CalledProcessError):
            CHECK.verify_tasks(self.root, self.published)

    def test_committed_product_and_routing_weakening_are_rejected(self):
        self.write("Cargo.lock", b"changed product dependency\n")
        self.commit()
        with self.assertRaises(ValueError):
            CHECK.verify_committed_scope(self.root, self.published)
        self.write("Cargo.lock", b"immutable product lock\n")
        path = "scripts/release/check-pr-ready.sh"
        self.write(path, policy.expected_controller(path, self.originals[path]) + b"weaken guard\n")
        self.commit()
        with self.assertRaises(ValueError):
            CHECK.verify_committed_scope(self.root, self.published)

    def test_dirty_and_untracked_source_are_rejected(self):
        self.write("Cargo.lock", b"unstaged dependency edit\n")
        with self.assertRaises(ValueError):
            CHECK.verify_committed_scope(self.root, self.published)
        self.write("Cargo.lock", b"immutable product lock\n")
        self.write("unexpected.rs", b"untracked product source")
        with self.assertRaises(ValueError):
            CHECK.verify_committed_scope(self.root, self.published)

    def test_evidence_symlink_and_gitlink_tree_entries_are_rejected(self):
        result = subprocess.run(["git", "-C", str(self.root), "hash-object", "-w", "--stdin"],
                                input=b"../../Cargo.lock", check=True, capture_output=True,
                                env=self.environment)
        blob = result.stdout.strip().decode()
        path = policy.EVIDENCE + "publication.md"
        for mode, oid in (("120000", blob), ("160000", self.published)):
            self.git("update-index", "--add", "--cacheinfo", f"{mode},{oid},{path}")
            self.commit(stage=False)
            with self.subTest(mode=mode), self.assertRaises(ValueError):
                CHECK.verify_committed_scope(self.root, self.published)

    def test_missing_and_duplicate_task_locations_fail(self):
        active = self.root / policy.ACTIVE / "tasks.md"
        active.rename(self.root / "held-task.txt")
        self.commit()
        with self.assertRaises(ValueError):
            CHECK.verify_tasks(self.root, self.published)
        self.write(policy.ACTIVE + "tasks.md", TASKS.encode())
        archive = f"openspec/changes/archive/2026-11-01-{policy.CHANGE}/tasks.md"
        self.write(archive, TASKS.replace("[ ]", "[x]").encode())
        self.commit()
        with self.assertRaises(ValueError):
            CHECK.verify_tasks(self.root, self.published)

    def test_non_ancestor_source_is_rejected(self):
        tree = self.git("write-tree").strip().decode()
        result = subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Fixture",
                                 "-c", "user.email=fixture@example.invalid", "commit-tree", tree,
                                 "-m", "別履歴fixture"], check=True, capture_output=True,
                                env=self.environment)
        unrelated = result.stdout.strip().decode()
        with self.assertRaises(subprocess.CalledProcessError):
            CHECK.verify_committed_scope(self.root, unrelated)

    def test_task_locations_come_from_head_not_staged_future_archive(self):
        archive = f"openspec/changes/archive/2026-11-01-{policy.CHANGE}/tasks.md"
        self.write(archive, TASKS.replace("[ ]", "[x]").encode())
        self.git("add", ".")
        self.assertEqual(CHECK.task_candidates(self.root), [policy.ACTIVE + "tasks.md"])
        with self.assertRaises(ValueError):
            CHECK.verify_committed_scope(self.root, self.published)


if __name__ == "__main__":
    unittest.main()
