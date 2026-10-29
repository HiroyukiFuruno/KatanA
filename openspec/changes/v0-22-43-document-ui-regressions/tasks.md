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

### 独立レビュー指摘

- [/] P2: closeと最初の次frameが時間の合否に入っていなかった検査を補強した。元の5秒を増やさずclose前から単一の絶対deadlineを共有し、既にworkerがidleでも期限切れを拒否する。実workerの代表4形式と実await入口の期限切れnegativeが成功。通常push・thread返信/resolve・current reviewは8.2/8.3で別に実施する。

- [/] Codex P2: 本文内の共通Loadingがviewport全高を消費して後続本文を押し出す問題を修正。実egui 320×600で旧row=600ptのRED、content-height horizontalへ変更後のgeometry回帰と文言2件がGREEN。全面表示callerの中央配置は維持し、通常push/個別reply/resolveは8.2/8.3へ残す。

## 8. Final Verification & Release Work

- [x] 8.1 自己レビュー、変更範囲のfmt/lint/回帰、文書検査、依存関係調査・更新を完了する。品質条件は下げない。
- [x] 8.2 関心別の署名付き通常commit、通常push hook一回の結果を回収し、失敗を修正する。重い同検査を直前に重複実行しない。
- [x] 8.3 Draft PRで自己・独立・Codex reviewを全ページ取得し、P0/P1修復・各thread reply/resolve・fresh確認を完了する。Readyへの昇格は次の8.4で行う。
- [ ] 8.4 レビュー完了後にReadyへ昇格し、新HEADの全OS/build/test/lint/coverage/security/packaging/host受入の必須CIを確認し、通常mergeする。元のRSS cold 196608KiB/steady 65536KiBと独立したphysical-footprint予算、KatanA所有資源の解放・通常終了の既存自動検査を維持する。KDV #70へ切り分けた同期open中のworker終了未達は明記し、解消済みとはしない。旧HEADの合格を流用しない。
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

### 並列テストの言語干渉修復

b5257652の通常pushはUI lib 1412成功/1失敗。移動したprovider検査が他テストの言語変更と干渉し、単独成功だけでは不十分だった。失敗は保持し、製品APIや基準を変えず、同じquery/assertを同一実行ファイルのexact子プロセスへ隔離する修正を適用した。子の実行成功markerも必須とし、0 tests成功を拒否する。全体並列lib回帰の再検証中なので8.1を未完了へ戻した。

言語干渉の隔離修正後、同じ通常profile/実workerでUI lib全体を並列再検証し1413成功・0失敗・既存ignored2/fixture filter2を確認した。子判定用の同名環境変数が親にある条件でもexact argvを確認して隔離し、親のlocaleを変更しない。結果: `tmp/v0.22.43-prep/palette-isolation-parallel-lib.log` / exit0。8.1を対象検証完了へ戻し、通常push成功の8.2は未完了を維持する。

共通ProcessServiceを使う最終隔離版でも、実workerを供給した並列UI lib 1413成功/0失敗。Windowsのheadless起動規約をテストでも維持し、ASTが拒否した直Command版は採用しない。最終検証logは `tmp/v0.22.43-prep/palette-isolation-parallel-lib-facade.log` / exit0。

### PR389独立P2の実経路再検証

公開thread `discussion_r4406997730` に指摘を記録。最終修正はclose前の絶対deadlineをforce close直後・次frame直後・worker idle前/中/後に検査し、元CLOSE_TIMEOUT=5秒を維持した。negativeはassert helperだけでなく、空の隔離HarnessとRenderEnvLockで実await_workers_idleへ期限切れを渡し、入口で拒否することを検証した。固定sleepや商用API変更なし。

実worker付き2 tests成功。HTML/DOCX/XLSX/PPTXの force_close_ms / close_to_next_frame_ms / worker_idle_wait_ms は順に1/3/0、0/2/0、0/1/11、0/2/0。正規の配布版受入・原本全close解消とは混同しない。証跡 `tmp/v0.22.43-prep/close-deadline-review-fix/close-deadline-idle-entry-regression.log` / exit0。

### PR389 Codex P2 — inline Loading

`PRRT_kwDORm09y86xWh4u` の指摘に従い、共通widgetをcontent-height horizontalへ修正した。文書別の重複widgetや商用test APIは追加せず、6系統callerは従来のfull-surface中央配置/inline本文配置を保持。実egui0.36.2の旧実装RED=viewport600ptを消費、修正後3 tests成功。初回のunused_mut compile失敗はREDとして扱わず別logへ保持した。証跡 `tmp/v0.22.43-prep/loading-inline-review-fix/inline-layout-red-retry.log` と `inline-layout-green.log`。

