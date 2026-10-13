# #338 固定sourceの正規producer実行

対象: https://github.com/HiroyukiFuruno/KatanA/issues/338

## 実行とsource

公式 `generate-sample-diagrams-reference.sh` を既存release/v0.22.42 worktreeでJOBS1実行した。session30723はactualexit0。開始からcapture後のsource検査までHEAD `458fabcf8ad79cc8f4f3d5abf2f27011113be268` とclean treeを維持し、別担当の編集・重build競合を避けた。本人アプリやTerms承認を操作していない。

原ログ: `tmp/issue338-458fabcf-capture-2026-10-14.log`。候補とraw: `tmp/issue338-458fabcf-capture-2026-10-14/`。rendererはin-process実Appであり、配布実行物の受入とは異なる。

| 入力・実行物・出力 | SHA256 |
|---|---|
| Cargo.lock | `31e2f449f83ee4d4efbd5ad1a52209d817db9fd643ae7257cd8805127a5ee7bd` |
| screenshot Cargo.lock | `8fd21cb4b180c862a02203337ac3301de921ef3214ee8b20f2be0b208f3189f4` |
| optimized Office worker | `90d5f0cc7d88c4bdf41b24342abd09d5537fe094f8f0f5adfff1558034df74b9` |
| optimized screenshot runner | `5046eee13e82f6030060961866ee15605961d6816d11454a8eaccc4885a79bd8` |
| normalized candidate | `2f998a05bd1663d62a0378bc83f43c8608c80373ee624bb6756e9909aabb8c5b` |

## 実geometry・同frame検査

frame1223とcompleted frame1223が一致し、Markdown区画17。diagram/image/copy/selection/hover/active-rangeの各controls counterは全0、active editor lineはnull、pointerはcontent外。scroll_y=0。

実full image2565×4774。physical cropはx88/y268/width2374/height4450、logical content1187×2225。指定crop条件を検査し、Box正規化後1280×2400を検査した。目視は数値検査の補助だけである。

## 限界・未完

旧ad4生成とcandidateのpixel hashは同一だったが、旧実行のsource driftを正当化しない。今回の固定source実行が新たなprovenance証拠であり、旧ログを固定sourceの証拠に差し替えていない。

state diagramのCommandNotFound文字が欠ける表示、unsupported CSS/SVG filter/shape警告は残る。独立visual/semantic/interaction/performance採点、95点、全OS配布版、原本Office/PDF・通常終了/RSS受入とは別であり、完了と偽らない。`assets/reference`を上書きせず、候補の参照採用・独立評価は未完である。#338とrelease全体を閉じない。
