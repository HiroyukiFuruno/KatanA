## Context

KatanA の現行 preview は Markdown document を中心に設計されている。Markdown 内の HTML block は既存の `HtmlParser` / `HtmlRenderer` 経由で表示できるが、`.html` / `.htm` ファイルそのものは標準 visible extension に含まれておらず、file open dialog や drag-and-drop の対象にもなっていない。

一方で、`katana-document-viewer` は `SourceKind::Html` / `DocumentKind::Html` と direct HTML source normalizer を既に持っている。Katana 側の実装では、最初から KDV / KRR の外部修正を前提にせず、Katana 内の open / preview routing を整える。KDV API が必要十分でない場合のみ、外部 repo の issue または OpenSpec change へ分離する。

この変更は将来の preview-driven local editing 計画の制約と整合させる。つまり、WebView、React、DOM runtime、bundled web app を導入しない。

## Goals / Non-Goals

**Goals:**

- `.html` / `.htm` を workspace tree、file open dialog、drag-and-drop から開ける。
- HTML file を Markdown としてではなく direct HTML preview として表示する。
- HTML preview は既存の HTML block 表示能力または KDV direct HTML contract を利用する。
- Markdown lint、Markdown format、Markdown export が HTML file へ誤適用されないことを検証する。
- KDV / KRR の作業要否を計画内の明示ゲートで判断し、不要な外部 repo 変更を避ける。

**Non-Goals:**

- CSS layout / JavaScript / iframe / form 実行を含むブラウザ相当の HTML viewer を実装しない。
- WebView、React、DOM runtime、bundled web app を追加しない。
- HTML file の PDF / PNG / JPEG export をこの change の必須範囲にしない。
- KRR public API をこの change 内で拡張しない。
- KDV / KRR の version bump、release prep、issue 作成を必要性確定前に行わない。

## Decisions

### Direct HTML は Katana の文書種別分岐で扱う

Katana 側で active document の拡張子を判定し、`.html` / `.htm` の場合は Markdown として section split しない direct HTML preview 経路へ渡す。`.drawio` と同じくファイル種別ごとの preview source normalization として扱うが、HTML は diagram fence へ包まず HTML document として扱う。

代替案として、HTML source をそのまま Markdown preview に流す方法がある。しかしそれでは Markdown parser の HTML block 判定に依存し、HTML document 全体の preview contract が曖昧になるため採用しない。

### 表示品質は safe readable preview に固定する

MVP の表示品質は「安全に読める preview」とする。対応対象は、既存 HTML renderer と KDV direct HTML normalizer が扱える見出し、段落、リンク、画像、details、table などの静的 HTML 表示である。CSS / JS の忠実再現は別 change の責務判断に回す。

代替案として、最初から WebView または headless browser を使う方法がある。しかし将来の preview-driven local editing 方針と衝突し、security / sandbox / asset policy / platform parity の設計が必要になるため採用しない。

### KDV は利用可能性を先に検証し、不足時だけ外部化する

KDV は既に HTML source kind と direct HTML normalizer を持つため、Katana 側から直接利用できるかを最初に確認する。公開 API や出力 model が不足する場合は、Katana change 内で暫定実装を正規化せず、KDV 側の issue または OpenSpec change として切り出す。

代替案として、最初から KDV issue を作ってから Katana へ戻る方法がある。しかし現状の API で足りる可能性が高く、不要な repo 越境を増やすため採用しない。

### KRR は MVP から外す

KRR は diagram / render-runtime として、入力を描画成果物へ変換する処理を担う候補である。HTML file preview の MVP は viewer routing と静的 HTML 表示であり、KRR の責務ではない。CSS layout、JS 実行、pixel faithful rendering が要求された時点で、KRR 側 change を別途検討する。

### Markdown 専用機能は extension gate を維持する

Diagnostics、formatting、Markdown export は Markdown document の機能として扱う。HTML file を開けるようにしても、これらの処理は `.md` / `.markdown` に限定され、HTML buffer を Markdown linter / formatter / exporter へ流さない。

## Risks / Trade-offs

- **Risk: ユーザーがブラウザ相当表示を期待する** -> MVP の表示品質を safe readable preview として spec に明記し、CSS / JS / iframe は non-goal とする。
- **Risk: Markdown parser 経由の HTML 表示で document 構造が落ちる** -> HTML file は Markdown document として扱わず、direct HTML preview 分岐を設ける。
- **Risk: KDV API が Katana から利用しづらい** -> Task 1 で API 適合性を検証し、不足があれば KDV issue / OpenSpec change に分離する。
- **Risk: HTML file に Markdown 操作が表示される** -> diagnostics / formatting / export の extension gate をテストで確認する。
- **Trade-off: fidelity より安全性と実装範囲を優先する** -> MVP では文書確認用途を優先し、忠実 rendering は後続判断に残す。
