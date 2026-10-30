# v0.22.44 文書読込・表示・入力の回帰修正

## 最新要件

> OfficeやHTMLを開いてロードのまま画像を開こうとすると、画像が開かない。
> PPTXを含む他のドキュメントもローディングを上下左右中央で統一。
> PPTXとかのキーボード操作によるページングが全然できていない。キーバインドの競合っぽい。
> token優先だけど、不具合は全部直して。報告だけで止まらず進めて。

## Definition of Ready

- [x] Homebrewの保存済みSecretをActionsで確認し、WinGetのローカル登録値/権限/対象forkを確認した。公開書込の証明とは区別する。
- [x] master clean/stash0を確認し、既存checkoutでrelease/v0.22.44を作成した。
- [x] #388/B01〜B11と新規報告をtmp/v0.22.44-prep/feedback-inventory.mdへ照合した。

## Branch Rule

作業はrelease/v0.22.44のみ。最新本人指示により、新規worktree/master編集/通常stash/hook迂回/強制pushは禁止。関心別通常commit、単一Draft PRへ統合する。古いHEADのCIを新HEADへ流用しない。

## Definition of Done

- [ ] 報告済み全件の実装/検証/未解決/上流境界を照合し、実行可能なKatanA所有の修正を完了する。
- [ ] 実egui/production入力経路、原本混在負荷、既存自動検査で検証し、検査失敗や未再現を完了扱いしない。
- [ ] `/openspec-delivery`相当の自己レビュー・関心別通常commit/push・Draft review/修正/reply/resolve・Ready・required checks・merge・公開・成果物確認・安全な後処理を実施する。本人の実機確認はClose条件に含めない。

## 1. 画像と混在読込 — #385/#386

- [x] 1.1 原本/混在未完了条件と既存v43の診断結果を照合し、違いと未再現を記録する。
- [x] 1.2 特定した画像選択の同期確認・allocator/HTML終了待ち・取消のKatanA所有処理を修正し、分類/実Explorer/終了寿命の回帰を追加する。原PNG30秒の全原因とMissingの全条件は未確定のまま#385へ残し、今回の局所修正を全面解消と呼ばない。
- [/] 1.3 修正後の画像表示・反復・正常終了は実workerで検証済み。元メモリ基準を含むcurrent HEADの必須自動検査をReady後に回収する（cold196608KiB/steady65536KiBを変更しない）。
- [x] 1.4 最終preview終了時のallocator走査をUIと共用mutexから外し、lifecycleの完了観測を維持する。
- [x] 1.5 HTML adapterの同期joinをUI破棄から外し、実HTMLの未完了/表示済みcloseと共有テスト環境の終了寿命を検証する。

## 2. 共通Loading

- [x] 2.1 旧共有部品の実矩形で水平中央の不合格を再現する。
- [x] 2.2 全callerを照合し、有限viewportとinline/fullscreenの両軸中央配置・文言回帰を通す。

## 3. キーボード入力

- [x] 3.1 文書選択/フォーカス/shortcutの実経路でPPTX/PDF左右キー未達を再現する。
- [x] 3.2 文書活性化と入力所有を修正し、text/search/modal/修飾/タブ/境界/二重処理を回帰する。

## 4. 全報告の照合

- [x] 4.1 B01〜B11の画像欠落/反復loading/メニュー/mode/slideshow/group/close/fullscreenと分割背景の実装・自動回帰を照合する。
- [x] 4.2 公開済み限定修正・今回修正・未解決・上流handoffをIssue/boardへ証拠とともに反映する。

## 5. Final Verification & Release Work

