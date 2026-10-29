# v0.22.43 公開実績

[GitHub Release](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.43) は2026-10-29 15:09:16 UTC（10月30日00:09:16 JST）に公開された。本人が更新した両lockを含む。公開tag/sourceは `0d139094a39887a7a5657ecdd63f53a6db012fde`、レビュー・必須CIを通したPR391 headは `95ce239cab09b5c9991af2c7daec507395f42b72`。commit identityを混同せず、同じGit treeを確認した。

## 製品と配布物

PR391の最新レビュー全ページ・未解決0を確認して通常merge。H95のCI40867263464全8jobsとReadiness40867263477は成功。元のWindows200ms描画条件、メモリ予算、coverage、各OSの検査条件を維持した。Release40881935779 attempt1の実checkoutはM2であり、Preflight・3OS build/package・macOS両architecture smoke・Publishの7必須jobsが成功した。

Linux、Windows、macOS Intel/Apple Siliconのclean-runner起動は同run artifact内の本体hashと照合済み。これは記録された自動検証であり、本人の実機確認・Office全体のChrome互換性を証明するものではない。

全5配布物（macOS ZIP/DMG、Linux tar.gz、Windows ZIP/MSI）とchecksums.txtを公開URLから取得。全5checksum、同attempt artifactとの全byte一致、既存3archive契約を確認した。6ファイルすべてのSLSA署名はsource M2、workflow、run40881935779/attempts/1、各digestを検証した。付属JSON/logはその実結果であり、v43で生成していない旧版3manifestを生成済みとしない。

## 公開後の失敗と復旧

Release全workflowの結論は **failure**。製品公開後のcleanupだけがsuffix付きrelease branchを認識せず、削除実行前に拒否されたためである。製品7jobs成功を全workflow成功へ言い換えない。対象branchは後でremoteに存在しないことを確認し、手元の不要3branchはM2への包含を確認して通常削除した。master未編集、stash0、sole worktreeを維持。現在の記録用branchは必要なため保持する。

最初のHomebrew/Linuxbrew同期はfailedのまま保存した。その後tap Issue5・DraftPR7のレビューを経て通常merge `edb377bcd697faf988c92e308f14340defe1f41c`。公開masterの4metadataと実Ruby/Cask/Formula評価を確認して復旧した。以前からの任意brew style違反6件はtap Issue6へ分離し、成功と偽らない。WinGetは付属JSONの観測時点を参照し、PR提出とcatalog採用を区別する。

CI資格情報の失敗理由は元の診断出力が捨てられており未確定。値や生レスポンスを出さない診断をIssue395で追跡する。個人にしかできない認証は非停止の本人フォローアップへ分離する。

## 既知の未解決

元PNG約30秒/Missing表示の全原因（#385）、同期Office/HTML読込中のclose遅延（#386、上流KDV70）は未解決を含む。今回修正したqueue再試行・共有UI制御・タブpopup・下部画像操作等の検証結果を、原報告全体の全面解消へ拡大しない。固定registry KDV0.5.12/KRR0.4.23の変換遅延・Chrome互換性・再起動永続artifact不足は今回の全面解消を主張しない。本人確認をIssue Close条件へ追加しない。

## Issue・仕様・ローカル整理

Issue357/376/381/382/383/384/387/392/394は固有DoDの実装・自動検証・M2公開確認に基づきClosed。ボード388のcomment6502005467で未解決385/386と後続362/395を分離した。canonical5仕様を同期し、2026-10-30付でchangeをarchiveした。必要な現在の記録branchを保持し、不要なlocal3branchのみ包含確認後通常削除済み。リモートbranchの手動削除は行っていない。
