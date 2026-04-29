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

## 2. Rust 管理 JS renderer spike の判定軸

Task 2 は「図形を表示できるか」だけを確認する作業ではない。現行実装も Mermaid / Draw.io を表示できている。今回の目的は、現行の表示品質を崩さずに、OS Chrome / Chromium アプリ依存を外し、描画速度と所有境界を改善できる runtime を選ぶことである。

評価トピックは次の2つに絞る。

- OS の環境に依存しない
- 高速かつ正確に表示する

As-Is:

- OS 依存
- 遅い

To-Be:

- OS 非依存
- 高速かつ正確

Rust 管理 JS renderer の候補は、まず公式 Mermaid.js / Drawio.js を実行する方向で比較する。

- `rquickjs`: QuickJS の Rust binding。軽量な JavaScript 実行環境として試す
- `boa_engine` / `boa_runtime`: Rust 製 ECMAScript engine。Boa runtime の Web API 実装余地も確認する
- `deno_core`: V8 ベースの Rust 管理 JavaScript runtime。重さと配布影響も同時に見る

Mermaid 専用 Rust renderer は、性能候補として記録するが、公式 Mermaid.js / Drawio.js を使うという本 change の条件と衝突するため別枠にする。

- `merman`: Mermaid `@11.12.3` parity を掲げる Rust renderer
- `mermaid-rs-renderer`: Mermaid の pure Rust SVG / PNG renderer
- `selkie-rs`: Mermaid.js の Rust 移植系 renderer

採用判断では、最低限次を確認する。

- Mermaid.js / Drawio.js の最小 render API が動くか
- `document`、`window`、SVG DOM、`getBBox()`、layout measurement がどこまで必要か
- flowchart / sequence / gantt など代表図形でサイズ、余白、中央寄せ、テーマ、特殊マーカーが崩れないか
- 初回描画と連続描画で現行 headless browser より体感上の改善があるか
- HTML export に埋め込む図形出力と cache key policy に統合できるか
- preview と export の通常経路で実行時 fallback を持たずに成立するか

### 2.1 V8 直接実行と最小 DOM / SVG shim の結果

`crates/katana-core/tests/mermaid_js_runtime_spike.rs` を追加し、Rust 管理の V8 で `~/.local/katana/mermaid.min.js` を読み込んだ。

DOM なしの実行結果:

```text
flowchart / sequence / class / state / gantt / pie がすべて document is not defined で失敗
```

最小 DOM / SVG shim ありの実行結果:

```text
flowchart: SVG 生成成功、A / B label 保持
sequence: SVG 生成成功、User / KatanA / Open label 保持
class: SVG 生成成功、PreviewPane / RenderedSection label 保持
state: SVG 生成成功、Pending / Image / success label 保持
gantt: SVG 生成成功、KatanA / Rendering label 保持
pie: SVG 生成成功、DrawIo / Mermaid label 保持
```

この結果から、公式 Mermaid.js を Rust 管理 JavaScript 実行で使う場合、JavaScript engine 単体では不足するが、KatanA 管理の最小 DOM / SVG shim で代表 Mermaid 図形の SVG 生成までは到達できる。採用判断の中心は、`document`、SVG DOM、`getBBox()`、layout measurement をどこまで正確に本体実装へ移せるかになる。

### 2.2 本体 Mermaid renderer への接続結果

Rust 管理 JavaScript 実行経路を本体 Mermaid renderer の既定経路へ接続した。

実装上の境界は次の通り。

- Rust 管理 V8 + 最小 DOM / SVG shim で公式 Mermaid.js を実行する
- 戻り値は `DiagramResult::Ok(svg)` とし、preview 側の既存 SVG rasterize 経路へ渡す
- preview cache key に Rust 管理 JS SVG profile を含め、旧 PNG キャッシュと混在させない
- Mermaid 描画ごとに独立した V8 isolate を使い、全描画を塞ぐ直列 lock は置かない
- `mathjax_svg` も同じ V8 グローバル状態を使うため、初期化所有者を揃えて後続の数式描画を壊さない

検証結果:

```text
cargo test -p katana-core --test mermaid_js_runtime_spike -- --ignored --nocapture
make test-specific T=cache_file_path_uses_rust_managed_svg_profile
make test-specific T=mermaid_cache_key_ignores_legacy_renderer_env
make test-specific T=mermaid_runtime_is_always_rust_managed_svg
make build-release
```

いずれも成功した。`mermaid_js_runtime_spike` では flowchart / sequence / class / state / gantt / pie の SVG 生成、label 保持、`resvg` による rasterize まで確認した。

`headless_chrome` は Chromium そのものではなく OS 上の Chrome アプリを制御する依存だったため、workspace 依存から削除した。Draw.io と PDF / PNG / JPEG export は、管理下 Chromium runtime 未接続の明示エラーへ切り替え、OS Chrome へ戻る fallback は残していない。