新P2を修正するため、旧d68200fcだけを検査中だった自分のpush session96989をSIGINTで終了しactual130を回収した。古い内容の途中検査を成功扱いせず、remoteはd0d3b475のまま、残存check/lease processなしを確認。両P2修復の新HEADを通常pushで改めて全検証する。

Loadingの最終実装は既存の共通AlignCenter(shrink_to_fit)を再利用した。直接ui.horizontalは既存AST horizontal-layout規約に拒否されたため採用せず、閾値・ルールを無効化しなかった。最終版もgeometry/locale3件成功、log `tmp/v0.22.43-prep/loading-inline-review-fix/inline-layout-align-center-green.log` / exit0。

### 実装構成3bb1ad10のDraftレビュー完了

実装修正を含む通常pushは `normal-push-3bb1ad10.log` / exit0、remote同期0/0。GitHub上で20commit全てVerified。Codexのcurrent commit3bb1ad10の機械receiptはstatus=completed/has_any_findings=false/has_unresolved_findings=false。全REST頁とGraphQL全connection/nestedコメントを回収し、2つのP2に個別reply/resolve後、未解決thread0をfresh確認した。High独立レビューの最終deltaもP0/P1/P2追加なし。

この台帳確定では実装・自動検証条件を変えない。8.3は実施済みDraftレビューと指摘解決の記録、まだ未実施のReady操作と新HEAD必須CI/mergeは8.4へ分け、将来イベントを完了扱いにしない。台帳のみの最終commit/push/自己レビュー後にReadyへ進み、現HEADの全必須CIを実施する。3bb1ad10の旧CIを新HEADへ流用しない。

### Ready CIで検出した撮影ツールの追従漏れ

新HEAD dbe1c08dのCI run40791121094、Linux job123176311422で `Test screenshot harness contracts` がE0423で失敗。`scripts/screenshot/src/executor_harness.rs:2352` に旧unit struct `AppCommandProvider` の生成が残っていた。未生成のheadless evidence upload失敗はこのコンパイル失敗に続くもので、本体文書の操作不合格と混同しない。旧成功やskipを代用せず、実ツールの修正・対象回帰・通常push・新レビューと新HEAD必須CIを行う。

- [/] 撮影側の重複provider検索を削除し、本体パレットへの実入力で検索・文書別制御を保つ。実UI回帰と撮影ツール全契約を回収する。

失敗ログ: `tmp/v0.22.43-prep/required-ci-dbe1c08d/job-123176311422-failure.log` と `job-123176311422-E0423.txt`。今回の実装修正に新公開APIや本体制御の緩和を追加しない。

修復後は既存debug target・同じlocked依存・診断OFFの通常Office worker入口で、撮影ツール全55 tests成功（ignore/filter0）。実KatanaApp・一時設定の実egui文字入力でquery更新とPDF pathの禁止コマンド除外も成功。ログ `tmp/v0.22.43-prep/screenshot-palette-ci-fix/all-tests-bash.log` / exit0。最初のscript直実行は非実行bitでexit126だったため、既存のbash入口で再実行した。release profileと全OSは新HEADの必須CIで再検証し、このdebug結果を代用しない。

### 本人の追加依存更新を公開前に統合する

2026-10-29の最新humanが、検証中に変更された両lockfileは本人のjust update由来であり「取りこんで公開」と明示した。v0.22.43はRelease/tagとも未作成（API404）で、旧公開run40806478547はCI gate待ち中に取消、旧Mのpush CI40806478312も取消した。公開済みassetやtagの上書きではない。PR389はmerge済みのまま保持する。

既存worktreeのorigin/master50ffcd07からrelease/v0.22.43-dependency-refreshへ本人差分をそのまま移し、stashもworktreeも作成していない。対象はctutils0.4.6→0.4.7と、撮影用lock内の既存syn/windows-sysの選択更新。KDV/KRRの公開registry固定は変更しない。

- [ ] 本人の両lock更新を精査・対象検証・通常commit/pushし、Draftレビューから新HEAD全必須CI/通常mergeを経てv0.22.43として公開する。旧fd/50ffの合格や取消を新lockの合格へ流用しない。

8.1〜8.3の完了記録はPR389までの実装修正に関する実績であり、今回追加更新のpush/レビュー結果を先取りしない。8.4〜8.6は依存更新を含む最終構成で継続する。両lockの元差分はtmp/v0.22.43-prep/concurrent-lock-update.diffにも保持したが、原本を退避/破棄せず正式履歴へ統合する。

追加依存構成で、通常Office worker入口・同じlocked graphの撮影ツール全55 testsが成功（ignore/filter0、dependency-refresh/screenshot-tests.log / exit0）。KML対象3文書・diff・pr-bootstrap phase検査成功。High独立レビューも依存要求・checksum・Windows向けlocked解決を確認して追加P0/P1/P2なし。これは対象検証結果であり、新HEADの通常push/レビュー/全必須CI/公開を完了扱いしない。
