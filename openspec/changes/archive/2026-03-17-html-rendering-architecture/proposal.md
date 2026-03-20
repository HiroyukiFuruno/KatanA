## Why

現在のHTMLレンダリングは正規表現ベースのアドホックなパース＋条件分岐の直書きで実装されており、拡張性・保守性・テスト可能性のすべてに問題がある。

具体的な不具合:
- inline/block要素の区別がないため改行制御が壊れる（`<a>`, `<img>` は横に流れるべきだがブロック的に縦に並ぶ）
- リンク処理が未分類（外部HTTP/内部ファイル/アンカーの区別なし、ブラウザ/エディタの開き先制御なし）
- パースとレンダリングが混在しUIコンテキスト必須 → ユニットテスト不可能
- 新しいHTML要素の追加が場当たり的 → 技術的負債の蓄積

将来的にはリンクナビゲーション（戻る/進む）、タブ管理、内部リンク解決など複雑な機能が必要になるため、今の段階で基盤を固める必要がある。

## What Changes

- **NEW**: `HtmlNode` 要素モデル — パース済みHTML/Markdownを構造化ツリーで表現
- **NEW**: `DisplayMode` (inline/block) — タグごとの改行制御ルールをコード化
- **NEW**: `LinkTarget` + `LinkAction` — リンク先分類と開き方の統一モデル
- **NEW**: `HtmlRenderer` — メソッドチェーン式のegui描画エンジン
- **REFACTOR**: `html_to_md()` → `HtmlParser::parse()` — `comrak` AST を活用した構造化パーサーへ（自前HTMLパースの最小化）
- **REFACTOR**: `RenderedSection::CenteredMarkdown(String)` → `HtmlBlock(Vec<HtmlNode>)` に汎化（非中央寄せHTMLも統一処理）
- **REFACTOR**: `render_centered_line()` → `HtmlRenderer::render()` — アドホック描画から統一レンダラーへ
- **REMOVE**: `preview_pane_ui.rs` 内のインラインパース関数群（`parse_image_src`, `parse_md_link`, `ensure_svg_extension` 等）
- **REMOVE**: `render_sections` の自動セパレーター挿入（Markdown の `---` は `CommonMarkViewer` が描画）

## Capabilities

### New Capabilities
- `html-element-model`: HTML要素の構造化表現（HtmlNode/DisplayMode）とパーサー
- `link-resolution`: リンク先分類（external/internal/anchor）とアクション決定ロジック
- `html-renderer`: メソッドチェーン式のegui描画エンジン

### Modified Capabilities
- `diagram-block-preview`: プレビューペインのHTML描画が新レンダラーに移行

## Impact

- **katana-core**: `HtmlNode`, `LinkTarget`, `HtmlParser` を追加（`preview.rs` のHTML処理を再構築）
- **katana-ui**: `preview_pane_ui.rs` のアドホック描画を `HtmlRenderer` に置き換え
- **依存関係**: 追加なし（既存の `regex` crate で十分。将来的に `html5ever` 等への移行パスを確保）
- **テスト**: パーサーがUI非依存になるため、UT/ITでの検証が大幅に容易になる
