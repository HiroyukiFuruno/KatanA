# #312 実PDFの現行テーマ検証

## 実行経路

- アプリの既存テーマ適用入口から `DiagramThemeSnapshot::current()`、既存 `ExportPoll::perform_tool_export` を通してPDFを生成した。
- SettingsServiceとCacheは既存in-memory実装で隔離。本人の起動中アプリ、設定ファイル、Terms承認は操作していない。
- ASCII本文・表・明示色Mermaidの同じ入力をcustom-dark/custom-light/preset-dark/preset-lightへ出力し、明示Poppler pdftoppmで各PDFの第一ページを120dpi PNGへ変換した。
- 実行入口は `just test-pdf-theme`。`KATANA_PDFTOPPM`が必須で、未設定・存在しない実行物は失敗する。既存 `external-fixture-acceptance` featureを使用し、通常検査の削除やrequired CIの代替はしない。

## 結果

- 初回 session79165 actualexit0、exact1PASS/0FAIL/0ignored、59.75秒。4テーマの背景・本文コントラスト4.5:1以上・表header/stripe/borderの色領域・明示Mermaid色が全て成功。
- 検証実装の背景同色境界で減算underflowを新回帰18044 actual101で検出。checked_subへ補正し、final95750 actualexit0、2PASS/0FAIL/0ignored、58.19秒。テーマを背景へ置換した負検査が同じ色領域検証器で拒否される。
- 生ログ: `tmp/issue312-real-pdf-current-first-2026-10-13.log`、`tmp/issue312-verifier-equal-background-red-2026-10-13.log`、`tmp/issue312-real-pdf-verifier-final-2026-10-13.log`。
- 初回出力4ページを実際に画像として確認。本文・表・図は読み取れ、ページ背景と文字はテーマごとに異なる。これは数値検査の補助であり、スクリーンショットだけの完了判定ではない。
- 保持artifact: `tmp/issue312-pdf-theme-2026-10-13/38770-1791899673482124000/`。final検査は別runディレクトリを生成する。

## 限界と残作業

- テーマ色がPDFへ届くことを検証した。全ドキュメントの文字重なり、全ページの意味的位置、KDVの文字組み精度、全OS配布や本人実機受入の証明ではない。
- 表の連続色領域と負検査はあるが、全ページのsemantic ROI評価とは主張しない。背景同色の表tokenは追加semantic検証が必要として拒否する。
- 現sourceの静的検査・自己review・正式commit/pushとIssue整理は未完。KDV側の文字重なり問題を#312へ混同して閉じない。
