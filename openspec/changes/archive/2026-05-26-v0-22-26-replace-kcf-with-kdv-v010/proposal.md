## Why

KatanA の preview / export 経路には `katana-canvas-forge`（kcf）依存が残っており、図形描画・Markdown 表示・出力責務の境界が `katana-document-viewer`（kdv）移行後の設計とずれている。
v0.22.26 では kcf 依存を排除し、次期 `katana-document-viewer` v0.1.0 を文書表示・出力の主経路へ完全載せ替えする。

## What Changes

- KatanA workspace から `katana-canvas-forge` dependency と kcf adapter を削除する。
- `katana-document-viewer` v0.1.0 を crates.io dependency として追加し、Markdown preview / HTML / PDF / PNG / JPEG export を kdv 経由に統一する。
- `katana-diagram-renderer`（kdr）は crates.io 経由の dependency として追加・維持し、図形描画は kdv から kdr を呼ぶ境界に寄せる。
- 既存の diagram cache、theme snapshot、export parity の契約を kdv 経由でも維持する。
- **BREAKING**: KatanA 内部実装は kcf API を呼び出さない。kcf 固有 DTO、adapter、依存整合チェックは撤去対象になる。

## Capabilities

### New Capabilities

- なし

### Modified Capabilities

- `diagram-block-preview`: Mermaid / Draw.io / PlantUML preview を kcf 前提から kdv v0.1.0 + crates.io kdr 前提へ変更する。
- `markdown-export`: HTML / PDF / PNG / JPEG export を kcf から kdv v0.1.0 へ完全移行する。
- `theme-settings`: テーマスナップショット伝播先を kcf から kdv / kdr 境界へ更新する。

## Impact

- 依存関係: workspace `Cargo.toml`、`Cargo.lock`、`crates/*/Cargo.toml`
- 実装境界: preview / export adapter、diagram rendering adapter、diagram cache の version / theme fingerprint 連携
- 検証: `./scripts/openspec validate v0-22-26-replace-kcf-with-kdv-v010 --strict`、`cargo tree` による kcf 排除確認、preview / export の回帰テスト、`just check-local`
