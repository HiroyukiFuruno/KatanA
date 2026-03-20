## 1. モデル定義（katana-core）

- [x] 1.1 `katana-core/src/html/` モジュールを作成し、`mod.rs` で公開
- [x] 1.2 `DisplayMode` enum（Block, Inline）を定義
- [x] 1.3 `TextAlign` enum（Left, Center, Right）を定義
- [x] 1.4 `HtmlNode` enum（Text, Image, Link, Heading, Paragraph, LineBreak, Emphasis, Strong）を定義
- [x] 1.5 `HtmlNode::display_mode()` メソッドを実装
- [x] 1.6 UT: 各バリアントの `display_mode()` が正しいモードを返すことを検証

## 2. リンク解決（katana-core）

- [x] 2.1 `LinkTarget` enum（External, InternalFile, Anchor）を定義
- [x] 2.2 `LinkTarget::resolve(href, base_dir)` を実装: http(s)→External, #→Anchor, それ以外→InternalFile
- [x] 2.3 `LinkAction` enum（OpenInBrowser, NavigateCurrentTab）を定義 ※ OpenInNewTab は YAGNI で保留
- [x] 2.4 `LinkTarget::default_action()` を実装
- [x] 2.5 UT: 外部/内部/アンカーそれぞれの resolve + default_action を検証

## 3. HTML パーサー（katana-core）

- [x] 3.1 `HtmlParser` struct を作成（base_dir を保持）
- [x] 3.2 `comrak` AST の `NodeValue::HtmlBlock` / `HtmlInline` から HTML 文字列を取得する連携を実装
- [x] 3.3 内部タグの属性抽出: `<img>` → `HtmlNode::Image`
- [x] 3.4 内部タグの属性抽出: `<a href>` → `HtmlNode::Link`（href を `LinkTarget::resolve` で分類）
- [x] 3.5 内部タグの属性抽出: `<p align>` → `HtmlNode::Paragraph`
- [x] 3.6 内部タグの属性抽出: `<h1>`-`<h6>` → `HtmlNode::Heading`
- [x] 3.7 内部タグの属性抽出: `<br>` → `HtmlNode::LineBreak`
- [x] 3.8 内部タグの属性抽出: `<em>`, `<strong>` → `HtmlNode::Emphasis`, `HtmlNode::Strong`
- [x] 3.9 Markdown記法のインライン解析: `[text](url)` → `HtmlNode::Link`, `![alt](src)` → `HtmlNode::Image`
- [x] 3.10 ネストされた要素の再帰的パース（`<a>` 内の `<img>` 等）
- [x] 3.11 UT: 既存テストケース（バッジ、中央寄せ、画像）を新パーサーで検証
- [x] 3.12 UT: README.md のバッジ行が正しくパースされ横並びの HtmlNode 構造になることを検証

## 4. HtmlRenderer（katana-ui）

- [x] 4.1 `HtmlRenderer` struct を作成（ui, base_dir, text_color, max_image_width）
- [x] 4.2 メソッドチェーン API: `text_color()`, `max_image_width()`
- [x] 4.3 `render()` メソッド: HtmlNode のマッチと描画ディスパッチ
- [x] 4.4 `render_inline_group()`: 連続インライン要素の収集とフラッシュ
- [x] 4.5 中央寄せ Paragraph 内のインライングループ: `ui.allocate_ui_with_layout()` で幅制限して中央配置
- [x] 4.6 `ensure_svg_extension()` をレンダラー層に配置し、Image 描画時に適用
- [x] 4.7 Link のクリック検出 + `LinkAction` 返却
- [x] 4.8 Link の視覚的表現: 下線付きテキスト、外部/内部で色分け

## 5. 統合・移行

- [x] 5.1 `RenderedSection::CenteredMarkdown(String)` → `HtmlBlock(Vec<HtmlNode>)` に汎化
- [x] 5.2 `preview.rs` の `html_to_md()` を `HtmlParser::parse()` に置き換え
- [x] 5.3 `preview_pane_ui.rs` の `show_section` HtmlBlock 分岐を `HtmlRenderer` に置き換え
- [x] 5.4 アドホック関数群を削除: `render_centered_line`, `render_text_with_links`, `parse_image_src`, `parse_md_link`, `CenteredElement`, `ensure_svg_extension`
- [x] 5.5 `render_sections` から自動セパレーター挿入を完全に削除（Markdown の `---` は `CommonMarkViewer` が描画）

## 6. テスト・品質

- [x] 6.1 既存の `preview_pane.rs` テストモジュール内のパース関数テストを新パーサーテストに移行
- [x] 6.2 IT: 外部リンクと内部リンクが正しく分類されることを検証
- [x] 6.3 `make check-light` が全て通ることを確認
