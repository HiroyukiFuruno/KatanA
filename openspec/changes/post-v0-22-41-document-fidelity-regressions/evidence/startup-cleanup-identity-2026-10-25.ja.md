# 起動後処理の重複identity確認

## 実失敗と診断

31fc8817の通常push45111は、preflightのtest-startup-evidence-retention.shでFAIL: startup cleanup exceeded its bounded termination graceとなり、最終exit1。ログはtmp/oct25-31fc8817-normal-push.log。

既存テストをbashの実行trace付きで1回だけ診断し、exit0だった。元の時間超過は未再現。tmp/oct25-startup-cleanup-diagnostic.log。traceのcleanup_startedは開始カウンタであり、elapsedそのものではない。PS4の秒値も全行同じため、個々のlsof所要時間はこのログから算出できない。

一方、同一PIDへのlsof・実行ファイルpath・SHA確認が直後に重複していることは実traceで確認した。terminate_verified_mainの外側で確認した後、startup_terminate_processでもTERM直前に同じ確認を行っている。

## 最小修復と安全境界

外側の重複だけを除く。startup_terminate_processによるTERM直前の完全なidentity確認と、猶予後KILL直前の再確認を維持する。確認に失敗したPIDへsignalを送らず、所有権・path・SHA・PID再利用の保護を弱めない。

2秒の終了待機、既存テストの3秒/6秒上限、実終了・証跡保持・無関係プロセスの生存条件は維持する。実cleanupのtraceで、通常終了はTERM前の1回、TERM無視はTERM/KILL前の2回という確認protocolを回帰する。identity関数のmockや検査の省略で時間を短縮しない。

## 修復後の検証とレビュー

修復前の新しい回帰は、通常終了ケースで実照合2回・期待1回となって失敗した。修復後はverified/discoverが1回、direct_ignoring/launcher_ignoringが2回で成功。identity関数やsignalをmockせず、同じ親shellでの実cleanupをtraceして確認した。

途中のtrace実装に動的FDとBASH_XTRACEFDへの新しい依存があり、独立レビューでBash 3.2非互換のP1を検出した。同じ親shellのbrace groupへstderrを保存する方式に修復し、固定FDの上書きも除いた。元PS4/xtrace状態の復元を検証する。途中のmarker不一致によるcount0の失敗も修復記録へ残した。

標準/bin/bash 3.2.57で構文検査・retentionテスト・process identity safetyテストが実exit0。4ケースの照合回数は1/1/2/2、SECONDS差分は1/0/3/5だった。通常PATHのbashでも同テストを主査側の別担当が実行しexit0、実バージョンは同じ3.2.57で、差分0/0/2/6。別のmodern Bashでの検証と偽らない。上限ちょうどのケースもあり、元の超過原因の確定や、全負荷で余裕があるという保証ではない。

実ログはtmp/oct25-startup-identity-portable-*.log、tmp/oct25-startup-identity-path-bash-final.log、修復記録はtmp/oct25-startup-cleanup-identity-fix.md。別の3回の実identity測定は約0.088から0.093秒だったが、shell起動等を含むためlsof単体の計測ではない。tmp/oct25-packaged-identity-latency.jsonに保存した。

最終独立レビューは実2SHA一致・P0/P1なしで、tmp/oct25-startup-cleanup-identity-final-independent-review.mdへ保存した。限定削除は保護を維持する。対象検証の成功と、未実行の通常push・新HEAD CI・公開は分けて回収する。
