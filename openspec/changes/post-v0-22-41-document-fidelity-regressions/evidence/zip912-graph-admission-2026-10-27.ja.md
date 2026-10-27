# ZIP 9.1.2更新後の依存経路照合（#354）

## 実際の失敗

通常pushのsource `83818382` ではMac側の製品テストが成功した後、別の2件で不合格となった。

- Linux検査はDocker内部のblob/metadataのI/Oエラーにより起動できなかった。
- 公開前検査は、本人が更新した `zip 9.1.2 → katana-core 0.22.42` を未承認の辺として拒否した。

後者は、既存検査のmodern ZIPの固定リストが9.0.5だけだったためである。
旧ZIPの問題APIを新たに使うようになったことや、新規advisoryを確認したことを意味しない。

## 最小の整合

既存の9.0.5を保持し、今回の実lockにある9.1.2とkatana-core 0.22.42の組だけを追加する。
modern ZIPの親は、正規リポジトリ内のkatana-core manifestであることと、
workspace rootsから到達できることも照合し、同名同版のgit/別path/孤立packageを認めない。
撮影ツールではcoreがworkspace memberではなくpath依存であるため、正当な到達経路を維持する。

旧ZIP0.6.6等の例外ID、期限、registry確認、KDV経由・迂回拒否は変更しない。
別版や別callerを一括許可せず、新しいadvisoryのignoreも追加しない。

## 照合結果の扱い

現在の両lockを使ったroot・撮影ツール・paint-metricsのlocked offline metadataを照合する。
rootと撮影ツールの同じ実graphは旧関数で9.1.2の辺を拒否し、修正関数で受理する。
ZIP辺のないpaint-metricsも既存のZstd検査と合わせて照合する。
ZIPの回帰27件、Zstdの回帰14件も成功した。
詳細はignoredの `tmp/oct27-zip912-admission/production-validation.json` に保存する。

これはcargo-denyの新しいadvisory DBによる検査の代替ではない。
通常pushの正式な供給網入口、新HEADのレビュー・CI・配布検査・公開は実結果で確認する。

## Docker検査環境の復旧

containerの実processは終了済みで、実runtimeのtask一覧が空であることを確認してから、
既存のColimaを設定・保護・データvolumeを保持したまま再起動した。
同じ検査用imageを再び読め、元のcompose入口でrustc 1.95.0を実行してexit0を確認した。
再起動による回復を、I/Oエラーの根本原因確定やLinux全検査成功とは扱わない。
