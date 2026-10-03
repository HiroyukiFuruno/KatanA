# 2026-09-13 公開監査と独立作業

## ライブ確認

- GitHub latest Release / crates.io max_version は KUC 0.3.10、KDV 0.5.5、KRR 0.4.19。
- KUC v0.3.11 / KDV v0.5.6 / KRR v0.4.20 の GitHub tag ref はすべて404。
- KUC #35はClosed、後続のtypography #52はOpen。既存担当が次版の検証中。
- KDV #50はDraft、remote HEAD `3b4ecb13396c8f067e838cacb2a13087dd88309d`。remote required preflightは失敗、3OS buildは成功。未コミットの担当側修正の結果とは区別する。
- KRR #77はDraft、remote HEAD `bfcbd5243f243cebd4da832919e53cadac1f789b`。表示された4OS CIとpreflightは成功。#73/#74/#76はOpen。担当は最新reviewのP2修正中と報告。
- KatanA latest Releaseはv0.22.41。既存Draft #341にはRelease Readiness/3OS build失敗が残る。現在worktreeの未公開修正が通過したことを意味しない。

上流の次版を先行採用していない。既存担当へ公開時の証跡引継ぎを依頼し、別repoは編集していない。

## KatanA独立作業

4.13の配布物同一性を既存packaged smokeに追加中。in-process screenshotを配布物受入へ読み替えない。
Windowsは展開exeのパスとSHA-256を起動前に記録し、実PIDのGet-Process.Pathとhashをfirst-frame後・検証終了前に照合する。
既存heartbeat/RSS/font閾値は維持。Windows用PowerShellの実行環境はこのホストにないため、構文/静的契約確認とWindows実行済みを区別する。

- 変更後の `test-packaged-startup-contract.sh` 成功（7 architecture tests、heartbeat負例含む）。
- workflow YAML構文検査、scoped diff check成功。
- Linux/macOS用helperを実装し、実システムsleep子プロセスでwrapper descendant探索、path/hash一致、誤path/hash・終了済みPID拒否が成功。main再レビューで物理親path解決、空PID拒否、子孫終了待ちも追加。
- first-frame後と検証終了前に同一PID/path/hashを再照合する。Unixの成功ログとWindowsのCI出力にexe/path/hash/PIDを記録する。既存アプリログは一時ファイルのため永続証跡はCI出力を参照する。
- 最終registry-only配布物のclean-machine/3OS実行、Office/HTML入力受入は未完了。
