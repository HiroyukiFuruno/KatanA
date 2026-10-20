"""Native Git input capture; no user repository or new worktree mutation."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from native_test_run_identity import _version, environment_identity, file_digest, source_identity


class RunIdentityTests(unittest.TestCase):
    def version_tools(self, root: Path) -> dict[str, str]:
        tools = root / "tools"
        tools.mkdir()
        cargo = tools / "cargo"
        cargo.write_text("#!/bin/sh\nprintf 'cargo fixture-%s\\n' \"$RUSTUP_TOOLCHAIN\"\n",
                          encoding="utf-8")
        cargo.chmod(0o755)
        rustc = tools / "rustc fixture"
        rustc.write_text("#!/bin/sh\nprintf 'rustc cwd=%s flags=%s\\n' \"$PWD\" \"$RUSTFLAGS\"\n",
                          encoding="utf-8")
        rustc.chmod(0o755)
        return {"PATH": str(tools), "RUSTC": str(rustc)}

    def test_environment_versions_use_execution_cwd_and_environment(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            environment = self.version_tools(root)
            environment.update({"CARGO": "ignored-cargo-wrapper", "RUSTUP_TOOLCHAIN": "fixture-toolchain",
                                "RUSTFLAGS": "fixture-flags"})
            observed = environment_identity(root, environment)
            self.assertEqual(observed["cargo"], "cargo fixture-fixture-toolchain")
            self.assertEqual(observed["rustc"], f"rustc cwd={root} flags=fixture-flags")

    def test_version_failure_is_unknown_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            self.assertIsNone(_version([str(root / "missing"), "-V"], root, dict(os.environ)))

    def repository(self, root: Path) -> None:
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        for arguments in (("init", "-q"), ("config", "user.name", "Inventory Test"),
                          ("config", "user.email", "inventory@example.invalid")):
            subprocess.run(["git", "-C", str(root), *arguments], env=environment, check=True)
        (root / "scripts/screenshot").mkdir(parents=True)
        for name in ("Cargo.lock", "scripts/screenshot/Cargo.lock"):
            (root / name).write_text("version = 4\n", encoding="utf-8")
        subprocess.run(["git", "-C", str(root), "add", "."], env=environment, check=True)
        subprocess.run(["git", "-C", str(root), "-c", "commit.gpgsign=false",
                        "commit", "-qm", "fixture"],
                       env=environment, check=True)

    def test_clean_tree_and_both_locks_are_measured(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.repository(root)
            observed = source_identity(root)
            self.assertTrue(observed["clean"])
            self.assertEqual(len(observed["head"]), 40)
            self.assertEqual(len(observed["tree"]), 40)
            self.assertEqual(observed["locks"]["Cargo.lock"], file_digest(root / "Cargo.lock"))
            self.assertEqual(set(observed["locks"]), {"Cargo.lock", "scripts/screenshot/Cargo.lock"})

    def test_dirty_untracked_and_missing_lock_never_look_clean(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.repository(root)
            (root / "Cargo.lock").write_text("version = 3\n", encoding="utf-8")
            self.assertFalse(source_identity(root)["clean"])
            (root / "untracked.rs").write_text("fn main() {}", encoding="utf-8")
            changed = source_identity(root)
            self.assertFalse(changed["clean"])
            (root / "scripts/screenshot/Cargo.lock").unlink()
            self.assertIn("unavailable", source_identity(root))

    def test_symlink_input_is_not_hashed_as_regular_lock(self) -> None:
        if os.name != "posix":
            return
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "real").write_text("input", encoding="utf-8")
            (root / "link").symlink_to(root / "real")
            with self.assertRaises(ValueError):
                file_digest(root / "link")


if __name__ == "__main__":
    unittest.main()
