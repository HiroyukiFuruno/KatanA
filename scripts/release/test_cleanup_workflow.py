"""Keep publication cleanup wired without weakening the existing push owner."""

from pathlib import Path
import unittest

WORKFLOW = Path(__file__).parents[2] / ".github/workflows/build-and-release.yml"


class CleanupWorkflowTests(unittest.TestCase):
    def job(self):
        source = WORKFLOW.read_text()
        self.assertIn("\n  cleanup_default:\n", source)
        return source.split("\n  cleanup_default:\n", 1)[1]

    def test_cleanup_runs_only_after_successful_publication_on_default_checkout(self):
        job = self.job()
        self.assertIn("needs: [preflight, publish]", job)
        self.assertIn("if: needs.publish.result == 'success'", job)
        self.assertIn("ref: ${{ github.event.repository.default_branch }}", job)
        self.assertIn("fetch-depth: 0", job)

    def test_normal_owner_tools_and_target_binding_precede_cleanup(self):
        job = self.job()
        owner = job.index("lefthook install")
        resolve = job.index("scripts/release/resolve-cleanup-target.py")
        execute = job.index("scripts/release/run-release-cleanup.py")
        self.assertLess(owner, resolve)
        self.assertLess(resolve, execute)
        for required in ("docker compose version", "cargo-deny@0.20.2", "setup-bun@",
                         '"--pull-request" "$SOURCE_PR"', '--expected "$EXPECTED"', "--remote"):
            self.assertIn(required, job)
        for forbidden in ("--no-verify", "--force", "LEFTHOOK: 0", "continue-on-error"):
            self.assertNotIn(forbidden, job)

    def test_receipts_are_collected_even_after_cleanup_failure(self):
        job = self.job()
        self.assertIn("if: always()", job)
        self.assertIn(".git/katana-cleanup-receipts/*.jsonl", job)
        self.assertIn("include-hidden-files: true", job)

    def test_version_ref_is_data_not_inline_code_and_assets_preserve_arguments(self):
        source = WORKFLOW.read_text()
        version = source.split("      - name: Determine version\n", 1)[1].split("      - name:", 1)[0]
        script = version.split("        run: |\n", 1)[1]
        self.assertIn("RELEASE_HEAD: ${{ github.event.pull_request.head.ref }}", version)
        self.assertIn('"$RELEASE_HEAD"', script)
        self.assertNotIn("${{ github.event.pull_request.head.ref }}", script)
        self.assertIn("FILES=(release-files/*)", source)
        self.assertIn('gh release upload "$TAG" "${FILES[@]}" --clobber', source)


if __name__ == "__main__":
    unittest.main()
