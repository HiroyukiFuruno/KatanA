## Why

KatanA は Mermaid の描画を `mmdc` 実行から、ローカル `mermaid.min.js` を使う自前の Mermaid.js renderer へ移行した。実行時の `mmdc` 依存は不要になったが、`mmdc` がきれいに出力するために暗黙に担っていた viewport、背景、テーマ、余白、切り抜き、拡大率の扱いは、KatanA 側へ取り込む必要がある。

ただし現行の Mermaid / Draw.io 経路は、見えない headless browser（画面を出さないブラウザ）であっても、実体として OS にインストールされた Chrome / Chromium アプリを起動している。通常 preview と HTML export（HTML出力）の diagram rendering が OS アプリに依存する状態は KatanA の native 方針と合わないため、まず Rust 管理 JS（Rust 側が所有する JavaScript 実行環境）で公式 Mermaid.js / Drawio.js を動かせるか試す。これが不採用なら、Mermaid と Draw.io は KatanA 管理下の高速な headless browser / WebView（アプリ内ブラウザ部品）/ Chromium（Chrome 系ブラウザエンジン）から単一の採用経路を選ぶ。

現在はガントチャートの「今日」線による大きな崩れだけを抑えている。残りの Mermaid 図形でも、`mmdc` 時代に近い安定した見た目を KatanA の renderer 内で再現できるようにする。

## What Changes

- `mmdc` を実行時依存として戻さず、参照実装として出力条件を抽出する。
- 通常 preview と HTML export の diagram rendering から OS にインストールされた Chrome / Chromium アプリ依存を外す。
- Mermaid と Draw.io の通常 preview / HTML export を、ユーザー環境のブラウザアプリではなく Rust 管理 JS または KatanA 管理下の高速な headless browser / WebView / Chromium から選んだ単一経路で扱う。
- 実行時の退避経路（fallback）は持たない。
- Mermaid.js renderer の viewport、container 幅、SVG 計測、PNG capture、余白、最大幅、背景、テーマ反映を KatanA 側の責務として整理する。
- flowchart / sequence / class / state / entity relationship / gantt / pie / journey / mindmap / timeline の fixture を使い、見た目の崩れを検出できるようにする。
- 差分をコードで吸収するもの、後続 versioned change に分けるもの、許容差分として文書化するものに分類する。
- `v0.22.10` のリリース対象として、実装、証跡、リリース準備まで完了させる。

## Capabilities

### New Capabilities

- `mermaid-rendering-compatibility-investigation`: `mmdc` 由来の安定した出力条件を KatanA 管理下の単一 Mermaid renderer に取り込むための調査、実装、fixture、比較証跡を扱う。

### Modified Capabilities

- `diagram-block-preview`: Mermaid / Draw.io の通常 preview から OS ブラウザアプリ依存を外し、Mermaid 図形の表示サイズ、余白、テーマ反映、特殊マーカー、キャッシュ条件を安定化する。
- `document-export`: HTML export 内の Mermaid / Draw.io rendering と、HTML から PDF / PNG / JPEG へ変換する export runtime を KatanA 管理下の単一経路へ寄せる。

## Impact

- `crates/katana-core/src/markdown/mermaid_renderer/`
- `crates/katana-core/src/markdown/export/`
- `crates/katana-core/tests/markdown_mermaid.rs`
- `crates/katana-core/tests/export_regression.rs`
- `crates/katana-ui/src/app/export.rs`
- `crates/katana-ui/src/app/export_poll.rs`
- `crates/katana-ui/tests/integration/preview_pane/diagrams.rs`
- `scripts/screenshot/` によるレビュー用スクリーンショット、動画の生成シナリオ
- Mermaid.js、旧 `mmdc`、ヘッドレスブラウザの viewport、描画コンテナ幅、PNG 化処理
