## Why

`v0.23.0` で local LLM provider と lint autofix の基盤が整えば、次の価値は文書作成そのものに local LLM を使えるようにすることにある。ユーザー要望では、新規ファイル生成、現在ファイルへの挿入、テンプレート生成を優先度なしで同時に扱いたい。

## What Changes

- local LLM を使った document generation workflow を追加する
- current document への挿入、新規 Markdown file 生成、template-based scaffolding を同じ release で提供する
- generation 前後の対象、保存先、反映内容を user が確認できる導線を追加する
- active document、selection、workspace context を generation input に使えるようにする
- file 作成や挿入後の editor / workspace refresh を統一する

## Capabilities

### New Capabilities

- `local-llm-document-generation`: local LLM を用いて current document、new file、template scaffold の 3 系統の文書生成を行う

### Modified Capabilities

## Impact

- 主な影響範囲は document generation action / service、`crates/katana-ui/src/views/panels/editor/*`、workspace file ops UI、`crates/katana-platform/src/settings/*`
- `v0.23.0` の active local provider 設定と availability 判定を前提にする
