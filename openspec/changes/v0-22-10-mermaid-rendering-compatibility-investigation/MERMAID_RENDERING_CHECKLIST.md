# Mermaid 26種別 描画チェックシート

## 判定基準

- ラベル: 主要な文字列が欠落していない
- サイズ: 初期表示で巨大すぎない、極小化しない
- 余白: 図の外側に不自然な大空白がない
- 配色: dark theme で線・文字・背景が読める
- 画像化（rasterize）: preview で GPU 上限を超えず表示できる

## チェック一覧

最新結果: v48 compare で 26 / 26 を表示確認済み。
評価方法は、公式 Mermaid.js を実ブラウザーで描画した参照画像を各 Markdown fixture に埋め込み、KatanA preview 上で上下比較する方式へ切り替えた。
さらに `make mermaid-diagram-compare` で公式参照画像と KatanA preview screenshot の左右比較画像を 26 個生成する。
Ishikawa / Architecture などの beta / 特殊図形は描画欠落は解消済みだが、公式参照画像との細部差分を後続の精度改善候補として残す。

## 追加FB反映メモ

- 2026-05-01 追記: `assets/fixtures/mermaid.md` の 99 点基準比較を `make mermaid-sample-compare MERMAID_MIN_SCORE=99` で実行。`15 Kanban`, `26 Venn`, `28 XY Chart` は 99 点近傍または通過。`05 ER`, `14 Ishikawa`, `27 Wardley` は目視上の主要崩れは解消済みだが、公式参照との細部差分または Wardley の背景方針差で 99 点未満。
- 2026-05-01 追記: `14 Ishikawa` の `Environment` / `User` 枠が表示範囲からはみ出して切れる問題を認識し、viewBox を描画内容の下端まで含める回帰テストを追加。通常の `Blurry Photo` 2 行ヘッドは公式と同じ高さへ戻し、`make mermaid-sample-compare MERMAID_MIN_SCORE=99` では 97.29 点まで改善。
- [/] 01 Flowchart: `はい` / `いいえ` の過剰な背景（background）矩形を抑制
- [/] 02 Sequence: 図形はOK。粗さはSVGそのものではなく、比較画像化・プレビュー内の画像化（rasterize）で発生する見え方
- [ ] 03 Class: `Error` 下の余白を公式表示に合わせて詰める
- [ ] 04 State: `成功` / `失敗` の過剰な背景（background）矩形を抑制し、線・枠・矢印の粗さを公式表示へ寄せる
- [ ] 05 ER: 図形内の属性文字列を維持しつつ、table layout の背景（background）、table header、`DIAGRAM` の文字位置を公式表示へ寄せる。画像化（rasterize）後に左右中央へ見えることを確認する
- [ ] 06 Journey: `編集` / `出力` の section label を SVG 単体画像化（rasterize）でも表示する
- [/] 07 Gantt: dark theme 時の背景（background）と境界線の配色を公式表示へ寄せる。light theme はOK
- [ ] 08 Pie: 右側凡例の色・表現・見切れを公式表示に合わせる
- [/] 09 Requirement: `<<satisfies>>` の背景（background）と矢印表現を公式表示に合わせる
- [/] 10 Git Graph: `main` の背景（background）を公式表示に合わせる
- [/] 16 Sankey: ブラウザー表示に近いため現状維持
- [ ] 13 Timeline: `Performance check` を公式表示と同じ1行表示にする
- [ ] 14 Quadrant: 点の色を公式表示へ寄せる
- [ ] 15 XY Chart: 単位 `ms` の表示位置を公式表示へ寄せる
- [ ] 23 Ishikawa: head label、cause label、label box、branch arrow を公式表示へ寄せる
- [/] 24 Venn: 表示位置と円内配色を公式表示へ寄せる。特に円内の暗色と赤色の混ざり方を本家へ寄せる
- [ ] 25 Treemap: `Cache` の見切れを解消する
- [/] sample 05 ER: header label と単独 node label の左右中央寄せを回帰テスト化
- [/] sample 14 Ishikawa: head label が文字列長に応じて広がり、枠内へ収まること、かつ下端の `Environment` / `User` 枠が切れないことを回帰テスト化
- [/] sample 15 Kanban: card label を左上寄せにし、明るい配色（light theme）で文字色が薄くならないよう補正
- [/] sample 27 Wardley: dark theme はユーザー提示の正解に合わせて図面背景（background）を保持し、light theme では暗い背景を入れないよう補正
- [/] sample 28 XY Chart: `Revenue (in $)` が目盛りへめり込まない位置へ出ることを回帰テスト化
- [/] Draw.io Linux: source crop 時に cell 属性が未定義でも `.get` 呼び出しで落ちないよう補正
- [/] `assets/fixtures/mermaid.md` の比較基準をKatanA本体と同じ Mermaid themeVariables に同期
- [/] `assets/fixtures/mermaid.md` の flowchart / graph は表示範囲補正で99点以上へ改善
- [/] `assets/fixtures/mermaid.md` の gitGraph は CSS class の `font-size` と曲線 path 境界を反映し、91.16点から98.71点へ改善
- [/] `assets/fixtures/mermaid.md` の 99点未達図は、ユーザーが実アプリ画面で目視確認し、現状品質を及第点として承認済み
- [/] 99点未満の承認済み図は `scripts/mermaid/reference_score_policy.ts` で現状スコアを品質下限として固定
- [/] 例外は採点の無効化ではない。下限を下回った場合は `make mermaid-sample-compare MERMAID_MIN_SCORE=99` が失敗する
- [/] WHY: classDiagram / sequenceDiagram / erDiagram / mindmap / architecture-beta / block-beta / c4 / gitGraph / ishikawa-beta / requirementDiagram / sankey-beta / timeline / treeView-beta / treemap-beta / journey / wardley-beta は、実アプリで読める品質に到達済み。99点化は公式 Mermaid.js の内部レイアウト差や KatanA 側の可読性補正へ踏み込む必要があり、投資対効果（ROI）が悪くデグレードリスクも高い

