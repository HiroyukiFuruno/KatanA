import json
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
import zipfile

from cargo_target_lease import target_lease
from test_cargo_clean_guard import run_clean_guard


ROOT = Path(__file__).resolve().parents[2]
INITIALIZER = ROOT / "scripts/ci/cargo_target_init.py"
SIGNATURE = b"Signature: 8a477f597d28d172789f06886806bc55"


class CargoTargetInitTest(unittest.TestCase):
    def test_unowned_fixture_child_is_preserved_by_actual_cargo_clean(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            manifest = root / "Cargo.toml"
            manifest.write_text('[package]\nname="fixture-clean"\nversion="0.0.0"\nedition="2024"\n')
            (root / "Cargo.lock").write_text('version = 3\n\n[[package]]\nname = "fixture-clean"\nversion = "0.0.0"\n')
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("pub fn probe() {}\n")
            child = root / "target/screenshot-fixtures"
            child.mkdir(parents=True)
            fixture = child / "existing.pptx"
            fixture.write_bytes(b"unchanged")
            environment = os.environ.copy() | {"CARGO_TARGET_DIR": str(child)}
            environment.pop("CARGO_BUILD_TARGET_DIR", None)
            cleaned = run_clean_guard(manifest, environment)
            self.assertNotEqual(0, cleaned.returncode)
            self.assertEqual(b"unchanged", fixture.read_bytes())
            self.assertFalse((child / "CACHEDIR.TAG").exists())

    @unittest.skipUnless(sys.platform == "darwin", "macOS system temporary alias")
    def test_macos_tmp_alias_shares_real_cache_and_lease_identity(self):
        from cargo_target_lease import lease_path

        with tempfile.TemporaryDirectory(dir="/tmp") as directory:
            alias = Path(directory) / "cache with spaces"
            canonical = alias.resolve()
            command = [sys.executable, str(INITIALIZER), "--allow-external-target", str(alias),
                       "--run", sys.executable, "-c", "import pathlib,sys; pathlib.Path(sys.argv[1]).write_text('ok')",
                       str(alias / "output")]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            self.assertEqual("ok", (canonical / "output").read_text())
            self.assertTrue((canonical / "CACHEDIR.TAG").is_file())
            self.assertEqual(lease_path(alias), lease_path(canonical))

    @unittest.skipUnless(sys.platform == "darwin", "macOS system temporary alias")
    def test_macos_var_alias_uses_the_same_cache_identity(self):
        from cargo_target_lease import lease_path

        with tempfile.TemporaryDirectory() as directory:
            canonical = Path(directory).resolve() / "cache with spaces"
            try:
                alias = Path("/var") / canonical.relative_to("/private/var")
            except ValueError:
                self.skipTest("temporary directory is not under /private/var")
            result = subprocess.run([sys.executable, str(INITIALIZER), "--allow-external-target", str(alias)],
                                    cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            self.assertTrue((canonical / "CACHEDIR.TAG").is_file())
            self.assertEqual(lease_path(alias), lease_path(canonical))

    def test_external_cache_requires_opt_in_and_safe_absolute_owned_path(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory).resolve()
            root = base / "repo"
            initializer = root / "scripts/ci/cargo_target_init.py"
            initializer.parent.mkdir(parents=True)
            shutil.copyfile(INITIALIZER, initializer)
            shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                            initializer.with_name("cargo_target_lease.py"))
            target = base / "external cache"
            prefix = [sys.executable, str(initializer)]
            default = subprocess.run(prefix + [str(target)], cwd=root, capture_output=True, text=True)
            self.assertNotEqual(0, default.returncode)
            self.assertFalse(target.exists())
            command = prefix + ["--allow-external-target", str(target), "--run", sys.executable,
                                "-c", "import sys; print(sys.argv[1])", "argument with spaces"]
            for _ in range(2):
                result = subprocess.run(command, cwd=root, capture_output=True, text=True)
                self.assertEqual(0, result.returncode, result.stdout + result.stderr)
                self.assertEqual("argument with spaces", result.stdout.strip())
            self.assertTrue((target / "CACHEDIR.TAG").read_bytes().startswith(SIGNATURE))
            for rejected in (base / "unowned", root / "src/cache", root / ".git/cache",
                             Path("../relative-cache")):
                if rejected == base / "unowned":
                    rejected.mkdir()
                    (rejected / "keep").write_text("unchanged")
                result = subprocess.run(prefix + ["--allow-external-target", str(rejected)],
                                        cwd=root, capture_output=True, text=True)
                self.assertNotEqual(0, result.returncode, str(rejected))
            self.assertEqual("unchanged", (base / "unowned/keep").read_text())
            invalid = base / "invalid"
            invalid.mkdir()
            (invalid / "CACHEDIR.TAG").write_text("invalid")
            result = subprocess.run(prefix + ["--allow-external-target", str(invalid)],
                                    cwd=root, capture_output=True, text=True)
            self.assertNotEqual(0, result.returncode)
            self.assertEqual("invalid", (invalid / "CACHEDIR.TAG").read_text())
            other_repository = base / "other-repository"
            (other_repository / ".git").mkdir(parents=True)
            result = subprocess.run(prefix + ["--allow-external-target", str(other_repository / "target")],
                                    cwd=root, capture_output=True, text=True)
            self.assertNotEqual(0, result.returncode)
            self.assertFalse((other_repository / "target").exists())
            if os.name == "posix":
                link = base / "linked"
                link.symlink_to(target, target_is_directory=True)
                result = subprocess.run(prefix + ["--allow-external-target", str(link / "nested")],
                                        cwd=root, capture_output=True, text=True)
                self.assertNotEqual(0, result.returncode)
                self.assertFalse((target / "nested").exists())

    def test_screenshot_fixture_is_generated_after_owned_target_initialization(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            initializer = root / "scripts/ci/cargo_target_init.py"
            initializer.parent.mkdir(parents=True)
            shutil.copyfile(INITIALIZER, initializer)
            shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                            initializer.with_name("cargo_target_lease.py"))
            generator = ROOT / "scripts/screenshot/generate_external_hyperlink_pptx.py"
            source = ROOT / "scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pptx"
            target = root / "target"
            fixture = target / "screenshot-fixtures/linked.pptx"
            result = subprocess.run(
                [sys.executable, str(initializer), str(target), "--run", sys.executable,
                 str(generator), "--source", str(source), "--output", str(fixture)],
                cwd=root, capture_output=True, text=True, timeout=60)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            self.assertTrue((target / "CACHEDIR.TAG").read_bytes().startswith(SIGNATURE))
            with zipfile.ZipFile(fixture) as archive:
                relationships = archive.read("ppt/slides/_rels/slide1.xml.rels")
            self.assertIn(b"rIdKatanaExternalHyperlink", relationships)

    @unittest.skipUnless(os.name == "nt", "Windows CreateProcess executable search regression")
    def test_windows_producer_uses_environment_path_before_system32(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            initializer = root / "scripts/ci/cargo_target_init.py"
            initializer.parent.mkdir(parents=True)
            shutil.copyfile(INITIALIZER, initializer)
            shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                            initializer.with_name("cargo_target_lease.py"))
            self.assertTrue((Path(os.environ["SystemRoot"]) / "System32/cmd.exe").is_file())
            binary = root / "bin/cmd.exe"
            binary.parent.mkdir()
            built = subprocess.run(
                ["rustc", "--edition", "2024", "-", "-o", str(binary)],
                input='fn main() { println!("{}", std::env::args().skip(1).collect::<Vec<_>>().join("|")); }',
                capture_output=True, text=True, timeout=60)
            self.assertEqual(0, built.returncode, built.stdout + built.stderr)
            environment = os.environ.copy()
            environment["PATH"] = str(binary.parent) + os.pathsep + environment.get("PATH", "")
            environment.pop("CARGO_TARGET_DIR", None)
            environment.pop("CARGO_BUILD_TARGET_DIR", None)
            for executable in ("cmd.exe", ".\\bin\\cmd.exe", str(binary)):
                with self.subTest(executable=executable):
                    result = subprocess.run(
                        [sys.executable, str(initializer), str(root / "target"), "--run", executable,
                         "explicit-path-producer", "argument with spaces"],
                        cwd=root, env=environment, capture_output=True, text=True, timeout=15)
                    self.assertEqual(0, result.returncode, result.stdout + result.stderr)
                    self.assertEqual("explicit-path-producer|argument with spaces", result.stdout.strip())
            environment["PATH"] = str(root / "missing")
            missing = subprocess.run(
                [sys.executable, str(initializer), str(root / "target"), "--run", "cmd.exe"],
                cwd=root.parent, env=environment, capture_output=True, text=True, timeout=15)
            self.assertNotEqual(0, missing.returncode)
            self.assertIn("producer executable not found on PATH", missing.stderr)

    def test_producer_lifetime_blocks_clean(self):
        self.assert_producer_lifetime(shared_parent=False)

    def test_producer_runs_under_existing_shared_parent_lease(self):
        self.assert_producer_lifetime(shared_parent=True)

    def test_nested_screenshot_targets_block_both_cleans_until_real_process_exits(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            initializer = root / "scripts/ci/cargo_target_init.py"
            initializer.parent.mkdir(parents=True)
            shutil.copyfile(INITIALIZER, initializer)
            shutil.copyfile(INITIALIZER.with_name("cargo_target_lease.py"),
                            initializer.with_name("cargo_target_lease.py"))
            manifest = root / "Cargo.toml"
            manifest.write_text('[package]\nname="nested-lease"\nversion="0.0.0"\nedition="2024"\n')
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("pub fn probe() {}\n")
            (root / "Cargo.lock").write_text('version = 3\n\n[[package]]\nname = "nested-lease"\nversion = "0.0.0"\n')
            parent = root / "target"
            child = parent / "screenshot-harness"
            driver = root / "producer.py"
            source = ('use std::io; fn main() { println!("running"); '
                      'let mut line = String::new(); io::stdin().read_line(&mut line).unwrap(); }')
            driver.write_text(
                "import subprocess,sys\nfrom pathlib import Path\n"
                f"binary=Path({str(child / 'probe.exe')!r})\n"
                f"source={source!r}\n"
                "subprocess.run(['rustc','--edition','2024','-','-o',str(binary)],"
                "input=source,text=True,check=True)\n"
                "raise SystemExit(subprocess.run([str(binary)]).returncode)\n")
            command = [sys.executable, str(initializer), str(parent), "--run",
                       sys.executable, str(initializer), str(child), "--run", sys.executable, str(driver)]
            process = subprocess.Popen(command, cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                       stderr=subprocess.STDOUT, text=True)
            try:
                self.assertEqual("running", self.read_stage(process))
                for target in (parent, child):
                    environment = os.environ.copy() | {"CARGO_TARGET_DIR": str(target)}
                    environment.pop("CARGO_BUILD_TARGET_DIR", None)
                    cleaned = run_clean_guard(manifest, environment)
                    self.assertEqual(75, cleaned.returncode, cleaned.stdout + cleaned.stderr)
                    self.assertTrue((target / "CACHEDIR.TAG").is_file())
                process.stdin.write("continue\n")
                process.stdin.flush()
                output, _ = process.communicate(timeout=60)
                self.assertEqual(0, process.returncode, output)
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.communicate(timeout=15)

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
                environment = os.environ.copy()
                shell = "bash"
                if os.name == "nt":
                    shell = environment.get("KATANA_GUARD_BASH_EXECUTABLE", "")
                    self.assertTrue(shell, "KATANA_GUARD_BASH_EXECUTABLE must identify the running Git Bash")
                    shell_path = Path(shell)
                    self.assertTrue(shell_path.is_absolute(),
                                    f"Git Bash must be a native absolute path: {shell}")
                    # WHY: recipe内の --run bash も、WSLではなく外側と同じGit Bashを使う。
                    environment["PATH"] = str(shell_path.parent) + os.pathsep + environment.get("PATH", "")
                (root / "justfile").write_text(
                    f'set shell := [{json.dumps(shell, ensure_ascii=False)}, "-uc"]\nRTK_CMD := ""\n' + recipe,
                    encoding="utf-8")
                environment.pop("CARGO_TARGET_DIR", None)
                environment.pop("CARGO_BUILD_TARGET_DIR", None)
                result = subprocess.run(["just", name], cwd=root, env=environment,
                                        capture_output=True, text=True, timeout=60)
                self.assertEqual(0, result.returncode, result.stdout + result.stderr)
                target = root / "target"
                contract_target = root / "tmp/contract-test-cache"
                tag = contract_target / "CACHEDIR.TAG"
                self.assertFalse(target.exists(), "standalone rustc must not create the Cargo target")
                self.assertTrue(tag.is_file(), "contract producer must tag its own cache")
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
                self.assertTrue(tag.is_file(), "Cargo cleanup must not consume the standalone cache")

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
