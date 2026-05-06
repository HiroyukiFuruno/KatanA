## Why

KatanA はMarkdownを「見る」「確認する」「必要箇所だけ直す」ことを主面に置く。現在はpreview、export、editor、fixture、metadata相当の情報が複数の実装へ分散し、KatanA側で既存libraryをhackして微細な表示調整を行っている。

このままでは、表（table/grid）、ホバー強調、コード位置同期、脚注、alert、badge、絵文字、PDFページング、LLM注釈、AST単位コピー・編集を同じ仕様で扱えない。KME（katana-markdown-engine）を文書モデルの中核として新規repository化し、KatanA ecosystem全体のMarkdown仕様を固定する。

## What Changes

- KMEを新規repositoryとして作成し、Markdown文書モデル、metadata schema、位置解決を所有させる
- KME v0の仕様fixtureを `assets/fixtures/sample.md`、README badge、alert、description listで固定する
- metadataはMarkdown本文へ埋め込まず、外部ファイルとして扱う
- editorは保存直後のmetadata同期を担い、KMEは新旧本文とmetadataからtarget再対応を判定する
- previewはFloem前提のネイティブ表示でKMEモデルを消費する
- kcfはKME本体ではなく、出力品質ゲートとHTML/PDF/PNG/JPG exportを担う
- 分離優先順位は P0 `katana-ast-lint`、P1 `katana-markdown-engine`、P2 `katana-ui-widget`、P3 その他とする
- `katana-ast-lint` を先に分離し、分離後repository間の品質統制を共通化する
- `katana-ui-widget` 分離を後続OpenSpecに切り、KME統合時に共通UI部品をKatanA本体へ残しすぎない

## Capabilities

### New Capabilities

- `kme-markdown-platform`: KMEを中心に、Markdown文書モデル、metadata、editor、preview、export、widget分離のrepository責務を定義する

### Modified Capabilities

- `markdown-authoring`: editor保存時metadata同期とAST単位編集を後続計画へ接続する
- `split-scroll-sync`: KMEのsource rangeとstable node idを同期基盤として使う
- `markdown-export`: kcfがKMEモデルと解決済みmetadataを使う後続移行を定義する

## Impact

- `openspec/changes/bootstrap-kme-document-model` in `katana-markdown-engine`: KME本体
- `katana-language-editor`: 保存時metadata同期
- `katana-document-preview`: Floem previewでKMEモデルを表示
- `katana-canvas-forge`: v0.1.2は品質ゲートに限定
- `katana-ui-widget`: 共通UI部品分離の後続計画
- `katana-ast-lint`: 分離後repository共通のAST lint
