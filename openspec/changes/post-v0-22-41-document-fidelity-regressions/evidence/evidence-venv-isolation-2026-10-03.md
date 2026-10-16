# Isolate the evidence Python environment from Cargo profiles

PR #346 comment 4171550246 correctly identified a P2 introduced by the
previous PNG-decoder bootstrap: `target/release-evidence-python` looks like a
target triple to the existing Cargo sweep scanner, but contains venv directories.
Its presence triggers `CleanupUnavailable` and safely skips the sweep.

The preflight environment is now `tmp/release-evidence-python`, outside Cargo's
target tree and covered by the existing `tmp` ignore rule. No scanner exclusions,
cleanup policy, source identity rules, or acceptance thresholds were relaxed.

## Reproduction and verification

- `test_preflight_evidence_environment_does_not_disable_cargo_sweep` reads the
  actual preflight path, creates representative real venv directories and Cargo
  profiles, and invokes the unmodified production profile scanner.
- Before the path repair it fails with `CleanupUnavailable: ambiguous target
  profile directory`, recorded in `tmp/evidence-venv-sweep-red.log`.
- After the repair all 56 acceptance/release-gate tests pass via the newly
  created environment (`tmp/evidence-venv-gates-green.log`).
- Existing `just test-cargo-sweep-contract` passes all four tests, including
  real Cargo build locking and safe rejection of ambiguous structures.
- `zsh -n`, `pip check`, `git check-ignore` and `git diff --check` pass.

The exact old 27 MiB environment was verified to be an ordinary directory with
our Python 3.14.3 venv configuration, then recoverably moved to
`tmp/trash/2026-10-03-evidence-venv.b1kiWn/release-evidence-python`. A fresh venv
was created at the new path rather than reusing relocated shebangs. Source,
Cargo build artifacts and other processes were not modified or removed.

## Separate existing limitation

Actual `just sweep` still safely skips for the pre-existing nested
`target/llvm-cov-target` structure. Its exit zero is not a successful sweep.
This is recorded separately; removing the evidence environment does not prove
that every existing cache layout is supported. No coverage artifacts were
deleted and no scanner bypass was added to claim recovery.

## Self-review

PASS for the bounded environment-isolation repair: the test exercises actual
profile discovery rather than a mocked scanner; the current environment is
outside `target`; the decoder and subprocess gates use one executable; existing
PNG corruption and all acceptance thresholds remain tested. Product acceptance,
upstream performance publication, all-target packaging and release are not
completed by this repair.
