## Why

`katana-core` には AI provider abstraction があるが、local LLM を user が選択・設定して使う導線はまだない。`v0.19.0` の markdownlint parity diagnostics を機械的に解釈できるようになった後は、lint の自動修正を local LLM へ接続するのが最初の実用価値になる。

## What Changes

- local LLM provider を user が設定・切り替えできるようにする
- `Ollama`、`LM Studio`、OpenAI 互換 local endpoint を選択肢として扱える provider settings を追加する
- lightweight な model を選びやすい初期導線と availability check を追加する
- markdownlint diagnostics を入力にした lint autofix workflow を追加する
- autofix は結果を確認してから適用できる安全な導線にする
- provider 未設定時や利用不可時の disabled state / recovery 導線を整える

## Capabilities

### New Capabilities

- `local-llm-lint-autofix`: local LLM を用いて markdownlint diagnostics の自動修正候補を生成し、確認後に適用する

### Modified Capabilities

- `ai-provider-abstraction`: local endpoint provider の設定、選択、利用可否判定を扱えるようにする

## Impact

- 主な影響範囲は `crates/katana-core/src/ai/mod.rs`、provider adapter 群、`crates/katana-platform/src/settings/*`、diagnostics UI、settings UI
- `v0.19.0` の official markdownlint diagnostics payload を前提にする
- `v0.24.0` と `v0.25.0` の local LLM 機能の基盤になる
