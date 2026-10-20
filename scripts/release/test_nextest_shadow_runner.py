import json
import importlib.util
import os
from pathlib import Path
import sys
import tempfile
import textwrap
import unittest

MODULE_PATH = Path(__file__).with_name("run-nextest-shadow.py")
SPEC = importlib.util.spec_from_file_location("run_nextest_shadow", MODULE_PATH)
RUNNER = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(RUNNER)
collect_shadow = RUNNER.collect_shadow
run_phase = RUNNER.run_phase
verify_tools = RUNNER.verify_tools


class NextestShadowRunnerTests(unittest.TestCase):
    def child(self, root: Path) -> Path:
        child = root / "child.py"
        child.write_text(textwrap.dedent("""
            import os
            import pathlib
            import sys
            if "-V" in sys.argv:
                print("cargo 1.90.0 (shadow-test)")
                raise SystemExit(0)
            if "build" in sys.argv:
                worker = pathlib.Path(os.environ["CARGO_TARGET_DIR"]) / "debug/kdv-office-worker"
                worker.parent.mkdir(parents=True, exist_ok=True)
                worker.write_bytes(b"worker")
                worker.chmod(worker.stat().st_mode | 0o100)
        """), encoding="utf-8")
        return child

    def version_child(self, root: Path) -> Path:
        child = root / "versions.py"
        child.write_text(textwrap.dedent("""
            import sys
            versions = {"nextest": "cargo-nextest 0.9.148", "llvm-cov": "cargo-llvm-cov 0.9.1"}
            print("cargo 1.90.0 (shadow-test)" if sys.argv[1] == "-V" else versions[sys.argv[1]])
        """), encoding="utf-8")
        return child

    def rustc_sentinel(self, root: Path) -> Path:
        directory = root / "tool with spaces"
        directory.mkdir()
        sentinel = directory / "rustc-sentinel"
        sentinel.write_text(
            "#!/bin/sh\ncase \"$RUSTFLAGS\" in *coverage*) printf 'rustc phase-vV\\n' ;; "
            "*) printf 'rustc root-vV\\n' ;; esac\n", encoding="utf-8")
        sentinel.chmod(0o755)
        return sentinel

    def test_verify_tools_returns_real_child_version_strings(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = self.version_child(root)
            versions = verify_tools([sys.executable, str(child)], dict(os.environ), root)
            self.assertEqual(versions, {
                "nextest": "cargo-nextest 0.9.148",
                "llvm-cov": "cargo-llvm-cov 0.9.1",
            })

    def test_tool_identity_uses_wrapper_and_rustc_sentinel_and_rejects_bad_tool(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = self.version_child(root)
            sentinel = self.rustc_sentinel(root)
            environment = {"RUSTC": str(sentinel)}
            identity = RUNNER.shadow_environment_identity(
                root, [sys.executable, str(child)], environment)
            self.assertEqual(identity["toolchain"]["cargo"], "cargo 1.90.0 (shadow-test)")
            self.assertEqual(identity["toolchain"]["rustc"], "rustc root-vV")
            phase_identity = RUNNER.shadow_environment_identity(
                root, [sys.executable, str(child)], {"RUSTC": str(sentinel),
                                                       "RUSTFLAGS": "--cfg coverage"})
            self.assertEqual(phase_identity["toolchain"]["rustc"], "rustc phase-vV")
            self.assertNotEqual(identity["toolchain"], phase_identity["toolchain"])
            bad = root / "bad.py"
            bad.write_text("raise SystemExit(9)\n", encoding="utf-8")
            with self.assertRaises(ValueError):
                RUNNER.verify_tools([sys.executable, str(bad)], {}, root)
            with self.assertRaises(ValueError):
                RUNNER.shadow_environment_identity(root, [sys.executable, str(child)], {"RUSTC": ""})

    def test_initial_receipt_records_actual_root_identity_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(__file__).resolve().parents[2]
            output = Path(directory) / "output"
            worker = Path(directory) / "worker"
            worker.write_bytes(b"worker")
            worker.chmod(0o755)
            environment = {
                "PATH": os.environ["PATH"],
                "CARGO_TARGET_DIR": str(Path(directory) / "target"),
                "CARGO_ENCODED_RUSTFLAGS": "-D\x1fwarnings",
                "KATANA_SHADOW_PROFILE_ID": "root-run",
                "KATANA_KDV_OFFICE_WORKER": str(worker),
            }
            identity = RUNNER.shadow_environment_identity(
                root, [sys.executable, str(self.version_child(Path(directory)))], environment)
            receipt = RUNNER.initial_receipt(
                root, output, [sys.executable, "cargo-wrapper"], environment,
                {"nextest": "cargo-nextest 0.9.148", "llvm-cov": "cargo-llvm-cov 0.9.1"},
                identity)
            self.assertEqual(receipt["cargo_argv"], [sys.executable, "cargo-wrapper"])
            self.assertEqual(receipt["environment_identity"]["environment"]["CARGO_TARGET_DIR"],
                             environment["CARGO_TARGET_DIR"])
            self.assertEqual(receipt["environment_identity"]["environment"]["CARGO_ENCODED_RUSTFLAGS"],
                             "-D\x1fwarnings")
            self.assertEqual(receipt["environment_identity"]["worker_sha256"],
                             RUNNER.file_digest(worker))

    def test_relative_worker_is_bound_to_child_repository(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            worker = root / "relative-worker"
            worker.write_bytes(b"child repository worker")
            environment = dict(os.environ)
            environment["KATANA_KDV_OFFICE_WORKER"] = worker.name
            identity = RUNNER.shadow_environment_identity(
                root, [sys.executable, str(self.version_child(root))], environment)
            self.assertEqual(identity["worker_sha256"], RUNNER.file_digest(worker))

    def test_identity_failure_prevents_child_stage_start(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "output"
            output.mkdir()
            with self.assertRaises(ValueError):
                run_phase(root, output, [str(root / "missing-cargo")], dict(os.environ),
                          {"phase": "blocked", "argv": []})
            self.assertFalse((output / "blocked").exists())

    def test_successful_child_without_store_writes_raw_phase_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = self.child(root)
            output = root / "output"
            output.mkdir()
            phase = {"phase": "phase-a", "argv": []}
            result = run_phase(root, output, [sys.executable, str(child)],
                               {**os.environ, "KATANA_SHADOW_PROFILE_ID": "test-run"}, phase)
            receipt = json.loads((output / "phase-a/phase.json").read_text())
            self.assertEqual(result["exit_code"], 0)
            self.assertEqual(receipt["exit_code"], 0)
            self.assertEqual(receipt["argv"], [sys.executable, str(child)])
            self.assertIn("record_unavailable", receipt)
            self.assertIn("environment_identity", receipt)
            self.assertEqual(receipt["environment_identity"]["explicit"], {
                "NEXTEST_STATE_DIR": str(output / "phase-a/nextest-state"),
                "LLVM_PROFILE_FILE_NAME": "katana-shadow-test-run-phase-a-%4m.profraw",
                "KATANA_KDV_OFFICE_WORKER": None,
            })

    def test_collect_shadow_returns_two_and_keeps_raw_success_stage(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = self.child(root)
            output = root / "output"
            output.mkdir()
            environment = dict(os.environ)
            environment["CARGO_TARGET_DIR"] = str(output / "coverage-target")
            identity = RUNNER.shadow_environment_identity(
                root, [sys.executable, str(child)], environment)
            receipt = {"stages": [], "environment_identity": identity}
            status = collect_shadow(root, output, [sys.executable, str(child)], environment, receipt)
            self.assertEqual(status, 2)
            self.assertEqual(receipt["stages"][1]["exit_code"], 0)
            self.assertIn("record_unavailable", receipt["stages"][1])

    def test_nonzero_child_exit_is_preserved_in_phase_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = root / "fail.py"
            child.write_text(textwrap.dedent("""
                import sys
                if "-V" in sys.argv:
                    print("cargo 1.90.0 (shadow-test)")
                else:
                    raise SystemExit(7)
            """), encoding="utf-8")
            output = root / "output"
            output.mkdir()
            result = run_phase(root, output, [sys.executable, str(child)], dict(os.environ),
                               {"phase": "phase-fail", "argv": []})
            receipt = json.loads((output / "phase-fail/phase.json").read_text())
            self.assertEqual(result["exit_code"], 7)
            self.assertEqual(receipt["exit_code"], 7)
            self.assertIn("record_unavailable", receipt)


if __name__ == "__main__":
    unittest.main()
