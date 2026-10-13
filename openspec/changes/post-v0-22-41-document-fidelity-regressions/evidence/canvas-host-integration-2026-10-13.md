# #258 読み取り専用Canvas host統合

## 正式履歴と対象

- core model: dd38236e724a0606f888c97a13ea58cbf94c2956。
- host/UI: dec02e313f7b9fda0e1b19604f8f1cf6d10e7221、通常commit81451 actualexit0・署名G。公開pushはこの時点で未完。
- workspace/file-open対応、既存reference文書経路、固定1worker/64queue、worker-only読み込み・SHA256 revision、typed source/JSON/geometry errors、世代取消、read-only保存/編集境界を接続した。
- カード・group・edge・loop/指定辺・ラベル・read-only accessibility、Scene pan/pinch/zoom/Fit、形式別menu、10localeを接続。
- 関連責務の境界をdocument preview policy/Fullscreen UIへ分けた。FullScreenの既存所有/監視/元fullscreen復元は変更しない。

## 実検査

- 初期描画の実RED81206: 30PASS7FAIL。routing/label/accessibility/rebasingを修復。初compile/FullOutput lifecycle失敗は別ログとして保持。
- reference placeholder/readonly backendの実RED89390/92408を修復し、7194で48PASS。
- 背景open表示mode変更32763、auto unchanged refresh pan喪失8902はactual101。activate限定とnonforce再読込/SHA一致surface保持で修復した。
- 追加host valid→validは新本文/geometry・実Text shapes・viewport/描画完了、valid→invalid→元bytesは型付き失敗描画から同SHA Readyへの復旧を検証。
- 最終22229 aggregateactualexit0: Canvas57PASS、既存fullscreen23PASS、公式AST23PASS、strict UI all-target/external-feature Clippy、fmt-check/diff-check。raw `tmp/canvas-current-final-ui-2026-10-13.log`、`tmp/canvas-existing-fullscreen-final-2026-10-13.log`、`tmp/canvas-pdf-ast-current-success-2026-10-13.log`、`tmp/canvas-pdf-strict-current-success-2026-10-13.log`。
- 旧61933はpipefail漏れで独立cargoexitを保存していないため、最終のpipefail付き22229を正本とする。

## 規約と制限

- 枠strokeは標準Shape/Painter経路へ統一し、rect/radius/width/clip/StrokeKind::Insideは同値。egui0.36.2 Painter::rect_strokeは同じRectShapeをaddする実装であり、描画を縮小して基準を変えない。
- 明示icon fill/AlignCenter/i18n labelと診断metadataの責務分離、数値constant、実行文字列Unicode同値を適用。新allow/除外/品質基準変更無し。
- テキストカードはliteral sourceであり完全なMarkdown整形ではない。ファイル参照はラベルのみ、URL/背景は未対応として表示する。浮動小数点coordinate literalは丸めず拒否し、任意のJSON Canvas/Obsidian文書への完全対応とは主張しない。
- 既存function-length lintのstart/end同span負債は別未対応として記録。AST合格だけで既存全関数30行条件を証明したとはしない。
- 本人のアプリ操作/Terms代行・上流編集・新worktree・通常stash・path/git override無し。
- full current-HEAD gates/review/allOS packaged受入/公開は未完。componentの成功をリリース完了や95点へ代用しない。
