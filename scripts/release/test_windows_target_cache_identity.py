import os
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/ci"))
from windows_target_cache_identity import (MAX_VERSION_BYTES, _cache_key, _source_stable, _version,
                                           build_cache_key, cache_identity)
from native_test_run_identity import source_identity


def _fixture() -> tuple[tempfile.TemporaryDirectory, Path]:
    temporary = tempfile.TemporaryDirectory()
    root = Path(temporary.name)
    (root / "scripts/screenshot").mkdir(parents=True)
    (root / "Cargo.lock").write_text("lock-a\n", encoding="utf-8")
    (root / "scripts/screenshot/Cargo.lock").write_text("lock-b\n", encoding="utf-8")
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    subprocess.run(["git", "config", "user.email", "cache@example.invalid"], cwd=root, check=True)
    subprocess.run(["git", "config", "user.name", "cache-test"], cwd=root, check=True)
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    subprocess.run(["git", "-c", "commit.gpgsign=false", "commit", "-qm", "fixture"],
                   cwd=root, check=True)
    return temporary, root


def _environment(root: Path) -> dict[str, str]:
    environment = {"PATH": os.environ["PATH"], "CARGO_BUILD_TARGET": "x86_64-pc-windows-msvc",
                   "RUSTFLAGS": "-D warnings", "CARGO_TARGET_DIR": str(root / "target"),
                   "CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0"}
    for key in ("SYSTEMROOT", "USERPROFILE", "HOME", "TEMP", "TMP"):
        if key in os.environ:
            environment[key] = os.environ[key]
    return environment


def _commit(root: Path, message: str) -> None:
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    subprocess.run(["git", "commit", "-qm", message], cwd=root, check=True)


