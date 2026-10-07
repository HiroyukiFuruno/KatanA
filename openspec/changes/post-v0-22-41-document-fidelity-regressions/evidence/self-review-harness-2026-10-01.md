# 文書受入ハーネス差分の自己レビュー

対象はin-process runnerの入力/geometry/typography/resource計測と原本fixture実行導線。packaged mainの受入を代替しない。

- P1: accessibility Node::rectへの倍率再適用を除去。pixel ratio1/2の実egui button入力回帰と、実XLSX下部タブのgeometry/0→1→0を確認。
- P1: ExportPngの文書未選択skipを拒否。旧実CLIはexit0、修正版はexplicit error/exit1でnegative契約成功。
- P1: viewport両指定の無言優先をrequest load時の拒否へ変更。旧実CLIはexit0、修正版はexit1。新単体回帰を含む39件成功。
- P1: cold baseline前のCloseActiveDocumentを固定待機から実open document/resource/Office workerのidle確認へ変更。同じmixed cycleが再実行成功し、10回後も全resource0、warm RSS増分1184KiB（65,536KiB上限は維持）、UI frame113→183。
- candidate生成は新規absolute出力を要求し参照フォルダへ書かない。controls-on拒否/同一frame controls-off2fixtureのactual render契約が成功。glyph mesh bandはraster inkとは区別し、回転はunsupportedで返す。
- 別Cargo root targetを分離、公開registry sibling graphをlockedで使用。本repo内tool依存はpathであり、未公開sibling採用ではない。root format styleを明示し通常fmt checkの結果を統一。
- runner release build、39tests、release全target strict Clippy、paint-metrics2tests、JSON、format/diff、unsafe candidate output/実export negative契約が成功。

残DoD: 原本HTML公開KRR受入、canonical score95、配布main/sidecar実入力、全coverage/combined gates。画像参照や品質閾値は変えていない。旧未採用候補は内容/ハッシュを保持してassets/candidatesへ区別した。
