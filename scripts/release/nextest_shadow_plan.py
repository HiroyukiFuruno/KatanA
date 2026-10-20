"""Build a diagnostic-only cargo-llvm-cov nextest shadow plan."""

from pathlib import Path


VERSION = "0.9.148"
PHASES = {
    "workspace-lib-bin": ("--workspace", "--lib", "--bins"),
    "workspace-integration": ("--workspace", "--test", "*", "--exclude", "katana-ui",
                               "--", "--skip", "fixture"),
    "ui-parallel": ("-p", "katana-ui", "--test", "ui_integration_parallel"),
    "ui-serial": ("-p", "katana-ui", "--test", "ui_integration_serial",
                  "--test-threads", "1"),
}
BASE_ARGUMENTS = ("llvm-cov", "nextest", "--no-report", "--no-cfg-coverage", "--locked",
                  "--no-fail-fast", "--retries", "0", "--profile", "default",
                  "--ignore-default-filter")


def phase_plan(repo: Path, output: Path) -> list[dict]:
    """Return four isolated invocations sharing one coverage target."""
    if not isinstance(repo, Path) or not isinstance(output, Path):
        raise TypeError("shadow plan paths must be Path values")
    config = repo / "scripts/release/nextest-shadow.toml"
    user_config = repo / "scripts/release/nextest-shadow-user.toml"
    target = output / "coverage-target"
    plans = []
    for phase, selection in PHASES.items():
        argv = [*BASE_ARGUMENTS, "--config-file", str(config), "--user-config-file",
                str(user_config), *selection]
        plans.append({"phase": phase, "argv": argv, "target_dir": str(target),
                      "receipt_dir": str(output / phase)})
    return plans


def shadow_contract(repo: Path) -> dict:
    """Return non-authoritative residuals and config paths for the runner."""
    if not isinstance(repo, Path):
        raise TypeError("shadow contract path must be a Path")
    return {"execution_equivalent": False, "reuse_authorized": False,
            "normal_remaining": True, "doctest_remaining": True,
            "non_instrumented_fixture_remaining": True,
            "config_file": str(repo / "scripts/release/nextest-shadow.toml"),
            "user_config_file": str(repo / "scripts/release/nextest-shadow-user.toml")}
