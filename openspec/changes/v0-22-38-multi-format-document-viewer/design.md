## Context

KatanA `v0.22.37` はMarkdown、画像、browser-equivalent HTMLを表示できるが、
PDF / DOCX / XLSX / PPTX viewerは持たない。旧 `v0.22.12` changeはKatanA UI内の
renderer、PDFium、WebViewを前提にしており、現在のKatanA -> KDV/KRR責務境界と
Rust-first方針に反するため再利用しない。

KDV `v0.5.1` がPDF / DOCX / XLSX / PPTXのengine evaluation、format routing、
worker lifecycle、materialization、neutral artifact、viewer state、diagnosticsを所有する。KatanAはengine選定へ介入せず、
承認済みかつ公開済みKDV contractだけをhost shellへ接続する。

## Goals / Non-Goals

**Goals:**

- local fileと直接取得可能なdocument URLを同じsource pipelineへ入れる
- KDVのpage / document / sheet / slide viewerをdocument tabへ接続する
- capabilityに応じたnavigation、zoom、fit、copy、open controlsを提供する
- format、source、operation、layer、causeを含むdiagnosticsを表示・log出力する
- KatanA `v0.22.38` のheadless acceptanceとrelease gateを機械化する

**Non-Goals:**

- KatanAでPDF / OOXMLをparseまたはlayoutしない
- KRRへdocument viewer APIを追加しない
- Chromium、WebView、PDFium、browser helperを追加しない
- Google Docs / Sheets / Slides、Office 365などのWeb applicationを埋め込み表示しない
- Office編集、PDF編集、annotation、form入力、macro、animationを追加しない
- CSV / SVG / WebP / AVIF viewerとPDF export paginationを同時実装しない

## Decisions

### D1. KatanAはthin hostに限定する

KatanAが所有するのはsource intake、tab identity、lifecycle、host command、
platform I/O、現行egui backendへの中立frame投影だけとする。KDVがformat detection後の
engine session、worker、render scale、XLSX materialization、artifact、viewer state、
capability、diagnostics、document surfaceを所有し、KUC implementationをprivateに利用する。
KatanAはKUCまたはKDV/KUC混成crateへ直接依存せず、KDV document surfaceのcommandと
frameだけをhost shellへ接続する。

KatanAは`PdfViewerSession`、`OfficeStaticViewerSession`、`SpreadsheetViewerSession`、
`SpreadsheetGridSurface`を直接構築せず、format別runtime enumも持たない。pointer座標を
KDV commandとして渡し、document hit-test、layout、selection algorithmを実装しない。

KatanA内にformat managerやprivate rendererを置く案は、責務重複とquality driftを
生むため採用しない。KRRは文書内diagram/mathでKDVが既存public APIを使う場合だけ
間接的に関与し、KatanAからdocument sourceを渡さない。

### D2. fileとdirect document URLを同じsource contractへ正規化する

local fileはcanonical file URL、bytes、MIME hint、revisionをsource descriptorへ
変換する。URLは `https` を既定とし、redirect、content length、timeout、MIME、
download policyをKatanA platform layerで検証してからbytesと最終URLをKDVへ渡す。

URLがHTML application、authentication page、unsupported scheme、MIME mismatchの
場合はbrowserとして解釈せず、typed fetch diagnosticsを返す。クラウドeditor URLを
WebViewで開く案はRust-first/offline contractに反するため採用しない。

### D3. controlsはKDV capabilityを唯一の正とする

PDF page、DOCX document/page、XLSX sheet、PPTX slideのnavigation差異をKatanA側の
format分岐へ複製しない。KatanAはKDVが返すcapabilityとcommand availabilityを使い、
未対応controlをdisabledまたは非表示にする。

### D4. diagnosticsはlayerを保持して表示する

error modelは `layer`、`operation`、`format`、`document`、`cause`、
`unsupported feature`、`resource limit` を保持する。UIには短いsummaryと展開可能な
detailsを表示し、structured logには同じfieldとsource chainを出力する。

worker停止だけを表示する案は原因追跡を妨げるため採用しない。KDV originのdiagnostics
は文字列へ潰さずKatanA boundaryまで保持する。

### D5. KDV publicationをrelease integrationのgateにする

production `Cargo.toml` とrelease harnessはcrates.ioのKDV `0.5.1`を利用し、
path / git dependencyを許可しない。KDV側のengine feasibilityとuser approval、
KDV v0.5.1 publication、KatanA dependency update、headless acceptanceの順で進める。

### D6. release targetはv0.22.38とし、将来minor計画を繰り上げない

最新公開版 `v0.22.37` に対する現在のmechanical SemVer contractは隣接patchだけを
許可するため、本changeは `v0.22.38` とする。v0.23.0〜v0.28.0の将来minor changeを
移動する必要はない。撤回済み `v0.29.0` は再利用せず、preview-driven local editingは
version-undecidedのまま保持する。

## Risks / Trade-offs

- [Risk] KDV engine選定が未完了のためKatanA contractが変動する
  -> KDV neutral API公開までKatanA production実装を開始しない。
- [Risk] Office layoutがMicrosoft Officeと一致しない
  -> KDV profileとunsupported capabilityをUIへそのまま表示し、互換性を誇張しない。
- [Risk] remote documentが巨大または偽MIMEである
  -> redirect、size、time、MIME、schemeの上限をfetch前後で検証する。
- [Risk] large PDF/XLSXでUI threadが停止する
  -> KDV workerを利用し、generation付き結果、cancel、bounded queueを契約テストする。
- [Risk] format別UI分岐がKatanAへ増殖する
  -> capability-driven controlsとneutral commandだけを使用するownership guardを置く。

## Migration Plan

1. KDV feasibility gateとユーザー承認を完了する。
2. 必要な場合だけKUC generic 2D gridを先行releaseする。
3. KDV `v0.5.1`をreleaseし、crates.io artifactを検証する。
4. KatanAを公開済みKDV `0.5.1`へ更新し、source routingとthin backend projectionを実装する。
5. format corpus、failure corpus、URL corpusをheadless acceptanceで検証する。
6. `v0.22.38` adjacent SemVer、ownership、registry dependency、release artifactを検証する。

KDV integrationに失敗した場合はKDV dependencyとroutingを戻し、`v0.22.37` の既存format
behaviorを維持する。unsupported formatを別rendererへfallbackしない。

## Resolved Decisions

- PDFはKDV `hayro` profileのstatic page artifactを表示する
- DOCX / PPTXはKDV `office2pdf` profileでcanonical PDFへ変換し、KDV `hayro` page / slide artifactを表示する
- XLSXはKDV `IronCalc` profileのsemantic sheet artifactをKDV private document surface経由のgeneric 2D gridとして表示する
- KatanAはKDV統一document session / `DocumentSurfaceFrame`だけを利用し、KUC node、format別session、materialization、layout、hit-test、rendererを所有しない
- direct document URLは最終 `http` / `https` URL、MIME、signature、256 MiB上限、30秒のtransport / host deadlineを検証する
