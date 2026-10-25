# #343 coverage工程間の候補重複診断

## 修復した独立未実装

従来の比較はcoverage各工程の候補集合をunionへ集約しており、工程間の重複候補を確認できなかった。各工程のunique candidate数と工程pairのintersectionを比較JSON/既存CI summaryへ追加した。

候補集合は実行回数、同等性、削減量ではない。元のnormal対coverage union、全test、argv、exit、hash/selection検査を維持し、execution_equivalent/reuse_authorizedはfalseのまま。Windows実測やtrusted reuseの完了ではない。

source不一致、重複run ID、不正record、固定4工程の欠落/selector不一致では新metricsをnull/unavailableにする。欠測を0件に置き換えない。

## 検証とレビュー

- 新field未実装のRED: `tmp/oct19-343-phase-overlap-red.log`。
- 正overlap/disjoint/union維持、source不一致・duplicate ID、実Cargo fixtureからの正常4工程CLIとselectorのみ/欠落のみの段階変異を確認。CLIのmetrics無効化2行を一時外すと実際にFAILした: `tmp/oct19-343-phase-overlap-reset-red.log`、actual exit1。製品差分は復元した。
- 関連focused群28件/7.002秒/actual0: `tmp/oct19-343-phase-overlap-related-final3.log`。
- 主査と独立レビューで42行のtest関数を検出し、fixture生成一回を維持してhelper分離。最終実CLI回帰1件/6.690秒/actual0: `tmp/oct19-343-phase-cli-helper-final.log`。
- 主査AST実測: 実装119/150行、tests115/127/154行。全関数30行以下、構文・diff検査成功。独立最終レビューのP0/P1/必須P2は0件。

初回の正overlap・CLI回帰不足と、その後の不正baselineでは拒否処理を検証できない不足を修復した。初回関連検証で全phase候補数が正という過剰な期待も失敗し、testを持たないphaseは0となるfixture契約へ訂正した。新しい候補指標を実検査の削減成功と扱わない。

## 残る受入

現行最終graphのWindows実取得・cold/warm、normal/coverageの完全な実行同等性、実重複削減、trusted reuse、導入前後総時間は未完。次の判断にcurrent Ready CIの実記録が必要であり、旧CIや今回の小fixtureから推定しない。安全性修正版のregistry採用後、通常push/current reviewと元の全品質条件を通す。Draft CI起動、hook迂回、上流実装、本人アプリ操作は行っていない。
