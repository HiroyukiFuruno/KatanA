## Context

KatanA は source editor より preview を主役にする方向へ寄っている。一方で、preview 周辺は Markdown parser、diagram rendering、table extension、emoji hack、anchor / scroll sync、UI action hook が密に絡んでいる。直接呼び出しを増やしたまま編集機能や renderer 置換を進めると、preview の責務がさらに広がる。

この change は新しい preview 体験を作るのではなく、現行 behavior を維持しながら adapter contract を定義する。

## Goals / Non-Goals

**Goals:**

- preview の input / output / metadata / action を KatanA-owned DTO にする。
- `katana-ui` から renderer-specific type を隠す。
- TOC、scroll sync、block highlight、search が必要とする metadata を明文化する。
- 現行 renderer を包む migration-first approach にする。

**Non-Goals:**

- preview 上で直接編集する UX。
- editor の read-only 化。
- WebView / React / DOM runtime の導入。
- vendor patch の完全削除。

## Decisions

### 1. Adapter-owned DTO を public surface にする

`PreviewInput`、`PreviewThemeSnapshot`、`PreviewWorkspaceContext`、`PreviewRenderMetadata`、`PreviewAction` 相当の型を KatanA 側で持つ。parser token や renderer node は adapter 外へ出さない。

Task 1 では、初期の public surface を `katana-core::preview::adapter` に置く。公開してよい型は adapter DTO、`std` 型、`serde` 可能な primitive / collection に限定する。`egui`、`egui_commonmark`、`comrak`、`pulldown-cmark`、vendor fork 固有の AST / render node は public adapter contract へ入れない。

source range は初期値として byte offset を採用する。line-column が必要になった場合は、既存 `PreviewSourceRange` を壊さずに別 DTO を追加する。

### 2. Current renderer wrapper first

現行 preview renderer を adapter implementation として包む。rewrite ではなく migration として進め、user-visible behavior を維持する。

### 3. Metadata は次の編集機能を見据えて stable にする

heading anchor、block anchor、source range、rendered identity は renderer-neutral DTO として返す。将来の preview local editing で stale check や source patch に使える形にする。

### 4. Vendor ownership を棚卸しする

preview-specific fork API は adapter implementation の内側に閉じる。platform input など preview 外の patch は別所有として記録する。

## Risks / Trade-offs

- [Risk] Metadata 移行で scroll sync がずれる → 既存 integration test と adapter metadata test を通す。
- [Risk] Adapter DTO が冗長になる → renderer internals への依存を減らすために許容する。
- [Risk] Vendor cleanup が広がる → この change では分類と containment を主目的にし、完全削除は必須にしない。

## Migration Plan

1. preview adapter DTO と trait / service boundary を定義する。
2. current renderer を adapter implementation として包む。
3. preview call site を adapter API へ移す。
4. metadata consumer を DTO 消費へ移す。
5. vendor ownership map を作る。

## Open Questions

- preview adapter を `katana-ui` 内に置くか、将来 crate 分離しやすい module に置くか。
- source range の単位を byte offset / line-column / both のどれにするか。
