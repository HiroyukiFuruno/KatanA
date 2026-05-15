## Why

KMM、katana-language-editor、katana-document-viewer、katana-diagram-renderer、katana-canvas-forge、katana-ui-widgetの責務が分かれても、KatanA本体の統合順序が曖昧だとviewer、editor、exportが別々の仕様へ戻る。さらにAST lintの統制が先に揃っていないと、repositoryごとに品質ゲートが割れる。

このchangeは、KatanAが各libraryをどの順序で取り込み、現行Markdown UXを落とさずKMMへ移行するかを固定する。

## What Changes

- KatanAはKMMをMarkdown文書仕様の正本として参照する
- KatanA統合前にP0 `katana-ast-lint` の共通品質ゲートを参照する
- viewerは `katana-document-viewer` のFloem実装経由でKMMモデルを表示する
- editorは `katana-language-editor` の保存時metadata同期を使う
- exportはKDVのviewer/export pipelineへ寄せる。KCF既存exportはKDV同等機能が入るまで維持する
- editor-viewer同期制御はKatanAが担い、KatanAがviewerまたはeditorへ命令する
- 共通UI部品は `katana-ui-widget` 分離計画へ逃がし、KatanA本体へ新しい汎用widgetを増やしすぎない

## Capabilities

### New Capabilities

- `kme-katana-integration`: KMM ecosystemをKatanAへ段階的に接続する

## Impact

- `Cargo.toml`: git dependencyの追加または差し替え
- `katana-ui`: viewer/editor/export接続部のadapter更新
- `katana-ast-lint`: 統合前の共通AST lint品質ゲート
- `openspec/changes/v0-29-0-preview-driven-local-editing`: KMM metadataとeditable node contractへ接続
- `assets/fixtures`: description list fixture追加
