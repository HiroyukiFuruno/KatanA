# 候補・生成成果物・公開後受入の分離

## 対象と状態

最新の本人承認は、KDV/KRRの追加公開を待たず、macOS隔離環境でKatanAの配布文書操作を確認し、全OS一括公開後に配布物を確認する方針である。Chrome比較の不能だけを全体停止の理由にしない。独立95点の達成、未知のhost不具合の延期、元メモリ予算の免除を承認されたものとは扱わない。

旧strict検査は維持する。host専用の固定版・固定承認・別canonical schemaを追加し、未実測の比較・他target文書操作・公開後検証を合格へ補完しない。コードは通常署名commit `e138d5aab6b885f18c9d874be8d36061617bd6a6`へ統合済み。下記対象検査も終了0だが、正式host manifest・公開・配布版受入は未完了である。

## 修復した不整合

- 旧host入口にも全target clean-machine/Chrome/Office独立参照を要求していた。今回は実macOS隔離host受入とstrict全体評価を分け、環境と未検証を実値で記録する。
- artifact検査がscopeを省略してstrictを再実行していた。host入口を明示しても、全5配布物の実payload・サイズ・archive/installer・architecture検査を省略しない。
- 公開前に独立Actionsビルドのbytesをローカル候補と同一にする循環を解消する。候補は最終clean source/lock/registryへ、各Actions成果物は同sourceと実main/worker/全5asset digestへ束縛する。公開直後は`postpublication_pending`とし、downloadした実Mac成果物の操作・解放・終了後に初めて公開後host受入を成立させる。

## 記録と拒否境界

producerは私的な原本・画面・ログをローカルに保持し、必要な数値・hash・操作状態だけを公開transcriptへ出す。実rawを読まずにPASS JSONを生成する入口はない。source/lock/公開registry、実commit、bundle、実PID/start、各cycleの実loaded source、実frame/index、HTML adapter/epoch/FNV/dirty/revisionを検査する。FNVをSHAと称さない。

外部collectorは実main/子processのidentity・メモリ・CPUと、既存heartbeatの実進行を記録する。Office workerを未観測のまま0とせず、HTMLへ存在しないworkerを要求しない。元200ms後のheartbeat進行検査とpartial readの再読条件を再利用し、観測欠落と実停止を区別する。新しい操作口・allocator・閾値は追加しない。

7原本とHTMLの各10反復、cold/steadyのRSS・physical footprint各196608/65536KiB、close/app終了5000msを維持する。PPTX初回15秒超過の既存狭い例外は、他形式・メモリ・終了・描画へ広げない。DEBUG=true/読取専用記録ON/heap・vmmap inspector OFFを実際の設定として保存し、DEBUG=falseの計測へ改名しない。

## 独立レビューと検査

初回の独立レビューで、canonical GitHub URLの拒否、存在しないbuild commitの受理、UI進行の欠測、各openの実loaded input未結合を検出し修復した。Mac targetのlaunch slice未結合、0ms初回表示の受理も修復した。rootが追加したWindows改行と通常検査入口についても再レビューした。

Windowsの実Git fixtureでは、同commitかつcleanでもautocrlf=trueのcheckoutでsource/Cargo.lockのbytesが変わることを確認した。Windows release runnerのcheckout前に変換を無効化し、hash対象・bytes検査を緩和せず一致させる。新33件の回帰が既存underscore discoverから外れていたため、既存preflightに2suiteの明示実行を追加した。

- host 29件、source/publication 6件、artifact 8件、gate 32件、strict 55件、計130件が独立検査で終了0。合成入力は単体検査専用で、実受入データへ転用しない。
- rootも新source/publication 6件を終了0で回収。preflightのzsh構文検査は終了0。
- 最終独立レビューと31ファイルの実SHAはローカル`tmp/oct25-host-admission-independent-final-review.md`と同`-source-sha256.json`に保存。未対応P0/P1/P2なし。Rust実行・新collector実動作・実配布受入の代用ではない。
- 公式AST30件（10652）、strict Clippy全4crate（83360）、既存fmt-check（48961）は終了0。
- 読取専用Rust観測の公式対象検査は初回が容量不足で終了101、再実行74721は終了0で15件成功・2232件filter（42suite）。失敗を成功へ書き換えず、元閾値・検査範囲を変更していない。

## 公開後の記録PRの経路

既存の旧docsブランチだけを扱う分岐では、今回指定されたrelease/*から公開後host判定へ到達できなかった。完全一致の`release/v0.22.42-public-release-evidence`だけを既存`--post-release-host`へ接続し、通常release/近似名/別版と旧docs経路は維持する。source commitを検証できるようreadiness checkoutは全履歴を取得する。公開済みstable Releaseを要求する既存guardは変更しない。実workflowのBash分岐13ケースを含むflow15件、Draft CI5件が独立検査で終了0。別2ファイルの独立レビューでも未対応P0/P1/P2なし。

## 残件

最終clean sourceの全構成package、実rawからのhost manifest生成、current review/CI、全OS公開、全5配布物と公開後Mac受入、Issue/OpenSpec/安全cleanupは未完了。現d668診断候補の大型XLSX steadyとPPTX coldは元メモリ予算に未達であり、このphase修復によって免除しない。
