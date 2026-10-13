# AST Linter タスク

## 1. Setup and AST Infrastructure

- [x] 1.1 `syn`, `ignore` を workspace dev-dependencies に追加
- [x] 1.2 `tests/ast_linter.rs` を作成
- [x] 1.3 `ignore` で `.rs` ファイルを走査するロジック構築

## 2. Core Visitor Implementation

- [x] 2.1 `syn::visit::Visit` による基本 Visitor を構築
- [x] 2.2 `visit_expr_method_call` / `visit_expr_call` のオーバーライド

## 3. i18n Hardcode Prevention Rule

- [x] 3.1 UI メソッド (`ui.label`, `RichText::new` 等) のハードコード文字列検知
- [x] 3.2 第1引数が `LitStr` かどうかの判定
- [x] 3.3 ファイルパス・行番号・内容を収集
- [x] 3.4 違反 0 件をアサート

## 4. Allowlist Implementation

- [x] 4.1 `is_allowed_string` (記号・絵文字・数値のみ文字列を許可)
- [x] 4.2 i18n ルールへの Allowlist 統合

## 5. Magic Number Detection Rule

- [x] 5.1 `MagicNumberVisitor` 実装 (const/static 外の数値リテラル検出)
- [x] 5.2 `is_allowed_number` 許可リスト (0, 1, -1, 2, 100)
- [x] 5.3 `#[cfg(test)]` ブロック (mod, fn, impl メソッド) のスキップ
- [x] 5.4 全クレートの違反 39件を名前付き定数に抽出
- [x] 5.5 定数名は用途・使用箇所が明確な命名に統一

## 6. Verification and CI Pipeline Alignment

- [x] 6.1 既存ハードコード文字列をリファクタリングしてパス
- [x] 6.2 `cargo test --workspace` で Linter が正しく機能することを確認
- [x] 6.3 `make lint` + `make test` 全パス

## 7. Shared Test Infrastructure

- [x] 7.1 `run_ast_lint()` を複数ディレクトリ対応の共通ランナーに
- [x] 7.2 `parse_file()` でファイル I/O を共通化
- [x] 7.3 `has_cfg_test_attr()` 共通ヘルパー

## 8. クレート構成の適正化

- [x] 8.1 `katana-linter` クレートを新規作成（ワークスペース共通ツール）
- [x] 8.2 `katana-ui/tests/ast_linter.rs` → `katana-linter/tests/ast_linter.rs` に移設
- [x] 8.3 `katana-ui` の dev-dependencies から `syn`, `ignore`, `proc-macro2` を削除
- [x] 8.4 マジックナンバー検知対象を全クレート (`katana-core`, `katana-platform`, `katana-ui`) に拡大

## 9. i18n 設計改善

- [x] 9.1 言語自称名 (`lang_english`, `lang_japanese`) をロケール JSON から削除
- [x] 9.2 `locales/languages.json` に言語マスター定義を切り出し
- [x] 9.3 `i18n::supported_languages()` で JSON を読み込む方式に変更
- [x] 9.4 言語選択 UI を `supported_languages()` ループで動的生成
