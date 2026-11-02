"""Verify the real shell environment defaults without invoking Cargo."""

import os
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile
import unittest


SCRIPT = Path(__file__).with_name("build-environment.sh")
RUNNER = Path(__file__).with_name("run.sh")


class BuildEnvironmentTests(unittest.TestCase):
    def runner_target_assignments(self) -> str:
        return "\n".join(
            line for line in RUNNER.read_text().splitlines()
            if line.startswith((
                "BUILD_TARGET_DIR=",
                "SCREENSHOT_TARGET_DIR=",
                "EXTERNAL_HYPERLINK_FIXTURE=",
            ))
        )

    def evaluate_runner_targets(self, assignments: str, target: str | None) -> tuple[str, str, str]:
        environment = dict(os.environ)
        if target is None:
            environment.pop("CARGO_TARGET_DIR", None)
        else:
            environment["CARGO_TARGET_DIR"] = target
        script = (
            "set -euo pipefail\nREPO_ROOT=/fixed/katana-root\n"
            f"{assignments}\nprintf '%s\\0%s\\0%s\\0' "
            "\"$BUILD_TARGET_DIR\" \"$SCREENSHOT_TARGET_DIR\" "
            "\"$EXTERNAL_HYPERLINK_FIXTURE\"\n"
        )
        result = subprocess.run(["bash", "-c", script], env=environment,
                                capture_output=True, check=True, text=False)
        values = result.stdout.decode().split("\0")
        self.assertEqual(values[-1], "")
        return tuple(values[:3])

    def test_screenshot_runner_target_assignments_preserve_shell_path_semantics(self):
        assignments = self.runner_target_assignments()
        self.assertEqual(assignments.count("\n") + 1, 3)
        expected_default = (
            "/fixed/katana-root/target",
            "/fixed/katana-root/target/screenshot-harness",
            "/fixed/katana-root/target/screenshot-fixtures/representative-with-external-hyperlink.pptx",
        )
        cases = ((None, expected_default), ("", expected_default))
        cases += tuple((target, (target, f"{target}/screenshot-harness",
                                  f"{target}/screenshot-fixtures/representative-with-external-hyperlink.pptx"))
                       for target in ("/tmp/canonical output", "relative canonical output"))
        for target, expected in cases:
            with self.subTest(target=target):
                self.assertEqual(self.evaluate_runner_targets(assignments, target), expected)

    def test_screenshot_runner_respects_caller_target_and_separates_child_lock(self):
        source = RUNNER.read_text()
        self.assertIn('BUILD_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target}"', source)
        self.assertIn('SCREENSHOT_TARGET_DIR="${BUILD_TARGET_DIR}/screenshot-harness"', source)
        self.assertIn('EXTERNAL_HYPERLINK_FIXTURE="${BUILD_TARGET_DIR}/screenshot-fixtures/', source)
        self.assertIn('CARGO_TARGET_DIR="${BUILD_TARGET_DIR}" cargo build --locked --release', source)
        self.assertIn('CARGO_TARGET_DIR="${SCREENSHOT_TARGET_DIR}" cargo build --locked --release', source)
        self.assertNotIn('BUILD_TARGET_DIR="${REPO_ROOT}/target"', source)

    def test_runner_rejects_unowned_target_before_creating_fixture(self):
        root = RUNNER.parents[2]
        with tempfile.TemporaryDirectory(dir=root / "tmp") as directory:
            target = Path(directory) / "unowned-target"
            target.mkdir()
            marker = target / "keep"
            marker.write_text("unowned", encoding="utf-8")
            environment = dict(os.environ) | {"CARGO_TARGET_DIR": str(target)}
            result = subprocess.run([str(RUNNER), "--help"], cwd=root, env=environment,
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("nonempty target has no ownership tag", result.stderr)
            self.assertEqual(marker.read_text(encoding="utf-8"), "unowned")
            self.assertEqual(list(target.iterdir()), [marker])

    def test_runner_initializes_target_before_real_fixture_generation(self):
        root = RUNNER.parents[2]
        with tempfile.TemporaryDirectory(dir=root / "tmp") as directory:
            target = Path(directory) / "new-target"
            environment = dict(os.environ) | {
                "CARGO_TARGET_DIR": str(target),
                "RUSTFLAGS": "-C katana_invalid_codegen_option",
            }
            environment.pop("CARGO_ENCODED_RUSTFLAGS", None)
            result = subprocess.run([str(RUNNER), "--help"], cwd=root, env=environment,
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("katana_invalid_codegen_option", result.stderr)
            self.assertTrue((target / "CACHEDIR.TAG").read_bytes().startswith(
                b"Signature: 8a477f597d28d172789f06886806bc55"))
            self.assertTrue((target / "screenshot-fixtures/representative-with-external-hyperlink.pptx").is_file())

    def test_legacy_fixture_is_regenerated_without_claiming_its_directory(self):
        root = RUNNER.parents[2]
        with tempfile.TemporaryDirectory(dir=root / "tmp") as directory:
            target = Path(directory) / "target"
            initializer = root / "scripts/ci/cargo_target_init.py"
            initialized = subprocess.run([sys.executable, str(initializer), str(target)], cwd=root,
                                         capture_output=True, text=True)
            self.assertEqual(0, initialized.returncode, initialized.stderr)
            old = target / "screenshot-fixtures/representative-with-external-hyperlink.pptx"
            generator = root / "scripts/screenshot/generate_external_hyperlink_pptx.py"
            original = root / "scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pptx"
            generated = subprocess.run([sys.executable, str(generator), "--source", str(original),
                                        "--output", str(old)], cwd=root, capture_output=True, text=True)
            self.assertEqual(0, generated.returncode, generated.stderr)
            before = hashlib.sha256(original.read_bytes()).hexdigest()
            environment = dict(os.environ) | {"CARGO_TARGET_DIR": str(target),
                                              "RUSTFLAGS": "-C katana_invalid_codegen_option"}
            environment.pop("CARGO_ENCODED_RUSTFLAGS", None)
            result = subprocess.run([str(RUNNER), "--help"], cwd=root, env=environment,
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("katana_invalid_codegen_option", result.stderr)
            self.assertEqual(before, hashlib.sha256(original.read_bytes()).hexdigest())
            with zipfile.ZipFile(old) as archive:
                relationships = archive.read("ppt/slides/_rels/slide1.xml.rels")
            self.assertIn(b"rIdKatanaExternalHyperlink", relationships)
            self.assertFalse((old.parent / "CACHEDIR.TAG").exists())

    @unittest.skipUnless(sys.platform == "darwin", "macOS system temporary alias")
    def test_runner_preserves_external_tmp_target_with_spaces(self):
        root = RUNNER.parents[2]
        with tempfile.TemporaryDirectory(dir="/tmp") as directory:
            target = Path(directory) / "external target"
            environment = dict(os.environ) | {
                "CARGO_TARGET_DIR": str(target),
                "RUSTFLAGS": "-C katana_invalid_codegen_option",
            }
            environment.pop("CARGO_ENCODED_RUSTFLAGS", None)
            result = subprocess.run([str(RUNNER), "--help"], cwd=root, env=environment,
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("katana_invalid_codegen_option", result.stderr)
            self.assertTrue((target.resolve() / "CACHEDIR.TAG").is_file())
            self.assertTrue((target.resolve() / "screenshot-fixtures/representative-with-external-hyperlink.pptx").is_file())

    @unittest.skipUnless(os.name == "posix", "filesystem symlink regression")
    def test_runner_rejects_linked_fixture_before_writing(self):
        root = RUNNER.parents[2]
        with tempfile.TemporaryDirectory(dir=root / "tmp") as directory:
            parent = Path(directory)
            target = parent / "target"
            initializer = root / "scripts/ci/cargo_target_init.py"
            initialized = subprocess.run(["python3", str(initializer), str(target)], cwd=root,
                                         capture_output=True, text=True)
            self.assertEqual(0, initialized.returncode, initialized.stderr)
            outside = parent / "outside"
            outside.mkdir()
            existing = outside / "representative-with-external-hyperlink.pptx"
            existing.write_bytes(b"existing fixture")
            before = hashlib.sha256(existing.read_bytes()).hexdigest()
            (target / "screenshot-fixtures").symlink_to(outside, target_is_directory=True)
            environment = dict(os.environ) | {"CARGO_TARGET_DIR": str(target)}
            result = subprocess.run([str(RUNNER), "--help"], cwd=root, env=environment,
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("fixture path contains symlink or junction", result.stderr)
            self.assertEqual(list(outside.iterdir()), [existing])
            self.assertEqual(before, hashlib.sha256(existing.read_bytes()).hexdigest())

    @unittest.skipUnless(os.name == "posix", "filesystem symlink regression")
    def test_runner_rejects_linked_fixture_file_without_touching_destination(self):
        root = RUNNER.parents[2]
        with tempfile.TemporaryDirectory(dir=root / "tmp") as directory:
            parent = Path(directory)
            target = parent / "target"
            initialized = subprocess.run(["python3", str(root / "scripts/ci/cargo_target_init.py"),
                                          str(target)], cwd=root, capture_output=True, text=True)
            self.assertEqual(0, initialized.returncode, initialized.stderr)
            destination = parent / "original"
            destination.write_bytes(b"unchanged")
            before = hashlib.sha256(destination.read_bytes()).hexdigest()
            fixtures = target / "screenshot-fixtures"
            fixtures.mkdir()
            (fixtures / "representative-with-external-hyperlink.pptx").symlink_to(destination)
            environment = dict(os.environ) | {"CARGO_TARGET_DIR": str(target)}
            result = subprocess.run([str(RUNNER), "--help"], cwd=root, env=environment,
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("fixture path contains symlink or junction", result.stderr)
            self.assertEqual(before, hashlib.sha256(destination.read_bytes()).hexdigest())

    @unittest.skipUnless(os.name == "posix", "filesystem hardlink regression")
    def test_runner_rejects_hardlinked_fixture_without_touching_destination(self):
        root = RUNNER.parents[2]
        with tempfile.TemporaryDirectory(dir=root / "tmp") as directory:
            parent = Path(directory)
            target = parent / "target"
            initialized = subprocess.run(["python3", str(root / "scripts/ci/cargo_target_init.py"),
                                          str(target)], cwd=root, capture_output=True, text=True)
            self.assertEqual(0, initialized.returncode, initialized.stderr)
            destination = parent / "original"
            destination.write_bytes(b"unchanged")
            before = hashlib.sha256(destination.read_bytes()).hexdigest()
            fixtures = target / "screenshot-fixtures"
            fixtures.mkdir()
            os.link(destination, fixtures / "representative-with-external-hyperlink.pptx")
            environment = dict(os.environ) | {"CARGO_TARGET_DIR": str(target)}
            result = subprocess.run([str(RUNNER), "--help"], cwd=root, env=environment,
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("fixture path is not an unshared regular destination", result.stderr)
            self.assertEqual(before, hashlib.sha256(destination.read_bytes()).hexdigest())

    def flags(self, supplied: str | None, encoded: str = "", platform: str = "darwin") -> tuple[str, str]:
        environment = dict(os.environ)
        environment.pop("RUSTFLAGS", None)
        if supplied is not None:
            environment["RUSTFLAGS"] = supplied
        environment["CARGO_ENCODED_RUSTFLAGS"] = encoded
        result = subprocess.run(
            ["bash", "-eu", "-c", 'OSTYPE="$2"; source "$1"; printf "%s\\0%s\\0" "${RUSTFLAGS-__unset__}" "$CARGO_ENCODED_RUSTFLAGS"',
             "build-environment-test", str(SCRIPT), platform],
            env=environment, capture_output=True, check=True,
        )
        flags, preserved, terminator = result.stdout.decode().split("\0")
        self.assertEqual(terminator, "")
        return flags, preserved

    def test_unset_flags_match_strict_just_default(self):
        self.assertEqual(self.flags(None), ("-D warnings", ""))

    def test_explicit_flags_and_empty_value_are_preserved(self):
        for flags in ("", "-D warnings -C debuginfo=0"):
            with self.subTest(flags=flags):
                self.assertEqual(self.flags(flags), (flags, ""))

    def test_encoded_flags_are_not_removed_or_rewritten(self):
        encoded = "-C\x1fdebuginfo=0"
        self.assertEqual(self.flags(None, encoded), ("-D warnings", encoded))

    def test_other_platforms_preserve_unset_target_configuration(self):
        for platform in ("linux-gnu", "msys", "cygwin"):
            with self.subTest(platform=platform):
                self.assertEqual(self.flags(None, platform=platform), ("__unset__", ""))
                self.assertEqual(self.flags("", platform=platform), ("", ""))
                self.assertEqual(self.flags("-C target-feature=+crt-static", platform=platform),
                                 ("-C target-feature=+crt-static", ""))


if __name__ == "__main__":
    unittest.main()
