# Windowsの画像更新通知を維持する修復

## 原因と対象

PR346の新HEAD `a2b58be3` に対する指摘4368177628と独立差分レビューで、notify 8.2.0のPollWatcherがmtimeを整数秒で比較し、既定では内容比較しないことを確認した。同じ秒内の更新やmtimeを保った画像上書きが通知されず、古い画像が残る。親ディレクトリの名前変更を拒否する常駐handleは復活させない。

## 修復

Windowsの親metadata監視は維持し、登録済み画像のalias/resolved経路だけを別PollWatcherで内容比較する。無関係なOffice/PDFを内容走査しない。内容snapshot確立後に登録完了を通知し、確立前にdecode済みの画像も対象ownerへの一回の再確認で更新する。通常の同期では再確認通知を繰り返さない。

欠損時の登録失敗・通知キューoverflowは既存の再試行へ戻す。owner解放、明示解除、親削除・再作成では内容監視も同期する。親pollingのpath付きNotFoundだけを実消失通知へ正規化し、PermissionDeniedなど未知のエラーは隠さない。

## 回収結果と残条件

- `tmp/oct25-windows-content-watch-tests-final.log`: 実ファイルを使う対象監視32件が成功。同mtime・同サイズのPNG更新、登録前更新、欠損復旧、別経路解除、owner解放、親rename・復旧を含む。
- `tmp/oct25-windows-content-watch-clippy.log`: UIとtestsの厳格Clippyが終了0。整形・差分検査も終了0。
- 初回のテスト引数型のコンパイル失敗と、fixtureのcanonical経路不一致による28/29の失敗は修復後の成功と区別する。
- 通常commitの正式ASTでerror-first・file-length・magic-numbers違反を検出したため、失敗処理と検査fixture・登録・解除を責務分割した。`tmp/oct25-windows-content-watch-ast-repair-{tests,ast,clippy,fmt-check}.log`で対象32件・正式AST30件・厳格Clippy・整形を再確認した。基準や検査の意味は変更していない。

これらはmacOS上の実PollWatcher・既存通知処理の回帰結果であり、Windows本体での成功や全性能合格ではない。通常pushの正式xwin検査、個別threadへの修正返信・resolve、新HEADのレビューとWindows CIを回収する。
