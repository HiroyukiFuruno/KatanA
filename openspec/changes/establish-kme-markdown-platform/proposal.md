## Why

KatanA はMarkdownを「見る」「確認する」「必要箇所だけ直す」ことを主面に置く。現在はpreview、export、editor、fixture、metadata相当の情報が複数の実装へ分散し、KatanA側で既存libraryをhackして微細な表示調整を行っている。

このままでは、表（table/grid）、ホバー強調、コード位置同期、脚注、alert、badge、絵文字、PDFページング、LLM注釈、AST単位コピー・編集を同じ仕様で扱えない。KMM（katana-markdown-model）を文書モデルの中核として新規repository化し、KatanA ecosystem全体のMarkdown仕様を固定する。

## What Changes

- KMMを新規repositoryとして作成し、Markdown文書モデル、metadata schema、位置解決を所有させる
- KMM v0の仕様fixtureを `assets/fixtures/sample.md`、README badge、alert、description listで固定する
- metadataはMarkdown本文へ埋め込まず、外部ファイルとして扱う
- editorは保存直後のmetadata同期を担い、KMMは新旧本文とmetadataからtarget再対応を判定する
- viewerはFloem前提のネイティブ表示でKMMモデルを消費する
- `katana-document-preview` は未リリース・未取り込みのため、`katana-document-viewer`（KDV）へ改名する
- KDVはviewerとHTML/PDF/PNG/JPG exportを担う
- KCFはMermaid、Draw.io、PlantUML、mathなどの外部描画へ責務を縮小する
- KCFの既存exportはKDVに同等機能が入るまで維持し、その後KDVへ移譲して削除する
- editor-viewer同期制御はKatanAが担い、KatanAがviewerまたはeditorへ命令する
- 分離優先順位は P0 `katana-ast-lint`、P1 `katana-markdown-model`、P2 `katana-ui-widget`、P3 その他とする
- `katana-ast-lint` を先に分離し、分離後repository間の品質統制を共通化する
- `katana-ui-widget` 分離を後続OpenSpecに切り、KMM統合時に共通UI部品をKatanA本体へ残しすぎない
- KMM public DTOとmetadata APIが固定されるまで、KDV、KLE、KCF、KatanA統合、KUWは独自document modelや独自metadata schemaを作らない
- KCF export関連の新規計画はKDVへ移譲する

## Capabilities

### New Capabilities

- `kme-markdown-platform`: KMMを中心に、Markdown文書モデル、metadata、editor、preview、export、widget分離のrepository責務を定義する

### Modified Capabilities

- `markdown-authoring`: editor保存時metadata同期とAST単位編集を後続計画へ接続する
- `split-scroll-sync`: KMMのsource rangeとstable node idをKatanAの同期材料として使う
- `markdown-export`: KDVがKMMモデルと解決済みmetadataを使うviewer/export pipelineを定義する

## Impact

- `openspec/changes/bootstrap-kme-document-model` in `katana-markdown-model`: KMM本体
- `katana-language-editor`: 保存時metadata同期
- `katana-document-viewer`: Floem viewerとHTML/PDF/PNG/JPG export
- `katana-diagram-renderer`: 外部描画（Mermaid / Draw.io / PlantUML / math）
- `katana-canvas-forge`: HTML / PDF / PNG / JPG export。既存exportはKDV移譲まで維持
- `katana-ui-widget`: 共通UI部品分離の後続計画
- `katana-ast-lint`: 分離後repository共通のAST lint
