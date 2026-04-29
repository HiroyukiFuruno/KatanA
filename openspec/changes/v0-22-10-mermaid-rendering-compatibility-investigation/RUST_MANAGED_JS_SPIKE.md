# Rust 管理 JavaScript 実行 spike

作成日: 2026-04-29

## 結論

Rust 管理 JavaScript 実行（Rust-managed JS runtime）は継続評価する価値がある。

V8 単体では `document is not defined` で止まるが、最小 DOM / SVG shim を V8 上に入れると、公式 Mermaid.js で代表 Mermaid 6種の SVG 生成まで到達した。少なくとも Mermaid 側は、WebView / Chromium なしで「公式 Mermaid.js + KatanA 管理の最小 browser API」という構成を spike 継続できる。

その後、この spike は本体 Mermaid renderer の既定経路へ接続済みである。Mermaid は OS 上の Chrome アプリを使わず、Rust 管理 JavaScript 実行で SVG を生成する。

## 実装した spike

場所:

```text
crates/katana-core/tests/mermaid_js_runtime_spike.rs
```

実行:

```bash
cargo test -p katana-core --test mermaid_js_runtime_spike -- --ignored --nocapture
```

アプリ起動:

```bash
make run-release
```

確認内容:

- `~/.local/katana/mermaid.min.js` を Rust 管理の V8 で読み込む
- 最小限の `window` / `self` / `navigator` / `performance` を用意する
- `document` / `Element` / SVG element / selector / `getBBox()` / DOMPurify 周辺 API の最小 shim を用意する
- `mermaid.initialize(...)`
- `mermaid.render(...)` で代表 Mermaid 図形を描画する

DOM なしの結果:

```text
flowchart / sequence / class / state / gantt / pie がすべて document is not defined で失敗
```

最小 DOM / SVG shim ありの結果:

```text
flowchart: ok, labels A/B preserved
sequence: ok, labels User/KatanA/Open preserved
class: ok, labels PreviewPane/RenderedSection preserved
state: ok, labels Pending/Image/success preserved
gantt: ok, labels KatanA/Rendering preserved
pie: ok, labels DrawIo/Mermaid preserved
```

本体接続後の追加確認:

```text
flowchart / sequence / class / state / gantt / pie の SVG 生成、label 保持、resvg rasterize に成功
preview cache key が Rust 管理 JS SVG profile を含むことを確認
描画ごとの V8 isolate を独立実行し、全描画を塞ぐ直列 lock を削除
`assets/fixtures/sample_mermaid_all.md` に Mermaid 図形種別の確認用 fixture を追加
全パターン fixture は 26 block すべてが SVG 生成と `resvg` rasterize まで成功
DOM / SVG shim に不足していた text content、append、layout size、getComputedStyle 初期値を追加
異常に大きい SVG が GPU texture 上限で preview を落とさないよう rasterize 側に最大辺 8192px の安全弁を追加
ガントチャートの期間外 today marker と SVG 最大幅 policy を Rust 管理 JS 経路へ移植
`headless_chrome` 依存を削除し、Draw.io と PDF / PNG / JPEG export は管理下 Chromium runtime 未接続の明示エラーへ変更
release build が成功
```

## 判定

Rust 管理 JavaScript 実行は、公式 Mermaid.js を使うため、手書き Rust ネイティブ描画より表示互換性の筋は良い。

V8 / QuickJS / Boa などの JavaScript engine 単体では Mermaid.js の描画条件を満たさない。一方で、V8 + 最小 DOM / SVG shim では、少なくとも代表 Mermaid 6種が SVG と label を返せるところまで進んだ。

次の評価は、「全パターンの SVG が返る」から一段進めて、サイズ、余白、テーマ、特殊マーカー、HTML export 埋め込み、連続描画速度を本体経路で確認する。

## 次に確認すること

- DOM / SVG shim を test から production module へ移す場合の責務境界
- `getBBox()` / `getBoundingClientRect()` / text measurement の精度
- テーマ変数、背景、余白、viewBox、最大幅の policy 反映
- gantt の today marker など特殊マーカーの扱い
- HTML export と preview が同じ SVG 出力を使えるか
- 全パターン fixture の表示品質を screenshot 証跡で確認できるか
- cache key に runtime / Mermaid version / policy / theme を含められるか
- 初回描画と連続描画の速度
- Drawio.js も同じ DOM 前提を持つため、Mermaid.js だけで判断しない