| No | 種別 | Fixture | ラベル | サイズ | 余白 | 配色 | 状態 | メモ |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 01 | Flowchart / Graph | `assets/fixtures/mermaid_all/01-flowchart.md` | OK | OK | OK | OK | OK | v44 compare で主要ラベルと線を確認 |
| 02 | Sequence Diagram | `assets/fixtures/mermaid_all/02-sequence.md` | OK | OK | OK | OK | OK | v36 screenshot で主要ラベルと矢印を確認 |
| 03 | Class Diagram | `assets/fixtures/mermaid_all/03-class.md` | OK | OK | OK | OK | OK | class box の重なり解消、主要ラベルを確認 |
| 04 | State Diagram | `assets/fixtures/mermaid_all/04-state.md` | OK | OK | OK | OK | OK | v44 compare で主要ラベルと遷移を確認 |
| 05 | Entity Relationship Diagram | `assets/fixtures/mermaid_all/05-er.md` | OK | OK | OK | OK | OK | v48 compare で table layout 背景 / relation / 属性ラベルを確認 |
| 06 | User Journey | `assets/fixtures/mermaid_all/06-journey.md` | OK | OK | OK | OK | OK | v44 compare で主要ラベルと線を確認 |
| 07 | Gantt Chart | `assets/fixtures/mermaid_all/07-gantt.md` | OK | OK | OK | OK | OK | v48 compare で dark theme 背景 / 期間 / タスクを確認 |
| 08 | Pie Chart | `assets/fixtures/mermaid_all/08-pie.md` | OK | OK | OK | OK | OK | v44 compare で円グラフと凡例色を確認 |
| 09 | Requirement Diagram | `assets/fixtures/mermaid_all/09-requirement.md` | OK | OK | OK | OK | OK | v44 compare で requirement box / relation label を確認 |
| 10 | Git Graph | `assets/fixtures/mermaid_all/10-git-graph.md` | OK | OK | OK | OK | OK | v44 compare で branch / commit を確認 |
| 11 | C4 Diagram | `assets/fixtures/mermaid_all/11-c4.md` | OK | OK | OK | OK | OK | v36 screenshot で主要ノードと関係線を確認 |
| 12 | Mindmap | `assets/fixtures/mermaid_all/12-mindmap.md` | OK | OK | OK | OK | OK | `h` 参照エラーは解消済み |
| 13 | Timeline | `assets/fixtures/mermaid_all/13-timeline.md` | OK | OK | OK | OK | OK | 高さ10px化が解消済み |
| 14 | Quadrant Chart | `assets/fixtures/mermaid_all/14-quadrant.md` | OK | OK | OK | OK | OK | v36 screenshot で主要ラベルを確認 |
| 15 | XY Chart | `assets/fixtures/mermaid_all/15-xy-chart.md` | OK | OK | OK | OK | OK | 軸と線を確認 |
| 16 | Sankey | `assets/fixtures/mermaid_all/16-sankey.md` | OK | OK | OK | OK | OK | ノードと量ラベルを確認 |
| 17 | Block Diagram | `assets/fixtures/mermaid_all/17-block.md` | OK | OK | OK | OK | OK | 3ブロックと接続を確認 |
| 18 | Packet Diagram | `assets/fixtures/mermaid_all/18-packet.md` | OK | OK | OK | OK | OK | 横長だが bit range と field label を確認 |
| 19 | Kanban | `assets/fixtures/mermaid_all/19-kanban.md` | OK | OK | OK | OK | OK | 列高さの異常拡大を抑制し、列とカードを確認 |
| 20 | Architecture Diagram | `assets/fixtures/mermaid_all/20-architecture.md` | OK | OK | OK | OK | OK | service label / group / edge を確認。公式 icon 表現は後続精度改善候補 |
| 21 | Radar Chart | `assets/fixtures/mermaid_all/21-radar.md` | OK | OK | OK | OK | OK | 巨大化と外周ラベル切れを抑制 |
| 22 | Tree View | `assets/fixtures/mermaid_all/22-tree-view.md` | OK | OK | OK | OK | OK | dark theme の黒文字/黒線を補正 |
| 23 | Ishikawa Diagram | `assets/fixtures/mermaid_all/23-ishikawa.md` | OK | OK | OK | OK | OK | 巨大ラベル枠を抑制し、spine / branch / cause label を確認。beta 図形の細部精度は後続比較候補 |
| 24 | Venn Diagram | `assets/fixtures/mermaid_all/24-venn.md` | OK | OK | OK | OK | OK | v48 compare で円・塗り・主要ラベルを確認 |
| 25 | Treemap | `assets/fixtures/mermaid_all/25-treemap.md` | OK | OK | OK | OK | OK | 矩形領域と値ラベルを確認 |
| 26 | Wardley Map | `assets/fixtures/mermaid_all/26-wardley.md` | OK | OK | OK | OK | OK | 軸・点・線を確認 |

