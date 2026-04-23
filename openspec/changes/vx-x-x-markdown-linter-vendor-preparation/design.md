# 設計: katana-markdown-linter ベンダー統合準備

## Context

`katana-markdown-linter` は以下の 5 Phase で開発される予定:

| Phase | 内容 | KatanA への影響 |
|-------|------|----------------|
| Phase 1 | Scaffold・骨格構築 | なし |
| Phase 2 | Rule Parity・品質 | 全 active rule の check 実装と config validation の source |
| Phase 3 | Public Release 準備 | `katana-markdown-linter` package / MIT license / installable artifact の確定 |
| Phase 4 | CLI | `kml` executable、`.markdownlint.jsonc`、`--format json` の利用可能性 |
| Phase 5 | Upstream 追従 | default branch 追従による ast_linter_stubs_parity 相当の機構 |

この OpenSpec は、Phase 2-3 が完成した時点で KatanA が `katana-markdown-linter` をスムーズにベンダー統合できるよう、KatanA 側の準備を行う。
実際の dependency switch は Phase 2 の全 active rule check 実装、Phase 2 の config validation、Phase 3 の package metadata が揃った後に行う。

## 責務の仕分け

### A. KatanA 側で本セッション内に実施したもの（この OpenSpec には含まない）

| 項目 | 状態 |
|------|------|
| `RulePropertyType::Enum` バリアントの追加 | ✅ 完了 |
| `rule_prop_enum!` マクロの追加 | ✅ 完了 |
| 全 58 ルールのプロパティ定義（スキーマ v0.40.0 準拠） | ✅ 完了 |
| `ast_linter_stubs_parity` テストの通過 | ✅ 完了 |
| UI ハードコード `get_known_choices` の排除 | ✅ 完了 |
| `regex_rule!` マクロへの properties サポート追加 | ✅ 完了 |
| Enum 選択肢の正確な値の修正 | ✅ 完了 |
| `MarkdownLintAdapter` trait と `InternalAdapter` 実装 | ✅ 完了 |
| `LintDiagnostic` / `LintFix` 中間表現型の定義 | ✅ 完了 |

### B. KatanA 側でこの OpenSpec として実施する内容

1. **`MarkdownRule::evaluate` シグネチャの拡張**
   - Config 参照を引数に追加し、各ルールが設定値を読めるようにする
   - マクロ (`official_rule!`, `regex_rule!`) の対応更新

2. **Config Validation 機構**
   - `MarkdownLintConfig` に `validate()` メソッドを追加
   - `RuleProperty` メタデータ（型・Enum 選択肢・デフォルト値）を活用した検証

### C. katana-markdown-linter 側に移譲する内容

| 項目 | 対応 Phase |
|------|-----------|
| `pulldown-cmark` ベースの AST パーサ | Phase 1 |
| 全 active rule check 実装（公式 docs / upstream implementation 準拠） | Phase 2 |
| 全 active rule の fixability 分類と安全な fix 実装 | Phase 2 |
| `.markdownlint.json` の生成・読み込み・検証 helper | Phase 2 |
| `.markdownlint.jsonc` の config 入力対応 | Phase 4 |
| `cargo install` 用 `kml` CLI | Phase 4 |
| upstream default branch 差分検出・追従機構 | Phase 5 |

### D. 将来的に katana-markdown-linter の機構を利用して行うこと

| 項目 | 前提条件 |
|------|---------|
| KatanA 内 Markdown ルール実装の全削除 | Phase 2 完了 + Adapter 層検証 + KatanA regression test 通過 |
| `katana-linter` の `Cargo.toml` に依存追加 | Phase 3 完了（crate 公開後または合意済み Git dependency） |
| stubs.rs / stubs_regex.rs の廃止 | vendor linter が全ルールカバー後 |
| `ast_linter_stubs_parity` テストの廃止 | Phase 5 の upstream 追従機構で代替 |

## Decisions

### 1. evaluate シグネチャは `&serde_json::Value` を受け取る

`MarkdownLintConfig` 全体ではなく、ルール個別の設定値（`Option<&serde_json::Value>`）を渡す。
理由: vendor linter 統合後も同じインターフェースで設定を渡せるようにするため。

### 2. Adapter は trait で抽象化する

```rust
pub trait MarkdownLintAdapter {
    fn convert(&self, result: &LintResult) -> MarkdownDiagnostic;
}
```

vendor linter 統合時に実装を差し替えるだけで済むようにする。
Phase 2 で `katana-markdown-linter` 側の `LintResult` が確定するまでは、KatanA 側 adapter は boundary trait と mapping test を先に用意し、dependency switch 時に実型へ接続する。

### 3. Config Validation は `RuleProperty` メタデータを活用する

既に全ルールに `properties` が定義済みのため、これを `MarkdownLintConfig::validate()` の source of truth として使う。新たなスキーマ定義は不要。

### 4. vendor dependency switch は feature flag で段階化する

`vendor-linter` feature flag を用意し、KatanA 内実装と `katana-markdown-linter` 実装を切り替えられるようにする。
切り替え前後で diagnostic count、rule id、range、message、fix availability の regression test を比較する。

## Integration Contract

- KatanA は `katana-markdown-linter` package / `katana_markdown_linter` module を依存として取り込む
- KatanA は CLI executable `kml` には依存しない
- KatanA は MIT license を third-party notices / license inventory に反映する
- KatanA は `.markdownlint.json` と `.markdownlint.jsonc` の両方を将来の config surface として扱う
- KatanA は Phase 5 の default branch drift report を `ast_linter_stubs_parity` の後継 gate として扱う

## Risks / Trade-offs

- evaluate シグネチャの変更は全ルール実装に波及する（マクロで吸収可能）
- vendor linter 完成までの過渡期に、KatanA 側と vendor 側で同じルール実装が二重に存在する
- Adapter 層の設計を先行しすぎると、vendor 側の API が変わった場合に手戻りが発生する
- `katana-markdown-linter` 側の `LintResult` が確定する前に KatanA 側 adapter の concrete mapping を固定すると、Phase 2 実装時に再調整が必要になる
