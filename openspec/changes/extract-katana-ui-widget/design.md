## Context

KatanAには既に `widgets` moduleがあり、egui依存の汎用部品が集まっている。Floem移行後、同じ構造をKatanA本体に残すと、KME・preview・editorが使う共通部品の所有者が曖昧になる。

ただしUI widget分離をKMEより先に進めると、metadata表示、unresolved target、AST単位copy/editの表示DTOが固まる前にUI側の型が先行する。`kcu` で見えている課題も踏まえ、P2としてP0/P1後に境界を確定する。

## Goals

- Floem前提の共通UI部品をKatanA本体から分離する。
- metadata表示、unresolved target表示、AST単位copy/edit actionを共有できるようにする。
- kdp/kle/KatanAが同じUI部品を使えるようにする。
- P0 `katana-ast-lint` の品質ゲートを前提にする。
- P1 KMEのmetadata/display DTOと整合するUI境界にする。

## Non-Goals

- egui widgetをそのまま移植すること。
- KatanA chrome全体をこのrepoへ移すこと。
- KME文書モデルやmetadata schemaをUI widget repoへ持たせること。
- KMEより先にUI widgetの表示型を固定すること。

## Decisions

### Floem Only

`katana-ui-widget` はFloem前提にする。egui互換層やWebView/Reactは持たない。

### P2 Extraction

`katana-ui-widget` はP2とする。P0 `katana-ast-lint` で品質ゲートを揃え、P1 KMEでmetadata/display DTOを定義した後に、UI部品の責務を確定する。

### Model-neutral Inputs

UI widgetはKMEやkdpの内部型を直接持たない。表示に必要なlabel、state、action descriptorなどの小さいDTOを受け取る。

### Extraction Order

最初の対象はmetadata/unresolved表示、copy/edit affordance、tab/toolbar、badge表示にする。KatanA shell固有のwindow/chromeは対象外にする。

### kcu Lessons

`kcu` で見えている課題は分離前の入力として扱う。ここでは詳細実装に踏み込まず、責務境界、DTO、品質ゲートに反映する。
