# ============================================================
# Tests / Checks
# ============================================================

# Run AST-based custom linters (comment style, etc.)
ast-lint:
    {{CARGO}} test -j {{JOBS}} -p katana-linter ast_linter -- --nocapture

# cargo check (type check only, fast)
type-check:
    {{CARGO}} check --workspace

# Run impacted unit tests (local default)
test: test-cargo-sweep-contract test-ci-resource-contract test-html-host-contract test-impacted

# デバッグ情報の容量対策で全test・coverage・受入を削減していないことを確認する。
test-ci-resource-contract:
    {{RTK_CMD}}python3 -B -m unittest discover -s scripts/ci -p test_ci_resource_contract.py -v

# 実Cargoのロック競合と不明なtarget構造で、cache整理が安全に停止することを確認する。
test-cargo-sweep-contract:
    {{RTK_CMD}}python3 -B -m unittest discover -s scripts/ci -p test_cargo_sweep_guard.py -v

# 原本・viewport・fragmentの誤一致を、描画実行前に検出する。
test-html-host-contract:
    {{RTK_CMD}}python3 scripts/ci/test_html_fixture_host_contract.py
    {{RTK_CMD}}python3 scripts/ci/test_html_host_process_group.py

# Run tests for katana-core only
test-core:
    {{CARGO}} test -p katana-core

# Run tests for katana-ui only
test-ui:
    {{CARGO}} test -p katana-ui

# Run the supplied Office and HTML fixtures through the explicit acceptance gate.
test-document-fixtures: test-html-host-contract
    scripts/ci/document-fixture-acceptance.sh

# Execute paint-metrics coordinate extraction independently from Office/V8 integration checks.
test-paint-metrics:
    {{CARGO}} test --locked --manifest-path scripts/paint-metrics/Cargo.toml

# Run tests with verbose output
test-verbose:
    {{CARGO}} test --workspace -- --nocapture

# Run a specific test (e.g., just T=test_name test-specific)
test-specific:
    {{CARGO}} test --workspace -- {{T}}

# Run integration tests - fixture tests only (requires: egui_kittest)
test-integration: check-native-office-worker-contract
    {{RTK_CMD}}bash scripts/ci/with-office-test-worker.sh "{{CARGO}} test --locked -j {{JOBS}} -q --workspace --test ui_integration_fixture -- --test-threads={{JOBS}} fixture"

# WHY: Linux workspace test は current_exe の推測ではなく実 Office worker を使う必要がある。
check-linux-office-worker-contract:
    {{RTK_CMD}}bash scripts/ci/check-linux-office-worker-contract.sh

# WHY: Native test entrypoint は実 Office worker の絶対パスを明示する必要がある。
check-native-office-worker-contract:
    {{RTK_CMD}}bash scripts/ci/check-native-office-worker-contract.sh

# Verify test execution in isolated Linux environment
check-linux: check-linux-office-worker-contract
    {{DOCKER}} compose -f platforms/linux/ci/compose.yml run --rm -e RUSTFLAGS="{{RUSTFLAGS}} -C link-arg=-fuse-ld=lld" ubuntu-test bash -c "ulimit -c 0 && cargo sweep --time 7 --dry-run && python3 scripts/ci/test_html_fixture_host_contract.py && python3 scripts/ci/test_html_host_process_group.py && cargo build --locked -p katana-ui --bin kdv-office-worker && KATANA_KDV_OFFICE_WORKER=/app/target/debug/kdv-office-worker cargo test --locked -q --workspace"

# Verify Windows cross-compilation without running tests
check-windows:
    {{DOCKER}} compose -f platforms/windows/ci/compose.yml run --rm windows-test bash -c "cargo sweep --time 7 --dry-run && cargo xwin check -q --workspace --target x86_64-pc-windows-msvc --tests"

# Verify test/compilation across all target platforms (Linux, Windows)
check-platforms: check-linux check-windows

# Run tests and verify 100% test coverage (requires cargo-llvm-cov)
coverage:
    JOBS={{JOBS}} {{RTK_CMD}}scripts/ci/coverage.sh

# Quick verification (skip slow fixture tests; ast-lint runs inside cargo test)
check-light: sweep fmt-check biome-js-lint lint-impacted
    {{RTK_CMD}}scripts/runner/impacted.py test -- --skip fixture
    @echo "✅ Light checks passed"

# Fast impacted verification (local default)
check: sweep fmt-check biome-js-lint lint-impacted test check-platforms
    @echo "✅ All checks passed"

# Full local verification incl. cross-platform checks
check-local: sweep fmt lint test-integration coverage check-platforms
    @echo "✅ All checks passed"

# Pre-push hook equivalent checks
pre-push: check-full
