## Context

公開版はv0.22.46。公式binaryの独立profileで、group popupの文字入力・Return・Cancel・Escが反応しないREDを確認した。同じprofileのファイル検索では入力できた。既存のGroupHeader単体検査はMainPanels/InteractionFacadeの本番経路を通らず、成功を本番動作へ一般化できない。原因候補のpopup所有UIと背景の入力抑制を、full KatanaAppで分離して検証する。

Open IssueのKatanA側残件は#403/#408/#409/#343/#372/#386/#339/#338、既存回帰維持と記録整理は#385/#354/#388。PR405/#411のActions更新とpin契約修正はcheckpoint b824d286にある。CI-only版番号例外は独立レビューで問題が見つかったため採用せず、通常の隣接patchへ統合する。

## Goals / Non-Goals

**Goals:**

- 操作退行を最優先に直し、実メニューから名前あり/空欄Enter、Cancel/Esc、作成後closeを自動入力と配布版で検査する。
- KatanA内で完結する残件を一回のv0.22.47に統合し、既存build/test/lint/coverage/CodeQL/供給網/文書QAを維持する。
- 上流待ち部分・既存公開済み部分・未検証部分を正確に分け、担当範囲を証跡付きでCloseする。

**Non-Goals:**

- KLE未公開APIやKDVの文書単位cancel・旧ZIP解消を他repoへ実装しない。
- 以前の未再現PNG遅延を新しい再現要求・本人確認待ちへ変えず、解決済みとも主張しない。
- 名前必須化、色だけの一意制約、新しい既定名の推測、既存groupの自動削除・改名・色変更を行わない。

## Decisions

1. **本番入力経路を回帰境界にする。** bare GroupHeaderの成功は補助に留め、full KatanaAppのmain panels・foreground判定・実pointer/contextmenu・別frame text/keyを通す。popup所有UIの操作を許し、他modalや背景への入力保護を維持する。原因はRED→修正→GREENで確定し、候補patchだけで完了扱いしない。
2. **更新helperの実失敗を境界にする。** 両UI導線でspawn失敗を表示して継続し、成功時だけ既存終了/更新動作を進める。実processの失敗とtemp lifecycleを検証し、テスト用に商用APIを変えない。
3. **公開/CIの安全条件を維持する。** remote cleanupはfalse/欠落を既定にして明示trueだけ既存のexact target/SHA/provenance/owner/hook条件を通す。CI再利用は完全な入力同等性と信頼できる成功runに結び、拒否ケースとcold/warm実測を残す。証拠がない条件を省略しない。
4. **source結合を検証する。** manual generated bumpは実Git/processと実GitHub metadata形状で複数commit chain・版ファイルだけの変更を検証する。同source dispatch/PRclosed/v42固定契約を維持する。今回の実公開receiptは実際の経路を明記し、no-bump dispatchをgenerated bumpの実公開実証に読み替えない。未達はそのまま記録する。
5. **hostと上流の責務を分ける。** UI close/切替/再open/別tab保持とKatanA所有環境整備はここで実行する。KDV worker固有cancelや未公開依存の修正は既存上流Issueへ残し、他のKatanA作業を止めない。
6. **一回の公開へ集約する。** release/v0.22.47でPR405の差分と今回の修正をまとめ、既存PRの最終title/bodyを実内容へ更新する。通常push→Draft review→指摘対応→Ready→必須CI→merge後、masterのv0.22.47をreuse_existing_version=trueの明示dispatchで一度だけ公開する。PR headはDependabotなので自動release/v経路には一致しない。既存成果物やtagを上書きしない。
7. **名前と色の組を一つの変更として扱う。** 空名を許可し、同一workspaceの別groupとの組の重複だけを拒否する。作成・rename・recolor・直接actionで共通検証し、自分自身は除外する。新規popupは未保存draftとし、確定前はmember移動・cleanup・永続化を行わない。既存編集も名前/色を一括確定し、失敗時は状態を変えず編集を継続する。Cancel/Escはdraftを破棄する。有効な名前付き直接作成は維持し、競合時は修正可能なdraftを開く。既存legacy重複はload時に破壊せず、無変更は保存せず終了できる。空白だけの名前は空名として比較し、それ以外のcase/Unicode/名前の値を勝手に正規化しない。色はRGBA値で比較する。
8. **実アプリの画像をREADMEへ掲載する。** 公開fixture・隔離profileを既存capture runnerで表示し、tree/editor/previewが分かる実描画をrootの日英READMEに掲載する。最終sourceと入力・画像hashを記録し、私有文書や推測した画面を使わない。
9. **分割表示の背景を残余領域へ限定する。** 先行panelはroot UIのcursorを狭めてもmax_rectを狭めないため、共有UIを受けるsplit背景にmax_rectを使わない。残余content矩形だけを塗り、top/side/statusを覆わない。実panelsを通した縦横・light/darkの境界回帰と、実App描画で再確認する。待機時間の追加・画像加工で隠さない。

## Risks / Trade-offs

- **popup自身を含む全体無効化やfocus初期化が残る** → full-Appのfield enabled/focus/valueと確認・Cancel結果を別framesで検査し、native実操作で補完する。
- **共有checkoutの並列編集が混在する** → group、updater、CI/release、環境の書込範囲を分け、branch操作/commit/push/全体formatは親が統合する。新worktreeは作らない。
- **依存/formatterがlockを更新する** → 版・checksum・実API・MSRV・全gateを確認して正式採否を決める。PR405作業で発生したglam/zip候補も比較記録してこのpatchで解決する。
- **他repoや本人確認がCloseを止める** → 真のupstream依存だけを明示し、本人操作は独立した非blocking follow-upとする。未実行のKatanA作業を外部待ちにしない。

## Migration Plan

保存済みgroup/tab/historyと設定を保持する。公開前candidateと実公開物の操作検査は独立profileで行い、元設定は変更しない。文書受入・reference captureはexact source/version/input/hashと結び、公開後記録・spec同期・Issue更新・local branch整理まで完了する。remote branchesは保持する。

## Open Questions

実測で確定する事項は、group popupのdisable/focusの直接原因、KatanA所有imageのprovenance、CI入力同等性と実測差、既存原本文書の受入残差。いずれも調査・自動検査タスクとして扱い、推測で合格にしない。