追加 fixture として、`assets/fixtures/sample_mermaid_all.md` に Mermaid 図形種別の確認用 Markdown を追加した。

全パターン fixture の Rust 管理 JS 評価では、26 block すべてが SVG 生成と `resvg` rasterize まで通った。

不足していた API は次の通り。

- text content / innerText の getter と setter
- `append` / `prepend` / `replaceChildren` / `contains`
- `clientWidth` / `clientHeight` / `offsetWidth` / `offsetHeight`
- `getComputedStyle()` の padding / margin / border 初期値
- Cytoscape layout 用の `getBBox()` width / height / w / h
- `style` / `defs` など非表示 SVG 要素を layout 計測から除外する処理

また、Mermaid の layout が異常に大きい SVG を返した場合でも GPU texture 上限で preview が panic しないよう、SVG rasterize 側で最大辺を 8192px に抑える安全弁を追加した。ガントチャートについては、期間外の today marker が canvas 幅を広げないよう描画前に `todayMarker off` を補い、SVG の width / height も最大幅 1200px の policy に合わせて正規化する。

26種別の個別 fixture と screenshot 評価では、初期の残NGは Mermaid.js 構文の未対応ではなく、SVG 生成前に Mermaid.js が期待するブラウザーAPIの再現不足が原因だった。

- `getBBox()` / `getComputedTextLength()` の精度不足で、Class / Requirement / ER の box overlap、Ishikawa の巨大 label box、Kanban の異常列高が発生した
- SVG/HTML 親要素の `offsetWidth` が `0` 扱いになり、Gantt の日付軸が負方向に潰れた
- CSS style / computed style の不足で、Tree View / Treemap / Venn の dark theme 配色や矩形表示が崩れた
- DOM move semantics、`:first-child`、`compareDocumentPosition()` の不足で、一部の図形の要素順序と計測が不安定だった
- SVG bounds normalization が横幅中心だったため、Radar / Venn / Ishikawa など縦横比が特殊な図形で初期表示が大きすぎた

これらに対して、DOM tree 操作、style parsing、text measurement、element metrics、content bounds、図形別の限定的な SVG normalization を追加した。v34 screenshot では 26 / 26 の主要ラベル、サイズ、余白、配色、rasterize を確認済みである。Ishikawa / Architecture は Mermaid beta / 特殊図形のため、公式ブラウザー描画との細部差分比較は後続の精度改善候補として残す。

### 2.3 公式ブラウザー描画との比較方式

26種別を人間がすべて正解判定する方式は維持しない。評価基準は、公式 Mermaid.js を実ブラウザーで描画した参照画像との比較へ切り替える。

追加した更新経路:

```bash
make mermaid-diagram-update
```

このコマンドは、`~/.local/katana/mermaid.min.js` を Playwright 管理 Chromium 上で実行し、`assets/fixtures/mermaid_all/official/*.png` を再生成する。さらに、26個の個別 Markdown fixture に公式参照画像を埋め込む。これにより、KatanA preview 上で「KatanA が描画した Mermaid」と「公式ブラウザー描画の画像」を上下に並べて比較できる。

Playwright 管理 Chromium は検証用の基準画像生成にだけ使う。通常 preview / HTML export の採用 runtime ではない。

### 2.4 `katana-renderer` 切り出しの前提

Mermaid preview 表示機構と検証機構は、HTML / PDF / PNG / JPEG export の境界も含めて、将来的に `katana-renderer` として別 repository 化できる可能性がある。ただし、現時点では interface がまだ KatanA のプレビュー都合に寄っている。

別 repository 化の前に、KatanA 内で次の境界を作る。

- 入力: Mermaid source、Mermaid.js 互換 config、size policy
- 出力: SVG または PNG、bounds、diagnostics
- 診断: render error、unsupported browser API、normalization warning
- 検証: 公式 Mermaid.js のブラウザー参照画像との差分比較

Mermaid.js に渡す値は互換性を保つ。`theme`、`themeVariables`、`securityLevel`、`flowchart`、`sequence` などは Mermaid.js の config 構造として扱い、KatanA 独自のサイズ制約、cache profile、診断情報は別の wrapper policy として外側に置く。

この境界整理は v0.22.11 `v0-22-11-renderer-runtime-interface-and-versioning` へ移管する。v0.22.10 では、まず公式 Mermaid.js の実ブラウザー描画との差分評価を行い、今回補正できる表示差分を補正する。

性能面では、Rust 管理 JS renderer は `mmdc` と大きな差が出る見込みがある。現時点の体感では、Mermaid 描画の読み込み（load）待ちをほぼ感じない。一方で、正確性はまだ担保済みではないため、公式比較画像との差分評価を v0.22.10 の主要完了条件として残す。

別 repository 化そのものは、interface が KatanA 内で安定してから判断する。v0.22.11 の完了条件は、別 repository を作ることではなく、別 repository 化できる責務境界へ寄せることである。
