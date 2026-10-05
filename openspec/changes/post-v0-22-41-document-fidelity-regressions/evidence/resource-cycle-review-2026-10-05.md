# 配布mainの資源サイクル受入検査

## 対象と欠落

PR346、HEAD77e53a89のreview5413776715、P1コメント4183452293
（thread `PRRT_kwDORm09y86pAtOT`）を対象とする。旧validatorは一回の
Office close後RSSだけを確認し、元resource-lifecycle specの十回混合切替、
旧世代close完了、worker/frame/texture/cacheのidle復帰を必須にしていなかった。

この修正は公開ゲートの欠落を塞ぐものであり、実配布mainによる入力操作・
正常close・表示品質の受入が完了したという意味ではない。

## 検査契約

- 全Office原本の各packaged_runに、実測サイクルのJSON artifactのpath/SHAを要求する。
- run ID、target、main/sidecar/入力SHAを同じpackaged_runへ束縛する。
- 十回以上の各cycleに原本HTMLと当該Office、session ID/世代、正常closeの所要時間、
  close後のRSS・physical footprint・資源カウンタを要求する。
- 開いた世代を列挙し、終了世代と一致させる。追加の旧世代があれば、その終了も必要とする。
  別sessionの同じ世代番号は許可し、同じsession/世代の証跡使い回しは拒否する。
- 元の正常close5000ms、cold RSS増分196608KiB、warm増分65536KiBを維持する。
  physical footprintは実測を記録するが、RSS上限を無根拠にその追加上限へ転用しない。
- 最終集約値は最後のclose実測値と一致し、最終worker/frame/texture/cacheは0とする。
  fixture/identity違い、欠落、非有限値、数値としてのbool、残留資源は拒否する。

## 検証中の記録

初稿をmainが既存専用Python環境で独立検査すると43件中1件が失敗した。
既存cold RSS上限ちょうどの正例が、サイクルartifactを同期せずrunの値だけ変更していた。
この失敗を成功扱いせず、実artifactのbyte/SHAと実測値を同期する回帰へ修正して再検証する。
原ログ: `tmp/resource-cycle-main-first-verification-2026-10-05.log`。

旧公開HEADのvalidatorをメモリ上で読み込む独立比較では、サイクルartifactを
削除しても旧検査が通り、新検査が拒否するREDを確認した。
原ログ: `tmp/resource-cycle-main-old-red-2026-10-05.log`。
初稿修正後の独立実行は46件とrelease gate17件が成功した。
原ログ: `tmp/resource-cycle-main-green-2026-10-05.log`。

追加した境界正例の195MiBは元cold上限192MiBを超えており、検査が正しく拒否した。
正例を191MiBへ訂正し、元上限ちょうど192MiB+warm64MiBも追加した。
その後48件のacceptance検査は成功したが、続くrelease gateはhelper分割編集中の
ファイルを読んで失敗したため、組み合わせた最終成功とは扱わない。
変更が止まった最終ソースで両suiteを再実行する。
原ログ: `tmp/resource-cycle-main-boundary-2026-10-05.log`、
`tmp/resource-cycle-main-corrected-green-2026-10-05.log`。

UT用のsynthetic artifactはvalidatorの試験入力だけに使用する。
実配布受入receiptには使用せず、PNGデコードの省略やfake完了値の補完は行わない。
正式push、個別reply/resolve、fresh review取得と実配布受入はまだ未完了である。

## 最終ソースの独立検証

責務別helperへ分割し、新規validator関数は最大21行、UT用artifact helperは15行。
統合作業中の重複helperを取り除き、単一定義を確認してからmainが両suiteを再実行した。
acceptance48件とrelease gate17件、計65件がexit0。Python構文検査とdiff checkも成功。
原ログ: `tmp/resource-cycle-root-stable-final-2026-10-05.log`。
通常preflightもexit0（`tmp/resource-cycle-pr-ready-2026-10-05.log`）。
自己レビューは入力境界・caller・世代使い回し・cold/warm別予算を確認しPASS。
Rust製品コード・原本・受入基準は変更していない。
