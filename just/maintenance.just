# ============================================================
# Maintenance
# ============================================================

# 新規profileの並行生成まで排他できないため、自動ビルドでは削除候補だけを確認する。
sweep:
    @{{RTK_CMD}}python3 scripts/ci/cargo_sweep_guard.py --dry-run

# Remove build artifacts
clean: sweep
    cargo clean

# Update dependency crates safely (respects Cargo.toml SemVer)
update-safe:
    {{CARGO}} update

# Upgrade ALL dependencies to absolute latest versions (including breaking changes)
update:
    {{CARGO}} upgrade -i
    {{CARGO}} update
    {{CARGO}} upgrade --manifest-path scripts/screenshot/Cargo.toml -i
    {{CARGO}} update --manifest-path scripts/screenshot/Cargo.toml

# List outdated dependencies (requires cargo-outdated)
outdated:
    @cp Cargo.toml Cargo.toml.bak
    @sed -e '/^\[patch\.crates-io\]/,$d' Cargo.toml.bak > Cargo.toml
    @{{CARGO}} outdated --workspace || (mv Cargo.toml.bak Cargo.toml && exit 1)
    @mv Cargo.toml.bak Cargo.toml

# Start the Linux verification environment
linux-up:
    bash platforms/linux/init.sh

# Stop the Linux verification environment
linux-down:
    bash platforms/linux/down.sh

# Run tests for impacted packages only
test-impacted: check-native-office-worker-contract
    {{RTK_CMD}}bash scripts/ci/with-office-test-worker.sh "{{RTK_CMD}}scripts/runner/impacted.py test -- --skip fixture"

# Run Clippy for impacted packages only
lint-impacted:
    {{RTK_CMD}}scripts/runner/impacted.py clippy

# Full verification (fmt + clippy + fixture IT + 100% coverage; ast-lint runs inside coverage)
check-full: test-cargo-sweep-contract test-ci-resource-contract fmt-check lint test-integration coverage check-platforms supply-chain
    @echo "✅ All checks passed"
