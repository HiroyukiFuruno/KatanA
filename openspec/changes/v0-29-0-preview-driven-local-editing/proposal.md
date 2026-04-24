## Why

KatanA は AI 時代の開発支援ツールとして、Markdown を「書く」よりも「見る」時間が長い前提で設計します。Draw.io や diagram block も preview で確認する価値が中心であり、日常の編集は全体を source editor で書き換えるより、preview 上の該当箇所だけを局所的に直す体験が重要です。

v0.28.0 で preview adapter を導入した後、その adapter が返す source span / render node metadata を使って、preview の AST / rendered node から直接ローカル編集 UI を開けるようにします。raw source は主役ではなく、read-only source inspector または明示的な fallback として扱います。

## What Changes

- Preview 上の block / inline node をクリックまたは action で選択し、その node に対応する局所編集 UI を開く。
- 編集対象は adapter が返す renderer-neutral な `EditableNodeDescriptor` と source range で表現し、KatanA UI は parser/vendor internals へ依存しない。
- 編集 commit は source range patch として in-memory Markdown buffer に適用し、dirty state と preview refresh は既存の Markdown authoring contract に従う。
- Source code panel は read-only source inspector を基本とし、直接編集が必要な場合だけ明示的な fallback source-edit mode として扱う。
- Paragraph、heading、fenced code、Mermaid、Draw.io、math、table、link、image などの主要 node に対して、node kind に応じた編集 surface を提供する。
- WebView、React、DOM runtime は導入しない。Typora と同等の full-document WYSIWYG を目標にしない。

## Capabilities

### New Capabilities

- `preview-driven-local-editing`: Preview の rendered node から局所編集 session を開き、source range patch として Markdown buffer に反映する。

### Modified Capabilities

- `markdown-authoring`: Markdown の主要な編集入口を full source editor typing から preview-driven local edits と明示的 fallback source-edit mode へ拡張する。

## Impact

- v0.28.0 preview adapter: editable node metadata、hit-test metadata、edit action を追加する。
- `crates/katana-ui`: preview selection、local edit surfaces、source inspector の state/action flow を追加または更新する。
- Markdown buffer management: source range patch、staleness validation、dirty state、cancel/rollback handling を追加する。
- Diagram/math/table handling: block-specific editor を追加し、preview fallback と patch commit を検証する。
- Tests: editable node discovery、patch application、stale range detection、dirty/save behavior、read-only source inspector を検証する。
