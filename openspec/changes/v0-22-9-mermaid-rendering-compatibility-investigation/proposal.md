## Why

Mermaid.js への描画経路移行により、`mmdc` 時代と比べて一部の図形の見た目、余白、基準線、サイズ感に差が出ている。今回のガントチャート修正では赤い「今日」線の影響は抑えたが、同種の差分が他の Mermaid 図形にも残っている可能性がある。

リリース直前の個別修正で全パターンを追うとスコープが肥大化するため、version に紐づかない調査 OpenSpec として、後続で体系的に扱える状態にする。

## What Changes

- Mermaid.js 描画と旧 `mmdc` 描画の差分を調査対象として明文化する。
- ガントチャート以外の Mermaid 図形について、見た目、サイズ、テーマ反映、基準線、余白、中央寄せの比較観点を定義する。
- 修正対象を即時実装ではなく、調査、fixture 作成、再現条件の整理、優先度判断に分ける。
- この change 自体は特定バージョンへ割り当てず、後続の versioned change へ移送できる調査バックログとして扱う。

## Capabilities

### New Capabilities

- `mermaid-rendering-compatibility-investigation`: Mermaid.js 描画を旧 `mmdc` 相当の見た目へ近づけるための調査、比較基準、fixture 管理を扱う。

### Modified Capabilities

- `diagram-block-preview`: 後続対応で Mermaid 図形の描画互換性要件を追加する可能性がある。

## Impact

- `crates/katana-core/src/markdown/mermaid_renderer/`
- `crates/katana-core/tests/markdown_mermaid.rs`
- `crates/katana-ui/tests/integration/preview_pane/diagrams.rs`
- `scripts/screenshot/` によるレビュー用スクリーンショット、動画の生成シナリオ
- Mermaid.js、`mmdc`、ヘッドレスブラウザの既定 viewport、描画コンテナ幅、PNG 化処理
