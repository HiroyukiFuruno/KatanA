"""Pure policy contracts; synthetic records are not public-release evidence."""
from pathlib import Path
import subprocess
import unittest

import v043_postpublication_scope as policy

ROOT = Path(__file__).resolve().parents[2]


def pull_request_trigger(content):
    if content.count(b"\n  pull_request:\n") != 1 or content.count(b"\n  push:\n") != 1:
        raise ValueError("release trigger is missing or duplicated")
    return content.split(b"\n  pull_request:\n", 1)[1].split(b"\n  push:\n", 1)[0]
BASE = {policy.ACTIVE + name for name in ("tasks.md", "proposal.md", "evidence/original.json")}
TASKS = "- [x] 1.1 implemented\n- [x] 8.4 merged\n- [ ] 8.5 publication\n- [ ] 8.6 archive\n"


class ScopePolicyTest(unittest.TestCase):
    def test_exact_route_and_actual_stable_source_are_required(self):
        policy.validate_route(policy.BRANCH, policy.VERSION, policy.REPOSITORY)
        release = {"tag_name": policy.TAG, "draft": False, "prerelease": False,
                   "published_at": "synthetic-only"}
        policy.validate_publication(release, policy.SOURCE, policy.SOURCE)
        for key, value in (("tag_name", "v0.22.42"), ("draft", True),
                           ("prerelease", True), ("published_at", None)):
            with self.subTest(key=key), self.assertRaises(ValueError):
                policy.validate_publication(release | {key: value}, policy.SOURCE, policy.SOURCE)
        for api, local in (("a" * 40, policy.SOURCE), (policy.SOURCE, "b" * 40)):
            with self.subTest(api=api), self.assertRaises(ValueError):
                policy.validate_publication(release, api, local)
        with self.assertRaises(ValueError):
            policy.validate_publication({"payload_verified": True}, policy.SOURCE, policy.SOURCE)
        for branch, version in ((policy.BRANCH + "-extra", policy.VERSION),
                                (policy.BRANCH, "0.22.44")):
            with self.assertRaises(ValueError):
                policy.validate_route(branch, version, policy.REPOSITORY)

    def test_only_exact_doc_spec_evidence_and_controller_paths_are_allowed(self):
        archive = f"openspec/changes/archive/2026-11-01-{policy.CHANGE}/tasks.md"
        allowed = [*policy.CONTROLLERS, *BASE, archive,
                   "openspec/specs/docker-resource-lifecycle/spec.md",
                   *(policy.EVIDENCE + name for name in policy.EVIDENCE_FILES)]
        policy.validate_paths(allowed, BASE)
        for path in ("Cargo.toml", "Cargo.lock", "scripts/screenshot/Cargo.lock",
                     "crates/katana-ui/Info.plist", "CHANGELOG.md", "crates/runtime.rs",
                     "scripts/release/postpublication_controller_scope.py",
                     ".github/workflows/build-and-release.yml", ".github/workflows/test-and-build.yml",
                     "docs/unrelated.md", policy.EVIDENCE + "unknown.json",
                     policy.ACTIVE + "evidence/new.json", policy.EVIDENCE + "../other.json"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                policy.validate_paths([path], BASE)
        with self.assertRaises(ValueError):
            policy.validate_paths([f"openspec/changes/archive/2026-99-99-{policy.CHANGE}/tasks.md"], BASE)

    def test_existing_routes_allow_only_the_exact_insertions(self):
        for path, (anchor, _addition) in policy.ROUTE_EDITS.items():
            original = b"preserved v42\n" + anchor + b"preserved strict\n"
            changed = policy.expected_controller(path, original)
            policy.validate_controller(path, original, changed)
            with self.subTest(path=path), self.assertRaises(ValueError):
                policy.validate_controller(path, original, changed.replace(b"preserved strict", b"weakened"))
            with self.assertRaises(ValueError):
                policy.expected_controller(path, changed)
            with self.assertRaises(ValueError):
                policy.expected_controller(path, original + anchor)

    def test_tasks_must_have_one_location_and_preserve_the_number_inventory(self):
        active = policy.ACTIVE + "tasks.md"
        archive = f"openspec/changes/archive/2026-11-01-{policy.CHANGE}/tasks.md"
        other = f"openspec/changes/archive/2026-11-02-{policy.CHANGE}/tasks.md"
        self.assertEqual(policy.select_tasks([active]), active)
        self.assertEqual(policy.select_tasks([archive]), archive)
        for paths in ([], [active, archive], [archive, other]):
            with self.subTest(paths=paths), self.assertRaises(ValueError):
                policy.select_tasks(paths)
        policy.validate_task_inventory(TASKS, TASKS.replace("[ ]", "[x]"))
        for text in ("", TASKS + "- [x] 1.1 duplicate\n", TASKS + "- [x] 9.9 added\n",
                     TASKS.replace("- [x] 1.1 implemented\n", "")):
            with self.subTest(text=text), self.assertRaises(ValueError):
                policy.validate_task_inventory(TASKS, text)

    def test_allowed_scope_is_covered_by_unchanged_release_path_filters(self):
        path = ".github/workflows/build-and-release.yml"
        baseline = subprocess.run(["git", "-C", str(ROOT), "show",
                                   f"{policy.SOURCE}:{path}"],
                                  check=True, capture_output=True).stdout
        current = subprocess.run(["git", "-C", str(ROOT), "show", f"HEAD:{path}"],
                                 check=True, capture_output=True).stdout
        # WHY: 公開後記録のPRがrelease対象外であることを固定し、手動入力の追加は許容する。
        for content in (current, (ROOT / path).read_bytes()):
            self.assertEqual(content.count(b"\njobs:"), 1)
            self.assertEqual(baseline.count(b"\njobs:"), 1)
            self.assertEqual(pull_request_trigger(content), pull_request_trigger(baseline))
        block = baseline.decode().split("paths-ignore:", 1)[1].split("  push:", 1)[0]
        for prefix in ("openspec/**", "evidence/**", "scripts/**", ".github/**"):
            self.assertIn(f"- '{prefix}'", block)
        permitted = [*policy.CONTROLLERS, *BASE, policy.EVIDENCE + "publication.md",
                     *(f"openspec/specs/{name}/spec.md" for name in policy.CAPABILITIES)]
        self.assertTrue(all(path.startswith(("openspec/", "evidence/", "scripts/", ".github/"))
                            for path in permitted))

    def test_release_path_filter_stays_exact_when_manual_inputs_change(self):
        baseline = b"name: release\non:\n  pull_request:\n    types: [closed]\n    paths-ignore:\n      - 'openspec/**'\n  push:\n    tags: [v*]\n  workflow_dispatch:\n    inputs:\n      version: {}\n"
        added = baseline.replace(b"      version: {}", b"      cleanup_remote: {}\n      version: {}")
        self.assertEqual(pull_request_trigger(baseline), pull_request_trigger(added))
        for changed in (baseline.replace(b"'openspec/**'", b"'source/**'"),
                        baseline.replace(b"types: [closed]", b"types: [opened]")):
            self.assertNotEqual(pull_request_trigger(baseline), pull_request_trigger(changed))


if __name__ == "__main__":
    unittest.main()
