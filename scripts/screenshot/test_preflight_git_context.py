import subprocess
import unittest
from pathlib import Path

import test_source_identity as fixture_support


class PreflightGitContextTest(unittest.TestCase):
    def test_preflight_uses_its_worktree_before_foreign_git_children(self) -> None:
        case = fixture_support.SourceIdentityTest()
        case.setUp()
        self.addCleanup(case.tearDown)
        preflight = fixture_support.SCRIPT.parents[2] / "scripts/release/preflight.sh"
        source = preflight.read_text(encoding="utf-8")
        self.assertIn("\nRED=", source)
        prefix = source.split("\nRED=", maxsplit=1)[0]
        donor, git_environment = case.donor_repository()
        donor_before = case.donor_snapshot(donor)
        environment = fixture_support.sanitized_environment()
        environment.update(git_environment)
        result = subprocess.run(
            ["zsh", "-c", prefix + "\ngit rev-parse --absolute-git-dir\n", str(preflight)],
            cwd=case.repo, env=environment, capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(Path(result.stdout.strip()).resolve(), (case.repo / ".git").resolve())
        self.assertEqual(case.donor_snapshot(donor), donor_before)


if __name__ == "__main__":
    unittest.main()
