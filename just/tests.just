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
test: test-cargo-sweep-contract test-ci-resource-contract test-html-host-contract test-commonmark-html-trace test-commonmark-backend test-custom-cursor-cache test-wgpu-native-surface-size test-impacted

# vendor workspaceの別lockを更新せず、依存なしの実要約moduleを直接検査する。
test-commonmark-html-trace:
    mkdir -p target/contract-tests
    {{RTK_CMD}}rustc --edition 2024 -D warnings --test vendor/egui_commonmark_upstream/egui_commonmark/src/parsers/pulldown/html_trace_summary.rs -o target/contract-tests/commonmark-html-trace.exe
    {{RTK_CMD}}target/contract-tests/commonmark-html-trace.exe

# root lockとUIのfeature集合で、workspace外の既存patched backendを検査する。
test-commonmark-backend:
    {{CARGO}} test --locked -j {{JOBS}} -p katana-ui -p egui_commonmark_backend --lib lazy_highlighting_tests

# workspace外のcursor修正も、UIと同じfeature集合で通常検査へ含める。
test-custom-cursor-cache:
    {{CARGO}} test --locked -j {{JOBS}} -p katana-ui -p egui-winit --lib cursor_cache::tests

# 実描画に使う寸法判定moduleを、追加のGPU環境なしで全OSの通常検査へ含める。
test-wgpu-native-surface-size:
    mkdir -p target/contract-tests
    {{RTK_CMD}}rustc --edition 2024 -D warnings --test vendor/egui-wgpu/src/native_surface_size.rs -o target/contract-tests/wgpu-native-surface-size.exe
    {{RTK_CMD}}target/contract-tests/wgpu-native-surface-size.exe

# デバッグ情報の容量対策で全test・coverage・受入を削減していないことを確認する。
test-ci-resource-contract:
    {{RTK_CMD}}python3 -B -m unittest discover -s scripts/ci -p test_ci_resource_contract.py -v
    {{RTK_CMD}}python3 -B -m unittest discover -s scripts/ci -p test_native_profiler_job_contract.py -v
    {{RTK_CMD}}python3 -B -m unittest discover -s scripts/runner -p test_impacted.py -v

# 実Cargoのロック競合と不明なtarget構造で、cache整理が安全に停止することを確認する。
test-cargo-sweep-contract:
    {{RTK_CMD}}python3 -B -m unittest discover -s scripts/ci -p 'test_cargo_*guard.py' -v

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

# WHY: テーマ設定値だけではPDFの実色を保証できないため、明示Popplerで実出力を検査する。
test-pdf-theme:
    {{CARGO}} test --locked -j {{JOBS}} -p katana-ui --lib --features external-fixture-acceptance shell_ui::pdf_theme_tests -- --test-threads=1 --nocapture

# Execute paint-metrics coordinate extraction independently from Office/V8 integration checks.
test-paint-metrics:
    {{CARGO}} test --locked --manifest-path scripts/paint-metrics/Cargo.toml

# Run tests with verbose output
test-verbose:
    {{CARGO}} test --workspace -- --nocapture

# Run a specific test (e.g., just T=test_name test-specific)
test-specific:
    {{CARGO}} test --workspace -- {{T}}

# WHY: 採取環境の成否を製品の必須検査に混ぜず、同じ実採取回帰を明示実行する。
test-native-html-diagnostics:
    @if [ "$(uname -s)" != "Darwin" ]; then printf '%s\n' 'Native HTML sampling requires macOS'; exit 1; fi
    {{CARGO}} test --locked -p katana-ui --lib --features native-profiler-diagnostics app::action::html_navigation::diagnostics::tests::sample_captures_owned_child_pid_to_workspace_failure_directory -- --exact --nocapture

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
    {{DOCKER}} compose -f platforms/linux/ci/compose.yml run --rm -e RUST_TEST_THREADS="{{RUST_TEST_THREADS}}" -e RUSTFLAGS="{{RUSTFLAGS}} -C link-arg=-fuse-ld=lld" ubuntu-test bash -c "ulimit -c 0 && cargo sweep --time 7 --dry-run && python3 scripts/ci/test_html_fixture_host_contract.py && python3 scripts/ci/test_html_host_process_group.py && cargo build --locked -p katana-ui --bin kdv-office-worker && KATANA_KDV_OFFICE_WORKER=/app/target/debug/kdv-office-worker cargo test --locked -q --workspace"

# Verify Windows cross-compilation without running tests
check-windows:
    {{DOCKER}} compose -f platforms/windows/ci/compose.yml run --rm windows-test bash -c "cargo sweep --time 7 --dry-run && cargo xwin check -q --workspace --target x86_64-pc-windows-msvc --tests"

# Verify test/compilation across all target platforms (Linux, Windows)
check-platforms: check-linux check-windows

# Run tests and verify 100% test coverage (requires cargo-llvm-cov)
coverage:
    JOBS={{JOBS}} {{RTK_CMD}}scripts/ci/coverage.sh

# WHY: 同等性を測る候補を既存の必須検査から分離し、未証明の省略を防ぐ。
nextest-shadow:
    CARGO={{quote(CARGO)}} {{RTK_CMD}}python3 -B scripts/release/run-nextest-shadow.py

# WHY: 比較候補の安全契約だけを、製品の重い再ビルド無しで検証する。
test-nextest-shadow-contract:
    {{RTK_CMD}}python3 -B -m unittest discover -s scripts/release -p 'test_nextest_shadow*.py' -v

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
pre-push: check
