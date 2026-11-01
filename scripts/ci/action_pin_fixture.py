"""Run the real workflow contracts against isolated action-pin variations."""

import re
import shutil
import subprocess
import sys


PIN_CASES = (
    (
        "Install Python for Windows inventory", "actions/setup-python",
        ("5fda3b95a4ea91299a34e894583c3862153e4b97", "6d4fd22b20465ccc0f7354476588e8df7c9ca0e0"),
        "scripts/ci/test_windows_ci_observation_contract.py",
        "WindowsCiObservationContractTest.test_windows_inventory_has_supported_python_and_real_pipe_regressions",
        (("python-version: '3.14'", "python-version: '3.13'"),
         ("runner.os == 'Windows'", "runner.os == 'Linux'")),
    ),
    (
        "Install coverage tool", "taiki-e/install-action",
        ("1e29709c0ec63babbe8f266397a465f29a825f91", "86f7a0ffb25f40114606501a311ebcfaf882ea32"),
        "scripts/release/test-release-flow-contract.py",
        "WorkflowContractTests.test_coverage_installer_is_pinned_before_execution",
        (("tool: cargo-llvm-cov@0.9.1", "tool: cargo-llvm-cov@0.9.0"),
         ("fallback: none", "fallback: cargo"),
         ("runner.os == 'macOS'", "runner.os == 'Linux'")),
    ),
)


def prepare_pin_fixture(root, fixture):
    for relative in (
        "scripts/ci/action_pin_fixture.py",
        "scripts/ci/test_ci_resource_contract.py",
        "scripts/ci/test_windows_ci_observation_contract.py",
        "scripts/release/test-release-flow-contract.py",
    ):
        target = fixture / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / relative, target)
    target = fixture / ".github/workflows/test-and-build.yml"
    target.parent.mkdir(parents=True, exist_ok=True)
    return target


def action_pin_variants(original, action, revisions, requirements):
    match = re.search(r"(?m)^        uses: [^\n#]+", original)
    if match is None:
        raise AssertionError("fixture must contain the actual uses line")
    uses = match.group(0).rstrip()
    variants = [(revision, original.replace(uses, f"        uses: {action}@{revision}"), True)
                for revision in revisions]
    for ref in ("main", "v7", revisions[0][:7], revisions[0] + "0", "g" * 40, revisions[0] + "/extra"):
        variants.append((ref, original.replace(uses, f"        uses: {action}@{ref}"), False))
    valid = f"        uses: {action}@{revisions[0]}"
    wrong = f"        uses: other/action@{revisions[0]}"
    variants.extend((
        ("wrong action", original.replace(uses, wrong), False),
        ("comment", original.replace(uses, f"        # {valid.strip()}"), False),
        ("run body", original.replace(uses, f"        run: |\n          {valid.strip()}"), False),
        ("other step", original.replace(uses, wrong) + f"\n      - name: Decoy\n{valid}\n", False),
    ))
    variants.extend((before, original.replace(uses, valid).replace(before, after), False)
                    for before, after in requirements)
    return variants


def run_pin_contract(fixture, script, test):
    # CLIの全体検査を省略せず、fixtureでは対象の実契約だけを別processで選ぶ。
    runner = (
        "import pathlib,runpy,sys,unittest; "
        "sys.path.insert(0,str(pathlib.Path(sys.argv[1]).parent)); "
        "scope=runpy.run_path(sys.argv[1]); "
        "cls,method=sys.argv[2].split('.',1); "
        "result=unittest.TextTestRunner().run(unittest.TestSuite([scope[cls](method)])); "
        "sys.exit(not result.wasSuccessful())"
    )
    return subprocess.run(
        [sys.executable, "-B", "-c", runner, str(fixture / script), test],
        cwd=fixture, capture_output=True, text=True, timeout=20,
    )
