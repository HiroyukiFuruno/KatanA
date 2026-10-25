"""Small real-child fixtures shared by Nextest shadow tests."""

import textwrap
from pathlib import Path


RUN_UUID = "123e4567-e89b-12d3-a456-426614174000"


def junit_xml(failed: bool) -> str:
    counts = f'tests="1" skipped="0" failures="{int(failed)}" errors="0"'
    outcome = "<failure />" if failed else ""
    return (f'<testsuites {counts} uuid="{RUN_UUID}"><testsuite name="bin" {counts}>'
            f'<testcase classname="bin" name="passes">{outcome}</testcase>'
            '</testsuite></testsuites>')


def phase(root: Path, output: Path, name: str, argv: list[str] | None = None) -> dict:
    template = root / "scripts/release/nextest-shadow.toml"
    if not template.is_file():
        template = root / "template.toml"
        template.write_text("[profile.default]\nretries = 0\n", encoding="utf-8")
    config = output / "junit" / f"{name}.toml"
    return {"phase": name, "argv": argv or ["--config-file", str(config)],
            "config_template": str(template), "junit_config": str(config),
            "junit_report": str(output / "junit" / f"{name}.xml")}


def recording_child(root: Path, exit_code: int = 0, failed_report: bool = False) -> Path:
    child = root / "recording-child.py"
    child.write_text(textwrap.dedent(f'''
        import ast
        import os
        from pathlib import Path
        import sys
        RUN_UUID = {RUN_UUID!r}
        if "-V" in sys.argv:
            print("cargo 1.90.0 (shadow-test)")
            raise SystemExit(0)
        state = Path(os.environ["NEXTEST_STATE_DIR"])
        store = state / "projects/project/records/runs" / RUN_UUID / "store.zip"
        store.parent.mkdir(parents=True)
        store.write_bytes(b"store")
        if {exit_code}:
            raise SystemExit({exit_code})
        config = Path(sys.argv[sys.argv.index("--config-file") + 1])
        report = next(ast.literal_eval(line.split("=", 1)[1].strip()) for line in
                      config.read_text(encoding="utf-8").splitlines()
                      if line.startswith("path = "))
        path = Path(report)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text({junit_xml(failed_report)!r}, encoding="utf-8")
    '''), encoding="utf-8")
    return child
