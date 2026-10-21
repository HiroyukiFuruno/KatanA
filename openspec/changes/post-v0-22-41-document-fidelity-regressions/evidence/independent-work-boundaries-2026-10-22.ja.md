# 独立作業と実施条件待ちの区別

## 確認対象

実装HEADは `82959ddb5d7adc71428d59ccb04b09d4381195ab`、確認時docs HEADは
`26278d3795aa178d3b4426fdd7468475db9d9806`。Open Issue全14件をライブ取得し、
元DoD・現source・既存回帰と照合した。ラベルは実装済み9件、部分対応4件、未実装1件。
新しい即実装可能な欠落は確認できなかった。全Issue完了という意味ではない。

## 残件と次の実行条件

| 対象 | 未完了の内容 | 実行条件・根拠 |
| --- | --- | --- |
| #343 Windows | compiled target cache導入判断、cold/warm・導入前後の実測 | 実Windowsでの検証が必要。Draftでは重CIを起動せず、公開sourceのレビュー後Readyで実行する |
| #343 重複削減/reuse | 実行同等性の判定、証明できる範囲の集約、信頼できる結果の再利用 | 保護CIの実run・環境・artifact証拠が必要。未証明なら既存検査を維持する元DoDは不変 |
| #335 | 実運用activation、published OIDでのcleanup receipt | `build-and-release.yml` のcleanupはpublish成功後。公開前のfixtureを実公開receiptへ代用しない |
| #338 | canonical採点・参照採用 | KDV所有のcrop manifest/PNG/full PNG/geometryと正規crop score結果が必要。候補生成・handoffは実施済み |
| #339 | 安全な公開registry採用、current packaged受入 | KDV #69の供給網修正版を公開後に採用。未公開path/git overrideで代用しない |
| #336 | KLE公開版採用・ABI整合・実editor host E2E | KLEの公開版が必要。rootで上流を重複実装しない |
| 実装済み9件 | 新HEADのreview/required CI、全OS配布物・最終受入 | 通常push後のDraft review→Ready→元必須CI→全配布受入を維持 |
| 本人受入 | 本人が行う実機確認・Terms | 本人操作。アプリや承認設定を代行しない |

## sourceとの照合

- Windows registry cacheは `test-and-build.yml` のtest/lint双方に実装済み。
  compiled targetはWindowsでは共有せず、cache hitでtestをskipしない。
  `test-windows-registry-cache-contract.py` がこの境界を検証する。
  registry cacheとinventoryの成功をtarget cache/cold-warm成功とは扱わない。
- `scripts/ci/observe-ci-command.py` は実command時間・元exit・run/source/lockを保存する。
  `build_cache_state=not_measured` / `cold_warm_equivalence=not_evaluated` を保持する。
- `ci_run_admission.py` / `verify-version-bump-ci.sh` は同一SHA・repo・固定workflow・
  全3OSの最新job成功を照合する。待機の受入をtest skip/reuse許可へ昇格しない。
- Issue #335のsetup/rollbackとpublish後cleanup接続はsource・実fixtureで検証済み。
  Issue #338のproducerはsource identity/clean capture/geometry/hashを生成するが、
  自らの候補をcanonicalや95点達成と判定しない。

## 検証の扱い

今回の再照合でsource・workflow・検査条件は変更していない。
既存の545件・318.076秒の成功は実装82959ddbの回帰結果として保持し、
ドキュメント変更だけで同じ重い検査を繰り返さない。
対象Markdownとdiffの検証を行い、元の未完DoD・Issueのpartial状態は維持する。
「今すぐ独立実行できる作業がない」と「上流以外の全DoDが完了」は区別する。
