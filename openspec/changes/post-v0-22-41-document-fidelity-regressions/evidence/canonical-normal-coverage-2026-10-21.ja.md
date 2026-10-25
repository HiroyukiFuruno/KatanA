# 通常testと正式coverageの同HEAD実測

## 条件と結果

- 対象は既存release worktreeの`fa9a71b5430e66e7c555dafd57052a172f23ca5f`。両工程の開始・終了でtracked clean、両Cargo.lockと共有Git設定ファイルのSHA不変を確認した。
- 既存Justfileと同じDEV/TEST DEBUG=0・STRIP=none、incremental=0、jobs/test threads=1、RUSTFLAGS=-D warningsを明示。同じ専用絶対targetを使用した。coverageの計測用環境差は比較JSONへ残した。
- normalは既存Office worker wrapper経由の`cargo test --workspace`を1回実行。session89740、UTC2026-10-20T16:27:13Zから16:43:28Z、actual exit 0。
- normalのmanifest、raw、入力記録をtarget外へ保存し、既存保護`just clean`で専用targetだけを回収した。session96731、actual exit 0、6545files/7.6GiB。保存SHAは前後一致。旧untagged root targetは変更していない。
- 既存`just coverage`を全4工程・既存report・strict document surface gateまで実行。session95104、UTC16:48:07Zから17:14:34Z、actual exit 0。strict document surfaceは100%・未実行行0。閾値、除外、skip、featureは変更していない。

## 比較結果と時間の範囲

既存compare CLIはactual exit 0。固定4工程の欠落・不正selectorはなく、選択集合比較が成立した。coverage各工程の候補は1770/228/144/18、6組の工程間intersectionはすべて0。通常との共通候補2160、通常だけ10、coverageだけ0であり、候補数を実行回数や再利用可能件数とは扱わない。

| 工程 | Cargo command秒 | 後処理診断秒 | exit |
| --- | ---: | ---: | ---: |
| normal | 485.801 | 6.351 | 0 |
| coverage lib/bin | 562.060 | 1.049 | 0 |
| coverage integration | 470.256 | 2.256 | 0 |
| coverage UI parallel | 52.852 | 0.441 | 0 |
| coverage UI serial | 9.019 | 0.432 | 0 |

coverage4工程のcommand合計は1094.187秒、診断合計4.178秒。これらはworkerの事前build、coverage setup/report、JSON保存、runner/CI待機を含まない。全job時間、cold/warm比較、短縮量には転用しない。

## 証跡と未完了範囲

- `tmp/oct21-normal-fa9-canonical.log`、`tmp/oct21-normal-fa9-canonical-metadata.md`、`tmp/oct21-canonical-main-identity-observation.md`
- `tmp/oct21-canonical-clean.log`、`tmp/oct21-coverage-fa9-canonical.log`、`tmp/oct21-canonical-comparison.log`
- `tmp/test-inventory/matched-fa9a71b5-20261021-canonical/normal.json`、`coverage/`の4manifest、`comparison.json`
- normal manifest SHA: `9bb260980abb1fefb9da29da872e508493f7d6be55dae3b17d59a1e1ef35eed5`。他入力SHAはcomparisonの`input_sha256`へ保存。
- 検査後も10証跡をtarget外へSHA保存し、専用cacheの既存保護cleanで13533files/11.4GiBを回収、全SHA一致・空き16GiBを主査確認。session78527のclean本体は0、終了記録wrapperはzsh予約変数代入により1であり、wrapper全成功とは扱わない。rawは`tmp/oct21-canonical-clean-after-coverage.log`、SHA一覧は`tmp/oct21-canonical-preserved-sha.txt`。旧rootは保持。

author-writableな診断でありCI由来の証明ではない。doctest一覧未捕捉、選択環境のみの記録、一覧取得時のbinary SHAという制限がある。全manifestのcomplete/reuseと比較のexecution_equivalent/reuse_authorizedはfalseのまま。通常だけの候補10件と計測環境・artifact差も保持し、coverageによる通常test代替やtrusted reuseを承認しない。

UI serialで記録されたworker SHAも他工程と異なる。記録は各工程前の観測であり、全工程のworker bytes固定や実行中の不変性まで証明しない。小さなfixtureでの先行観測を製品全体へ一般化せず、この差の原因と固定方法は別途切り分ける。

Issue #343のWindows実機cold/warm、欠落のない実行同等性、実重複削減、trusted verifier、導入前後総時間は未完了。全OS・packaged受入・current公開review・リリース完了もこのMac検査から推定しない。
