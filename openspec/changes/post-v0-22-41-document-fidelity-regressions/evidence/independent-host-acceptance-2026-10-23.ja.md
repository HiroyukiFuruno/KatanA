# 現行graphの独立した原本・PDFテーマ受入

## 結論と対象

通常検査だけでは未実施だった `external-fixture-acceptance` の全追加5テストを実行し、すべて成功した。製品コード、期限、判定、既存ignoreは変更していない。対象は既存 `release/v0.22.42` の署名HEAD `9d5898721e2d04a787add283209ee9d58841691d`、tree `39d0b1f020f45e41cbbe0506cf989bd687e65687`。各実行の開始・終了でHEAD・tree・tracked clean・両lockの一致を確認した。

root lock SHA256は `927fac6b5c168bf578c0e787ea7ea17f0bd0c079a9cd13557a4c7134836c51dc`、撮影lockは `a0f0abdedb6751225121368cb633c0a0238e8b262d42497b3e185c50f9c5b5d5`。本人のアプリ・設定・Terms、master、兄弟repoを操作していない。

## 原本Office・PPTX操作・HTML

元の `just test-document-fixtures` を専用targetで実行した。session22823のactual exitは0、終了UTCは `2026-10-23T09:08:24Z`。前提の実契約31件と、以下の原本テスト3件が成功した。テスト用release workerは元scriptがbuildし、絶対pathで明示している。

worker SHA256: `87883abf95f94709e1443b6ceedad65819d4e8a26b8f7b2e1bfefd2831cf9a1a`。release buildは32分20秒、external-feature test buildは8分54秒。別repoの並行buildも観測したため、所要時間を製品性能や短縮効果へ換算しない。

| 原本 | 形式・item数 | 初回frame ms |
| --- | --- | ---: |
| `libre-chat_vs_loom.pptx` | Pptx・9 | 6159 |
| `librechat_entra_oidc_vs_saml.pptx` | Pptx・11 | 1777 |
| `shopchannel_analysis.xlsx` | Xlsx・5 | 26 |
| `【681303_発注依頼書】…モードコム.xlsx` | Xlsx・1 | 75 |
| `【チャット型AIエージェント】…PoC提案_r3_20260616.pptx` | Pptx・26 | 4580 |
| `視聴購入data_セグ別201805_20260803.xlsx` | Xlsx・1 | 8019 |

全6件active_index=0、PptxはPage、XlsxはGrid。Officeテストは20.71秒。PPTX原本3件のready/next/stale_rejected/closeは各true、テスト10.72秒。HTML原本は `#s15`、scroll_y=3048.824、content_height=9539、first_frame_ms=10985、sticky_sidebar_ratio=0.9352、テスト10.99秒。原本6件とHTMLのSHAを実行中と終了後に照合して一致した。

これは元scriptのin-process/実worker受入であり、本人GUIの無反応解消・Chrome完全一致・高速化・アプリ正常終了・全メモリ予算の証明ではない。

## PDFの4テーマ

同target・同featureで元の `just test-pdf-theme` を直列実行した。session88363 actual exit0、2テスト成功/1320filtered、47.01秒、終了UTC `2026-10-23T09:09:37Z`。明示したPopplerは26.05.0、実行ファイルSHA256は `de772e88ab9977ccde25def9b403bf42675d75f5dd82b19fbd7d8123ad183159`。

`custom-dark`、`custom-light`、`preset-dark`、`preset-light` の4件それぞれで実PDFを出力し、120dpiのPNGへ変換した。元の背景/文字画素・4.5:1以上のコントラスト・表の連続色領域・図のaccent・背景色への置換拒否をすべて検査した。画面や設定値だけの確認ではない。

実PDF4点・PNG4点は `tmp/issue312-pdf-theme-2026-10-13/85237-1792746530208120000/` に保持した。ディレクトリ名の旧日付を今回の実行日と誤認しない。raw、原本hash照合、計12証跡のSHA256は `tmp/oct23-external-acceptance-retained.sha256` で一致を確認した。

## 通常検査との照合とIssue状態

現行normal inventoryのUI libは1317件、今回の実binary listingは1322件。名前集合の差分は原本Office・PPTX操作・HTMLの3件と、PDF実画素・背景同色拒否の2件だけで、通常側だけの名前は0件だった。追加5件は今回すべて実行した。listing比較は欠測検査の発見用であり、feature間の実行同等性や通常検査省略の認可ではない。

Open15件の元DoDとOpenSpec未完66項目を照合した。未読込Markdown診断、実Bold/Italic描画、取消済みFIFO、HTML reload/scrollは既存sourceと現normalの名前付きPASSを確認し、未実装と誤分類しない。上流の再起動render cacheは既定の次期範囲を維持する。今回発見した独立検証の欠測は上記2入口で補った。

- #345/#349: 今回のOffice6件・PPTX原本3件の操作を追加証拠とする。実packaged/全OS/clean-machine/本人受入・公開は未完。
- #312: 今回の実PDF4テーマを追加証拠とする。実packaged/現required CI・公開は未完。
- #339/#350: KDV #69の安全公開版採用・全供給網の成功は別の未完。TLS更新や今回の受入で旧推移ZIP advisoryを解消したことにはしない。
- #343: Windows cache配線と観測は実装済み、native cold/warm・全体比較は未完。trusted artifact結果再利用は未実装だが、元の段階評価と「同等性を証明できない検査は維持」に従い既存検査を残す。独断で別の証明基盤を追加したり、作者PASS fileで代用したりしない。
- #335/#338/#336: 公開OID後のactivation/cleanup、KDV ownerの採点/参照採用、KLE公開版採用は、それぞれ別の未完条件。その他implemented Issueも公開・最終配布受入の完了ではない。

Windows native、全対象packaged/clean-machine、本人受入/Terms、通常push/current review/Ready/required CI、公開・Issue closure・archive・cleanupは未完のまま保持する。この記録をKatanA全DoDや全Issue完了へ読み替えない。

## 生証跡

- `tmp/oct23-current-original-fixtures.sh` / `.log`
- `tmp/oct23-current-pdf-theme.sh` / `.log`
- `tmp/oct23-original-input-before.sha256` / `tmp/oct23-original-input-after.log`
- `tmp/oct23-external-fixture-test-list.txt` / `tmp/oct23-external-only-test-names.txt`
- `tmp/oct23-external-acceptance-retained.sha256`

以前の現graph全品質結果は [通常・coverage・クロス・撮影検証](current-graph-cross-and-screenshot-validation-2026-10-23.ja.md) に保持し、今回重複実行していない。
