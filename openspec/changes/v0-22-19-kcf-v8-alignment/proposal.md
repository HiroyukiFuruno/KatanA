## Why

`katana-diagram-renderer` 0.1.0 は `v8 = "=147.4.0"` を要求する一方、KatanA が参照している `katana-canvas-forge` 0.1.6 は `v8 = "=139.0.0"` を要求している。V8 は 1 プロセス内で複数バージョン（version）を安全に共存できないため、Mermaid / Draw.io の描画ワーカー（worker）が起動時に停止し、プレビュー（preview）上では `[Mermaid] Diagram render worker disconnected before producing a result.` が表示される。

kcf 側は v0.1.7 で `v8 = "=147.4.0"` へ追従済みのため、KatanA v0.22.19 では課題（issue）[#293](https://github.com/HiroyukiFuruno/KatanA/issues/293) として依存バージョンを揃え、描画と出力（export）の回帰を確認する。

## What Changes

- `katana-canvas-forge` を `0.1.6` から `0.1.7` へ更新する。
- 作業領域（workspace）の `v8` 固定指定（pin）を `=139.0.0` から `=147.4.0` へ更新する。
- `Cargo.lock` を `katana-canvas-forge` と `v8` の整合が取れた状態へ更新する。
- Mermaid / Draw.io プレビューがワーカー切断（worker disconnect）で全面失敗しないことを確認する。
- HTML / PDF / PNG / JPEG 出力が kcf 0.1.7 経由で回帰していないことを確認する。

## Capabilities

### New Capabilities

- なし

### Modified Capabilities

- `diagram-block-preview`: V8 を使う描画依存関係（V8-backed renderer dependency）のバージョン不整合で、対応済み Mermaid / Draw.io ブロックがワーカー起動前に失敗しないことを明確にする。
- `markdown-export`: kcf 経由の HTML / PDF / PNG / JPEG 出力が、V8 を使う依存関係（V8-backed dependency）のバージョン不整合で停止しないことを明確にする。

## Impact

- `Cargo.toml`: 作業領域の依存関係（workspace dependencies）の `katana-canvas-forge` と `v8`
- `Cargo.lock`: `katana-canvas-forge` / `v8` / 関連する間接依存（transitive dependencies）
- `crates/katana-core`: 図形バックエンド（diagram backend）と出力経路のビルド（build）/ テスト（test）
- `crates/katana-ui/tests/integration/preview_pane/diagrams.rs`: Mermaid / Draw.io プレビューの回帰確認
