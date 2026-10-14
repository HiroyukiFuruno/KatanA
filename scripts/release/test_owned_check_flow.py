"""Keep the owner command singular and its native contract tools available."""

import ast
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
OWNER_ENTRY = "run: python3 scripts/release/run-owned-push-check.py"
TOOLS_ENTRY = "run: brew install lefthook just"


def verify_tools_before_readiness(workflow: str):
    native_job = workflow.split("  build_macos:", 1)[0]
    tools = native_job.find(TOOLS_ENTRY)
    readiness = native_job.find("./scripts/release/check-pr-ready.sh")
    if "runs-on: macos-latest" not in native_job or tools < 0 or readiness < tools:
        raise ValueError("native hook contract tools must be installed before readiness")


class OwnedCheckFlowTests(unittest.TestCase):
    def test_pre_push_has_one_fixed_owner_entrypoint(self):
        source = (ROOT / "lefthook.yml").read_text()
        push = source.split("pre-push:\n", 1)[1].split("pre-pr:\n", 1)[0]
        self.assertEqual(push.count(OWNER_ENTRY), 1)
        self.assertNotIn("run: just check\n", push)
        tree = ast.parse((ROOT / "scripts/release/owner_verification.py").read_text())
        commands = [ast.literal_eval(node.value) for node in tree.body if isinstance(node, ast.Assign)
                    and any(isinstance(target, ast.Name) and target.id == "COMMAND" for target in node.targets)]
        self.assertEqual(commands, [["just", "check"]])

    def test_native_cloud_gates_install_contract_tools_first(self):
        for name in ("build-and-release.yml", "release-readiness.yml"):
            with self.subTest(name=name):
                workflow = (ROOT / ".github/workflows" / name).read_text()
                verify_tools_before_readiness(workflow)
                with self.assertRaises(ValueError):
                    verify_tools_before_readiness(workflow.replace(TOOLS_ENTRY, "run: echo skipped"))


if __name__ == "__main__":
    unittest.main()