class WindowsTargetCacheIdentityTests(unittest.TestCase):
    def test_key_is_stable_and_contains_no_raw_path_or_secret(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        environment = _environment(root)
        key = build_cache_key(root, environment, "dev", ("external-fixture",), ("workspace",))
        self.assertEqual(key, build_cache_key(root, environment, "dev", ("external-fixture",), ("workspace",)))
        self.assertNotIn(str(root), key)
        self.assertNotIn("secret", key)

    def test_key_changes_for_target_flags_profile_features_and_selection(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        environment = _environment(root)
        baseline = build_cache_key(root, environment, "dev", (), ("workspace",))
        for changed in (
            ({**environment, "CARGO_BUILD_TARGET": "aarch64-pc-windows-msvc"}, "dev", (), ("workspace",)),
            ({**environment, "RUSTFLAGS": "-C opt-level=2"}, "dev", (), ("workspace",)),
            ({**environment, "RUNNER_ARCH": "ARM64"}, "dev", (), ("workspace",)),
            ({**environment, "CARGO_TERM_COLOR": "always"}, "dev", (), ("workspace",)),
            (environment, "release", (), ("workspace",)),
            (environment, "dev", ("external-fixture",), ("workspace",)),
            (environment, "dev", (), ("ui-serial",)),
        ):
            self.assertNotEqual(baseline, build_cache_key(root, *changed))

    def test_dirty_unknown_and_missing_source_inputs_fail_closed(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        environment = _environment(root)
        (root / "Cargo.lock").write_text("dirty\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            build_cache_key(root, environment, "dev", (), ("workspace",))
        temporary2, root2 = _fixture()
        self.addCleanup(temporary2.cleanup)
        with self.assertRaises(ValueError):
            build_cache_key(root2, {**_environment(root2), "CARGO_UNKNOWN_OPTION": "1"},
                            "dev", (), ("workspace",))
        (root2 / "Cargo.lock").unlink()
        with self.assertRaises(ValueError):
            build_cache_key(root2, _environment(root2), "dev", (), ("workspace",))

    def test_untracked_and_invalid_options_fail_closed(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        environment = _environment(root)
        (root / "untracked.txt").write_text("untracked\n", encoding="utf-8")
        result = cache_identity(root, environment, "dev", (), ("workspace",))
        self.assertFalse(result["cache_eligible"])
        self.assertEqual(result["key"], "")
        with self.assertRaises(ValueError):
            build_cache_key(root, environment, "", (), ("workspace",))
        with self.assertRaises(ValueError):
            build_cache_key(root, environment, "dev", ("\x00",), ("workspace",))
        with self.assertRaises(ValueError):
            build_cache_key(root, environment, "dev\x00", (), ("workspace",))

    def test_diagnostic_ci_paths_and_run_ids_do_not_change_key(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        environment = _environment(root)
        first = {**environment, "RUNNER_NAME": "host-a", "RUNNER_TEMP": str(root / "tmp-a"),
                 "RUNNER_TOOL_CACHE": str(root / "tool-a"), "RUNNER_WORKSPACE": str(root / "ws-a"),
                 "RUNNER_ENVIRONMENT": "github-hosted", "KATANA_TEST_INVENTORY_ROOT": "run-a",
                 "GITHUB_RUN_ID": "1"}
        second = {**first, "RUNNER_NAME": "host-b", "RUNNER_TEMP": str(root / "tmp-b"),
                  "KATANA_TEST_INVENTORY_ROOT": "run-b", "GITHUB_RUN_ID": "2"}
        self.assertEqual(build_cache_key(root, first, "dev", (), ("workspace",)),
                         build_cache_key(root, second, "dev", (), ("workspace",)))

    def test_clean_committed_source_and_lock_changes_change_key(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        environment = _environment(root)
        first = build_cache_key(root, environment, "dev", (), ("workspace",))
        (root / "Cargo.lock").write_text("lock-c\n", encoding="utf-8")
        _commit(root, "lock")
        second = build_cache_key(root, environment, "dev", (), ("workspace",))
        self.assertNotEqual(first, second)
        (root / "README.md").write_text("source\n", encoding="utf-8")
        _commit(root, "source")
        self.assertNotEqual(second, build_cache_key(root, environment, "dev", (), ("workspace",)))

    def test_source_change_during_tool_version_collection_disables_cache(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        before = source_identity(root)
        (root / "Cargo.lock").write_text("changed\n", encoding="utf-8")
        self.assertFalse(_source_stable(before, root))

    def test_actual_tool_versions_change_key(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        script = root / "version_child.py"
        script.write_text("import sys\nprint(sys.argv[1])\n", encoding="utf-8")
        first = _version([sys.executable, str(script), "rustc-1.90"], root, _environment(root))
        second = _version([sys.executable, str(script), "rustc-1.91"], root, _environment(root))
        self.assertNotEqual(first, second)
        self.assertNotEqual(_cache_key({"toolchain": {"rustc": first, "cargo": "cargo-1"}}),
                            _cache_key({"toolchain": {"rustc": second, "cargo": "cargo-1"}}))
        self.assertNotEqual(_cache_key({"toolchain": {"rustc": first, "cargo": "cargo-1"}}),
                            _cache_key({"toolchain": {"rustc": first, "cargo": "cargo-2"}}))

    def test_cli_emits_only_a_digest_at_the_github_output_boundary(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        environment = _environment(root)
        script = ROOT / "scripts/ci/windows_target_cache_identity.py"
        output = root / "github-output"
        result = subprocess.run([sys.executable, str(script), "--repo", str(root),
                                 "--profile", "dev", "--feature", "external-fixture",
                                 "--selection", "workspace", "--github-output", str(output)], cwd=ROOT, env=environment,
                                capture_output=True, text=True, check=True)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["cache_eligible"])
        self.assertRegex(payload["key"], r"^windows-target-v1-[0-9a-f]{64}$")
        self.assertNotIn(str(root), result.stdout)
        self.assertEqual(output.read_text(encoding="utf-8"),
                         f"cache_eligible=true\ncache_key={payload['key']}\n")

    def test_invalid_cli_input_returns_fresh_fallback_without_key(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        script = ROOT / "scripts/ci/windows_target_cache_identity.py"
        output = root / "github-output"
        result = subprocess.run([sys.executable, str(script), "--repo", str(root),
                                 "--profile", "dev", "--github-output", str(output)], cwd=ROOT,
                                env={**_environment(root), "CARGO_UNKNOWN_OPTION": "secret"},
                                capture_output=True, text=True, check=True)
        self.assertEqual(json.loads(result.stdout)["cache_eligible"], False)
        self.assertEqual(json.loads(result.stdout)["key"], "")
        self.assertNotIn(str(root), result.stdout)
        self.assertEqual(output.read_text(encoding="utf-8"), "cache_eligible=false\ncache_key=\n")

    def test_tool_failure_and_oversize_output_are_bounded(self):
        temporary, root = _fixture()
        self.addCleanup(temporary.cleanup)
        missing = root / "missing-tool"
        with self.assertRaises(ValueError):
            _version([str(missing)], root, _environment(root))
        noisy = root / "noisy.py"
        noisy.write_text("print('x' * 70000)\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            _version([sys.executable, str(noisy)], root, _environment(root))
        noisy.write_text("import sys\nsys.stderr.write('x' * 70000)\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            _version([sys.executable, str(noisy)], root, _environment(root))
        self.assertEqual(MAX_VERSION_BYTES, 65536)