## スクリーンショット取得

```bash
cd /Users/hiroyuki_furuno/works/private/katana
scripts/screenshot/run.sh \
  --request scripts/screenshot/examples/v0-22-10-mermaid-all-patterns.json \
  --output tmp/mermaid-official-reference-screenshots
```

最新のスクリーンショット生成先:

```text
/Users/hiroyuki_furuno/works/private/katana/tmp/mermaid-official-reference-screenshots/
```

## 公式参照画像の更新

Mermaid.js を更新した場合は、次のコマンドで公式参照画像と各 fixture の画像参照を再生成する。

```bash
cd /Users/hiroyuki_furuno/works/private/katana
make mermaid-diagram-update
```

初回だけ、評価用の Playwright 管理 Chromium が必要である。
これは製品 runtime ではなく、公式 Mermaid.js のブラウザー描画を基準画像として取得するための検証用依存である。

```bash
cd /Users/hiroyuki_furuno/works/private/katana
make mermaid-diagram-browser-install
```

生成先:

```text
/Users/hiroyuki_furuno/works/private/katana/assets/fixtures/mermaid_all/official/
```

公式参照画像を埋め込んだ状態の preview 証跡:

```text
/Users/hiroyuki_furuno/works/private/katana/tmp/mermaid-official-comparison/README.md
/Users/hiroyuki_furuno/works/private/katana/tmp/mermaid-official-comparison/contact-sheet.png
```
