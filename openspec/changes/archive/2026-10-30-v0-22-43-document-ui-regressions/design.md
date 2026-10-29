## Context

本人の直近11報告をIssue #388と対応ボードB01〜B11で追跡する。約100KB PNGの約30秒遅延が最優先であり、単体decode・文言修正だけでは解決としない。公開済みv0.22.42は変更せず、既存checkoutのrelease/v0.22.43へ通常統合する。

## Goals / Non-Goals

**Goals:**

- 文書操作設定を形式とlive runtime状態から一元評価し、全sidebar・shortcut・palette・直接実行へ適用する。
- 同じLoading部品と局所状態で全形式の待機を表示する。
- 原本の要求→保存→frame→texture準備を分け、変更なし反復と複合タブの遅延を実経路で調べる。
- タブグループの一回だけのpopup開始と確定/取消後の入力解放、画像下部制御領域、HTML/Office終了の応答性を回帰させる。
- 配布とDockerの所有・失敗・保持状態を明確にする。

**Non-Goals:**

- KDV/KRRの別repository実装、旧版asset差し替え、本人の確認やTerms承認の代行。
- 原因未確定の全面刷新、検証閾値の緩和、別テスト環境の成功を実配布版の証明にすること。

## Decisions

1. 既存PreviewMenuAvailabilityを共通DocumentOperationSettingsへ拡張する。mode可否をToolsの代用にせず別capabilityとして定義する。AppStateへ投影するruntime情報は永続化せず、文書identity/revision・preview generation・実readyへ結合し、変更時にpaletteを更新する。
2. Loaderの待機・retry・decode状態をLoading widgetへ複製しない。widgetは局所状態と解決済みの文言だけを受け取る。
3. グループの名称draft/focus/open markerを一つのpopup状態管理へまとめ、一回だけ開き、確定/取消actionはgroup IDを結合する。他グループの編集状態を消さない。
4. 画像の制御は独立領域をallocateする。小さい画像でも全画面ボタンの領域を確保し、全画面では画像clipと下部制御を分ける。
5. HTMLは依存adapter Dropの同期join、Officeはworkerの非同期終了を別に調べる。共通保存・preview drop・allocator返却の実時間を採り、支配的なKatanA側原因にだけ修正を入れる。候補だけで原因確定にしない。
6. Dockerはowner/purpose/run IDラベルとhostへの証跡保存記録を用いる。未知・共有・参照中・未保存は保持し、後日回収は対象を明示する。正常終了の既存cleanupと必須UI受入条件は維持する。

## Risks / Trade-offs

- callback単体の成功が30秒遅延を覆い隠す → 元入力のfull app経路と複合タブで別測定し、未再現を明記する。
- 非同期解放による残存資源 → worker/drain・正常終了・元メモリ予算を維持する。
- snapshotのstale readiness → identity/generationを更新してから全入口で同じ設定を読む。
- volumeの誤回収 → owner/用途/実行ID/停止中参照/保存hashを検査し、force/pruneを提供しない。
- 共有crateの同時編集 → 担当pathとnative buildの枠を調整し、結果回収後に次の実検証を進める。

## Migration Plan

対象検証後に関心別の通常commit、通常push hook一回、Draft review、全指摘回収とP0/P1修復・個別reply/resolve、Ready、必須CI、通常merge、Actions公開、全5asset/checksum・配布検証・Issue/OpenSpec更新と安全な後処理を行う。元の品質条件を維持し、本人の実機確認は公開後の任意フィードバックとして分離する。

## Open Questions

約100KBの30秒遅延とHTML/Office closeの支配的な時間は未確定。検証結果に応じて設計へ原因と最小差分を追記する。
