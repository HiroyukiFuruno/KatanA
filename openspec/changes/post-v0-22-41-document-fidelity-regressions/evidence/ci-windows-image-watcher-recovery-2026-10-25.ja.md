# Windowsの画像監視エラーと局所復旧

## 一次情報

[Windows CIの実失敗](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40169880630/job/121098387421)は6494f8d3。UI1341件成功/1件失敗/既存ignore2、実exit101だった。native_directory_rename_detaches_exact_descendants_and_keeps_unrelated_watchで、rename後にOS error 2の監視エラーが返った。

コネクターで取得した元ログの該当部分はtmp/oct25-6494-windows-rename-failure-excerpt.logに保存した。gh apiでの全ログ保存は端末escape保護で拒否され、tmp/oct25-6494-windows-rename-failure.logはその拒否メッセージであり生ログではない。元の実ジョブURLを一次情報とする。

## 判別できたことと未確定事項

notify 8.2.0のerror.rsではパスがあればDisplayにaboutとパス一覧が付く。今回の実ログにはなく、パス情報が空であることは判別できる。Displayだけではerror.kindを厳密に確定できないため、Io(NotFound)と断定したり、新Windows CIでの成功を先に宣言したりしない。

現Backendのパス付きNotFound正規化は維持する。匿名エラーは現在Coordinatorで全監視先へ失敗通知される。実rename後の消失先だけを識別できれば、既存のDirectoryRecoveryへ局所的に戻せる。

## 実装の境界

匿名Io(NotFound)に限り、登録ディレクトリのmetadataがNotFoundになった対象だけRemoveへ正規化する。集合が空なら元のエラーを保持する。他のerror kind、パス付きエラー、権限エラー、全登録先が生存する場合を変換根拠にしない。

Coordinatorとnative rename回帰で同じ正規化を通し、既存のrecoverからnotify_eventへの順序を維持する。テストだけエラーを無視しない。元のworker・queue・timeout・ignore・品質条件と無関係な監視先の保護を維持する。

## 連続する匿名通知への追加修復

独立レビューで、最初の消失を処理して登録を解除した後、同じpollから後続の匿名エラーが届くと、全登録先が生存するため元Errが維持され、既存の全画像への失敗通知を起こすP1を確認した。単一通知の正規化だけを最終合格とはしない。

正規化関数は元Errを保持する。共通dispatcherではパス情報のないIo(NotFound)だけ、元kind/pathを含む警告と既存のoverflow再確認経路へ渡す。削除を捏造せず、エラーを無言で破棄しない。PermissionDenied・Other・Generic・パス付きエラーは従来の失敗通知を維持する。連続2件、全登録先の生存、他kindとパス付きエラーの回帰を追加する。

新しいタイマー、worker/queue上限の変更、無期限tombstoneやbatch基盤は追加しない。既存の再確認処理を使うことと、未知の連続通知で再decodeが発生しないことは別であり、後者は証明したと主張しない。

## 修復後の自動検証

匿名の連続通知を含む対象回帰6件、関連watcher group31件（対象を含む）、形式検査、AST30件、影響範囲Clippy、差分検査はすべて実exit0。ログはtmp/oct25-windows-watcher-recovery-final-errors.log、同final-watcher-group.log、同final-fmt.log、同final-ast.log、同final-lint.logに保存した。初回ASTのwatcher.rs非test201行違反は、共通dispatcherをDirectoryRecoveryへ移して修復した。元の上限200を維持する。

連続通知は実filesystemでold/nestedを消した後に共通dispatcherへ匿名エラーを2回渡すunit回帰で検証した。実native rename回帰はmacOSで成功し、元assertを維持する。native queueに届く後続通知をすべてdrainして検証したという意味ではない。WindowsネイティブCIの再確認は未完了であり、元failureのkindまで確定したとは扱わない。

設計レビューはtmp/oct25-windows-watcher-recovery-design-review.md、修復と実検査はtmp/oct25-windows-watcher-recovery-fix.md。最終独立レビューは4ファイルの実SHA一致、P0/P1なしを確認し、tmp/oct25-windows-watcher-recovery-final-independent-review.mdへ保存した。native後続queueの未測定は証跡の境界として保持し、追加の公開条件にしない。[担当Issue](https://github.com/HiroyukiFuruno/KatanA/issues/358)で通常commit/push・新HEADのreview/CI・公開を追跡し、本人確認をClose条件へ含めない。
