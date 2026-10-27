# 配布起動検査の共有終了猶予（Issue #371）

対象: https://github.com/HiroyukiFuruno/KatanA/issues/371

## 問題と修正

旧HEAD1c387c7eのReadiness40467962993/job122067856092は、所有mainとlauncherを直列に終了させる処理で7秒かかり、元の6秒条件に失敗した。アプリの描画失敗とは別の検査終了処理の不具合である。元実装のローカル成功はCI失敗を否定しない。

検証済みmainと所有launcherへ先にTERMを送り、既存の2秒猶予を共有する。残存する対象は同じcheckerで再確認してからKILLし、次の2秒待ちも共有する。同一PIDの信号重複を除去する。main単独caller、証跡保持、非対象PID保護、元6秒/各2秒の基準を維持する。

## 検証

- 最終Mac Bash3.2実プロセス5ケース成功（10.804秒）。cleanup単体は通常0秒、TERM無視3ケース各2秒。全identity再確認・無関係PID生存・証跡/設定保持が成功。
- 同じ最終testと旧helperでは元4ケース成功後、新しい親TERM時点の実child生存検査がdead-mainで不合格（14.289秒）。直列実装を確実に検出する。
- dedupだけ削除したmutationはTERM2回を検出して不合格。KILL前のidentity再確認だけ削除したmutationは1回/期待2回で不合格。
- 変更2ファイルのShellCheck -xとBash syntax成功。変更しないOffice caller全体のShellCheckは従来の動的source警告SC1090が残るため、その全体成功とはしない。実caller回帰は別に回収する。
- 独立静的レビューP0/P1/P2なし。Linux実5ケースとMac Office restore callerは最終sourceでともにexit0。Linux cleanupは2/0/2/2/2秒、identity1/1/2/2/2回、元6秒・証跡保持成功。前後source hash一致。`tmp/oct27-startup-cleanup-final-linux-caller.json`参照。

## 証跡と境界

実行log/実exit/秒数/最終source SHAは `tmp/oct27-startup-cleanup-final-mutations.json`、静的検査は `tmp/oct27-startup-cleanup-final-static.json`、独立レビューは `tmp/oct27-startup-cleanup-batch-final-review.md`。ソースの最終hash:

- `scripts/release/startup-process-cleanup.sh`: `af5b1c7b9561fbee4ba1ec824574451a0fe597d11a50b37fe77619b49fafe2b4`
- `scripts/release/test-startup-evidence-retention.sh`: `23e311168b24d1ecebb687f57e0fec88b0d083ea713b2233347c83a4132fec20`

新HEADの通常commit/push・current review・全CI・実配布検査・公開はまだ未完である。旧CIを新HEAD成功へ流用しない。
