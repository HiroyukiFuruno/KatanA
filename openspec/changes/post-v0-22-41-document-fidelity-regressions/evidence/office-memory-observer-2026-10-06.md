# Office終了後メモリの診断observer

## 目的と非完了条件

旧5f9の通常原本Office5受入は2回とも元RSS上限196608KiBを超えて失敗した。idle資源counter0は漏れの原因も受入成功も証明しない。既存KDV #59担当からKatanA #339 comment5998964487へ引き継がれた、同じ親runner PIDのcold/各正常close直後のmalloc割当・VM内訳を取得する。診断追加やKDV0.5.12公開をRSS修正成功へ読み替えない。

## 実装境界

変更はscripts/screenshot/src/main.rs、executor_harness.rs、新memory_observer.rsのみ。製品API、描画、原request、正常close期限、元RSS/品質基準は不変。任意 --memory-diagnostics は既定falseで、通常実行では診断コマンドとファイル処理を行わない。enabledはmacOSだけ、rootをcanonicalizeし、std::process::id()の親自身に固定argv /usr/bin/vmmap -summary PID と /usr/bin/heap -s PIDを使用する。shellや外部PIDを受け付けない。

既存RecordRuntimeSnapshot、成功したCloseActiveDocument/CloseAllDocuments、最終RSS assertion前に接続する。stdout/stderrは直接ファイルへ送って親に大量bufferを保持しない。create_newで既存file/symlinkの上書きを拒否。poll中の合計8MiB/15秒安全上限でowned diagnostic childをkill+waitし、metadata/try_wait失敗でも回収する。時刻エラーを隠さない。manifestはdiagnostic_only、canonical root、step/phase、親PID、固定command/argv、時刻、exit status、timeout/出力超過を記録する。診断失敗は通常成功へfallbackしない。

## 主担当レビューと検証

初稿の50件成功だけでは、終了後のみの出力量判定、monitor失敗時のchild回収、実mac probe不足を証明できないため統合しなかった。既存childへ同じ編集境界で補正を依頼し、主担当が最終差分と実probe rawを照合した。補正後childは54件test/strict Clippy exit0を報告。主担当の公式CIと同じlocked release screenshot契約検査1164もexit0、54pass/失敗0/ignore0。rawはtmp/memory-observer-canonical-release-contracts-2026-10-06.log。主担当のrelease all-target strict Clippy68303とfmtもexit0、rawはtmp/memory-observer-canonical-clippy-2026-10-06.log。基準/除外/受入assertは変更していない。

実mac probeは親test PID72806だけを対象とし、両manifestで固定argv/exit0/diagnostic_only=true/上限未超過を確認。rawはlocal /private/var/folders/ql/4640yx8s22zg367pjjld7yc00000gn/T/katana-memory-observer-owned-probe-1791220721960284000に保存。これは検査processの観測能力だけを示し、原本Office runnerの保持内訳やRSS改善ではない。

KDV0.5.12固定採用後の公式実worker/HTML関連8件、host契約24件/実process-group7件、供給網4分類は成功。変更後の原本診断、通常flag-off受入、全gate、正式commit/push/current review、packaged全OS/独立各95点/公開後処理は未完了。

通常commit21579 exit0で依存差分000a2869、別commit63596 exit0でobserver9401a5b0を正式履歴へ統合した。rawはtmp/kdv0512-adoption-source-commit-2026-10-06.logとtmp/memory-observer-source-commit-2026-10-06.log。新release runner/workerを13063でlocked build中であり、旧binaryの原本結果を新sourceへ流用しない。通常push、新HEAD review、全gate/原本診断・受入・公開DoDは後続として継続する。
