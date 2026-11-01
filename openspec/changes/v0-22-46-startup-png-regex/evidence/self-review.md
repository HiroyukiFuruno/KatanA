# Self-Review: v0.22.46

## 確認結果

- 目的: 起動不能を優先し、同一パッチ版にフィルター検証の修正を含める。
- 画像: 単独/Markdown内の両upload経路を共通化し実contextの上限を使用。元寸法・alpha・cache世代/背景の無効化を保持。
- 入力: query/mode/case/word/否定のpattern生成を共通化。不正regex時の全表示は維持。
- 新規Rustファイルは200行以内。既存テストの期待値を弱めず、必要なtexture wrapperとsource寸法へ更新。
- serialized設定や元ファイルを変更しない。別repo編集/未公開依存/新規worktreeなし。
- 依存は両lockの互換2件だけを採用。破壊的更新と上流exact競合の実調査を保存。
- 独立した画像レビューでP0/P1/P2なし。releaseレビューのP2（事前検証3.2をfuture許可する条件）を修正し、再確認で残指摘なし。

## 指摘と対応

- 追加画像テストの色指定がAST規約で拒否されたため、テーマ背景とRGBA画像入力fixtureへ修正。規約や除外設定は変更しない。
- 配布時の上限超過はGPUでfatalになるため、unitのupload寸法に加えnative保存タブ復元でも検査した。
- ユーザーの実入力 *.png の不一致を再現し通常検索のwildcard照合へ修正。標準regexの仕様は維持し、regex modeでは無効な *.png をエラーとして扱う。

- preflightで停止した既存safe ZIP edgeのhost版登録をexact v0.22.46へ追加。未知の版は拒否、旧脆弱版の例外・期限は不変、core source不変と実3manifest supply-chain成功を確認し、独立レビューで指摘なし。

## 判定

差分レビューはPASS。通常commit/push、必須CI、公開後asset確認はtasks.mdで実結果を回収し、未実施を成功扱いしない。

## Draft後の再レビュー

CodexのWindows path P2と独立reviewのUnix filename P2を採用した。OSのMAIN_SEPARATORでwildcard生成だけを切り替え、通常literal/regexの既存挙動を保持。28回帰が成功し、High再reviewで残指摘なし。GitHub threadは修正push後に個別返信・resolveして再取得する。