- [x] 5.1 docs/coding-rules.ja.mdとself-reviewで差分を確認する。
- [x] 5.2 Markdownとコードを整形し、対象回帰を回収する。
- [x] 5.3 関心別通常commitの後、通常push hook一回を正式ゲートとし、重い同検査を手動で重ねない。
- [x] 5.4 Draft PR→全reviewページ/threads取得→P0/P1修正と個別reply/resolve→fresh確認を完了する。
- [ ] 5.5 Ready後のcurrent required checksを回収し、不合格は修正する。
- [ ] 5.6 通常merge→Actions公開→全5asset/checksum・配布自動検査→Issue更新→OpenSpec archive→安全なbranch/worktree後処理を実施する。

## KatanA CLI Entry Point

OpenSpecはリポジトリの`./scripts/openspec`を使用する。既存品質基準・公開済みregistry依存を維持する。

## 現在の対象検証

- 画像参照分類: 旧実装3件失敗→修正後3件成功。`tmp/v0.22.44-prep/image-reference-{red,green}.log`。PNG30秒待ち全体の原因確定とは区別する。
- 共有Loading: 5件、実callerのstandalone/inline配置1件成功。`loading-widget-final.log`、`loading-caller-final.log`。
- 左右キー: `keyboard-final.log`で7件成功（読込中に旧フォーカスが失効する回帰を含む）。正式ASTで検出した規約違反は修正後に全30件成功。
- Memory: `worker-memory-tests.log`で12件成功。HTML surface32件成功。初期化直後の実HTML終了回帰を含め33件成功。実HTML/Officeのclose検査も元5秒条件で成功。
- 正式AST: 初回21件成功/2件失敗→規約準拠で修正後、全30件成功。ゲート変更なし。

- v0.22.44表記と両lock更新後の対象検査: Loading5件、実caller1件、画像参照分類3件成功。最終pushゲート・公開は別工程。

## 2026-10-30 現在の統合結果

