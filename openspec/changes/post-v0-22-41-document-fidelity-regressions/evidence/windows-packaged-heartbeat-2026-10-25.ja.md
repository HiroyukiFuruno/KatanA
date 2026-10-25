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
