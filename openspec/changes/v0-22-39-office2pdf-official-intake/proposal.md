## Why

KatanA v0.22.38 は PDF / DOCX / XLSX / PPTX を KDV 経由で表示できるが、PPTX の
段落行送りと font fallback を含む upstream 修正 #745 が当時の crates.io
`office2pdf 0.6.5` には未公開だった。このため KDV v0.5.2 は一時的な
`office2pdf-katana 0.6.6` registry package を利用した。

公式 `office2pdf 0.6.7` は #745 を含み crates.io に公開済みであり、KDV v0.5.3 も
その公式 package だけを使って公開された。KatanA は公開済み KDV を registry から
intake し、temporary maintenance package を最終配布経路から除去する。

## What Changes

- `katana-document-viewer = "=0.5.3"` を crates.io registry dependency として固定する
- lockfile が公式 `office2pdf 0.6.7` だけを解決し、`office2pdf-katana` を含まないことを
  release contract で検証する
- v0.22.38 -> v0.22.39 の隣接 patch SemVer guard と release evidence を更新する
- PDF / DOCX / XLSX / PPTX の既存 KDV surface と KatanA thin host を変更せず、3 OS の
  headless acceptance と配布 asset を再検証する

## Non-Goals

- KatanA に Office/PDF parser、renderer、font resolver、layout 補正を追加しない
- KDV/KRR/KUC の公開 API、format profile、UI backend boundary を変更しない
- Chromium、WebView、PDFium、LibreOffice、git/path dependency を導入しない
