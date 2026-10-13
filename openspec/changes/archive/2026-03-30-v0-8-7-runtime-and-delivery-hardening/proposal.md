## Why

`v0-8-4-rust-code-refactoring` はコード構造の保守性改善に集中している一方で、実行時の失敗耐性と配布導線の一貫性には未解決の負債が残っています。現状は設定保存失敗の黙殺、破壊的な更新置換、ローカル release と GitHub Actions release の検証差分が併存しており、ユーザー設定の消失や不完全な配布物を招く余地があります。

## What Changes

- 設定永続化を原子的な書き込みに改め、破損ファイルの退避と回復可能エラーの可視化を追加する
- in-app update のインストール経路を、破壊的な `rm -rf` 前提から staged swap + rollback 前提へ変更する
- `make release` と `.github/workflows/release.yml` が同一の preflight と artifact 契約を使うようにする
- release helper script 群に dry-run / smoke-test 可能な入口を用意し、CI から publish なしで検証できるようにする

## Capabilities

### New Capabilities

- `update-install-safety`: 更新アーカイブの検証、段階的な置換、失敗時のロールバックを扱う
- `release-pipeline-consistency`: ローカル release と GitHub Actions release の事前検証と成果物契約を統一する

### Modified Capabilities

- `settings-persistence`: 設定保存を atomic write 化し、破損時の退避と保存失敗の通知を追加する

## Impact

- `crates/katana-platform/src/settings/repository.rs`
- `crates/katana-platform/src/settings/service.rs`
- `crates/katana-ui/src/shell.rs`
- `crates/katana-ui/src/settings_window.rs`
- `crates/katana-core/src/update/download.rs`
- `crates/katana-core/src/update/installer.rs`
- `scripts/release/release.sh`
- `scripts/release/publish-github.sh`
- `scripts/release/update-homebrew.sh`
- `.github/workflows/release.yml`
- `Makefile`
- release / settings / update 関連のテストとドキュメント
