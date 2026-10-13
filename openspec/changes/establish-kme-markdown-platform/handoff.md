# KMM Platform Handoff

## 結論

KMM Platformは、KMMを出発点にした一方向依存で進める。

KMMが所有するのはMarkdown文書モデル、source mapping、metadata schema、metadata target解決である。viewer、editor、export、UI widget、KatanA統合は、KMM public contractを消費する側であり、KMM未確定の仕様を独自に作らない。

editor-viewer同期制御はKatanAが所有する。KatanAはKMMの位置情報を使ってviewerまたはeditorへ命令し、KMM/KLE/KDVは互いの状態を知らない。

## 現在の状態

- P0 `katana-ast-lint`: 分離済み。KMMは `katana-ast-lint = "0.1.0"` を品質ゲートとして利用する。
- P1 `katana-markdown-model`: repository初期構築済み。OpenSpecは次セッションが実装を継続できる粒度へ更新中である。
- P2 `katana-ui-widget`: OpenSpecは `katana` 側にあるが、repositoryは未作成である。これはblocking riskである。
- P3 `katana-document-viewer`: `katana-document-preview` から改名する。KMM public DTOとKUW境界待ちである。
- P3 `katana-language-editor`: KMM metadata schemaとtarget resolution API待ちである。
- P3 `katana-render-runtime`: 外部描画（Mermaid / Draw.io / PlantUML / math）専用 crate として独立。
- P3 `katana-canvas-forge`: export 専用へ責務を縮小する。既存exportはKDV移譲まで維持する。
- P3 `katana`: 各libのpublic contract待ちである。

## 次の順序

1. KMM OpenSpecを、別セッションがそのまま実装開始できる状態にする。
2. KMMでcanonical fixture、table/grid source range、emoji node、metadata conflict DTOを固める。
3. KUW repository作成のDoRを、KMM metadata/display DTO確定後として再確認する。
4. KDVとKLEはKMM APIのconsumerとして進める。
5. KCFは外部描画専用へ縮小し、export関連計画をKDVへ移譲する。
6. KatanA本体は最後に統合する。

## 検証結果

2026-05-06 に以下を確認済みである。

- `katana`: `scripts/openspec validate "establish-kme-markdown-platform" --strict`
- `katana`: `scripts/openspec validate "adopt-kme-in-katana" --strict`
- `katana`: `scripts/openspec validate "extract-katana-ui-widget" --strict`
- `katana`: `scripts/openspec validate "extract-katana-ast-lint" --strict`
- `katana-markdown-model`: `scripts/openspec validate "bootstrap-kme-document-model" --strict`
- `katana-document-viewer`（現repo名は `katana-document-preview`）: `npx -y @fission-ai/openspec validate "adopt-kme-preview-model" --strict`
- `katana-language-editor`: `npx -y @fission-ai/openspec validate "sync-kme-metadata-on-save" --strict`
- `katana-canvas-forge`: `npx -y @fission-ai/openspec validate "v0-1-3-export-css-debug" --strict`
- `katana-ast-lint`: `scripts/openspec validate "shared-ast-lint" --strict`
- `katana-ast-lint`: `scripts/openspec validate "v0-2-0-configurable-shared-rules" --strict`

## 禁止事項

- KDV、KLE、KCF、KUW、KatanA本体でKMMの代替document modelを作らない。
- KMM未確定のmetadata schemaを各repoで先に固定しない。
- KUW未作成のままKatanA本体へ共通UI部品を増やし続けない。
- KCFへ新規export責務を追加しない。
- KMM、KLE、KDVへeditor-viewer同期制御を持たせない。
- KMM public DTOへthird-party parser ASTを漏らさない。
