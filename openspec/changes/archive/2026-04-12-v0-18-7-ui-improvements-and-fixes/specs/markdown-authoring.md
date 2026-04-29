## MODIFIED Requirements

### Requirement: 平文URLの自動リンク検出

システムは、Markdown文書内の平文URL（例: `<https://example.com`）を検出し、自動的にクリック可能なリンクとしてレンダリングしなければならない（SHALL）。>

#### Scenario: 平文URLのリンク化

- **WHEN** 文書中に `<https://dummy.com`> というテキストが含まれている時
- **THEN** プレビュー画面ではこれが青文字の下線付きリンクとして表示され、クリックでブラウザが開く
