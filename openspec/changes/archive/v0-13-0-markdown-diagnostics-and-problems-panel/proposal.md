## Why

KatanA は Markdown workspace でありながら、文書品質の問題は現状 CI や目視確認に寄っており、執筆中には見えません。
`katana-linter` には既に lint の基盤があるため、`v1.0.0` 前は local LLM ではなく deterministic な diagnostics を app 内で見せる方が筋が良いです。

## What Changes

- Markdown と repository docs を対象にした diagnostics を app 内で実行・表示できるようにする
- diagnostics を一覧表示する Problems Panel を追加する
- Problems Panel から editor / preview の該当箇所へ jump できるようにする
- 初期 rule set として、heading structure、`*.md` と `*.ja.md` の見出し同期、broken relative links、missing local assets など deterministic な文書問題を扱う
- manual refresh と save 時 refresh を定義し、編集中の体験を阻害しない更新方針にする

## Capabilities

### New Capabilities

- `markdown-diagnostics`: Markdown 文書や関連 docs の問題を app 内で検出し、Problems Panel と editor / preview 導線で扱える機能

### Modified Capabilities

## Impact

- `crates/katana-linter`: 既存 rule 再利用と Markdown diagnostics rule 拡張
- `crates/katana-ui`: Problems Panel、diagnostics navigation、editor/preview highlight、status summary
- `openspec/specs`: in-app diagnostics の要件定義
- 開発体験: CI 以前に文書品質問題を見つけられるようになる
