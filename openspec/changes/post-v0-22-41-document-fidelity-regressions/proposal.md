## Why

v0.22.41 の実ファイル確認で、Office/HTML に適用できないサイドメニューが操作可能なまま、XLSX の下部シートタブが画面外へ押し出され、HTML の CSS/JavaScript 互換性、Office の表示品質、PPTX 初回表示時間、DOCX/XLSX/PPTX 共通の OOXML ZIP 互換性にも未解消差分が残っている。加えて KatanA の実プロセスは空 workspace でも約 700 MiB、観測中に約 1.1 GiB RSS と 1.7 GiB peak footprint に到達しており、起動時ベースラインと文書切り替え後の資源解放を分離してフリーズ回帰を計測可能な契約にする必要がある。

## What Changes

- Office/HTML で意味を持たない目次、ダウンロード、スライドショー、表示系サイドメニューを非活性にし、ショートカットや直接アクションからも開けないようにする。
- XLSX のシートタブを表の下端に常時確保し、実シート名で切り替えられるようにする。フィルターは解析状態と操作を KDV が所有し、対応するデータがある場合だけ KatanA が薄く投影する。
- 指定 HTML を通常ブラウザーと差分比較し、KRR が所有する CSS/JavaScript/Web API の不足を明示して完全経路で回帰検証する。
- DOCX/XLSX/PPTX の安全な ZIP 解析、表示品質、PPTX 初回表示時間を実ファイルで測定し、原因層で修正する。
- `DEBUG=true` の時だけ文書取得、変換、解析、初回フレーム、テクスチャ更新、セッション終了の時間と資源量を出す共通診断を追加する。
- 空 workspace の定常メモリを計測し、文書の反復切り替え・閉鎖で worker、RGBA フレーム、GPU テクスチャ、変換成果物が解放され、RSS/footprint が無制限に増えない回帰ハーネスを追加する。
- Office/PDF の常時表示対象追加後もエクスプローラーがワークスペース全ツリーを毎フレーム複製・全行描画しないようにし、操作遅延をフレーム時間で検知する。
- 配布アセットの CPU architecture、最低 OS、sidecar を検査し、宣言した対応環境で packaged app が起動することを自動確認する。

## Capabilities

### New Capabilities

- `document-preview-resource-lifecycle`: 文書プレビューの時間計測、資源解放、メモリ上限、フリーズ回帰検知を定義する。
- `desktop-startup-compatibility`: 配布アセットの対応 architecture、sidecar、現在の無償配布方針と実起動 smoke test を定義する。

### Modified Capabilities

- `html-file-preview`: 実 HTML の CSS/JavaScript/Web API 互換性と適用不能メニューの非活性化を追加する。
- `multi-format-document-preview`: XLSX 下部タブ、フィルター、OOXML ZIP 互換性、Office 表示品質、PPTX 初回表示時間、適用不能メニューの要件を追加する。

## Impact

KatanA のプレビューサイドバー、ドキュメント面レイアウト、エクスプローラー描画、診断ログ、配布契約、受入ハーネスが対象になる。HTML のブラウザー意味論は KRR、Office/PDF/スプレッドシート解析・変換・フィルター・キャッシュ方針は KDV と office2pdf 境界に留める。sibling に修正が必要な場合は KatanA に代替実装を持ち込まず、上流を公開して registry 依存から検証する。
