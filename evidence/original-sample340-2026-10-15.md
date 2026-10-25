# Issue340 原本sampleのexportとhost本文回帰

対象: https://github.com/HiroyukiFuruno/KatanA/issues/340

## 固定sourceの実export

既存公式`generate-sample-export-reference.sh`をsession97005で一回実行し、actual exit 0を回収した。開始・終了の`source_identity.py begin/verify`はHEAD `3c6b2c79c2ac90dae9142f7dbf732ba61f7fc9c2`とclean treeを確認。ビルド待ちを理由に旧runnerへ切り替えず、本人アプリ・設定・Termsを操作していない。

原ログ: `tmp/issue340-sample-export-2026-10-15.log`。
候補: `tmp/issue340-sample-export-2026-10-15/sample.png`。

| Identity | SHA256 |
| --- | --- |
| 元assets/fixtures/sample.md（Issue記載値と一致） | `489360a81d60af20d67f8ea47e251732a194983431ea1ab261627490cc9009e1` |
| Cargo.lock | `a6035ec9364f734fd132d58ca909cfed0b9c9e13c25ffc894165d79060db2af4` |
| scripts/screenshot/Cargo.lock | `ef7151771258e762d1962647ccf4625dd668d48ec3a5246b60bcb48734955896` |
| 今回のoptimized runner | `d911b0fcece10d85bdce38373f2b418a23a998bd22a3956687686c69be7e8670` |
| 新規PNG（1280×19282） | `067326d731a000ea9a7b615ef891fc96b1425871059dfcf1eaefcf985edda0e7` |

全5steps/quit完了。export stepは5113890416ns。これはビルド時間やアプリ起動時間ではない。

## 描画境界を分けた観測

PNG候補の先頭1280×2600 cropを検査。`#dddddd`の連結領域は128×128、左上576/1594、面積15639pixelsで、原SVGの128×128 rectに対応する。OCRと目視でもsection1.6の見出し・説明・画像を確認したが、OCRや画像だけを本文非漏出の自動証明にしていない。`assets/reference`を上書きせず、候補を参照へ採用していない。

同実行のhost previewには、元URIの`xmlns=%22<http://www.w3.org/2000/svg%22>`についてSVG rasterizationの構文失敗警告がある。PNG exportはKDV export adapterの別経路であり、export画像の成功をhost画像表示の成功へ読み替えない。SVG/CSSのunsupported警告も残る。元入力の不正namespaceを補正・差し替えしていない。

## 恒久host回帰

`html_export.rs`に元sampleを`include_str!`し、section1.6を境界で抽出する回帰を追加した。実egui TextShapeから見出しと説明文の存在を肯定assertし、その同じ描画出力に`%3Csvg`/`%3Crect`/`%3Ctext`等がないことをassertする。空の描画で否定assertだけが通ることを防ぐ。mock、snapshot更新、元fixture変更、製品のtestport追加はない。

- 対象新test: 33088 actual exit 0、1PASS/0.15秒（RTK集約）。最初の別ログはJustの変数指定位置誤りで未実行であり、製品REDに数えない。
- 関連host3件: 72142 actual exit 0、3PASS/2.81秒。raw `tmp/issue340-original-header-related-2026-10-15.log`。
- 正式AST: 61012 actual exit 0、23PASS/23.30秒。raw `tmp/issue340-original-header-ast-target-audit-2026-10-15.log`。最初の圧縮ログだけで件数を推測せず、生出力を回収した。
- 対象test-inclusive strict Clippy: 45823 actual exit 0、3.61秒。
- 公式format: 初回6282 actual exit 1をformatterで修正し、54071 actual exit 0。`git diff --check`成功。

既存parser修復に対する原本coverage拡張であり、新testが旧製品sourceでREDだったとは主張しない。独立95点、全OS配布版、正常終了/RSS、Issue340全DoD・公開の完了ではない。hostの画像警告とpackaged受入を残してIssueを閉じない。
