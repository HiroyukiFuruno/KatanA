## Why

KMM統合では、metadata表示、unresolved表示、AST単位コピー、局所編集、tab、toolbar、badge、inline actionなど、共通UI部品が増える。これらをKatanA本体の `katana-ui/src/widgets` に積み続けると、Floem移行後もアプリ本体が肥大化し、kdp/kle/kcfとの境界が曖昧になる。

`katana-ui-widget` をP2として分離し、Floem前提の共通UI部品をKatanA ecosystemで共有できるようにする。P0 `katana-ast-lint` とP1 `katana-markdown-model` の境界を受けてから、metadata表示やAST単位操作のUI責務を決める。

## What Changes

- `katana-ui-widget` repositoryの分離計画を作る
- 対象をFloem前提の共通UI部品に限定する
- KMM metadata表示、unresolved target表示、copy/edit action、tab/toolbar、badge表示を候補にする
- egui widgetの移植先ではなく、Floem時代の共通UI部品として設計する
- `kcu` で見えている課題を踏まえ、KMMのmetadata/display DTOが固まる前にUI側だけを先走らせない

## Capabilities

### New Capabilities

- `katana-ui-widget`: Floem前提の共通UI部品をKatanA本体から切り出す

## Impact

- `katana-ui/src/widgets`: 分離候補の棚卸し
- `katana-ast-lint`: P0品質ゲート
- `katana-markdown-model`: P1 metadata/display DTO
- `katana-document-preview`: metadata表示やcopy/edit actionの共通部品利用
- `katana-language-editor`: editor toolbarやmetadata unresolved actionの共通部品利用
- `katana`: shell/chromeと汎用widgetの境界整理
