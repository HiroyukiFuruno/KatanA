"""Keep the actual Git capability prerequisite ahead of evidence checks."""

from pathlib import Path
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[2]
CHECK = "scripts/release/check-git-capability.sh"


class GitToolchainContractTests(unittest.TestCase):
    def test_current_git_supports_the_required_no_fetch_capability(self):
        subprocess.run(["bash", str(ROOT / CHECK)], check=True, capture_output=True)

    def test_preflight_checks_capability_before_test_discovery(self):
        source = (ROOT / "scripts/release/preflight.sh").read_text()
        self.assertLess(source.index(f"bash {CHECK}"),
                        source.index("python3 -m unittest discover"))
        self.assertEqual(source.count(f"bash {CHECK}"), 1)

    def test_setup_checks_active_git_before_installing_other_tools(self):
        source = (ROOT / "scripts/setup/setup.sh").read_text()
        check = f'bash "${{REPO_ROOT}}/{CHECK}"'
        self.assertGreater(source.index(check), source.index("brew install git"))
        self.assertLess(source.index(check), source.index("# 2b. just"))
