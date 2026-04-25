## Context

v0.26.0 / v0.27.0 では preview と editor の分離が検討されていましたが、KatanA の方向性は「コード editor を主役にする」よりも「preview を主役にして、必要な箇所だけ修正する」ことへ寄っています。この v0.28.0 はその前段として、現行 preview の user-visible behavior を変えず、KatanA 本体と rendering 実装の間に adapter 境界を作る移行だけを扱います。

## Goals

- Native egui preview を維持したまま、KatanA UI から parser/vendor/rendering internals を隠す。
- KatanA 側の contract を DTO / trait / event に寄せ、`katana-markdown-linter` と同じように疎結合な境界を作る。
- 現行の preview behavior、TOC、scroll sync、anchor、diagram、math、table、emoji 表示を維持する。
- preview-specific vendor hack を adapter implementation に閉じ込め、所有者と残存理由を明文化する。
- v0.29.0 の preview-driven local editing に必要な source span / block identity / hit-test metadata を追加できる余地を作る。

## Non-Goals

- preview からの局所編集 UX を実装すること。
- source editor の read-only 化または UI 方針変更を行うこと。
- Typora と同等の full WYSIWYG editor を作ること。
- WebView、React、DOM runtime を導入すること。
- すべての vendor patch をこの変更だけで必ず削除すること。

## Decisions

### 2026-04-25 Active 整理の反映

無印の `preview-adapter-contract` は active 対象から外し、`openspec/changes/archive/2026-04-25-superseded-preview-adapter-contract/` へ移した。`05341608 feat: preview adapter契約を追加` で入った initial DTO / contract 実装は、この v0.28.0 の既存前提として扱う。

v0.28.0 の残作業は、既存 DTO を再定義することではなく、現行 renderer の adapter implementation 化、preview call site の移行、TOC / scroll sync / block highlight / search / action hook metadata の contract 固定、vendor ownership の整理である。

### Adapter-Owned Contract

Preview の public surface は adapter が所有します。KatanA UI は `PreviewInput`、`PreviewThemeSnapshot`、`PreviewWorkspaceContext`、`PreviewRenderMetadata`、`PreviewAction` のような KatanA 側の型だけを扱い、Markdown parser token、`egui_commonmark` の内部型、vendor fork 固有 API を直接扱いません。

### Current Renderer Wrapper First

初期実装は現行 renderer を adapter implementation として包む migration-first approach とします。renderer を新規実装し直すのではなく、既存挙動を保ったまま呼び出し境界を差し替えます。

### Metadata As Contract

TOC、scroll sync、block highlight、search/action hook に必要な heading anchor、block anchor、source range、rendered rect identity は `PreviewRenderMetadata` として adapter から返します。v0.29.0 で編集対象に拡張するため、metadata は renderer-specific な borrow や widget id へ依存しない stable DTO とします。

### Vendor Hack Containment

Preview specific な vendor patch は adapter implementation の内側に閉じ込めます。KatanA UI が fork-specific API を呼ぶ状態は migration 完了条件に含めません。`egui-winit` など platform input 全体に関わる patch が preview 外の責務で残る場合は、preview adapter の対象外として明示的に inventory へ記録します。

### Contract Tests

Fixture-driven tests で adapter 入力、render metadata、diagram fallback、theme propagation、emoji handling の contract を確認します。UI snapshot 全体に依存しすぎず、adapter の戻り値と observable behavior を主な検証対象にします。

## Risks / Trade-offs

- **Risk: migration regressions** - 既存 preview の挙動を包み直すだけでも anchor や scroll sync がずれる可能性がある。既存 integration tests と metadata contract tests を移行条件にする。
- **Risk: vendor patch removal is blocked** - preview 以外の call site が root patch に依存している可能性がある。patch 削除を絶対条件にせず、inventory と ownership 分離を完了条件にする。
- **Trade-off: adapter DTO duplication** - parser/rendering 内部型と KatanA 側 DTO の変換が増える。代わりに KatanA 本体の依存方向を安定させ、v0.29 以降の編集機能を renderer implementation から切り離せる。
