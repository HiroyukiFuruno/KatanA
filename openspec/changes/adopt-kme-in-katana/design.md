## Context

KatanAは最終利用側として、KME、editor、preview、export、widgetを接続する。統合時にKatanA本体がKMEの内部nodeや各libraryの実装型を直接持つと、外部化した意味がなくなる。

## Goals

- KatanA本体はKME文書モデルのpublic DTOだけを扱う。
- preview/editor/exportは同じKMEモデルとmetadata targetを共有する。
- 既存fixtureの表示を落とさず移行する。
- Floem前提で接続し、egui継続を統合条件にしない。
- P0 `katana-ast-lint` を統合品質ゲートとして使う。

## Non-Goals

- KME parser内部をKatanA UI stateへ持ち込むこと。
- KME移行と同時に全UI widgetをKatanA本体で作り直すこと。
- kcf v0.1.2をKME統合完了条件にすること。

## Decisions

### Adapter Boundary

KatanAはKME、kdp、kle、kcfの公開DTOだけを扱う。parser AST、renderer internals、widget stateはKatanAの永続stateへ入れない。

### Quality Gate Order

統合順序はP0 `katana-ast-lint`、P1 `katana-markdown-engine`、P2 `katana-ui-widget`、P3 KatanA統合とする。KatanA統合では、P0の共通AST lintを使って、分離repositoryごとの独自品質ゲートに戻らないことを確認する。

### Fixture-first Migration

統合は `sample.md`、README badge、alert、description listのfixtureで差分を確認してから広げる。fixtureが通らない状態で旧実装を置き換えない。

### Metadata Flow

保存時はeditorがKME位置解決APIを呼び、metadataを更新する。previewはunresolvedを表示し、exportは解決済みmetadataだけをページングや出力制御へ使う。

### UI Widget Separation

KME統合中に増えるmetadata表示、copy/edit操作、tab、toolbarは `katana-ui-widget` 分離対象として扱う。
