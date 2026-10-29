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
- [ ] 1.2 待ち行列/監視登録/取消/再読込/終了のKatanA所有原因を特定し、実経路の失敗回帰を追加して修正する。
- [ ] 1.3 修正後の画像表示・反復・正常終了と既存メモリ条件を検証する。
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
- [ ] 4.2 公開済み限定修正・今回修正・未解決・上流handoffをIssue/boardへ証拠とともに反映する。

## 5. Final Verification & Release Work

- [x] 5.1 docs/coding-rules.ja.mdとself-reviewで差分を確認する。
- [x] 5.2 Markdownとコードを整形し、対象回帰を回収する。
- [ ] 5.3 関心別通常commitの後、通常push hook一回を正式ゲートとし、重い同検査を手動で重ねない。
- [ ] 5.4 Draft PR→全reviewページ/threads取得→P0/P1修正と個別reply/resolve→fresh確認を完了する。
- [ ] 5.5 Ready後のcurrent required checksを回収し、不合格は修正する。
- [ ] 5.6 通常merge→Actions公開→全5asset/checksum・配布自動検査→Issue更新→OpenSpec archive→安全なbranch/worktree後処理を実施する。

## KatanA CLI Entry Point

OpenSpecはリポジトリの`./scripts/openspec`を使用する。既存品質基準・公開済みregistry依存を維持する。

## 現在の対象検証

- 画像参照分類: 旧実装3件失敗→修正後3件成功。`tmp/v0.22.44-prep/image-reference-{red,green}.log`。PNG30秒待ち全体の原因確定とは区別する。
- 共有Loading: 5件、実callerのstandalone/inline配置1件成功。`loading-widget-final.log`、`loading-caller-final.log`。
- 左右キー: `keyboard-final.log`で6件成功。正式ASTで検出した規約違反は修正後に全30件成功。
- Memory: `worker-memory-tests.log`で12件成功。HTML surface32件成功。初期化直後の実HTML終了回帰を含め33件成功。実HTML/Officeのclose検査も元5秒条件で成功。
- 正式AST: 初回21件成功/2件失敗→規約準拠で修正後、全30件成功。ゲート変更なし。

- v0.22.44表記と両lock更新後の対象検査: Loading5件、実caller1件、画像参照分類3件成功。最終pushゲート・公開は別工程。
