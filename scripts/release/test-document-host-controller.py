"""P/C境界を隔離した実Git repositoryで検査する。公開native証跡ではない。"""

import ast
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import document_host_controller as controller
from postpublication_controller_scope import verify_controller_scope
from document_host_acceptance import provenance
from document_host_io import HostEvidenceError
from generated_asset_identity import module
from generated_identity_fixtures import repository
from published_git_source import git_source_identity

HERE = Path(__file__).parent


class HostControllerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name).resolve()
        repository(self.root, "v0.22.47")
        self.git("config", "core.autocrlf", "false")
        for name in ("Cargo.toml", "Cargo.lock"):
            path = self.root/name
            path.write_bytes(path.read_bytes().replace(b"\r\n", b"\n"))
        self.path = "scripts/release/postpublication_controller_scope.py"
        self.write(self.path, "published verification helper\n")
        for name in ("document_host_transcript.py", "document_host_acceptance.py", "process_memory_sampler.py"):
            self.write("scripts/release/"+name, (HERE/name).read_text(encoding="utf-8"))
        self.commit()
        self.published = self.git("rev-parse", "HEAD")
        self.source = git_source_identity(self.root, self.published)
        self.anchor = patch.object(controller, "PUBLISHED_SOURCE", self.published)
        self.anchor.start()
        self.write(self.path, "reviewed verification helper\n")
        self.commit()
        self.commit_id = self.git("rev-parse", "HEAD")

    def tearDown(self):
        self.anchor.stop()
        self.temp.cleanup()

    def git(self, *args):
        return subprocess.run(["git", "-C", str(self.root), "-c", "core.hooksPath=/dev/null",
                               "-c", "user.name=Host Controller Unit", "-c", "user.email=unit@example.invalid",
                               *args], check=True, capture_output=True, text=True).stdout.strip()

    def write(self, path, text):
        target = self.root/path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8", newline="\n")

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-q", "-m", "synthetic controller unit")

    def binding(self):
        return controller.verify_controller(self.root, self.published, self.commit_id)

    def test_published_product_and_controller_have_distinct_complete_git_identities(self):
        binding = self.binding()
        self.assertEqual(binding["published_source"], self.published)
        self.assertEqual(binding["controller_commit"], self.commit_id)
        self.assertNotEqual(self.source["source_tree_sha256"], binding["controller_source_tree_sha256"])
        self.assertEqual(controller.verify_binding(self.root, self.source, binding), binding)
        with self.assertRaises(HostEvidenceError):
            provenance(self.root, self.source)
        self.assertEqual(provenance(self.root, self.source, controller=binding), self.source)
        admitted = verify_controller_scope(self.root, self.published)
        self.assertEqual(admitted["published_source"], self.published)
        self.assertEqual(admitted["controller_commit"], self.commit_id)

    def test_later_records_head_can_differ_but_source_must_equal_c(self):
        binding = self.binding()
        self.write("evidence/releases/v0.22.47/unit-record.txt", "unit record only\n")
        self.commit()
        self.assertNotEqual(self.git("rev-parse", "HEAD"), self.commit_id)
        self.assertEqual(controller.verify_binding(self.root, self.source, binding), binding)
        self.write(self.path, "unreviewed later source\n")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "differs from selected controller"):
            controller.verify_binding(self.root, self.source, binding)

    def test_compiled_build_workflow_lock_assets_and_unknown_scripts_are_rejected(self):
        for path in ("crates/main.rs", "assets/fixture.txt", ".github/workflows/new.yml", "Cargo.lock",
                     "scripts/screenshot/run.sh", "scripts/release/arbitrary.py", "docs/unknown.md"):
            with self.subTest(path=path), self.assertRaisesRegex(HostEvidenceError, "outside approved"):
                controller.validate_paths([path])

    def test_real_unknown_source_commit_is_rejected(self):
        self.write("crates/new.rs", "fn changed() {}\n")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "outside approved"):
            controller.verify_controller(self.root, self.published)

    def test_dirty_and_untracked_source_are_rejected(self):
        self.write(self.path, "dirty\n")
        with self.assertRaisesRegex(HostEvidenceError, "must be committed"):
            self.binding()
        self.write(self.path, "reviewed verification helper\n")
        self.write("scripts/untracked.py", "untracked\n")
        with self.assertRaisesRegex(HostEvidenceError, "must be committed"):
            self.binding()

    def test_checkout_line_ending_filters_cannot_hide_different_runtime_bytes(self):
        self.git("config", "core.autocrlf", "true")
        (self.root/self.path).write_bytes(b"reviewed verification helper\r\n")
        self.assertEqual(self.git("diff", "--name-only", "HEAD"), "")
        with self.assertRaisesRegex(HostEvidenceError, "checkout bytes differ"):
            self.binding()

    def test_binding_tampering_and_foreign_p_are_rejected(self):
        original = self.binding()
        for field, value in (("schema", "unknown"), ("published_source", "0"*40),
                             ("controller_commit", self.published), ("controller_source_tree_sha256", "0"*64)):
            with self.subTest(field=field), self.assertRaises((HostEvidenceError, subprocess.SubprocessError)):
                controller.verify_binding(self.root, self.source, original | {field:value})
        for field in self.source:
            changed = copy.deepcopy(self.source)
            changed[field] = {} if field == "published_dependencies" else "0"*64
            with self.subTest(source_field=field), self.assertRaises(ValueError):
                controller.verify_binding(self.root, changed, original)

    def test_git_symlink_in_allowed_path_is_rejected(self):
        target = self.root/"tmp/link-target"
        target.parent.mkdir()
        target.write_bytes(b"missing")
        blob = self.git("hash-object", "-w", str(target))
        self.git("update-index", "--cacheinfo", "120000,"+blob+","+self.path)
        self.git("commit", "-q", "-m", "synthetic Git symlink")
        with self.assertRaisesRegex(HostEvidenceError, "regular committed file"):
            controller.verify_controller(self.root, self.published)

    def test_deleted_allowed_path_is_rejected(self):
        (self.root/self.path).unlink()
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "regular committed file"):
            controller.verify_controller(self.root, self.published)

    def test_git_mode_change_in_allowed_path_is_rejected(self):
        self.git("update-index", "--chmod=+x", self.path)
        self.git("commit", "-q", "-m", "synthetic mode change")
        with self.assertRaisesRegex(HostEvidenceError, "file mode changed"):
            controller.verify_controller(self.root, self.published)

    def test_protected_timing_memory_and_sampler_functions_are_immutable(self):
        for name, original, changed in (
            ("document_host_transcript.py", "196608", "196609"),
            ("document_host_transcript.py", "5000", "5001"),
            ("document_host_acceptance.py", "15000", "15001"),
            ("process_memory_sampler.py", "DARWIN_PHYSICAL_FOOTPRINT_OFFSET)[0]", "DARWIN_RESIDENT_SIZE_OFFSET)[0]"),
            ("process_memory_sampler.py", "DARWIN_PHYSICAL_FOOTPRINT_OFFSET = 72", "DARWIN_PHYSICAL_FOOTPRINT_OFFSET = 73"),
            ("process_memory_sampler.py", '"rss_bytes": None', '"rss_bytes": 0'),
            ("document_host_processes.py", 'require(not result["children"],', 'require(True,'),
            ("collect-document-host-memory.py", "default=1200", "default=1201"),
        ):
            raw = (HERE/name).read_bytes()
            mutated = raw.replace(original.encode(), changed.encode(), 1)
            self.assertNotEqual(raw, mutated)
            with self.subTest(name=name, value=original), self.assertRaises(HostEvidenceError):
                controller.validate_protected_contracts("scripts/release/"+name, raw, mutated)

    def test_heartbeat_constants_are_immutable(self):
        path = "scripts/release/document_host_heartbeat.py"
        raw = (HERE/"document_host_heartbeat.py").read_bytes()
        for original, changed in (("200_000_000", "200_000_001"),
                                  ("READ_RETRIES = 3", "READ_RETRIES = 4"),
                                  ("READ_RETRY_SECONDS = 0.05", "READ_RETRY_SECONDS = 0.06")):
            with self.subTest(original=original), self.assertRaisesRegex(HostEvidenceError, "constants changed"):
                controller.validate_protected_contracts(path, raw, raw.replace(original.encode(), changed.encode(), 1))
        with self.assertRaisesRegex(HostEvidenceError, "constants changed"):
            controller.validate_protected_contracts(path, raw, raw+b"\nREAD_RETRIES = 3\n")

    def test_entire_heartbeat_verify_function_is_immutable(self):
        path = "scripts/release/document_host_heartbeat.py"
        raw = (HERE/"document_host_heartbeat.py").read_bytes()
        tree = ast.parse(raw)
        function = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "verify")
        function.body.append(ast.Pass())
        with self.assertRaisesRegex(HostEvidenceError, "contract changed: verify"):
            controller.validate_protected_contracts(path, raw, ast.unparse(tree).encode())
        with self.assertRaisesRegex(HostEvidenceError, "missing or duplicated: verify"):
            controller.validate_protected_contracts(path, raw, raw+b"\ndef verify():\n    pass\n")

    def test_entire_heartbeat_observer_class_is_immutable(self):
        path = "scripts/release/document_host_heartbeat.py"
        raw = (HERE/"document_host_heartbeat.py").read_bytes()
        tree = ast.parse(raw)
        observer = next(node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == "HeartbeatObserver")
        observer.body.append(ast.Pass())
        for mutated in (ast.unparse(tree).encode(), raw.replace(b"stop_event.wait(0.1)", b"stop_event.wait(0.2)")):
            with self.subTest(mutated=mutated[-80:]), self.assertRaisesRegex(HostEvidenceError, "poll/lifecycle contract changed"):
                controller.validate_protected_contracts(path, raw, mutated)
        with self.assertRaisesRegex(HostEvidenceError, "observer missing or duplicated"):
            controller.validate_protected_contracts(path, raw, raw+b"\nclass HeartbeatObserver:\n    pass\n")

    def test_heartbeat_reader_only_change_preserves_protected_contracts(self):
        path = "scripts/release/document_host_heartbeat.py"
        raw = (HERE/"document_host_heartbeat.py").read_bytes()
        mutated = raw.replace(b"before = os.fstat(handle.fileno())", b"before = os.fstat(handle.fileno())\n                pass", 1)
        self.assertNotEqual(raw, mutated)
        controller.validate_protected_contracts(path, raw, mutated)

    def test_heartbeat_correction_adds_only_the_two_approved_closure_paths(self):
        controller.validate_paths(["scripts/release/document_host_heartbeat.py", "scripts/release/test-document-host-acceptance.py"])
        self.assertEqual(len(controller.HOST_CONTROLLER_PATHS), 30)

    def test_native_after_snapshot_stays_at_actual_c_but_records_verification_can_move(self):
        snapshot = module("_unit_controller_snapshot", "record-document-host-snapshot.py")
        binding = self.binding()
        stage = self.root/"tmp/stage"
        stage.mkdir(parents=True)
        main, worker, archive = (stage/name for name in ("main", "worker", "archive"))
        for path in (main, worker, archive):
            path.write_bytes(path.name.encode())
        bundle = {"provenance":self.source, "controller":binding, "target":"macos-arm64",
                  "main_path":str(main), "worker_path":str(worker),
                  "main_sha256":hashlib.sha256(main.read_bytes()).hexdigest(),
                  "sidecar_sha256":hashlib.sha256(worker.read_bytes()).hexdigest(),
                  "archive_sha256":hashlib.sha256(archive.read_bytes()).hexdigest()}
        bundle_path=stage/"bundle.json"
        bundle_path.write_text(json.dumps(bundle), encoding="utf-8")
        run, profile = stage/"run", stage/"config"
        run.mkdir();profile.mkdir()
        (run/"app.exit").write_text("0\n", encoding="utf-8")
        snapshot.run_snapshot(self.root, run, bundle_path, archive, profile, "after")
        self.assertEqual((run/"head.after").read_text(encoding="utf-8").strip(), self.commit_id)
        self.assertEqual(json.loads((run/"source-identity.after.json").read_text(encoding="utf-8")), self.source)
        self.assertEqual(json.loads((run/"controller-identity.after.json").read_text(encoding="utf-8")), binding)
        self.write("docs/unit-record.txt", "record only\n");self.commit()
        self.assertEqual(controller.verify_binding(self.root, self.source, binding), binding)
        with self.assertRaisesRegex(HostEvidenceError, "native run HEAD"):
            snapshot.run_snapshot(self.root, run, bundle_path, archive, profile, "after")


if __name__ == "__main__":
    unittest.main()
