import os
from contextlib import nullcontext
from pathlib import Path
import queue
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest

from cargo_target_lease import target_lease
from test_cargo_clean_guard import run_clean_guard


ROOT = Path(__file__).resolve().parents[2]
INITIALIZER = ROOT / "scripts/ci/cargo_target_init.py"
SIGNATURE = b"Signature: 8a477f597d28d172789f06886806bc55"


class CargoTargetInitTest(unittest.TestCase):
    def test_producer_lifetime_blocks_clean(self):
        self.assert_producer_lifetime(shared_parent=False)

    def test_producer_runs_under_existing_shared_parent_lease(self):
        self.assert_producer_lifetime(shared_parent=True)

    def assert_producer_lifetime(self, shared_parent):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            initializer = root / "scripts/ci/cargo_target_init.py"
            initializer.parent.mkdir(parents=True)
            shutil.copyfile(INITIALIZER, initializer)
            shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                            initializer.with_name("cargo_target_lease.py"))
            manifest = root / "Cargo.toml"
            manifest.write_text('[package]\nname="producer-lease-probe"\nversion="0.0.0"\nedition="2024"\n')
            (root / "Cargo.lock").write_text(
                'version = 3\n\n[[package]]\nname = "producer-lease-probe"\nversion = "0.0.0"\n')
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("pub fn probe() {}\n")
            target = root / "target"
            binary = target / "contract-tests/producer.exe"
            driver = root / "producer.py"
            source = ('use std::io; fn main() { println!("running"); '
                      'let mut line = String::new(); io::stdin().read_line(&mut line).unwrap(); }')
            driver.write_text(
                "import subprocess,sys\nfrom pathlib import Path\n"
                f"binary=Path({str(binary)!r})\n"
                "binary.parent.mkdir(parents=True,exist_ok=True)\n"
                "compiler=subprocess.Popen(['rustc','--edition','2024','-','-o',str(binary)],"
                "stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)\n"
                "print('compiling',flush=True)\nsys.stdin.readline()\n"
                f"output,_=compiler.communicate({source!r},timeout=60)\n"
                "if compiler.returncode: raise SystemExit(output)\n"
                "raise SystemExit(subprocess.run([str(binary)]).returncode)\n")
            environment = os.environ.copy()
            environment.pop("CARGO_BUILD_TARGET_DIR", None)
            environment["CARGO_TARGET_DIR"] = str(target)
            # WHY: push検査の親leaseを残したまま、子producerの初期化と実行を検査する。
            parent_lease = target_lease(target, exclusive=False) if shared_parent else nullcontext()
            with parent_lease:
                process = subprocess.Popen(
                    [sys.executable, str(initializer), "--run", sys.executable, str(driver)],
                    cwd=root, env=environment, stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                try:
                    for stage in ("compiling", "running"):
                        self.assertEqual(stage, self.read_stage(process))
                        cleaned = run_clean_guard(manifest, environment)
                        self.assertEqual(75, cleaned.returncode, cleaned.stdout + cleaned.stderr)
                        if os.name == "posix":
                            self.assertIn("target lease", cleaned.stderr)
                        elif os.name == "nt" and not shared_parent:
                            self.assertIn("unsupported lock filesystem platform", cleaned.stderr)
                        self.assertTrue((target / "CACHEDIR.TAG").is_file())
                        if stage == "running":
                            self.assertTrue(binary.is_file())
                        process.stdin.write("continue\n")
                        process.stdin.flush()
                    output, _ = process.communicate(timeout=60)
                    self.assertEqual(0, process.returncode, output)
                finally:
                    if process.poll() is None:
                        process.terminate()
                        process.communicate(timeout=15)
            cleaned = run_clean_guard(manifest, environment)
            self.assertEqual(0 if os.name == "posix" else 75,
                             cleaned.returncode, cleaned.stdout + cleaned.stderr)
            if os.name == "nt":
                self.assertIn("unsupported lock filesystem platform", cleaned.stderr)
            self.assertEqual(os.name != "posix", target.exists())

    def read_stage(self, process):
        received = queue.Queue()
        threading.Thread(target=lambda: received.put(process.stdout.readline()), daemon=True).start()
        try:
            return received.get(timeout=15).strip()
        except queue.Empty:
            self.fail("producer did not reach the next process barrier")

    def test_real_contract_producers_then_build_clean_and_reclean(self):
        recipes = (ROOT / "just/tests.just").read_text(encoding="utf-8")
        for name in ("test-commonmark-html-trace", "test-wgpu-native-surface-size"):
            with self.subTest(recipe=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                (root / "src").mkdir()
                (root / "src/lib.rs").write_text("pub fn probe() {}\n")
                manifest = root / "Cargo.toml"
                manifest.write_text('[package]\nname="cache-init-probe"\nversion="0.0.0"\nedition="2024"\n')
                for source in (
                    "vendor/egui_commonmark_upstream/egui_commonmark/src/parsers/pulldown/html_trace_summary.rs",
                    "vendor/egui-wgpu/src/native_surface_size.rs",
                ):
                    destination = root / source
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(ROOT / source, destination)
                if INITIALIZER.exists():
                    destination = root / "scripts/ci/cargo_target_init.py"
                    destination.parent.mkdir(parents=True)
                    shutil.copyfile(INITIALIZER, destination)
                    shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                                    destination.with_name("cargo_target_lease.py"))
                recipe = re.search(rf"(?m)^{name}:.*\n(?:[ \t]+.*\n)+", recipes).group()
                (root / "justfile").write_text('set shell := ["bash", "-uc"]\nRTK_CMD := ""\n' + recipe)
                environment = os.environ.copy()
                environment.pop("CARGO_TARGET_DIR", None)
                environment.pop("CARGO_BUILD_TARGET_DIR", None)
                result = subprocess.run(["just", name], cwd=root, env=environment,
                                        capture_output=True, text=True, timeout=60)
                self.assertEqual(0, result.returncode, result.stdout + result.stderr)
                target = root / "target"
                tag = target / "CACHEDIR.TAG"
                self.assertTrue(tag.is_file(), "contract producer must tag its new target before Cargo runs")
                self.assertTrue(tag.read_bytes().startswith(SIGNATURE))
                environment["CARGO_TARGET_DIR"] = str(target)
                built = subprocess.run(["cargo", "build", "--offline", "--manifest-path", str(manifest)],
                                       env=environment, capture_output=True, text=True, timeout=60)
                self.assertEqual(0, built.returncode, built.stderr)
                for _ in range(2):
                    cleaned = run_clean_guard(manifest, environment)
                    self.assertEqual(0 if os.name == "posix" else 75,
                                     cleaned.returncode, cleaned.stdout + cleaned.stderr)
                if os.name == "posix":
                    self.assertFalse(target.exists())

    def test_initializer_preserves_unowned_invalid_and_linked_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            initializer = root / "scripts/ci/cargo_target_init.py"
            initializer.parent.mkdir(parents=True)
            shutil.copyfile(INITIALIZER, initializer)
            shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                            initializer.with_name("cargo_target_lease.py"))
            for name in ("unowned", "invalid", "linked"):
                target = root / name
                if name == "linked":
                    if os.name != "posix":
                        continue
                    target.symlink_to(root / "unowned", target_is_directory=True)
                else:
                    target.mkdir()
                    (target / "keep.txt").write_text("keep")
                    if name == "invalid":
                        (target / "CACHEDIR.TAG").write_text("invalid")
                result = subprocess.run([sys.executable, str(initializer), str(target)],
                                        capture_output=True, text=True, timeout=10)
                self.assertNotEqual(0, result.returncode)
                self.assertIn("refused safely", result.stderr)
                self.assertEqual("keep", (target / "keep.txt").read_text())
                if name == "invalid":
                    self.assertEqual("invalid", (target / "CACHEDIR.TAG").read_text())
                else:
                    self.assertFalse((target / "CACHEDIR.TAG").exists())

    def test_empty_target_and_existing_valid_tag_are_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            initializer = root / "scripts/ci/cargo_target_init.py"
            initializer.parent.mkdir(parents=True)
            shutil.copyfile(INITIALIZER, initializer)
            shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                            initializer.with_name("cargo_target_lease.py"))
            target = root / "target"
            target.mkdir()
            command = [sys.executable, str(initializer)]
            result = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(0, result.returncode, result.stderr)
            tag = target / "CACHEDIR.TAG"
            original = tag.read_bytes()
            (target / "keep.txt").write_text("keep")
            result = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(0, result.returncode, result.stderr)
            self.assertEqual(original, tag.read_bytes())
            self.assertEqual("keep", (target / "keep.txt").read_text())

    def test_parallel_initializers_share_the_creation_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            initializer = root / "scripts/ci/cargo_target_init.py"
            initializer.parent.mkdir(parents=True)
            shutil.copyfile(INITIALIZER, initializer)
            shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                            initializer.with_name("cargo_target_lease.py"))
            code = ("import runpy,sys; from pathlib import Path; "
                    "sys.stdin.read(1); path=sys.argv[1]; sys.path.insert(0,str(Path(path).parent)); "
                    "sys.argv=[path]; runpy.run_path(path,run_name='__main__')")
            children = [subprocess.Popen([sys.executable, "-c", code, str(initializer)],
                                         stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                         stderr=subprocess.PIPE, text=True) for _ in range(6)]
            try:
                for child in children:
                    child.stdin.write("x")
                    child.stdin.flush()
                for child in children:
                    output, errors = child.communicate(timeout=15)
                    self.assertEqual(0, child.returncode, output + errors)
            finally:
                for child in children:
                    if child.poll() is None:
                        child.terminate()
                    child.communicate(timeout=15)
            self.assertTrue((root / "target/CACHEDIR.TAG").read_bytes().startswith(SIGNATURE))


if __name__ == "__main__":
    unittest.main()
