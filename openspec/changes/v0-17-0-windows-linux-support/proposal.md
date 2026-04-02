## Why

KatanA は `eframe`、`rfd`、`dirs` などのクロスプラットフォーム基盤を既に使っている一方で、実際の起動、テーマ検出、ロケール検出、ネイティブメニュー、更新、配布導線は macOS 前提のまま残っている。`v0.18.0` では Windows / Linux を「将来の余地」ではなく、実際にビルド・起動・配布できる対象へ引き上げるため、runtime と release の両面で対応範囲を定義する必要がある。

## What Changes

- Windows x86_64 と Linux x86_64 を `v0.18.0` の正式サポート対象に加え、workspace / editor / preview の主要フローが各 OS で起動する状態まで持っていく。
- OS 依存処理を `katana-platform` を中心に整理し、テーマ検出、ロケール検出、ショートカットの primary modifier、メニュー surface、フォント探索を macOS 固定実装から切り離す。
- macOS のネイティブメニューは維持しつつ、Windows / Linux では同じ `AppAction` 群へ到達できる in-app command surface を追加する。
- release と配布物を macOS 専用から拡張し、macOS は既存の `.dmg` / `.zip` を維持しつつ、Windows は `.zip`、Linux は `.tar.gz` を GitHub Releases で配布できるようにする。
- in-app update は platform-aware な asset 解決へ切り替え、macOS は既存の auto-install を維持し、Windows / Linux は manual download / release page へ誘導する。
- README、development guide、support matrix、badge、CI を macOS only 前提から更新する。

## Capabilities

### New Capabilities

- `desktop-platform-support`: macOS / Windows / Linux の各 OS でシェル起動と主要 Markdown ワークフローを維持する。
- `desktop-release-distribution`: 対応 OS ごとの release artifact、CI build、インストール導線を提供する。
- `platform-update-policy`: OS ごとに更新 asset と更新導線を切り替え、未対応 install path で壊れないようにする。

### Modified Capabilities

- `settings-persistence`: 設定ファイル保存先の契約を各 OS の標準 config directory に揃える。
- `theme-settings`: 初回起動時の default theme を macOS 以外でも OS theme へ追従できるようにする。
- `i18n`: 初回起動時の default language を system locale へ追従できるようにする。
- `font-settings`: Windows / Linux でも editor / preview の default font fallback が成立するようにする。
- `menu-enhancement`: ネイティブメニュー非対応 OS でも同等の command access を提供する。
- `app-branding`: アプリケーションアイコンの表示契約を macOS Dock 限定から対応 desktop OS 全体へ拡張する。

## Impact

- 主な影響範囲は `crates/katana-platform/src/os_theme.rs`、`crates/katana-platform/src/os_fonts.rs`、`crates/katana-platform/src/settings/*`、`crates/katana-ui/src/main.rs`、`crates/katana-ui/src/native_menu.rs`、`crates/katana-ui/src/shell_ui.rs`、`crates/katana-ui/src/font_loader/*`、`crates/katana-core/src/update/*`、`Makefile`、`scripts/package-mac.sh`、`scripts/dmg.sh`、`scripts/release/*`、`.github/workflows/*`、`README*.md`、`docs/development-guide*.md`。
- 追加の platform helper や軽量 dependency が必要になっても、導入箇所は `katana-platform` に閉じ込める。
- native installer は `v0.18.0` の必須条件に含めず、Windows は portable `.zip`、Linux は portable `.tar.gz` を初期配布形とする。
