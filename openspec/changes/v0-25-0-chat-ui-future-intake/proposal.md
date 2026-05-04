## Why

`katana-chat-ui` の document generation・translation overlay 機能が完成した段階で KatanA に取り込む。v0.24.0 の chat UI 基盤の上に、LLM による文書生成と dynamic text の翻訳 overlay を追加する。

## DoR（Definition of Ready）

- `katana-chat-ui` v0.x.0（document generation + translation overlay 対応）が release 済みであること
- 対象バージョンは katana-chat-ui 側の roadmap 確定後に更新する

## What Changes

- `katana-chat-ui`、`katana-chat-ui-egui` を当該バージョンに bump する
- KatanA UI に document generation action（current doc 挿入・新規ファイル生成・template scaffold）を追加する
- dynamic / external English text への translation overlay を追加する（`katana-chat-ui` の translation service 経由）

## Capabilities

### New Capabilities

- `llm-document-generation`: LLM による current doc 挿入・新規ファイル生成・template scaffold
- `dynamic-translation-overlay`: dynamic / external English text の LLM 翻訳表示

## Impact

- 追加: `Cargo.toml` の `katana-chat-ui`、`katana-chat-ui-egui` をバージョン bump
- 追加: document generation action / translation overlay 接続の薄い adapter
- katana-chat-ui 側の実装は [katana-chat-ui openspec](https://github.com/HiroyukiFuruno/katana-chat-ui) を参照
