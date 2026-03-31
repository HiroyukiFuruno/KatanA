## Why

現在の KatanA では、ファイル検索、各種アクション、今後追加される Markdown content search や diagnostics 導線が別々の入口に分かれています。
local LLM を `v1.0.0` 以降に回すなら、その前に keyboard-first な unified navigation を整え、日常操作の摩擦を先に下げる方が価値が高いです。

## What Changes

- global command palette を追加し、commands / file navigation / Markdown content navigation を 1 つの入口にまとめる
- query に応じて command results、workspace file results、Markdown content results を grouped に表示する
- keyboard-first な selection / execution / dismissal を定義する
- empty query 時には common actions や recent items を表示できるようにする
- 既存の dedicated file-search modal は互換性のため残しつつ、palette を主要な高速導線として定義する

## Capabilities

### New Capabilities

- `command-palette`: commands、files、Markdown content results を横断して実行・移動できる keyboard-first palette

### Modified Capabilities

## Impact

- `crates/katana-ui`: palette state、result rendering、keyboard navigation、action dispatch
- `crates/katana-ui` existing search flow: file search modal と Markdown content search との統合導線
- `AppAction` / search state: command execution payload と navigation payload の整理
- `openspec/specs`: unified navigation の contract 追加
