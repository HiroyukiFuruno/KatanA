## Why

KatanAはPDF / DOCX / XLSX / PPTXをKatanA内で閲覧できず、ドキュメントレビューの
途中で外部アプリへ切り替える必要がある。KDV `v0.5.2` のbackend-neutral multi-format
viewerを、KatanA `v0.22.38` の薄いhost integrationとして取り込む。

## What Changes

- local fileと直接取得可能なdocument URLからPDF / DOCX / XLSX / PPTXを開く
- sourceの取得、MIME / extension判定、tab identity、host command処理をKatanAが担う
- parsing、layout、page / sheet / slide artifact、viewer state、diagnosticsをKDVへ委譲する
- page / sheet / slide navigation、zoom、fit、copy、openの対応状況をKDV capabilityに従って表示する
- password protected、corrupt、unsupported、resource limit、URL取得失敗を原因別に表示する
- WebView、Chromium、PDFium、KatanA内format parser / rendererを導入しない
- KDV `v0.5.2` 公開後にcrates.io dependencyとして取り込み、path / git dependencyを残さない
- KatanAはKUCまたはKDV/KUC混成crateへ直接依存せず、KDV document surfaceだけを利用する
- release targetを公開済み `v0.22.37` の隣接patch `v0.22.38` に固定し、撤回済み
  `v0.29.0` を引き続き拒否する

## Capabilities

### New Capabilities

- `multi-format-document-preview`: KDV経由でPDF / DOCX / XLSX / PPTXのfile/URL sourceを閲覧し、navigation、capability、diagnosticsをKatanA shellへ統合する

### Modified Capabilities

- `workspace-shell`: explorer、tab、direct document URLからmulti-format document previewを開く

## Impact

- `crates/katana-core`: supported document source、MIME / extension routing、host command
- `crates/katana-ui`: document tab、KDV viewer bridge、controls、diagnostics
- `crates/katana-platform`: local file readとdirect document URL fetch
- `Cargo.toml` / `Cargo.lock`: 公開済み `katana-document-viewer 0.5.2`
- `scripts/screenshot`: PDF / DOCX / XLSX / PPTXのheadless acceptanceと証跡
- `scripts/release`: adjacent SemVer、registry dependency、ownership、format corpusのrelease gate
- `katana-render-runtime`: 変更なし
