# v0.22.43 文書操作・表示回帰修正

## 要件(Verbatim Requirements)

> 各対象ドキュメントのプレビューとコードの切り替えが出来てはならない場合に出来てします。
>
> pptxでのスライドショーが有効になっていない。
>
> 右メニューの制御が要件通りになっていない。
>
> これらの制御は設定のclassを作成して、ドキュメントごとに定義を持つって制御すればこんなことにはならないはず。
>
> 右メニューの全てにしたいしてドキュメントごとの制御を最適化してくださいね。
>
> ローティングなんて汎用部品作って文言渡すだけで、全てのドキュメントで統一した仕様で提供できるのに...まじでありえんって
>
> グループ作成はバックグラウンドで完了しているのに、確定ができない。そしてグループ名のウィンドが開いたままになり、グループ解除や、グループを閉じないと、タブの操作が一切できない致命的なバグ
>
> png(おそらくpngだけではなく画像全般)を開くと、かなりの高頻度で更新がかかり画像がローディングされ続けている。
>
> あとフルスクリーンのボタンと制御のボタンが重なっていてくそ。制御のボタンは描画対象が小さい場合でもかならず、画像の下部に配置して、この問題を回避できる。githubそうしている。
>
> 100k前後のpngのロードに30sかかることだ。
>
> 閉じようとするとフリーズするのはhtmlだけではなくoffice系もでした。

### この要件から導出される制約(MUST)

- 全右メニュー・modeの制御を文書ごとの共通設定で行う。
- Loadingは共通部品で文言を渡す。文言だけを直して画像30秒問題を解決扱いしない。
- グループ作成後の確定と入力解放、画像の再loading/欠落、HTML/Office終了を別々に追跡する。
- 小さい画像でも制御は画像下部の独立領域にする。

### 設計判断時の参照義務

設計変更時には上記要件とB-IDを照合する。矛盾する変更でDoDをすり替えない。

## Definition of Ready

- [x] 最新本人報告11件をIssue #388とB01〜B11へ照合した。
- [x] 既存checkoutのrelease/v0.22.43を使用し、公開済みv0.22.42は変更しない方針を記録した。

## Branch Rule

既存checkoutのrelease/v0.22.43のみで作業する。本人の現行指示を優先し、新規worktree・feature/fix/wipブランチ・master編集・通常stash・hook迂回・強制pushは行わない。関心別通常commitの後、単一のDraft PRへ統合する。

## Definition of Done

- [ ] 対象不具合を実装・実経路の自動回帰で確認し、未解決を解決扱いしない。
- [ ] `/openspec-delivery` の自己レビュー・通常commit/push・Draft review・指摘対応/reply/resolve・Ready・必須CI・通常mergeを実行し、Actions公開、全5asset/checksumと配布検証、Issue更新、OpenSpec archive、安全な後処理まで証拠を残す。

本人の実機確認とTerms承認はClose条件に含めない。上流repositoryは編集せずIssueへ切り分ける。独立した実装・検証を上流や本人の返答待ちで止めない。

## 1. 画像の遅延と安定性 — #385 / B01〜B04

- [ ] 1.1 元PNGのhash/sizeを結合し、image-onlyとOffice/HTML混在の選択→保存→full frame→初回texture準備を計測する。
- [ ] 1.2 未変更の反復とタブ往復でtexture/revision/要求回数を調べ、再loadingとMissing表示の原因を切り分けて修正する。
- [ ] 1.3 確認したworker完了wake修正を、実callerの回帰と一緒に正式統合する。単体の高速decodeだけで30秒問題を閉じない。

## 2. 共通操作設定とLoading — #383 / B04〜B08

- [ ] 2.1 形式×状態の可否表を一元化し、TOC/refresh/search/export/story/tools/infoとCode/Split/Slideshowを全入口へ適用する。
- [ ] 2.2 PDFを含む全binaryの禁止mode、ready PPTXのsettings/start、same-path reopenのstale readinessを自動回帰する。
- [ ] 2.3 submenuのSplit/Tangochouも共通設定で表示可否を制御し、実widget状態を検証する。
- [ ] 2.4 全6系統のLoadingを共通化し、ローカライズとplaceholder未露出を検証する。

## 3. グループ操作 — #384 / B09

- [ ] 3.1 popupの一回開始、draft/focus保持、Enter/confirm/Escape/外側clickの終了を一つの状態管理へまとめる。
- [ ] 3.2 実eguiイベントとdispatcherで名前反映・matching IDのみの解除・後続タブ操作を回帰し、変更関数の規約を確認する。

## 4. HTML・Office close — #386 / B10

- [ ] 4.1 隔離fixtureでHTML/DOCX/XLSX/PPTXのsave・preview drop・allocator返却・最初のUI frame・worker idleを段階別に測る。
- [ ] 4.2 計測したKatanA側の停止原因を修正し、背景teardown完了と元のメモリ/正常終了条件を維持する。
- [ ] 4.3 診断専用instrumentationを整理し、永続的な実経路回帰と証拠を残す。

## 5. 画像制御配置 — #387 / B11

- [ ] 5.1 通常/全画面の画像clipと下部制御領域を分離し、1px/小画像/通常/縦長・viewport別の非重複と操作を検証する。

## 6. 配布とDocker — #381 / #382 / #376

- [ ] 6.1 対象別同期結果・失敗検知・receipt保持を正式統合し、元の必須公開検査を維持する。
- [ ] 6.2 Homebrew生成DSL・zero SHA拒否・fixture認証/コマンド隔離を再発回帰と一緒に正式統合する。
- [ ] 6.3 owner/purpose/run ID、実producerの保存先、停止後の証跡export/hash、対象限定just入口と保持方針を統合する。
- [ ] 6.4 統合後のunitと実Dockerで作成/稼働/停止参照、正常終了/失敗/保存失敗/再実行を検証し、元のLinux/Windows/UI条件を維持する。
- [ ] 6.5 既存Open Issue #357/#372/#362/#343/#338/#339の独立残件を一次情報で処理し、実装/検証/上流待ちをIssueと台帳へ反映する。

## 7. User Review (Pre-Final Phase)

- [ ] 7.1 ボードB01〜B11に対応する実装・自動検査の結果を提示し、追加報告を同じ台帳へ登録する。本人の返答や実機確認を公開停止条件にしない。
- [ ] 7.2 未対応フィードバックを残さず、個別劣後が本人から指示された場合だけ根拠を記録する。

## 8. Final Verification & Release Work

- [ ] 8.1 自己レビュー、変更範囲のfmt/lint/回帰、文書検査、依存関係調査・更新を完了する。品質条件は下げない。
- [ ] 8.2 関心別の署名付き通常commit、通常push hook一回の結果を回収し、失敗を修正する。重い同検査を直前に重複実行しない。
- [ ] 8.3 Draft PRで自己・独立・Codex reviewを全ページ取得し、P0/P1修復・各thread reply/resolve・fresh確認後にReadyへ進める。
- [ ] 8.4 新HEADの全OS/build/test/lint/coverage/security/packaging/host受入の必須CIを確認し、通常mergeする。旧HEADの合格を流用しない。
- [ ] 8.5 GitHub Actionsの公開と全5asset/checksum・配布検証・外部配布同期の実結果を回収する。
- [ ] 8.6 Issue状態更新・OpenSpec archive・branch/worktree/stashの安全な後処理を完了する。リモートbranchは明示指示なしに削除しない。
