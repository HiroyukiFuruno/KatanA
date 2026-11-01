"""v46 の段階別条件と preflight の実際の分岐を検証する。"""

from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
RELEASE = ROOT / "scripts/release"
CHANGE = "v0-22-46-startup-png-regex"
ALLOWED = {
    "strict": set(),
    "pr-bootstrap": {"5.2", "5.3", "5.4", "5.5", "5.6"},
    "release-artifact-pending": {"5.4", "5.5", "5.6"},
    "post-release-evidence": {"5.6"},
}
IDS = {"1.1", "2.1", "3.1", "3.2", "5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "9.9"}


def tasks(incomplete):
    return "".join(f"- [{' ' if task in incomplete else 'x'}] {task} task\n" for task in sorted(IDS))


class Phase46Tests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.release = self.root / "scripts/release"
        self.release.mkdir(parents=True)
        for name in ("check-openspec-task-phase.py", "check-openspec-task-completion.py"):
            shutil.copyfile(RELEASE / name, self.release / name)
        self.directory = self.root / "openspec/changes" / CHANGE
        self.directory.mkdir(parents=True)
        self.canonical = self.directory / "tasks.md"

    def tearDown(self):
        self.temporary.cleanup()

    def check(self, mode, incomplete, *, version="0.22.46", change=CHANGE, path=None):
        self.canonical.write_text(tasks(incomplete))
        return subprocess.run([sys.executable, str(self.release / "check-openspec-task-phase.py"),
                               version, change, mode, str(path or self.canonical)],
                              capture_output=True, text=True, timeout=60)

    def test_only_the_designated_tasks_are_allowed_in_each_phase(self):
        for mode, allowed in ALLOWED.items():
            with self.subTest(mode=mode):
                result = self.check(mode, allowed)
                self.assertEqual(0, result.returncode, result.stderr)
            for task in IDS - allowed:
                with self.subTest(mode=mode, forbidden=task):
                    self.assertNotEqual(0, self.check(mode, allowed | {task}).returncode)

    def test_exact_version_change_path_and_complete_reviews_are_required(self):
        for version in ("0.22.45", "0.22.47", "v0.22.46"):
            self.assertNotEqual(0, self.check("strict", set(), version=version).returncode)
        self.assertNotEqual(0, self.check("strict", set(), change=CHANGE + "-extra").returncode)
        other = self.root / "other.md"
        other.write_text(tasks(set()))
        self.assertNotEqual(0, self.check("strict", set(), path=other).returncode)
        self.canonical.write_text("- [/] 2.1 review incomplete\n")
        result = subprocess.run([sys.executable, str(self.release / "check-openspec-task-phase.py"),
                                 "0.22.46", CHANGE, "pr-bootstrap", str(self.canonical)],
                                capture_output=True, text=True, timeout=60)
        self.assertNotEqual(0, result.returncode)

    def preflight(self, mode):
        source = (RELEASE / "preflight.sh").read_text(encoding="utf-8")
        dispatch = "VERSION_DASHED=$(" + source.split("VERSION_DASHED=$(", 1)[1].split(
            'success "All preflight checks passed', 1)[0]
        script = self.root / "preflight-task-dispatch.zsh"
        script.write_text('set -e\nVERSION=0.22.46\nTASK_GATE_MODE="$1"\n'
                          'error() { print -u2 -- "$*"; }\nsuccess() { print -- "$*"; }\n' + dispatch)
        return subprocess.run(["zsh", str(script), mode], cwd=self.root,
                              capture_output=True, text=True, timeout=60)

    def test_preflight_dispatch_enforces_each_phase_and_rejects_unbound_change(self):
        for mode, allowed in ALLOWED.items():
            self.canonical.write_text(tasks(allowed))
            result = self.preflight(mode)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            self.assertIn(f"OpenSpec change '{CHANGE}' satisfies", result.stdout)
            self.canonical.write_text(tasks(allowed | {"2.1"}))
            self.assertNotEqual(0, self.preflight(mode).returncode)
        unbound = self.root / "openspec/changes/v0-22-46-unbound"
        unbound.mkdir()
        (unbound / "tasks.md").write_text(tasks(set()))
        self.canonical.write_text(tasks(ALLOWED["pr-bootstrap"]))
        self.assertNotEqual(0, self.preflight("pr-bootstrap").returncode)


if __name__ == "__main__":
    unittest.main()
