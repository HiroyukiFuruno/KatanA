#!/usr/bin/env python3
"""Guard the all-platform release workflow and trigger contract."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).parents[2]
WORKFLOW = ROOT / ".github/workflows/build-and-release.yml"


def job_section(workflow: str, job: str) -> str:
    pattern = rf"(?ms)^  {re.escape(job)}:\n(.*?)(?=^  [A-Za-z0-9_]+:|\Z)"
    match = re.search(pattern, workflow)
    if match is None:
        raise ValueError(f"workflow job missing: {job}")
    return match.group(1)


def require_all_platforms(workflow: str) -> None:
    if "      target:\n" in workflow or "github.event.inputs.target" in workflow:
        raise ValueError("release workflow must not select a single platform")
    for job in ("build_macos", "smoke_macos", "build_linux", "build_windows"):
        if re.search(r"(?m)^    if:", job_section(workflow, job)):
            raise ValueError(f"platform job is conditionally skipped: {job}")


def require_publication_contract(workflow: str) -> None:
    publish = workflow.split("  publish:", 1)[1]
    needs = "needs: [preflight, build_macos, smoke_macos, build_linux, build_windows]"
    success = ("needs.build_macos.result == 'success' && needs.smoke_macos.result == 'success' "
               "&& needs.build_linux.result == 'success' && needs.build_windows.result == 'success'")
    if needs not in publish or success not in publish:
        raise ValueError("publication must require every build and macOS smoke")
    for asset in ("KatanA-Desktop-*.dmg", "KatanA-macOS.zip",
                  "KatanA-linux-x86_64.tar.gz", "KatanA-windows-x86_64.zip",
                  "KatanA-windows-x86_64.msi"):
        if asset not in workflow:
            raise ValueError(f"publication asset missing: {asset}")
    if "check-published-artifact-identity.py" not in publish:
        raise ValueError("publication identity check is missing")


class AllOsReleaseContractTests(unittest.TestCase):
    def test_single_platform_guard_cannot_be_reintroduced(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        require_all_platforms(workflow)
        for job in ("build_macos", "smoke_macos", "build_linux", "build_windows"):
            altered = workflow.replace(f"  {job}:\n", f"  {job}:\n    if: false\n", 1)
            with self.subTest(job=job), self.assertRaises(ValueError):
                require_all_platforms(altered)

    def test_step_condition_does_not_count_as_job_skip(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        altered = workflow.replace("      - name: Upload Linux Artifacts\n",
                                   "      - name: Upload Linux Artifacts\n        if: success()\n", 1)
        self.assertNotEqual(altered, workflow)
        require_all_platforms(altered)

    def test_trigger_has_no_legacy_target_argument(self) -> None:
        recipe = (ROOT / "just/release.just").read_text(encoding="utf-8")
        self.assertNotIn("-f target=", recipe)
        self.assertIn(
            "gh workflow run build-and-release.yml -f version={{VERSION}} -f force=$BOOL_FORCE",
            recipe,
        )

    def test_publication_rejects_partial_or_unverified_release(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        require_publication_contract(workflow)
        mutations = (
            workflow.replace("needs.build_linux.result == 'success'", "true", 1),
            workflow.replace("check-published-artifact-identity.py", "missing-identity.py", 1),
            workflow.replace("KatanA-windows-x86_64.msi", "missing.msi", 1),
        )
        for altered in mutations:
            with self.assertRaises(ValueError):
                require_publication_contract(altered)


if __name__ == "__main__":
    unittest.main()
