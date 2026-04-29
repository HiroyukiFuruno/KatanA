# Findings: v0.22.10 Mermaid Rendering Compatibility Investigation

## 1. `mmdc` 由来の出力契約

### 旧 KatanA の呼び出し

旧 Mermaid renderer は、`4f273fff^` 時点の `crates/katana-core/src/markdown/mermaid_renderer/render.rs` で `mmdc` を直接起動していた。

- 入力: 一時 `.mmd` ファイルを `-i` へ渡す
- 出力: 入力ファイルと同じ一時ディレクトリの `.png` を `-o` へ渡す
- 背景: `DiagramColorPreset::current().background` を `--backgroundColor` へ渡す
- テーマ: `DiagramColorPreset::current().mermaid_theme` を `--theme` へ渡す
- ログ: `--quiet` を付け、標準出力と標準エラーは捨てる
- 形式: `mmdc` が出力ファイル拡張子 `.png` から PNG を選ぶ

旧コードは `mmdc` を使う理由を「Puppeteer / Chrome で PNG 化し、`resvg` が苦手な `<foreignObject>` の欠落を避けるため」と記録していた。ここで必要だったのは `mmdc` 自体ではなく、公式 Mermaid.js をブラウザ互換の描画環境で実行し、PNG として安定して切り出す契約である。

### `mmdc` 11.12.0 の既定値

ローカルの `/opt/homebrew/bin/mmdc` と npm registry はどちらも `@mermaid-js/mermaid-cli` 11.12.0 を指していた。

`mmdc -h` で確認した主な既定値は次の通り。

- ページ幅（`--width`）: `800`
- ページ高さ（`--height`）: `600`
- 拡大率（`--scale`）: `1`
- 背景色（`--backgroundColor`）: `white`
- テーマ（`--theme`）: `default`
- 出力形式（`--outputFormat`）: 出力ファイル拡張子から決定

旧 KatanA は width / height / scale を明示していなかったため、`mmdc` 側の `800 x 600` と `scale = 1` が暗黙の基準だった。一方、背景色とテーマは KatanA の現在テーマから明示的に渡していた。

### `mmdc` を依存として戻さない条件

`mmdc` は実行時依存として戻さない。戻さない理由は次の通り。

- Node.js と Mermaid CLI の導入をユーザーに要求し、KatanA のネイティブアプリ方針から外れる
- `mmdc` の内部 runtime は Puppeteer / Chrome 依存であり、OS や配布形態ごとの差分が KatanA の外へ漏れる
- 現行の Mermaid.js renderer はローカル `mermaid.min.js` を KatanA 側で管理しており、必要な契約を KatanA 側の描画 policy として持てる
- 実行時の退避経路（fallback）を追加すると、preview と export の失敗条件、キャッシュ、検証対象が二重化する

### KatanA renderer に取り込む policy

KatanA 側で明示的に持つべき出力契約は次の通り。

- render width: `mmdc` 既定の `800` を基準値にする
- page height: `mmdc` 既定の `600` は初期表示領域の参考にし、最終 PNG は図形内容に合わせて切り出す
- background: KatanA の `DiagramColorPreset.background` を HTML、SVG、PNG capture に通す
- theme: KatanA の `DiagramColorPreset.mermaid_theme` を Mermaid 初期化へ通す
- scale: 既定は `1` を互換基準とし、KatanA 側の高精細化は policy と cache key で明示する
- capture target: ページ全体ではなく図形コンテナを切り出し、過度な余白を作らない
- content bounds: SVG の `getBBox()` と `viewBox` を使い、図形内容と最小余白を基準にする
- max width: 横長化を抑える上限を設け、ガントチャートの範囲外 today marker のような特殊マーカーは対象を限定して扱う
- cache key: policy を変更した場合は cache version を更新し、旧 PNG と混在させない

現行コードは `MERMAID_RENDER_WIDTH = 800`、`MERMAID_CAPTURE_SCALE = 1.25`、`MERMAID_CAPTURE_MAX_WIDTH = 1200` を持っている。`scale = 1.25` は `mmdc` 既定値とは異なるため、画質向上のための KatanA policy として扱い、検証と cache key の対象に含める。
