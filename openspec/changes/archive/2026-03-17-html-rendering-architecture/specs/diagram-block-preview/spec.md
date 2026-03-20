## MODIFIED Requirements

### Requirement: プレビューペインの HTML セクション描画
プレビューペインは `RenderedSection::CenteredMarkdown` のレンダリングを、アドホックなパース＋描画ロジックから `HtmlParser` + `HtmlRenderer` の3層アーキテクチャに移行しなければならない（SHALL）。

#### Scenario: 既存の CenteredMarkdown セクションが新レンダラーで描画される
- **WHEN** HTML の `<p align="center">` ブロックを含む Markdown をプレビューする場合
- **THEN** `HtmlParser` でパースされた `HtmlNode` ツリーを `HtmlRenderer` で描画する

#### Scenario: 既存のバッジ表示が維持される
- **WHEN** shields.io バッジを含む README をプレビューする場合
- **THEN** バッジがテキスト付きで横並びに中央寄せで表示される

#### Scenario: セクション間にセパレーターが自動挿入されない
- **WHEN** 複数の `RenderedSection` を順に描画する場合
- **THEN** Markdown ソースに `---` がない限りセパレーターは表示されない
