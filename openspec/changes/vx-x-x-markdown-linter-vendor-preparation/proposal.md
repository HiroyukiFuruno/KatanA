# 変更提案: katana-markdown-linter ベンダー統合準備

## 背景

`katana-markdown-linter` が独立クレートとして Phase 1〜5 の計画で開発される。
現在 KatanA の `katana-linter` クレート内に存在する Markdown ルール群は、将来的にすべて `katana-markdown-linter` へ移譲される。
取り込み対象の crate/package 名は `katana-markdown-linter`、Rust module 名は `katana_markdown_linter`、CLI executable 名は `kml`、license は MIT として確定している。

しかし、移譲の「受け入れ側」が完成するまでの間、KatanA 側では以下の責務境界を明確化し、移行コストを最小化する準備が必要である。

### 現状の課題

1. **evaluate シグネチャの設計欠陥**: `MarkdownRule::evaluate(&self, file_path, content)` に設定値（Config）が渡らない。`.markdownlint.json` / `.markdownlint.jsonc` で設定した値が Linter の評価ロジックに一切反映されない
2. **Adapter 層の不在**: `katana-markdown-linter` が提供する `LintResult` と KatanA 内部の `MarkdownDiagnostic` を変換する Adapter 層が存在しない
3. **Config Validation の不在**: `MarkdownLintConfig` は JSON の read/write のみ。スキーマに対する値のバリデーション機構がない

## 目的

- [ ] KatanA 側の Markdown ルール評価機構に `.markdownlint.json` 設定値を渡せるようにする（evaluate シグネチャの拡張）
- [ ] `katana-markdown-linter` 統合時に必要となる Adapter 層のインターフェースを定義する
- [ ] Config Validation の基盤を整備し、不正値を検出できるようにする

## スコープ

### 対象

- `MarkdownRule` トレイトの `evaluate` シグネチャ拡張
- `eval.rs` の `evaluate_all` に Config 参照を追加
- Adapter 層インターフェースの設計・定義
- `MarkdownLintConfig` への Validation 機構の追加
- `.markdownlint.jsonc` を将来の vendor crate と同じ config surface として扱う準備

### 対象外（katana-markdown-linter 側で実施）

- 全 active rule の check 実装（Phase 2）
- `pulldown-cmark` ベースの AST パーサへの移行（Phase 1-2）
- CLI の実装（Phase 4）
- upstream markdownlint の更新追従機構（Phase 5）
- crates.io 公開と `kml` executable の install 準備（Phase 3-4）

### 対象外（将来バージョンで実施）

- 各 active rule の設定値による評価分岐の完全実装
- `katana-linter` からの Markdown ルール削除（`katana-markdown-linter` 完成後）

## 影響分析

- **アーキテクチャ**: `MarkdownRule` トレイトのシグネチャ変更は全ルール実装に波及する。マクロ (`official_rule!`, `regex_rule!`) の対応が必要
- **データベース**: なし
- **API**: `MarkdownRule::evaluate` の引数追加（破壊的変更だが、内部 API のため影響は `katana-linter` 内部に閉じる）
- **セキュリティ**: なし
