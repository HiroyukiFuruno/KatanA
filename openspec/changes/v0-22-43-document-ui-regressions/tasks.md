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

- [x] 1.1 元PNGのhash/sizeを結合し、image-onlyとOffice/HTML混在の選択→保存→full frame→初回texture準備を計測する。
- [x] 1.2 未変更の反復・実watch更新・キュー満杯時の要求状態を調べ、確認したKatanA側の再loading/完了通知/再試行を修正・回帰する。原本の約30秒待ちとMissing表示の全原因確定は未達として#385へ残し、全面解消とは扱わない。
- [x] 1.3 確認したworker完了wake修正を、実callerの回帰と一緒に正式統合する。単体の高速decodeだけで30秒問題を閉じない。

## 2. 共通操作設定とLoading — #383 / B04〜B08

- [x] 2.1 形式×状態の可否表を一元化し、TOC/refresh/search/export/story/tools/infoとCode/Split/Slideshowを全入口へ適用する。
- [x] 2.2 PDFを含む全binaryの禁止mode、ready PPTXのsettings/start、same-path reopenのstale readinessを自動回帰する。
- [x] 2.3 submenuのSplit/Tangochouも共通設定で表示可否を制御し、実widget状態を検証する。
- [x] 2.4 全6系統のLoadingを共通化し、ローカライズとplaceholder未露出を検証する。

## 3. グループ操作 — #384 / B09

- [x] 3.1 popupの一回開始、draft/focus保持、Enter/confirm/Escape/外側clickの終了を一つの状態管理へまとめる。
- [x] 3.2 実eguiイベントとdispatcherで名前反映・matching IDのみの解除・後続タブ操作を回帰し、変更関数の規約を確認する。

## 4. HTML・Office close — #386 / B10

- [x] 4.1 隔離fixtureでHTML/DOCX/XLSX/PPTXのsave・preview drop・allocator返却・最初のUI frame・worker idleを段階別に測る。
- [x] 4.2 計測したKatanA側のタブ誤選択を修正し、代表4形式のclose・preview解放・次frameを実経路で回帰する。元のメモリ/正常終了の自動検査は8.4で維持し、同期open中のKDV所有worker終了は既存の上流非停止方針に従いKDV #70へ切り分け、未達を明記する。
- [x] 4.3 診断専用instrumentationを整理し、永続的な実経路回帰と証拠を残す。

## 5. 画像制御配置 — #387 / B11

- [x] 5.1 通常/全画面の画像clipと下部制御領域を分離し、1px/小画像/通常/縦長・viewport別の非重複と操作を検証する。

## 6. 配布とDocker — #381 / #382 / #376

- [x] 6.1 対象別同期結果・失敗検知・receipt保持を正式統合し、元の必須公開検査を維持する。
- [x] 6.2 Homebrew生成DSL・zero SHA拒否・fixture認証/コマンド隔離を再発回帰と一緒に正式統合する。
- [x] 6.3 owner/purpose/run ID、実producerの保存先、停止後の証跡export/hash、対象限定just入口と保持方針を統合する。
- [x] 6.4 統合後のunit11件と実Docker7件で作成/稼働/停止参照、正常終了/失敗/保存失敗/再実行を検証する。元のLinux/Windows/UI条件は8.4の新HEAD必須CIで引き続き検証する。
- [x] 6.5 既存Open Issue #357/#372/#362/#343/#338/#339をlive本文と現差分で照合し、実装済み・未対応・検証待ち・上流待ちを台帳へ記録する。#357/#362の本文を現実装へ更新し、他Issueの独立残作業を完了扱いにせず保持する。

## 7. User Review (Pre-Final Phase)

- [x] 7.1 ボードB01〜B11に対応する実装・自動検査の結果を提示し、追加報告を同じ台帳へ登録する。本人の返答や実機確認を公開停止条件にしない。
- [x] 7.2 B01〜B11を共通操作/Loading/画像/グループ/close/配置の実装と自動回帰へ照合し、原本全事象の未確認とKDV #70を未解決として明記する。既存の自動検証優先・本人確認非停止・上流非停止方針を維持し、Issueを全面解決として閉じない。

## 8. Final Verification & Release Work

- [x] 8.1 自己レビュー、変更範囲のfmt/lint/回帰、文書検査、依存関係調査・更新を完了する。品質条件は下げない。
- [ ] 8.2 関心別の署名付き通常commit、通常push hook一回の結果を回収し、失敗を修正する。重い同検査を直前に重複実行しない。
- [ ] 8.3 Draft PRで自己・独立・Codex reviewを全ページ取得し、P0/P1修復・各thread reply/resolve・fresh確認後にReadyへ進める。
- [ ] 8.4 新HEADの全OS/build/test/lint/coverage/security/packaging/host受入の必須CIを確認し、通常mergeする。元のRSS cold 196608KiB/steady 65536KiBと独立したphysical-footprint予算、KatanA所有資源の解放・通常終了の既存自動検査を維持する。KDV #70へ切り分けた同期open中のworker終了未達は明記し、解消済みとはしない。旧HEADの合格を流用しない。
- [ ] 8.5 GitHub Actionsの公開と全5asset/checksum・配布検証・外部配布同期の実結果を回収する。
- [ ] 8.6 Issue状態更新・OpenSpec archive・branch/worktree/stashの安全な後処理を完了する。リモートbranchは明示指示なしに削除しない。

