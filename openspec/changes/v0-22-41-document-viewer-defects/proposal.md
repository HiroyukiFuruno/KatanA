## Why

実ファイルで HTML の操作・スクロール、Office/PDF のエクスプローラー選択、Excel のシート操作、目次、Office 変換が一貫して動作していない。修正版の公開依存だけを使う v0.22.41 として、添付された全不具合を下流まで検証して配布する必要がある。

## What Changes

- HTML プレビューのブラウザー入力面を維持し、JavaScript 操作とスクロールを実ホストで可能にする。
- PDF/Office を開いた時に、対応するファイルをエクスプローラーで選択・表示する。
- XLSX のページ矢印を廃止し、実シート名を使う下部タブで切り替える。
- DOCX/XLSX/PPTX では目次を非活性にし、PDF では解析済み階層目次から対象ページへ移動する。
- 公開済みの `office2pdf-katana 0.6.10`、KDV 0.5.5、KRR 0.4.17 を registry 依存として取り込み、依存更新後も既存の品質ゲートを維持する。

## Capabilities

### New Capabilities

なし。

### Modified Capabilities

- `html-file-preview`: HTML の操作・スクロールを実ブラウザー入力面で保証する。
- `multi-format-document-preview`: Office の実シートタブ、全添付不具合、公開 registry 依存の要件を追加する。
- `table-of-contents`: Office の非活性化と PDF 階層目次・ページ移動を追加する。
- `workspace-shell`: 開いている PDF/Office ファイルのエクスプローラー選択・表示を追加する。

## Impact

KatanA のプレビュー入力、ドキュメント操作、目次、エクスプローラー状態、KDV/KRR 依存とリリース契約が対象になる。KDV 0.5.5 と `office2pdf-katana 0.6.10` は KatanA より先に crates.io へ公開し、path/git 依存は最終成果物に残さない。
