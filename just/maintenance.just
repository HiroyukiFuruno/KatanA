# ============================================================
# Maintenance
# ============================================================

# 新規profileの並行生成まで排他できないため、自動ビルドでは削除候補だけを確認する。
sweep:
    @{{RTK_CMD}}python3 scripts/ci/cargo_sweep_guard.py --dry-run

# Remove build artifacts
clean: sweep
    @{{RTK_CMD}}python3 scripts/ci/cargo_clean_guard.py --manifest-path Cargo.toml --cargo "{{CARGO}}"

# Update dependency crates safely (respects Cargo.toml SemVer)
update-safe:
    {{CARGO}} update

# Upgrade ALL dependencies to absolute latest versions (including breaking changes)
update:
    {{CARGO}} upgrade -i
    {{CARGO}} update
    {{CARGO}} upgrade --manifest-path scripts/screenshot/Cargo.toml -i
    {{CARGO}} update --manifest-path scripts/screenshot/Cargo.toml

# WHY: 実manifestを書換えると中断時に作業差分が残るため、一時領域だけで診断する。
outdated:
    @{{RTK_CMD}}python3 -B scripts/dependency-diagnostic/real_contract.py --root . --cargo "{{CARGO}}"

# WHY: 診断専用toolを通常検査の必須依存へ追加せず、実tool回帰を明示実行する。
dependency-diagnostic:
    @{{RTK_CMD}}python3 -B scripts/dependency-diagnostic/fixture_contract.py --cargo "{{CARGO}}"

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

# 所有ラベルのあるDocker資源の保持状態と参照を確認する（削除しない）。
docker-volumes-preview:
    {{RTK_CMD}}python3 -B scripts/docker/volume-lifecycle.py preview

# 証跡保存済みで保持期間を過ぎた一回分だけを回収する。
docker-volume-delete-ui-run run_id:
    {{RTK_CMD}}python3 -B scripts/docker/volume-lifecycle.py delete-ui-run {{quote(run_id)}}

# 未参照の所有CIキャッシュ一つを、名前の再入力後に削除する。
docker-volume-delete-cache volume:
    {{RTK_CMD}}python3 -B scripts/docker/volume-lifecycle.py delete-cache {{quote(volume)}}