## 最新の実装・検証証跡（2026-10-29）

`[x]` は当該工程の実装・対象検証完了を示し、公開完了を意味しない。未完了の通常タスクは `[ ]` のまま進捗と証跡を記録する。公開は8.3〜8.6で別に確認する。

- グループ入力: 通常commit `452094e9`。実egui/dispatcherの4回帰が最新依存構成でも成功。
- 画像: 通常commit `dd48838e`。未変更auto保持・手動reload・実watch変更、worker完了wake、実global queue満杯から完了/取消後の表示復旧を検証。本人の約30秒待ちとMissing表示すべての原因を確定したとはしない。
- 共通操作・Loading・下部画像制御: 通常commit `d952a162`。7形式ポリシー、実SelectAndJump、登録コマンド、Loading2件、controller7件等を含む最新構成の42対象回帰成功。新構成のslideshow16件も成功し、実PPTX fixtureでready/next/stale拒否/closeを検証した。
- close: 通常commit `078b861d`。手前タブを閉じた際の誤選択をRED→GREENで修正。代表HTML/DOCX/XLSX/PPTXの描画後closeと資源解放も自動回帰成功。ただし原本の全closeフリーズが解消したとはしない。通常診断OFFの大型PPTXは表示40.410秒、close0.758ms、次HTML/保存/解放を確認。大型XLSXは約51秒のworker読込制限で表示前に失敗し、close結果ではない。描画開始直後の原本closeを追加診断中。
- Docker資源管理: 通常commit `11f06971`、unit11件/実Docker7件成功。既存volume不変、診断用fixture残留なし。全OS/配布受入は今後の正規工程。
- 公開元照合: 通常commit `6ee75e45`、binding7/admission14/flow24とv42実API照合成功。新v43の公開後source/3manifest/5asset照合は公開後に実施する。
- 依存更新: 通常commit `73f1efe4`、公開KML0.20.0と互換依存を両lockへ採用。KDV0.5.12/KRR0.4.23固定を維持。`just outdated` の一時診断はlibc最新要求とKDVの固定要求で失敗したが、実manifest/lockの解決失敗とは区別し、通常のcargo更新と最新構成の対象回帰は成功。
- 検証環境TLS: 通常commit `6bece25f`。保持2venvをpip26.2.2へ更新、更新前後とも同一pip標準contextで正常対照/不正4経路を検査。observerのCA bundle欠落を修正した実imageでも5条件成功。旧image ID所有判定等の残件はIssue372へ保持。
- 正式AST/Clippyは通常commit hookで成功。初回の失敗は取り消さず証跡へ保持。通常pushは容量不足と統合テストの旧構築方法、v43工程設定不足で失敗。結果を回収し修復を続けており、Draft review・Ready CI・公開は未実施。

### close追加診断と上流責任の切分け

原本の描画開始直後に実CloseDocumentを実行した3独立プロセスで、HTMLはclose0.524ms/次UI1.360ms・即時解放、PPTXは0.513ms/次UI2.012ms、XLSXは0.322ms/次UI1.689ms。Office2件はUIが応答していてもworkerが元の5秒条件内でidleにならなかった。この未達を成功へ変換しない。隔離runnerの所有する残存workerだけを回収し、原本SHA前後一致を確認した。

KatanAのtab除去と次frameは同期openの完了を待たない。KDVのDocumentSession::open中は文書単位のcancel/PID handleがまだ返らず、PPTX/Spreadsheetの子processはKDVが所有するため、他の文書を巻き込むprocess-list killで回避しない。[KDV Issue #70](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/70)へ実測・API境界・自動回帰/公開DoDを引き継いだ。既存#65〜#67とは区別した。通常eframe終了時の孤児化は未検証であり、test runner終了後の残存をその証明にしない。

本人の既存方針「KatanA側で改善できるか否かが判断軸」「KDV, KRRはissue立ててあなたは作業しなくて良い」に従い、KatanAの実装・レビュー・必須CI・公開をこの上流対応待ちで停止しない。初回30秒報告やclose全体が解消したと偽らず、既存Issueの結果区分を維持する。

### 再開後の全体検査と修復

通常push `normal-push-resume.log` の結果を回収し、公開前検査の13段階は成功、v43工程設定不足で最終段階が失敗した。全体testは旧unit struct構築の統合テストでコンパイルが失敗した。テスト都合で製品APIを公開せず、単体検証を同一crate内へ移し、単体1件と実UI/キーボード3件を成功させて通常commit `8920f4d4`へ統合した。

v43専用の工程契約は7件、既存release-flow契約は24件成功。未知version/change/path/modeと実装工程未完了を拒否し、push実行結果・review・CI/merge・公開・後処理をそれぞれ証拠が存在する段階に残す。strictの完了条件は維持する。OpenSpec strict validateと変更文書KML検査が成功した。新HEADの通常push成功はまだ得られていないため8.2は未完了のまま。

Open Issue6件はlive本文と現ソースを照合し、#357/#362は実装済み・対象回帰成功・未公開へ本文を更新した。#343/#338/#339/#372の独立残件は解決済みとせず、Issueと作業台帳へ保持する。
