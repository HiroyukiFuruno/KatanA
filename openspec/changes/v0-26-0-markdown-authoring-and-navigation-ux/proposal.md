## Why

KatanA には markdownlint diagnostics、画像取り込み、TOC、タブ DnD などの個別基盤はあるが、日常的な Markdown authoring workflow としては未接続なまま残っている。`tmp/対応したい改善/memo.md` で挙がっている課題は、既存機能の不足や仕様不整合を跨いでおり、局所修正ではなく authoring / navigation / file operation UX を一体で整理し直す必要がある。

## What Changes

- markdownlint diagnostics の出荷スコープを拡張し、Problems Panel だけでなく editor 上の inline underline、hover popup、quick-fix entry point、theme 連動の装飾色まで含む user-facing contract に整理する
- Markdown editor に常設の authoring toolbar を追加し、見出し・装飾・リスト・表・画像挿入を GUI から実行できるようにする
- 画像取り込み workflow を再統合し、clipboard paste、外部 image drag-and-drop、既存 file attach を同じ ingest pipeline に揃える
- image ingest の保存先、命名、挿入位置、asset refresh を明文化し、既存実装の `./asset/img` 規約を source of truth として再定義する
- Markdown から参照されている local image を explorer 上でも遅延サムネイル表示し、preview 以外でも asset の存在を把握できるようにする
- 単一ファイルを「一時ワークスペース」として開く flow、現在ワークスペースに追加して開く flow、外部 file drag-and-drop open を定義する
- explorer から tab への drag-and-drop、explorer 内の file move drag-and-drop、move confirmation 設定を追加し、file operation を mouse 主体でも完結できるようにする
- TOC から不要な塗り背景を除去し、accordion 展開、全開 / 全閉、階層ガイド線、設定永続化を追加する

## Capabilities

### New Capabilities

- `markdownlint-diagnostics-experience`: markdownlint diagnostics の rule coverage、inline decoration、hover detail、quick-fix entry、theme-linked visual settings を扱う
- `markdown-asset-ingest`: image file attach、clipboard paste、external image drop、保存先規約、命名、cursor-aware insert を扱う
- `workspace-file-operations`: 単一ファイル open、temporary workspace、外部 file drop open、explorer-to-tab drop、explorer 内 move を扱う

### Modified Capabilities

- `markdown-authoring`: source-first editor に GUI authoring controls と discoverable command surface を追加する
- `local-asset-preview`: local image reference を preview だけでなく explorer thumbnail と asset refresh flow まで拡張する
- `table-of-contents`: TOC の表示構造を accordion 化し、expand/collapse all と guide line 設定を追加する
- `theme-settings`: markdown diagnostics の underline / warning decoration color を theme 設定から変更できるようにする

## Impact

- 主な影響範囲は `crates/katana-linter/src/markdown.rs`、diagnostics state / Problems Panel / editor rendering、`crates/katana-ui/src/views/panels/editor/*`、`crates/katana-ui/src/app/action/image_ingest.rs`、explorer / top bar drag-drop、`crates/katana-ui/src/views/panels/toc.rs`、`crates/katana-platform/src/settings/*`
- active change `v0-23-0-local-llm-lint-autofix` とは役割を分離し、本 change は diagnostics UX と deterministic quick-fix entry を対象にする
- archive 済み `2026-04-12-v0-22-0-markdown-authoring-and-image-assets` の image ingest 設計、および `2026-04-12-v0-19-0-markdownlint-parity-diagnostics` の rule parity 契約を再利用する
