"""Exercise the default receipt and the guarded workflow boundary."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class CleanupOptInTests(unittest.TestCase):
    def test_default_creates_truthful_receipt_in_real_git_repository(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run([sys.executable, "-B", str(ROOT / "scripts/release/record-skipped-cleanup.py"),
                            "--controller", str(root), "--repository", "owner/repo",
                            "--version", "0.22.47", "--published", "a" * 40], check=True)
            receipt, = (root / ".git/katana-cleanup-receipts").glob("*.jsonl")
            rows = [json.loads(line) for line in receipt.read_text().splitlines()]
            self.assertEqual([("cleanup", "started"), ("remote", "not-requested"),
                              ("local", "not-requested"), ("cleanup", "completed")],
                             [(row["action"], row["state"]) for row in rows])
            self.assertFalse(rows[0]["detail"]["include_remote"])

    def test_workflow_requires_explicit_true_before_resolution(self):
        workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text()
        self.assertIn("cleanup_remote:\n        description:", workflow)
        self.assertIn("CLEANUP_REMOTE: ${{ github.event.inputs.cleanup_remote }}", workflow)
        block = workflow.split("      - name: Resolve and clean the published target", 1)[1].split(
            "      - name: Preserve cleanup receipts", 1)[0]
        gate = block.index('case "$CLEANUP_REMOTE" in')
        skipped = block.index("record-skipped-cleanup.py")
        done = block.index("exit 0", skipped)
        resolver = block.index("resolve-cleanup-target.py")
        deletion = block.index("--remote", resolver)
        self.assertLess(gate, skipped)
        self.assertLess(skipped, done)
        self.assertLess(done, resolver)
        self.assertLess(resolver, deletion)

    def test_workflow_default_false_and_invalid_skip_target_resolution(self):
        workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text()
        block = workflow.split("      - name: Resolve and clean the published target", 1)[1].split(
            "      - name: Preserve cleanup receipts", 1)[0]
        shell = "\n".join(line[10:] for line in block.split("        run: |\n", 1)[1].splitlines())
        for value in (None, "", "false", "0", "TRUE"):
            with self.subTest(value=value), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                subprocess.run(["git", "init", "-q", str(root)], check=True)
                (root / "scripts").mkdir()
                (root / "scripts/release").symlink_to(ROOT / "scripts/release", target_is_directory=True)
                env = os.environ | {"REPOSITORY": "owner/repo", "VERSION": "0.22.47",
                                    "PUBLISHED": "a" * 40, "SOURCE_PR": ""}
                env.pop("CLEANUP_REMOTE", None)
                if value is not None:
                    env["CLEANUP_REMOTE"] = value
                result = subprocess.run(["bash", "-e", "-c", shell], cwd=root, env=env,
                                        capture_output=True, text=True)
                if value in (None, "", "false"):
                    self.assertEqual(result.returncode, 0, result.stderr)
                    receipt, = (root / ".git/katana-cleanup-receipts").glob("*.jsonl")
                    rows = [json.loads(line) for line in receipt.read_text().splitlines()]
                    self.assertIn(("remote", "not-requested"),
                                  [(row["action"], row["state"]) for row in rows])
                else:
                    self.assertEqual(result.returncode, 2, result.stderr)
                    self.assertIn("Invalid cleanup_remote input", result.stderr)


if __name__ == "__main__":
    unittest.main()
