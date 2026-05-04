# タスクリスト: 次世代 i18n フレームワークへの移行

## Task 1: 基礎インフラの整備

- [ ] `katana-ui/Cargo.toml` に `fluent`, `fluent-langid`, `unic-langid` を追加。
- [ ] `crates/katana-ui/src/i18n/engine.rs` を新規作成し、`FluentBundle` の管理ロジックを実装。
- [ ] テストコードを作成し、単純なメッセージのルックアップが Fluent 経由で動作することを確認する。

## Task 2: ロケールファイルの変換と配置

- [ ] 既存の `locales/*.json` を `locales/*.ftl` に変換するスクリプトを `scripts/i18n/convert_json_to_ftl.py` として作成。
- [ ] 全言語の FTL ファイルを生成し、`crates/katana-ui/locales/` に配置。
- [ ] `build.rs` または `include_str!` 等を利用して、バイナリに FTL を埋め込む仕組みを構築。

## Task 3: API の移行と統合

- [ ] `I18nOps::t` および `I18nOps::tf` を `engine.rs` の Fluent ロジックを使用するように書き換え。
- [ ] 既存の `I18nMessages` 構造体を使用している箇所を、文字列キーベースのアクセスに順次変更。
- [ ] 複数形が必要な箇所（例：検索結果、問題数表示）を検出し、FTL 側で `[one]`, `[other]` 等の分岐を定義。

## Task 4: 検証とクリーンアップ

- [ ] 複雑な複数形ルールを持つ言語（ロシア語、ポーランド語など、将来的に追加する場合を考慮）での動作をシミュレートするテストケースの追加。
- [ ] 古くなった JSON ロケールファイルおよび `types/*.rs` (i18n 用) の削除。
- [ ] UI 上での表示崩れや翻訳漏れがないか、全言語でランタイムチェックを実施。
