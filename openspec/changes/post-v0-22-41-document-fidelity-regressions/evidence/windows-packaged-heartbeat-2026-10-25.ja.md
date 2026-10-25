# Windows配布起動の画面更新記録

## 実際の不合格

[公開run40192198622 / Windows job121167342877](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40192198622/job/121167342877) はビルド・パッケージ・identity生成に成功した後、配布起動検査で `last_frame=8; current_frame=8`、exit1となった。全decodedログは `tmp/oct25-canonical-windows-package-failure.log`。この時点ではプロセスは生存していた。

元runはLinuxも失敗、macOS両archのnative smokeは成功、Publish Releaseはskippedだった。旧sourceの成功を修正後sourceへ流用しない。[Issue364](https://github.com/HiroyukiFuruno/KatanA/issues/364)。

## 確認できた観測上の欠陥

UI完了時のheartbeat記録は100ms未満の呼出しを間引いており、実際にUIフレームが完了してもカウンタが進まない。eguiの再描画予約は予測フレーム時間を差し引くため、100msの再描画要求と100msの書込み間隔は同じイベントを表さない。

診断用環境変数が有効なときだけ、既存カウンタをUI完了の各呼出しで記録する。通常起動の冒頭return、100ms再描画要求、Windowsの200ms間隔×10回観測、RSS/font/worker/identityの各判定は維持する。Windowsの不合格例外へ、既存stderrログも添える。

## 検証の境界

プロセス分離した実ファイルへの連続記録を回帰検査する。間引きがWindowsの実不合格を起こした原因だったかは未確定であり、修正後のnative配布起動検査で別途確認する。判定のtimeout延長や閾値変更で不合格を消さない。

## 対象検査の結果

先に新しい実ファイル・独立子プロセス回帰だけを旧実装へ追加し、既存 `just test-specific` でexit101を回収した。実子のファイルは2回目も `frame=1`、期待値 `frame=2` と不一致（`tmp/oct25-heartbeat-red-raw.log:177-180`）。その後、間引き状態を除去すると `startup_heartbeat` filterは4件成功/exit0、`just fmt-check` もexit0。旧間引き専用の純粋テストは、要求する実フレーム記録の回帰へ置換した。

この4件には既存のTerms modal表示中のheartbeat回帰も含む。Termsへの同意操作や通常利用profileの変更は行っていない。新Windows native配布起動、current review/CI、公開は未完であり、対象検査の成功と区別する。

追加対象検証はTerms modal単独1件、既存packaged startup契約、AST30件、fmt、impacted lint、diff checkすべてexit0。独立最終レビューは実REDの子assertと最終source2件のSHA一致を確認し、P0/P1なし。heartbeat `09f5d8bf2b95edc36db2a1764ed03e3bfa02543e19d5607738c8fb727661861e`、workflow `126a0d4dc62947a684f19e187a434cec3f5ceed9b31cfb9b1d93aa253d45aa0e`。結果詳細は `tmp/oct25-startup-heartbeat-completed-frames-result.ja.md`。

## 新HEAD CIでのカバレッジ修復

f14de30dの[macOS CI job121184046310](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40198372205/job/121184046310) は通常テスト・撮影契約・複数形式受入後、coverageで新回帰テストの診断引数2行（117/118、stdout/stderrのString::from_utf8_lossy）が未実行となり失敗した。これはassert失敗時だけ評価する引数で、同ファイルの先行テストはlet stdout/let stderrで先に評価する既存形式を使っている。同じ形式へ揃え、失敗時の出力情報と検査assertを維持する。商用処理、coverage checker/exclude、閾値、timeoutは変更しない。

失敗を無視してmergeせずPR365をDraftへ戻した。converted_to_draftによるconcurrency取消で旧runはcancelled、残Ubuntu/Windowsはjob全体の成功ではない。旧Readiness・Lint・CodeQL等の成功を新HEADへ流用しない。修復後の対象検査・通常commit/push・current review・新Ready CIで再確認する。raw `tmp/oct25-f14de30d-macos-coverage-failure.log`。

修復後の対象子回帰1件/AST30件/fmt/diffは実exit0。独立最終レビューはsource SHA `8f1c420865fef717ed8f0c6cac40be6495caa5bc9394b3a68135913ec8c1208a`、workflow不変SHA `126a0d4dc62947a684f19e187a434cec3f5ceed9b31cfb9b1d93aa253d45aa0e` でP0/P1なしを確認。子失敗assertと実frame1→2の比較、商用コード・品質ゲートは維持される。完全coverageは新CIの実結果で確認する。

## 新CI成功後のWindows実配布再検証

`d352db0d` の[CI40204301916](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40204301916)は全8ジョブ成功、[Readiness40204301912](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40204301912)も成功。macOSのstrict文書coverageは100%・未実行0行、coverage gateも成功した。全レビューの未解決0を再取得して[PR365](https://github.com/HiroyukiFuruno/KatanA/pull/365)を通常mergeした。source `5cb787cf7c8a4d10101cd0f823b0aa574080a20a` は元headと完全Git treeが一致する。

そのsourceの[全OS公開run40212332637 / Windows job121228922985](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40212332637/job/121228922985)は、build・zip/MSI構成・identity生成後、既存200ms観測で `last_frame=7; current_frame=7` / exit1だった。追加したstderrには字体9件・owned55825700bytesの記録だけがあった。raw `tmp/oct26-canonical-windows-frame7-failure.log`。100msの記録間引き修復だけでnativeの不合格は解消しておらず、恒久的なUI停止か、一時停止・観測の問題かをまだ確定しない。

同runのmacOS build・arm64/x86_64 native起動は成功、Linux/Windowsは失敗、Publish Release・cleanupはskipped。tag/Releaseが404のままなのを再確認した。macOSの成功をWindowsや次sourceの成功へ流用しない。

## 次の原因判定に必要な観測

mainのtracing subscriberは標準出力へ書く一方、Windows起動stepは標準エラーだけを保存していた。そのため再描画の既存traceやheartbeat書込みエラーを判断できる観測が不足している。製品を変更せず、別のfresh fileへstdoutを採取し、当該stepだけの既存trace・失敗後のraw heartbeatと同PIDの状態を保存する。元の例外・200ms×10回・RSS/font/worker/identity・全OS公開条件は維持し、診断採取の成功だけをnative成功にはしない。

ソースでは100ms再描画予約がTerms分岐より前にあり、単純なTerms早期returnによる予約漏れではない。PowerShell公式readerはReadWrite共有を要求し、排他的readerや内容キャッシュが原因だと決めつけない。updater結果の設定保存はUI内の同期JSON/write経路を通るが、今回その処理が発火して200msを超えたかは未観測である。各仮説を次の実nativeログで判別し、推測による待機延長やbusy repaintで不合格を消さない。

限定診断は公開workflowと既存契約検査だけに実装した。元workflowのstdout未採取は局所検査でexit1、修正後はstartup契約exit0、全OS公開契約4件/exit0、bash構文・diff検査exit0。診断の任意状態取得が失敗してもstdout/stderrの採取へ進め、元の失敗を再throwする。製品・元の測定ループ・判定は変更していない。

抽出したPowerShell本文は公式image `mcr.microsoft.com/powershell@sha256:73c08403182e3cd1a62176b6723645b2d2037cda8deefc0e2c2c01cb814abe43` のLinux/amd64 PowerShell7.4.2で `Parser.ParseFile` が構文有効・exit0。これはLinux上の構文検証で、実Windows起動の代用ではない。抽出本文SHA256は `b971aa04faba93ea546dca65984dcba2bed7e7d25023401db09115563ccbe70a`、結果は `tmp/oct26-windows-startup-diagnostics-result.md` と各実ログへ保存した。

対象source SHA256はworkflow `8cf51a91eed6768a793add3875a5d55c92af09c7700f1fbfaff8b63f6249f875`、startup契約 `6894089f9136903ec2b623d95d6e592b314a69bd4c415908e4fc7bacd4058994`。新current review/CI、実原因の確定・必要な修復・新native成功・公開は未完であり、この診断追加だけでIssue364を解決済みにしない。
