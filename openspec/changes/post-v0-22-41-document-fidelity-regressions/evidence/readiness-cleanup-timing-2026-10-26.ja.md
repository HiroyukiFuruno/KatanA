# Readinessの終了処理回帰: 時間超過の内訳を保存する

## 実際の不合格

source `1517014a5680ff04e891f1461402933959cacc1c` の[Readiness run40241669703](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40241669703)、job121314586274は、実プロセスを使う終了処理回帰で不合格になった。

`launcher_ignoring` は `start=5 end=12 actual_delta=7` となり、既存の6秒上限を超えた。先行する `verified` / `discover` / `direct_ignoring` はそれぞれ0 / 0 / 2秒、identity検査は1 / 1 / 2回で成功した。時間超過の判定は後段のidentity回数・PID消失等のassertより前なので、不合格ケースの安全条件が全部成功したとは主張しない。

rawログは `tmp/oct26-151-readiness-failure.log`、SHA-256 `38b5063aecd02e97e72ce024e75f81d2055571e115d81b7713b0867fde72c89f`。ログ行1965–1969に上記の実値がある。隣接する出力のwall時刻差にはfixture起動・子探索も含まれ、各処理の所要時間には代用できない。

これは共通startup cleanupの回帰であり、Rust UIやWindows配布版の実行結果ではない。既存のxtraceはrunnerの一時ディレクトリへ退避されるが、失敗時に内容を標準エラーへ出しておらず、元ログだけではidentity確認・終了猶予・PID回収・証跡移動のどこが長かったか確定できなかった。製品不具合・fixture不備・一時的な実行環境負荷のいずれかは未確定として残す。

## 最小の観測修正

`test-startup-evidence-retention.sh` のPS4へBash組込み `SECONDS` の行を追加し、各コマンドの時刻を同じtraceへ保存する。既存の `+STARTUP_CLEANUP_TRACE: ` 行は維持し、identity検査の正確な回数確認を変更しない。

6秒超過を判定した後、元のexit1より前にtraceを標準エラーへ出す。出力は計測終了後に行い、計測区間へ外部時計プロセスや追加のファイル読込を入れない。実traceに時刻行が存在することも確認する。

trace読取りが失敗した場合も警告を出し、元のFAIL理由とexit1を維持する。追加の診断が元の不合格を隠さないようにする。

製品・共通cleanup helper・2秒の個別猶予・6秒の合計条件・4ケース・identity再確認・無関係PIDの生存・実child/launcher消失・呼出側PS4/xtrace復元は変更しない。並列終了、再確認削減、閾値・timeout・ignoreの変更は行わない。

## 現時点の検証と限界

旧sourceの既存スクリプトをローカルで1回実行したbaselineはexit0、4ケースのdeltaは0 / 1 / 2 / 5秒だった。これはクラウドの7秒不合格を再現・解消した証明ではない。

修正後の同じ実プロセス回帰もexit0、delta0 / 1 / 2 / 5秒、identity1 / 1 / 2 / 2回で成功した。旧実trace4件には時刻行がなく、新実trace4件には時刻行があることを照合した。これは観測不足の修復確認であり、時間超過原因の修復確認ではない。

結果は `tmp/oct26-cleanup-timing-target.log`、旧新trace照合は `tmp/oct26-cleanup-clock-red-green.json`、原因診断は `tmp/oct26-151-readiness-termination-diagnosis.md`。新HEADの通常レビュー・必須CI・全OS配布起動と公開は引き続き確認する。再び超過した場合は保存される時刻付きtraceで内訳を判別し、必要な修復を行う。

通常PATHのBash5.3.20に加え、macOSの `/bin/bash` でも同じ既存回帰を実行してexit0を確認した。後者のdeltaは0 / 0 / 3 / 4秒、identity1 / 1 / 2 / 2回。ログは `tmp/oct26-cleanup-timing-bash32.log`。別の実行環境での成功も、クラウドでの原因確定や安定解消の証明にはしない。

同sourceのCI40241669711はSupply Chainと3OS lintが成功し、残るbuild/test・CodeQLはcancelledで終端した。Readinessのfailureとは区別し、全CI成功として流用しない。Draftへ戻した後のrun40242610466 / 40242610519はskippedであり、合格扱いしない。
