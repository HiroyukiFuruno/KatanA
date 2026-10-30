"""Exact v45 task-phase contracts and the real preflight dispatch fragment."""

from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
RELEASE = ROOT / "scripts/release"
STORAGE = "v0-22-45-update-storage-preflight"
IDENTITY = "v0-22-45-build-identity"
EXPECTED = {
    STORAGE: {"strict": set(), "pr-bootstrap": {"3.3", "3.4", "3.5", "3.6"},
              "release-artifact-pending": {"3.5", "3.6"}, "post-release-evidence": {"3.6"}},
    IDENTITY: {"strict": set(), "pr-bootstrap": {"3.1", "3.2", "3.3", "3.4", "3.5"},
               "release-artifact-pending": {"3.2", "3.3", "3.4", "3.5"}, "post-release-evidence": {"3.5"}},
}
IDS = ("1.1", "1.2", "1.5", "2.1", "2.2", "2.3", "3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "9.9")


def tasks(incomplete):
    return "".join(f"- [{' ' if task in incomplete else 'x'}] {task} task\n" for task in IDS)


class Phase45Tests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.release = self.root / "scripts/release"
        self.release.mkdir(parents=True)
        for name in ("check-openspec-task-phase.py", "check-openspec-task-completion.py"):
            shutil.copyfile(RELEASE / name, self.release / name)
        for change in EXPECTED:
            path = self.root / "openspec/changes" / change
            path.mkdir(parents=True)

    def tearDown(self):
        self.temporary.cleanup()

    def check(self, change, mode, incomplete, *, version="0.22.45", path=None):
        canonical = self.root / "openspec/changes" / change / "tasks.md"
        canonical.write_text(tasks(incomplete))
        return subprocess.run([sys.executable, str(self.release / "check-openspec-task-phase.py"),
                               version, change, mode, str(path or canonical)],
                              capture_output=True, text=True, timeout=60)

    def test_each_pair_allows_only_exact_future_tasks_in_each_mode(self):
        for change, modes in EXPECTED.items():
            for mode, allowed in modes.items():
                with self.subTest(change=change, mode=mode):
                    result = self.check(change, mode, allowed)
                    self.assertEqual(0, result.returncode, result.stderr)
                for task in set(IDS) - allowed:
                    with self.subTest(change=change, mode=mode, forbidden=task):
                        self.assertNotEqual(0, self.check(change, mode, allowed | {task}).returncode)

    def test_implementation_review_partial_path_version_and_mode_cannot_be_waived(self):
        for change in EXPECTED:
            for version in ("0.22.43", "0.22.44", "0.22.46", "v0.22.45"):
                with self.subTest(change=change, version=version):
                    self.assertNotEqual(0, self.check(change, "pr-bootstrap", set(), version=version).returncode)
            elsewhere = self.root / "other-tasks.md"
            elsewhere.write_text(tasks(set()))
            self.assertNotEqual(0, self.check(change, "strict", set(), path=elsewhere).returncode)
            other = next(candidate for candidate in EXPECTED if candidate != change)
            other_path = self.root / "openspec/changes" / other / "tasks.md"
            other_path.write_text(tasks(set()))
            self.assertNotEqual(0, self.check(change, "strict", set(), path=other_path).returncode)
            self.assertNotEqual(0, self.check(change, "unknown", set()).returncode)
            canonical = self.root / "openspec/changes" / change / "tasks.md"
            canonical.write_text("- [/] 2.1 review incomplete\n")
            result = subprocess.run([sys.executable, str(self.release / "check-openspec-task-phase.py"),
                                     "0.22.45", change, "pr-bootstrap", str(canonical)],
                                    capture_output=True, text=True, timeout=60)
            self.assertNotEqual(0, result.returncode)

    def preflight(self, mode):
        source = (RELEASE / "preflight.sh").read_text()
        dispatch = source.split("VERSION_DASHED=$(", 1)[1]
        dispatch = "VERSION_DASHED=$(" + dispatch.split('success "All preflight checks passed', 1)[0]
        script = self.root / "preflight-task-dispatch.zsh"
        script.write_text('set -e\nVERSION=0.22.45\nTASK_GATE_MODE="$1"\n'
                          'error() { print -u2 -- "$*"; }\nsuccess() { print -- "$*"; }\n' + dispatch)
        return subprocess.run(["zsh", str(script), mode], cwd=self.root,
                              capture_output=True, text=True, timeout=60)

    def test_real_preflight_dispatch_checks_both_pairs_and_rejects_any_unbound_change(self):
        for change, modes in EXPECTED.items():
            (self.root / "openspec/changes" / change / "tasks.md").write_text(tasks(modes["pr-bootstrap"]))
        result = self.preflight("pr-bootstrap")
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        for change in EXPECTED:
            self.assertIn(f"OpenSpec change '{change}' satisfies", result.stdout)
        path = self.root / "openspec/changes" / IDENTITY / "tasks.md"
        path.write_text(tasks(EXPECTED[IDENTITY]["pr-bootstrap"] | {"2.2"}))
        self.assertNotEqual(0, self.preflight("pr-bootstrap").returncode)
        path.write_text(tasks(EXPECTED[IDENTITY]["pr-bootstrap"]))
        unbound = self.root / "openspec/changes/v0-22-45-unbound"
        unbound.mkdir()
        (unbound / "tasks.md").write_text(tasks(set()))
        self.assertNotEqual(0, self.preflight("pr-bootstrap").returncode)


if __name__ == "__main__":
    unittest.main()