- 関心別通常コミット: 画像選択 `aa49cf80`、メモリ返却 `cdffd3c3`、Loading `22be2e03`、初期化取消 `4522382d`、HTML終了 `1578a616`、ページ操作 `ca0f4c9c`、全テーマ背景回帰 `4a2ee3ba`、Explorer混在回帰 `527efbea`。全て通常hookを通過。
- 公開fixtureのPPTX/HTML読込中に、実ExplorerのPNG行クリックから描画・10フレーム維持・表示完了後closeまで成功。原本HTMLと104907Bの原本PNGを使った組合せでも約149ms/157msで描画した。大型原本PPTXを含む全条件の解消を意味しない。
- 大型原本PPTXの読込途中closeは、KDVの`DocumentSession::open`から復帰せず5秒idle未達。KatanA側ではnative open直前/復帰後の取消確認と不要な初回描画抑止を追加。既に実行中の上流呼出しを中断するAPIはなく、[KDV #72](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/72)へ証跡を引き渡した。
- 元の表示完了後close5秒とメモリ予算は変更しない。原本の失敗診断は`tmp/v0.22.44-prep/original-explorer-diagnostic-source/`へ保持し、公開fixtureの成功へ置換しない。
- 最終ソースで文書close/Explorerの実worker検査3件成功（`final-document-close-and-explorer.log`）。通常push・Draft review・Ready CI・公開は未完了。

- board #388を最新commit・対象結果・KDV #72境界・token認証済みへ更新済み。実worker/出力/コマンド順序6件も成功。初回環境指定漏れの失敗は保持し、正式worker入口の再実行結果と区別する。

## 正式push失敗の修復

- 初回通常pushはUI1455成功/1失敗と、安全ZIP依存のrelease版識別漏れで不合格。合格へ読み替えずログを保持。
- HTML自身の終了21件は全て完了していたが、別テストのPPTX native openが共用globalカウントへ混入していた。検証環境が所有するHTML終了ハンドルだけを元5秒で回収する検査寿命へ修正し、製品のglobal lifecycle/実文書close5秒基準は維持。旧実装の赤失敗と、実HTML結線・worker失敗回収を含む3成功、並列全UI1458成功を確認。`420b627f`へ通常commit。
- 安全ZIP9.1.3/core0.22.44の完全一致許可を追加。別版・親・source・pathは拒否し、旧脆弱ZIPの例外を拡大していない。27契約とroot/screenshot両実locked graphの照合成功。core実装はmasterから不変。
- 次の通常pushで正式gateを再実行し、まだremote反映/必須CI/公開完了とは扱わない。

## 公開判定の範囲

- 本人の既存方針「自動テストにないものはリリースの品質担保の対象外」「リリース後に動作確認して未解決なら改めて対応」を引き続き適用する。実行可能な自動検査の失敗は除外しない。今回の改善の公開と、報告全体を解決してIssueをCloseすることは別。
- #385は30秒/Missingの全条件が未確定のためOpenを維持する。合計native起動のadmission未実装という設計懸念も物理監査へ残すが、30秒遅延の原因と断定しない。
- 最初のKML呼出しはinstalled CLI0.16.1の未対応helpにより対象外Markdownを整形してしまった。直前cleanを根拠に、その差分patchだけを逆適用し全て復旧、tracked cleanを再確認。途中のpush検査は明示停止し採用しない。以後は実装を確認した`--file`による対象指定だけを使う。

## Draftレビュー到達

- HEAD `5691242406d548fd1125b770f04f9e17280572d4` の通常pushは元ローカル/Linux/Windows向け/PR前ゲートを通過しexit0。local/origin 0/0、tracked cleanを確認。
- [Draft PR #397](https://github.com/HiroyukiFuruno/KatanA/pull/397)を作成。DraftのCI/Readinessは全job SKIPPEDでrunnerを割り当てていない。
- current HEADのCodex summaryはcompleted/has_any_findings=false/has_unresolved_findings=false。formal review/会話/threads/nested commentsを全ページ取得し未解決0、独立レビューもP0/P1なし。`tmp/v0.22.44-prep/pr397-review-56912424-completed-all-pages.json`。
- この記録更新後も最新HEADのレビューを再取得し、Ready後の必須CI・メモリ条件・公開結果はまだ完了扱いしない。

## Ready前のタブ経路修復

- 通常のタブクリックは検証済みだが、`SelectNextTab` / `SelectPrevTab`によるlocal PDF/PPTXへの移動ではページfocus ticketが立たないことを実呼出経路で確認。3.2を再開し、元の7成功だけで完了扱いしない。
- 次/前タブ両経路とPDF/PPTX、no-change/他formatを回帰し、既存の形式判定と共通active transitionへ統合する。新しい入口別の分散ifや公開設定は追加しない。

- タブ経路は新回帰で旧実装のfocus要求欠落を再現（`tab-keyboard-red.log`）。同時に出たtext側の失敗は、テストが旧TextEditを描かず消した問題であり製品の不具合へ数えない。
- 操作名ごとの分岐は増やさず、実際にactive pathが変わった場合だけ既存の形式別focus処理へ接続。次/前×PDF/PPTX・旧focus失効・一タブno-change・TextEdit保持を含め10件成功（`tab-keyboard-green-all.log`）、独立レビューP0/P1なし。
- この実装修正により最新HEADのDraftレビューを再開する。以前の56912424のレビュー結果を新コードの完了証跡へ流用しない。

## タブ修復後のDraftレビュー完了

- `e59d9cfa311f5ca7c27815cca6076c9c020e40fa` の通常push66979はexit0、元の全検査を維持して成功。remote HEAD一致・cleanを確認。
- 同HEADのCodex reviewはcompleted/指摘なし。formal review/会話/thread/nested commentsを全頁取得し未解決0、GitHub上の全15commit Verified。`tmp/v0.22.44-prep/pr397-review-e59d9cfa-completed-all-pages.json`。
- この後の差分は本ファイルの完了証跡だけで、実装/設定/両lockは変更しない。記録差分を査読してReadyへ進め、新HEADの必須CIとReadyイベントで開始される追加レビューはmerge前に回収する。過去CIの合格を新HEADへ流用しない。
