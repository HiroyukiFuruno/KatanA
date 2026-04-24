## Why

`katana-core` には AI provider abstraction があるが、local LLM を user が設定して実際に会話・修正へ使う導線はまだない。最初の価値は、Ollama 経由でローカルモデルへ接続し、KatanA 内に他機能から独立した chat UI を置いた上で、markdownlint diagnostics の autofix へ段階的に接続することにある。

## What Changes

- Ollama を初期接続先として扱う local LLM 設定を追加する
- 1桁GB級の軽量モデルを選びやすい推奨導線と availability check を追加する
- `katana-ui` 内に、VS Code 風の端アイコンから開閉できる chat サイドパネルを追加する
- chat はまず user prompt と assistant response の往復を扱い、ファイル変更は明示 action に限定する
- MVP の chat はアプリ起動中の一時的な会話に限定し、履歴の保存・一覧・管理は後続 task に分離する
- MVP では Ollama モデルの選択を必須にし、temperature などの細かい生成設定は後続 task に分離する
- markdownlint diagnostics と KML (`katana-markdown-linter`) の一括 fix 後 content を入力にした lint autofix workflow を追加する
- autofix は file 単位の差分 preview で確認してから適用できる安全な導線にする
- provider 未設定時や利用不可時の disabled state / recovery 導線を整える
- Vertex AI、Bedrock、OpenAI または OpenAI-compatible provider、音声入力は後続 milestone の拡張点として扱う

## Capabilities

### New Capabilities

- `local-llm-chat`: Ollama 経由の local LLM と会話する独立 chat UI を提供する
- `local-llm-lint-autofix`: local LLM を用いて file 単位の lint 自動修正候補を生成し、差分確認後に適用する

### Modified Capabilities

- `ai-provider-abstraction`: Ollama adapter、model 設定、利用可否判定を扱えるようにする

## Impact

- 主な影響範囲は `crates/katana-core/src/ai/mod.rs`、Ollama adapter、`crates/katana-platform/src/settings/*`、chat UI、diagnostics UI、settings UI
- `v0.19.0` の official markdownlint diagnostics payload を前提にする
- `v0.24.0` と `v0.25.0` の local LLM 機能の基盤になる
