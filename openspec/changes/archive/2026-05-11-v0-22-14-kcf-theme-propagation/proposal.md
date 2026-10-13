## Why

KatanA が light テーマで動作していても、kcf 経由の Mermaid / Draw.io 描画が dark 的な色で返る経路がある。調査の結果、KatanA はテーマ情報を `RenderInput` に入れているが、kcf 側の実描画がその情報を使わず内部の既定テーマに依存しているため、v0.22.14 で kcf 修正の取り込みと KatanA 側の再発防止を行う。

## What Changes

- kcf 側 issue [HiroyukiFuruno/katana-canvas-forge#4](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/4) の修正を前提に、KatanA の `katana-canvas-forge` dependency をテーマ伝播対応版へ更新する。
- KatanA 側 adapter が、現在のテーマスナップショットを kcf の実描画へ届く形で渡していることを明示的に検証する。
- Mermaid / Draw.io preview の cache key が、kcf 実描画で使われるテーマ差分と一致して変化することを保証する。
- HTML / PDF / PNG / JPEG export でも、preview と同じテーマ情報で図形ブロックが描画されることを保証する。
- kcf の crate version / runtime / renderer profile を手書き文字列で固定せず、実際の依存版と出力メタデータに追従する。

## Capabilities

### New Capabilities

- なし

### Modified Capabilities

- `diagram-block-preview`: kcf 経由の Mermaid / Draw.io 描画が、KatanA の現在テーマを使う契約へ更新する。
- `theme-settings`: テーマ切替と同一モード内の色変更が、kcf backed diagram preview に反映される契約を強化する。
- `markdown-export`: export 時の図形ブロックも、現在テーマと同じ色で描画される契約を追加する。

## Impact

- `Cargo.toml` / `Cargo.lock`: テーマ伝播対応済みの `katana-canvas-forge` へ更新する。
- `crates/katana-core/src/markdown/diagram_backend/`: kcf `RenderInput` へ渡すテーマ情報と version/profile 管理を見直す。
- `crates/katana-core/src/markdown/export/`: export 用 HTML 生成と kcf backed diagram render のテーマ一貫性を確認する。
- `crates/katana-ui/src/preview_pane/`: preview cache key と refresh trigger のテーマ差分検証を追加する。
- `scripts/screenshot/`: light テーマで Mermaid / Draw.io が dark 配色へ戻らない証跡を生成する。
