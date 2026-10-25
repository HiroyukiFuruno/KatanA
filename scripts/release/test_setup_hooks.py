from __future__ import annotations

import importlib.util
import os
import sys
import tempfile
import unittest
from pathlib import Path


HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("setup_hooks", HERE / "setup_hooks.py")
SETUP_HOOKS = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.path.insert(0, str(HERE))
SPEC.loader.exec_module(SETUP_HOOKS)


class SetupHookTests(unittest.TestCase):
    def test_normal_setup_installs_dispatcher_after_lefthook(self) -> None:
        setup = (HERE.parent / "setup" / "setup.sh").read_text(encoding="utf-8")
        self.assertIn('KATANA_RELEASE_REPO:-HiroyukiFuruno/KatanA', setup)
        self.assertIn(
            'python3 -B "${REPO_ROOT}/scripts/release/setup_hooks.py"',
            setup,
        )


class SetupHookFileTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.hook = Path(self.temp.name) / "pre-push"
        self.backup = Path(self.temp.name) / "pre-push.katana-existing"

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_restore_dispatcher_preserves_snapshot_bytes_and_mode(self) -> None:
        snapshot = (b"old dispatcher\n", 0o751)
        generated = (b"generated owner\n", 0o755)
        self.hook.write_bytes(generated[0])
        os.chmod(self.hook, generated[1])
        SETUP_HOOKS.restore_dispatcher(self.hook, snapshot, generated)
        self.assertEqual(self.hook.read_bytes(), snapshot[0])
        self.assertEqual(self.hook.stat().st_mode & 0o777, snapshot[1])

    def test_restore_refuses_changed_bytes_without_overwrite(self) -> None:
        generated = (b"generated owner\n", 0o755)
        changed = b"concurrent writer\n"
        self.hook.write_bytes(changed)
        os.chmod(self.hook, generated[1])
        with self.assertRaises(RuntimeError):
            SETUP_HOOKS.restore_dispatcher(self.hook, (b"old\n", 0o751), generated)
        self.assertEqual(self.hook.read_bytes(), changed)

    def test_restore_refuses_changed_mode_without_overwrite(self) -> None:
        generated = (b"generated owner\n", 0o755)
        self.hook.write_bytes(generated[0])
        os.chmod(self.hook, 0o700)
        with self.assertRaises(RuntimeError):
            SETUP_HOOKS.restore_dispatcher(self.hook, (b"old\n", 0o751), generated)
        self.assertEqual(self.hook.read_bytes(), generated[0])
        self.assertEqual(self.hook.stat().st_mode & 0o777, 0o700)

    def test_snapshot_rejects_unknown_wrapper_and_unsafe_symlink(self) -> None:
        self.hook.write_bytes(b"# KATANA_PRE_PUSH_DISPATCHER_V1\n")
        with self.assertRaises(ValueError):
            SETUP_HOOKS.snapshot_dispatcher("ACME/KatanA", self.hook, self.backup)
        target = Path(self.temp.name) / "foreign"
        target.write_text("foreign\n", encoding="utf-8")
        self.hook.unlink()
        self.hook.symlink_to(target)
        with self.assertRaises(ValueError):
            SETUP_HOOKS.snapshot_dispatcher("ACME/KatanA", self.hook, self.backup)


if __name__ == "__main__":
    unittest.main()
