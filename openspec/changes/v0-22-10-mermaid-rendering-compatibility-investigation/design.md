## Context

現在の Mermaid 描画は、旧 `mmdc` 呼び出しから、ローカル Mermaid.js をヘッドレスブラウザで描画して PNG 化する経路へ移行している。ガントチャートでは、Mermaid.js が親要素の横幅を拾うこと、赤い「今日」線がチャート期間外に出ることにより、`mmdc` 時代と見た目が大きく乖離した。

今回のリリースではガントチャートの明確な崩れだけを修正対象にした。残りは、個別修正を積む前に、どの図形で何が `mmdc` と違うかを整理する。

## Goals

- Mermaid 図形ごとの `mmdc` 互換差分を再現可能な fixture として整理する。
- 見た目の比較軸を、サイズ、余白、中央寄せ、テーマ色、フォント、特殊マーカー、エラー表示に分ける。
- 修正優先度を、ユーザー文書で発生しやすい図形と、表示崩れの大きさで判断できるようにする。
- 後続の versioned change に移せる粒度でタスクを分解する。

## Non-Goals

- この change では実装を行わない。
- この change では Mermaid.js 以外の Draw.io / PlantUML の互換性調査を主対象にしない。
- この change ではユーザー設定を増やさない。
- この change では `mmdc` へ戻す判断をしない。

## Approach

1. 旧 `mmdc` 経路の出力条件を確認する。
   - KatanA が `mmdc` に渡していた引数
   - `mmdc` の既定 viewport / output size
   - テーマ、背景色、フォント、スケールの扱い
2. Mermaid.js 経路の描画条件を確認する。
   - ヘッドレスブラウザの viewport
   - Mermaid render 対象 container の幅
   - SVG `viewBox` / `getBBox()` / PNG capture 対象
   - キャッシュキーに含めるべき描画条件
3. 代表 fixture を作る。
   - flowchart
   - sequence
   - class
   - state
   - entity relationship
   - gantt
   - pie
   - journey
   - mindmap
   - timeline
4. `mmdc` 出力と Mermaid.js 出力を比較する。
   - 画像サイズ
   - 図形本体の位置
   - テーマ色の一致
   - 文字色、線色、背景色
   - 図形固有マーカーの扱い
5. 修正方針を分類する。
   - Mermaid 初期化設定で解決できるもの
   - HTML container / viewport の調整で解決できるもの
   - SVG 後処理が必要なもの
   - Mermaid.js 側の仕様差として許容または明示するもの

## Risks

- `mmdc` の出力は Mermaid CLI と Puppeteer のバージョンに依存するため、完全一致を目標にすると更新不能になる。
- 図形ごとに最適な container 幅が異なる場合、単一の固定幅では別の図形を崩す可能性がある。
- SVG 後処理を増やしすぎると、Mermaid.js 公式更新への追従が難しくなる。

## Open Questions

- `mmdc` 互換の許容範囲を、ピクセル一致ではなく「ユーザーが見て違和感がない」基準にするか。
- 図形ごとの固定幅を持つべきか、全 Mermaid 図形で共通の既定幅を維持するか。
- `mmdc` 比較を CI に入れるか、重いため手動検証・スクリーンショット証跡に限定するか。
