## ADDED Requirements

### Requirement: HtmlRenderer はメソッドチェーンで設定を組み合わせ可能にする
`HtmlRenderer` は Builder パターンのメソッドチェーンで設定（テキスト色、画像最大幅等）を組み合わせ可能でなければならない（SHALL）。

#### Scenario: メソッドチェーンによるレンダラー設定
- **WHEN** `HtmlRenderer::new(ui, base_dir).text_color(color).max_image_width(400.0).render(&nodes)` を呼んだ場合
- **THEN** 指定されたテキスト色と画像最大幅でノードが描画される

### Requirement: inline 要素は横に流れ、block 要素で改行する
レンダラーは連続するインライン要素を `ui.horizontal()` でまとめて横並びに描画し、ブロック要素に遭遇した時点でインライングループをフラッシュし改行しなければならない（SHALL）。

#### Scenario: バッジ行（複数インライン画像）の横並び描画
- **WHEN** 4つの `HtmlNode::Image`（バッジ）が連続するインライン要素として存在する場合
- **THEN** 4つのバッジが同一行に横並びで描画される

#### Scenario: 段落内テキスト＋画像のインラインフロー
- **WHEN** `HtmlNode::Paragraph` の children に Text("English | "), Link(Text("日本語")) が含まれる場合
- **THEN** "English | " と "日本語"（リンク）が同一行に描画される

#### Scenario: ブロック要素による改行
- **WHEN** `HtmlNode::Paragraph` の後に別の `HtmlNode::Paragraph` が存在する場合
- **THEN** 2つの段落の間に改行が入る

### Requirement: center 属性付き Paragraph は中央寄せで描画する
`align: Some(TextAlign::Center)` を持つ `HtmlNode::Paragraph` の子要素は中央寄せレイアウトで描画しなければならない（SHALL）。

#### Scenario: 中央寄せテキストの描画
- **WHEN** `HtmlNode::Paragraph { align: Some(Center), children: [Text("hello")] }` を描画する場合
- **THEN** "hello" が利用可能幅の中央に描画される

#### Scenario: 中央寄せ画像の描画
- **WHEN** `HtmlNode::Paragraph { align: Some(Center), children: [Image { src, alt }] }` を描画する場合
- **THEN** 画像が利用可能幅の中央に描画される

### Requirement: SVG 拡張子補正はレンダラー層で行う
egui の SVG ローダーは URI が `.svg` で終わることを要求するため、既知のバッジサービスの URL に `.svg` を付加する処理はレンダラー層で行わなければならない（SHALL）。モデル層の URL は変更しない。

#### Scenario: shields.io URL の SVG 拡張子補正
- **WHEN** `Image { src: "https://img.shields.io/github/v/release/org/repo" }` を描画する場合
- **THEN** egui には `"https://img.shields.io/github/v/release/org/repo.svg"` として渡される

#### Scenario: 既に .svg がある URL は変更しない
- **WHEN** `Image { src: "https://img.shields.io/badge/License-MIT-blue.svg" }` を描画する場合
- **THEN** URL は変更されずそのまま使用される

### Requirement: リンクのクリック時に LinkAction を返す
レンダラーはリンクテキストをクリック可能に描画し、クリック時に対応する `LinkAction` を返さなければならない（SHALL）。

#### Scenario: 外部リンクのクリック
- **WHEN** `LinkTarget::External("https://github.com")` のリンクがクリックされた場合
- **THEN** `LinkAction::OpenInBrowser` を返す

#### Scenario: 内部リンクのクリック
- **WHEN** `LinkTarget::InternalFile("README.ja.md")` のリンクがクリックされた場合
- **THEN** `LinkAction::NavigateCurrentTab` を返す
