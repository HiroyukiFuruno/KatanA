"""Actual Lefthook selection and fixed Just wiring, not production gate evidence."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from owner_verification import VerificationSession

HERE = Path(__file__).resolve().parent
REPOSITORY_ROOT = HERE.parent.parent


class OwnedHookSelectionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        self.environment = {key: value for key, value in os.environ.items()
                            if not key.startswith(("GIT_", "LEFTHOOK", "KATANA_OWNER_CHECK"))}
        subprocess.run(["git", "init", "-q", "-b", "release/v0.22.42", str(self.root)], check=True, env=self.environment)
        source = self.root / "scripts/release"
        source.mkdir(parents=True)
        for name in ("owner_verification.py", "run-owned-push-check.py"):
            shutil.copy2(HERE / name, source / name)
        ci = self.root / "scripts/ci"
        ci.mkdir(parents=True)
        shutil.copy2(REPOSITORY_ROOT / "scripts/ci/cargo_target_lease.py", ci / "cargo_target_lease.py")
        shutil.copy2(REPOSITORY_ROOT / "lefthook.yml", self.root / "lefthook.yml")
        (self.root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["crates/fixture"]\nresolver = "2"\n'
        )
        crate = self.root / "crates/fixture"
        crate.mkdir(parents=True)
        (crate / "Cargo.toml").write_text(
            '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2021"\n'
        )
        (crate / "src").mkdir()
        (crate / "src/lib.rs").write_text("pub fn fixture() {}\n")
        subprocess.run(["cargo", "generate-lockfile", "--manifest-path", str(self.root / "Cargo.toml")],
                       check=True, env=self.environment, capture_output=True)
        (self.root / "Justfile").write_text("check:\n    @printf 'fixed owner command executed\\n'\n")
        subprocess.run(["git", "add", "."], cwd=self.root, check=True, env=self.environment)
        subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture"],
                       cwd=self.root, check=True, env=self.environment)
        self.revision = subprocess.run(["git", "rev-parse", "HEAD"], cwd=self.root, check=True,
                                       capture_output=True, text=True, env=self.environment).stdout.strip()

    def tearDown(self):
        self.temporary.cleanup()

    def run_hook(self, session, path, extra_environment=None):
        environment = self.environment | {key: value for key, value in session.environment().items()
                                          if key.startswith("KATANA_OWNER_CHECK")}
        environment.update(extra_environment or {})
        environment["LEFTHOOK_EXCLUDE"] = "pr-ready-check,check-light," + environment.get("LEFTHOOK_EXCLUDE", "")
        return subprocess.run(["lefthook", "run", "pre-push", "--file", path,
                               "--no-auto-install", "--no-tty"], cwd=self.root, env=environment,
                              capture_output=True, text=True)

    def test_root_and_independent_dependency_graphs_issue_receipts(self):
        for path in ("Cargo.toml", "Cargo.lock", "scripts/screenshot/Cargo.toml",
                     "scripts/paint-metrics/Cargo.lock", "crates/katana-ui/src/lib.rs", "lefthook.yml"):
            with self.subTest(path=path), VerificationSession(self.root, self.revision) as session:
                result = self.run_hook(session, path)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(session.verified_source(), self.revision, result.stdout)

    def test_docs_only_selection_does_not_claim_quality_execution(self):
        with VerificationSession(self.root, self.revision) as session:
            result = self.run_hook(session, "docs/release-notes.md")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIsNone(session.verified_source())

    def test_excluded_quality_command_never_claims_execution(self):
        with VerificationSession(self.root, self.revision) as session:
            result = self.run_hook(session, "Cargo.lock", {"LEFTHOOK_EXCLUDE": "check-strict"})
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIsNone(session.verified_source())


if __name__ == "__main__":
    unittest.main()
