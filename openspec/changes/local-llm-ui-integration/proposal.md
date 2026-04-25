## Why

Ollama provider、chat UI、file-level autofix の土台が分離して進んだため、次はユーザーが設定から chat / diagnostics autofix まで迷わず辿れる導線を整える必要がある。ここを独立 change に切り出すことで、内部 pipeline を広げずに UI の到達性と復旧導線を先行して固める。

## What Changes

- AI 設定画面に Ollama endpoint、model 選択、接続確認、autofix 有効化状態を一貫して扱う UI を整備する。
- chat パネル、Problems パネル、settings の間に明確な移動導線を追加する。
- provider 未設定、model 未選択、Ollama unavailable、timeout、invalid response の理由をユーザーに見える状態として表示する。
- diagnostics 上の autofix entry point を、利用可能時と利用不可時で分かる状態にする。
- UI snapshot または同等の確認結果を残し、ユーザーフィードバックをこの change の tasks に反映する。
- chat 履歴の永続化、remote provider、音声入力、document generation はこの change に含めない。

## Capabilities

### New Capabilities

- `local-llm-ui-integration`: Local LLM 設定、chat、diagnostics autofix をひとつのユーザー導線として接続する。

### Modified Capabilities

- `local-llm-chat`: chat request の provider recovery 導線を UI 上で明確化する。
- `local-llm-lint-autofix`: diagnostics からの autofix 起動可否と recovery 導線を UI 上で明確化する。

## Impact

- `crates/katana-ui/src/settings/*`
- `crates/katana-ui/src/views/panels/problems/*`
- `crates/katana-ui/src/state/chat*` / `state/autofix*`
- `crates/katana-ui/src/views/app_frame/*` と chat panel 表示制御
- i18n locale files
- UI integration tests and snapshot-free semantic assertions
