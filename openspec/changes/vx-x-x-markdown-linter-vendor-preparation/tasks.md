# 実装タスク: katana-markdown-linter ベンダー統合準備

## Definition of Ready

- [x] 全 58 ルールのプロパティ定義が `OfficialRuleMeta` に反映済み
- [x] `RulePropertyType::Enum` が `types.rs` に定義済み
- [x] `ast_linter_stubs_parity` テストが通過済み
- [ ] `proposal.md` と `design.md` の内容がユーザーと合意されていること
- [ ] `katana-markdown-linter` 側の確定事項（package 名、module 名、MIT license、`kml` executable、全 active rule check 必須、JSONC support、default branch 追従）がこの change に反映済みであること

## 1. evaluate シグネチャの拡張

- [ ] 1.1 `MarkdownRule::evaluate` に Config 引数（`Option<&serde_json::Value>`）を追加する
- [ ] 1.2 `official_rule!` マクロの evaluate 生成を更新する
- [ ] 1.3 `regex_rule!` マクロの evaluate 生成を更新する
- [ ] 1.4 `eval.rs` の `evaluate_all` で `.markdownlint.json` からルール設定を取得し evaluate に渡す
- [ ] 1.5 手動実装ルール（mod.rs, heading.rs, list.rs 等）の evaluate シグネチャを更新する

## 2. Config Validation 機構

- [ ] 2.1 `MarkdownLintConfig` に `validate(&self, rules: &[Box<dyn MarkdownRule>]) -> Vec<ConfigError>` を追加する
- [ ] 2.2 `ConfigError` 型を定義する（ルール ID、プロパティ名、エラー種別: 不正な型 / Enum 範囲外 / 未知プロパティ）
- [ ] 2.3 `RulePropertyType::Enum` の選択肢に対するバリデーションを実装する
- [ ] 2.4 Config Validation のユニットテストを追加する
- [ ] 2.5 `.markdownlint.jsonc` を将来の config surface として扱うため、JSONC support の差分を adapter / validation 設計に記録する

## 3. Quality Gates

- [ ] 3.1 evaluate シグネチャ変更後に全既存テスト（105 個）が通過すること
- [ ] 3.2 Config Validation のユニットテストを追加する
- [ ] 3.3 `ast_linter_stubs_parity` テストが引き続き通過すること
- [ ] 3.4 `vendor-linter` feature flag の切り替え境界を定義し、dependency switch 前後で比較する項目を記録する
- [ ] 3.5 `katana-markdown-linter` の MIT license を KatanA 側 license inventory に反映する必要があるか確認する

## Definition of Done

- [ ] `MarkdownRule::evaluate` が Config 参照を受け取り、設定値へのアクセスが可能であること
- [ ] `MarkdownLintConfig::validate()` が不正値をルール ID + プロパティ名付きで検出できること
- [ ] 全既存テスト（105 個）が通過すること
- [ ] `ast_linter_stubs_parity` テストが引き続き通過すること
- [ ] `katana-markdown-linter` への dependency switch 条件が Integration Contract として文書化されていること
