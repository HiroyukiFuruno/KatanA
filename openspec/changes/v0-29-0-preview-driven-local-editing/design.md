## Context

v0.29.0 は v0.28.0 の preview adapter migration を前提にします。KatanA の主面は preview であり、source editor は「常に書く場所」ではなく、source を確認する inspector または fallback です。Typora 風の滑らかな入力感は参考にしますが、KatanA では full-document WYSIWYG ではなく、preview node から必要箇所だけを修正する体験を優先します。

## Goals

- Rendered preview node から局所編集 session を開始できる。
- Adapter が返す editable metadata を通じて、KatanA UI と parser/vendor/rendering internals を疎結合に保つ。
- Local edit commit を source range patch として in-memory Markdown buffer に反映し、既存の dirty/save semantics を維持する。
- Paragraph、heading、fenced code、Mermaid、Draw.io、math、table、link、image の主要編集を扱える。
- Source panel は read-only inspector を基本にし、明示的な fallback source-edit mode と区別する。

## Non-Goals

- Typora と同等の full WYSIWYG editor を実装すること。
- 全 Markdown 文法を inline で常時編集可能にすること。
- WebView、React、DOM runtime を導入すること。
- v0.28.0 の adapter migration を飛ばして renderer internals へ直接依存すること。
- Linter、AI provider、workspace shell の contract を変更すること。

## Decisions

### Editable Node Descriptor

Preview adapter は `EditableNodeDescriptor` 相当の DTO を返します。Descriptor には stable node id、node kind、source range、display label、current source snippet、supported edit commands、rendered hit rect identity を含めます。KatanA UI は descriptor と command だけを扱い、parser node や vendor widget state を保持しません。

### Source Range Patch Engine

Commit は Markdown buffer への source range patch として扱います。Patch には target source range、expected original text または source snapshot hash、replacement text、edit command kind を含めます。Commit 時に range が stale になっていないことを検証し、成功時だけ buffer を更新して dirty にします。

### Block-Level First, Inline Where Stable

初期の主対象は paragraph、heading、fenced code、diagram、math、table など block-level node です。Link や image は preview action から URL / alt text / title を編集する局所 UI とし、複雑な inline cursor editing は full WYSIWYG とみなして避けます。

### Source Inspector By Default

Source code panel は read-only inspector を基本とします。ユーザーが raw Markdown 全体を直接編集する必要がある場合は、明示的に fallback source-edit mode を開きます。Fallback を残す場合でも、通常の preview-driven local edit flow と dirty/save semantics を共有します。

### Adapter Owns Hit Testing Metadata

Rendered rect と source range の対応は adapter の責務です。UI は pointer event や keyboard action から adapter-provided hit-test metadata を参照し、選択された editable descriptor に対して edit surface を開きます。

## Risks / Trade-offs

- **Risk: stale source ranges** - AI update、external reload、fallback source edit により range が古くなる可能性がある。Expected original text と buffer snapshot hash で検証し、失敗時は再選択を求める。
- **Risk: malformed Markdown after patch** - 局所 patch が Markdown 構造を壊す可能性がある。Commit 前後で parser validation と preview fallback を行い、失敗時は buffer 更新を行わない。
- **Risk: IME and cursor complexity** - full-document editor を preview 内で再現すると IME / selection / layout が複雑になる。Local edit surface は通常の egui text input や structured form に限定する。
- **Trade-off: source inspector is less powerful by default** - 直接 source を常時編集する体験は弱くなる。代わりに view-first の主体験と source safety を優先し、明示的 fallback を残す。
