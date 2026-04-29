# Mermaid 26種別 描画チェックシート

## 判定基準

- ラベル: 主要な文字列が欠落していない
- サイズ: 初期表示で巨大すぎない、極小化しない
- 余白: 図の外側に不自然な大空白がない
- 配色: dark theme で線・文字・背景が読める
- 画像化（rasterize）: preview で GPU 上限を超えず表示できる

## チェック一覧

最新結果: v34 screenshot で 26 / 26 を表示確認済み。
評価方法は、公式 Mermaid.js を実ブラウザーで描画した参照画像を各 Markdown fixture に埋め込み、KatanA preview 上で上下比較する方式へ切り替えた。
Ishikawa / Architecture は Mermaid beta / 特殊図形のため、参照画像との差分を後続の精度改善候補として残す。

| No | 種別 | Fixture | ラベル | サイズ | 余白 | 配色 | 状態 | メモ |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 01 | Flowchart / Graph | `assets/fixtures/mermaid_all/01-flowchart.md` | OK | OK | OK | OK | OK | v34 screenshot で主要ラベルと線を確認 |
| 02 | Sequence Diagram | `assets/fixtures/mermaid_all/02-sequence.md` | OK | OK | OK | OK | OK | v34 screenshot で主要ラベルと矢印を確認 |
| 03 | Class Diagram | `assets/fixtures/mermaid_all/03-class.md` | OK | OK | OK | OK | OK | class box の重なり解消、主要ラベルを確認 |
| 04 | State Diagram | `assets/fixtures/mermaid_all/04-state.md` | OK | OK | OK | OK | OK | v34 screenshot で主要ラベルと遷移を確認 |
| 05 | Entity Relationship Diagram | `assets/fixtures/mermaid_all/05-er.md` | OK | OK | OK | OK | OK | entity / relation / 属性ラベルを確認 |
| 06 | User Journey | `assets/fixtures/mermaid_all/06-journey.md` | OK | OK | OK | OK | OK | v34 screenshot で主要ラベルと線を確認 |
| 07 | Gantt Chart | `assets/fixtures/mermaid_all/07-gantt.md` | OK | OK | OK | OK | OK | 軸の負方向潰れと左欠けが解消、期間とタスクを確認 |
| 08 | Pie Chart | `assets/fixtures/mermaid_all/08-pie.md` | OK | OK | OK | OK | OK | v34 screenshot で円グラフと凡例を確認 |
| 09 | Requirement Diagram | `assets/fixtures/mermaid_all/09-requirement.md` | OK | OK | OK | OK | OK | requirement box / relation label を確認 |
| 10 | Git Graph | `assets/fixtures/mermaid_all/10-git-graph.md` | OK | OK | OK | OK | OK | v34 screenshot で branch / commit を確認 |
| 11 | C4 Diagram | `assets/fixtures/mermaid_all/11-c4.md` | OK | OK | OK | OK | OK | v34 screenshot で主要ノードと関係線を確認 |
| 12 | Mindmap | `assets/fixtures/mermaid_all/12-mindmap.md` | OK | OK | OK | OK | OK | `h` 参照エラーは解消済み |
| 13 | Timeline | `assets/fixtures/mermaid_all/13-timeline.md` | OK | OK | OK | OK | OK | 高さ10px化が解消済み |
| 14 | Quadrant Chart | `assets/fixtures/mermaid_all/14-quadrant.md` | OK | OK | OK | OK | OK | v34 screenshot で主要ラベルを確認 |
| 15 | XY Chart | `assets/fixtures/mermaid_all/15-xy-chart.md` | OK | OK | OK | OK | OK | 軸と線を確認 |
| 16 | Sankey | `assets/fixtures/mermaid_all/16-sankey.md` | OK | OK | OK | OK | OK | ノードと量ラベルを確認 |
| 17 | Block Diagram | `assets/fixtures/mermaid_all/17-block.md` | OK | OK | OK | OK | OK | 3ブロックと接続を確認 |
| 18 | Packet Diagram | `assets/fixtures/mermaid_all/18-packet.md` | OK | OK | OK | OK | OK | 横長だが bit range と field label を確認 |
| 19 | Kanban | `assets/fixtures/mermaid_all/19-kanban.md` | OK | OK | OK | OK | OK | 列高さの異常拡大を抑制し、列とカードを確認 |
| 20 | Architecture Diagram | `assets/fixtures/mermaid_all/20-architecture.md` | OK | OK | OK | OK | OK | service label / group / edge を確認。公式 icon 表現は後続精度改善候補 |
| 21 | Radar Chart | `assets/fixtures/mermaid_all/21-radar.md` | OK | OK | OK | OK | OK | 巨大化と外周ラベル切れを抑制 |
| 22 | Tree View | `assets/fixtures/mermaid_all/22-tree-view.md` | OK | OK | OK | OK | OK | dark theme の黒文字/黒線を補正 |
| 23 | Ishikawa Diagram | `assets/fixtures/mermaid_all/23-ishikawa.md` | OK | OK | OK | OK | OK | 巨大ラベル枠を抑制し、spine / branch / cause label を確認。beta 図形の細部精度は後続比較候補 |
| 24 | Venn Diagram | `assets/fixtures/mermaid_all/24-venn.md` | OK | OK | OK | OK | OK | 円と主要ラベルを確認 |
| 25 | Treemap | `assets/fixtures/mermaid_all/25-treemap.md` | OK | OK | OK | OK | OK | 矩形領域と値ラベルを確認 |
| 26 | Wardley Map | `assets/fixtures/mermaid_all/26-wardley.md` | OK | OK | OK | OK | OK | 軸・点・線を確認 |

## スクリーンショット取得

```bash
cd /Users/hiroyuki_furuno/works/private/katana
scripts/screenshot/run.sh \
  --request scripts/screenshot/examples/v0-22-10-mermaid-all-patterns.json \
  --output tmp/mermaid-all-pattern-screenshots
```

最新の一覧画像:

```text
/Users/hiroyuki_furuno/works/private/katana/tmp/mermaid-all-pattern-screenshots/contact-sheet.png
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
/Users/hiroyuki_furuno/works/private/katana/tmp/mermaid-official-reference-screenshots/contact-sheet.png
```
