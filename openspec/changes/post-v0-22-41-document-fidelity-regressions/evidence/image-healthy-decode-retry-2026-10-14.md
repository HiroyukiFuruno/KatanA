# 画像decode失敗の期限付き再試行

## 対象

- Issue #348、PR #346 review 5513716841 / comment 4267334667。
- healthy watcher下の一時decode失敗を永久にFailedとして保持しない。
- 既存の成功cache・背景別key・世代/path revision取消・batch結果保持・容量制限を維持する。

## 現時点の実証

- 既存APIだけのrepaint回帰94218はcompile成功後、`retry repaint`欠落でexit101。rawは`tmp/issue348-healthy-decode-repaint-red-2026-10-14.log`。
- 初期の別retry map案48316は46成功/1失敗。元のbatch全失敗結果保持を壊すため不採用。
- `DecodeFailure`を既存ready/activeのErrへ埋め込んだ候補1245は画像loader対象47件成功、2.34秒、exit0。全release gateの証明ではない。
- 再試行延期後にreadyが除去される境界54092は、期限切れactiveを参照する1assertでexit101。rawは`tmp/issue348-rearm-copy-deadline-red-2026-10-14.log`。ready除去を延期より先にした初fixture65528の成功はこの不備の再現証拠ではない。
- rootは両cacheへ同じ期限を設定する修正を追加。98982はloader49件成功、2.43秒、exit0。
- 正式AST60643は22成功/1失敗（early-returnと待機時間の定数化）、impacted Clippy44338は3規約違反でexit101。除外せず規約へ修正し、飽和回帰は境界テストへ責務分離した。
- 規約補正後99161はloader49件成功、2.38秒、exit0。正式AST25379は23件成功、22.36秒、exit0。rustfmt checkとdiff checkも成功。最終impacted Clippy37596はexit0、追加テストを含むUI全target/external-fixture-acceptance strict Clippy91773もexit0。

## 自己レビュー

- 公開の`LocalImageStatus::Failed(String)`・store result入力契約は不変、埋込み型は内部cacheのみ。
- 新allocator/追加thread/allow/閾値変更/テスト削除なし。元1worker/queue2、ready16件/64MiB、active frame猶予を維持。
- 成功/reset/path invalidationで失敗履歴も破棄。due/Pending中は履歴を保持し、attemptsは飽和加算。
- 最終readonlyレビューは期限同期・lock順・実UI repaint導線・呼出順と追加関数30行/ファイル200行を確認しP0/P1残なし。実queue full統合検査の留保は維持。
- この対象検査を全OS・全coverage・packaged受入や公開完了へ読み替えない。

## 証明していない範囲

- unitの期限操作と実PNG decoder復旧は、OSイベント欠落/sharing violationそのものの実再現ではない。
- global queue fullから再試行timerまでの実統合経路は未実測。cache境界回帰とcaller監査を区別する。
- 最新候補の通常commit/push・current review返信解決・全gate・packaged受入・全OS・公開は未完。
- 本人のアプリ・設定・Termsは操作していない。
