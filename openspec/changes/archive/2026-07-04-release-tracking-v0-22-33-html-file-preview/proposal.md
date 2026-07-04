## Why

KatanA は Markdown だけでなく、workspace 内の HTML ファイルも確認対象として扱う必要がある。現状の preview 経路は Markdown 中の HTML 断片には対応しているが、`.html` / `.htm` ファイルそのものは標準の表示・open 対象ではなく、開けた場合も Markdown preview として処理される。

この変更では、HTML ファイルを KatanA の view-first workflow に取り込みつつ、WebView / React / DOM runtime を導入しない既存方針を維持する。

## What Changes

- `.html` / `.htm` を workspace tree、file open dialog、drag-and-drop の対象に追加する。
- HTML ファイルを active document として開いた場合、Markdown としてではなく direct HTML preview として表示する。
- direct HTML preview は、既存の HTML block renderer または KDV の direct HTML source contract を使い、KatanA 内にブラウザ相当の runtime を追加しない。
- Markdown lint、Markdown format、Markdown export など Markdown 専用機能が HTML ファイルに誤適用されないことを確認する。
- KDV / KRR の変更は最初から前提にせず、Katana 側で不足が確定した場合だけ KDV / KRR 側の issue または OpenSpec change へ切り出す。

## Capabilities

### New Capabilities

- `html-file-preview`: `.html` / `.htm` ファイルを workspace から開き、KatanA preview pane で安全な direct HTML preview として表示する。

### Modified Capabilities

- `workspace-file-filter`: 標準表示・open 対象に HTML ファイルを追加する。

## Impact

- `crates/katana-core/src/workspace/`: HTML 拡張子を標準 visible/openable file contract に追加する。
- `crates/katana-ui/src/app/action/file_open.rs`: file open dialog と drag-and-drop の対応拡張子を更新する。
- `crates/katana-ui/src/app/preview.rs` / `crates/katana-ui/src/preview_pane/`: active document の拡張子に応じて HTML preview 経路を選ぶ。
- `crates/katana-ui/src/app/action/process_diagnostics.rs` / Markdown formatting / export 周辺: HTML ファイルに Markdown 専用機能が誤適用されないことを確認する。
- `katana-document-viewer`: 既存の `SourceKind::Html` / direct HTML normalizer を利用できるか確認する。公開 API 不足が確定した場合だけ外部 change を作る。
- `katana-render-runtime`: MVP では対象外。CSS / JS / pixel faithful rendering が要求された場合のみ、別 change として責務判断する。
