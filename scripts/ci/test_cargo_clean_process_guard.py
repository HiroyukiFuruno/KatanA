import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import unittest

import cargo_sweep_guard as sweep_guard
from test_cargo_clean_guard import (
    assert_cleaned,
    build_environment,
    build_release,
    create_crate,
    create_target_triple_profiles,
    run_clean_guard,
)


class CargoCleanProcessGuardTest(unittest.TestCase):
    def test_actual_just_cargo_wrapper_preserves_metadata_json(self):
        configured = configured_cargo()
        with tempfile.TemporaryDirectory() as directory:
            manifest = create_crate(Path(directory))
            result = run_clean_guard(manifest, os.environ.copy(), configured)
            self.assertEqual(0, result.returncode, result.stderr)
            self.assertIn("target is absent", result.stdout)

    def test_actual_just_cargo_wrapper_cleans_idle_target(self):
        configured = configured_cargo()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = create_crate(root)
            target, barrier, environment = build_environment(root)
            barrier.mkdir()
            barrier.joinpath("release").touch()
            create_target_triple_profiles(target)
            built = build_release(manifest, environment)
            self.assertEqual(0, built.returncode, built.stdout + built.stderr)
            write_cache_tag(target)
            artifact = target / "release" / "libsweep_guard_probe.rlib"
            cleaned = run_clean_guard(manifest, environment, configured)
            if sweep_guard.fcntl is None or os.name != "posix":
                self.assertEqual(75, cleaned.returncode, cleaned.stderr)
                self.assertTrue(artifact.exists())
            else:
                assert_cleaned(self, cleaned, artifact)

    def test_actual_child_failure_is_propagated_after_clean(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = create_crate(root)
            target, barrier, environment = build_environment(root)
            barrier.mkdir()
            barrier.joinpath("release").touch()
            create_target_triple_profiles(target)
            built = build_release(manifest, environment)
            self.assertEqual(0, built.returncode, built.stdout + built.stderr)
            write_cache_tag(target)
            wrapper = root / "cargo-wrapper.py"
            wrapper.write_text(
                "import subprocess, sys\n"
                "result = subprocess.run(['cargo', *sys.argv[1:]])\n"
                "raise SystemExit(23 if sys.argv[1] == 'clean' and result.returncode == 0 else result.returncode)\n",
                encoding="utf-8",
            )
            spec = f"{shlex.quote(Path(sys.executable).as_posix())} {shlex.quote(wrapper.as_posix())}"
            result = run_clean_guard(manifest, environment, spec)
            artifact = target / "release" / "libsweep_guard_probe.rlib"
            if sweep_guard.fcntl is None or os.name != "posix":
                self.assertEqual(75, result.returncode, result.stderr)
                self.assertTrue(artifact.exists())
            else:
                self.assertEqual(23, result.returncode, result.stderr)
                self.assertFalse(artifact.exists())


def configured_cargo():
    return subprocess.run(
        ["just", "--evaluate", "CARGO"], capture_output=True, text=True, check=True
    ).stdout.strip()


def write_cache_tag(target):
    target.joinpath("CACHEDIR.TAG").write_text(
        "Signature: 8a477f597d28d172789f06886806bc55\n", encoding="ascii"
    )


if __name__ == "__main__":
    unittest.main()
