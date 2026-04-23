## ADDED Requirements

### Requirement: HtmlNode enum で HTML 要素を構造化表現する

システムは HTML/Markdown の要素を `HtmlNode` enum で構造化ツリーとして表現しなければならない（SHALL）。各バリアントは Text, Image, Link, Heading, Paragraph, LineBreak, Emphasis, Strong を含む。

#### Scenario: img タグのパース

- **WHEN** `<img src="icon.png" alt="icon">` を含む HTML をパースした場合
- **THEN** `HtmlNode::Image { src: "icon.png", alt: "icon" }` を生成する

#### Scenario: リンク内画像（バッジパターン）のパース

- **WHEN** `<a href="LICENSE"><img src="badge.svg" alt="License"></a>` をパースした場合
- **THEN** `HtmlNode::Link { target: InternalFile("LICENSE"), children: [HtmlNode::Image { src: "badge.svg", alt: "License" }] }` を生成する

#### Scenario: 中央寄せ段落のパース

- **WHEN** `<p align="center">text</p>` をパースした場合
- **THEN** `HtmlNode::Paragraph { align: Some(TextAlign::Center), children: [HtmlNode::Text("text")] }` を生成する

#### Scenario: br タグのパース

- **WHEN** `<br>` または `<br/>` を含む HTML をパースした場合
- **THEN** `HtmlNode::LineBreak` を生成する

### Requirement: DisplayMode でタグごとの表示モードを分類する

各 `HtmlNode` バリアントは `display_mode()` メソッドで `DisplayMode::Block` または `DisplayMode::Inline` を返さなければならない（SHALL）。

#### Scenario: inline 要素の分類

- **WHEN** `HtmlNode::Text`, `HtmlNode::Image`, `HtmlNode::Link`, `HtmlNode::LineBreak`, `HtmlNode::Emphasis`, `HtmlNode::Strong` の `display_mode()` を呼んだ場合
- **THEN** `DisplayMode::Inline` を返す

#### Scenario: block 要素の分類

- **WHEN** `HtmlNode::Heading`, `HtmlNode::Paragraph` の `display_mode()` を呼んだ場合
- **THEN** `DisplayMode::Block` を返す

### Requirement: HtmlParser は UI 非依存でテスト可能である

`HtmlParser` は egui や UI フレームワークに一切依存せず、`katana-core` クレートに配置しなければならない（SHALL）。パース結果は `Vec<HtmlNode>` として返す。

#### Scenario: UI 依存なしでのパーステスト

- **WHEN** `HtmlParser::parse("<p>hello</p>")` を egui コンテキストなしで呼んだ場合
- **THEN** `vec![HtmlNode::Paragraph { align: None, children: [HtmlNode::Text("hello")] }]` を返す

### Requirement: 既存の `html_to_md` テストケースが新パーサーでも同等の結果を返す

既存の `html_to_md` 関数のテストケース（バッジ、中央寄せテキスト、画像等）は、新パーサー経由でも同等のレンダリング結果を得なければならない（SHALL）。

#### Scenario: shields.io バッジの変換

- **WHEN** `[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)` を含む HTML をパースした場合
- **THEN** Link ノードの子に Image ノードが含まれ、src が `https://img.shields.io/badge/License-MIT-blue.svg` である
