## Context

KatanA の interactive HTML path は KatanA -> KDV worker adapter -> KRR in-process Rust/V8 session で構成される。v0.22.33 で基本の HTML/CSS/JavaScript/input/navigation は接続済みだが、remote resource と embedded SVG の不足、session start error で KDV worker 自体が終了する lifecycle、抽象的な `browser worker has stopped` に一次原因が上書きされる error state が実利用を妨げた。画像/diagram surface では scroll と zoom の入力が同時に解釈され、拡大寸法が戻る問題と、透明 controls が light theme/白画像上で消える問題が残る。

制約は、HTML semantics を KRR に限定すること、KDV/KatanA に parser/CSS/JavaScript/hit-test を追加しないこと、Chromium/WebView/helper process を使用しないこと、coverage gate を緩和しないこと、release dependency を crates.io のみにすることである。

## Goals / Non-Goals

**Goals:**

- KRR が許可済み subresource と embedded SVG を browser frame に反映する
- 個別 resource failure と browser operation failure を追跡可能にする
- KDV worker が startup failure 後も command を受け、navigation により session を再生成できるようにする
- fullscreen scroll を zoom から分離し、上下左右 pan で拡大寸法を保持する
- すべての画像 overlay controls を固定背景と border で判別可能にする
- registry-only chain と headless evidence を v0.22.34 release gate で証明する

**Non-Goals:**

- Chromium、WebView、platform browser、external helper/runtime archive の導入
- KDV/KatanA での HTML parse、CSS cascade/layout、JavaScript evaluation、hit-test
- iframe execution、credential-bearing URL、HTTPS から HTTP への mixed content の許可
- coverage threshold、package size、SemVer guard の緩和

## Decisions

1. **HTML semantics は KRR に維持する。** KRR の既存 html5ever/CSS/V8/layout pipeline に resource と SVG support を追加する。OS browser を利用する案は Rust/V8 ownership と配布契約を破るため採用しない。

2. **subresource failure は resource 単位で非致命にする。** policy rejection または transport failure は layer、operation、document、resource kind/reference、cause を warning log に残し、stylesheet/script/image だけを省略して主文書を描画する。main document/session failure は typed error のまま致命扱いとする。

3. **embedded SVG は DOM subtree を KRR 内で serialize し、layout 時に配置する。** SVG 要素を通常 HTML child として flatten せず、namespace、case-sensitive attribute、viewBox、CSS width/height を保持した markup として raster pipeline へ渡す。

4. **KDV worker は session の有無と worker lifetime を分離する。** startup failure でも receiver loop を終了せず、最新 viewport/origin を保持する。valid resize を記録し、次の navigation で session を再生成する。worker stop/panic は channel/thread failure のみを表す。

5. **error context は各境界で失わず、KatanA は一次原因を保持する。** KDV は KRR error を operation と document origin で包む。KatanA は receive/update の layer を付けて UI と tracing log に同じ構造を出し、後発 WorkerStopped/WorkerPanicked で既存の具体的 error を置換しない。

6. **fullscreen input は scroll と zoom を排他的に解釈する。** smooth scroll delta がある frame は x/y pan のみに使い zoom を変更しない。scroll がない pinch/zoom input だけが zoom を更新する。

7. **overlay control colors は theme token ではなく固定 contrast token にする。** 最大化、pan/zoom/reset/info、fullscreen close は半透明黒の固定背景、白 icon、1 px 白 border を使用する。hover/active/focus は alpha のみ変更し、画像や active theme から色を導出しない。

8. **公開は KRR -> KDV -> KatanA の順に行う。** KRR v0.4.4 と KDV v0.3.2 の GitHub Release/crates.io を確認後、KatanA lockfile を registry-only で更新する。local path patch は開発検証に限定し、release artifact へ残さない。

## Risks / Trade-offs

- [remote resource が応答しない] -> KRR の既存 transport timeout と policy を維持し、失敗を resource 単位で記録する
- [worker recovery 後に stale frame が表示される] -> KatanA は exact viewport と current session generation に一致しない初期/stale frame を破棄する
- [SVG serialization が HTML attribute 正規化で壊れる] -> case/namespace/viewBox と実 Mermaid SVG の pixel test を追加する
- [固定 control 背景が画像を一部覆う] -> control 面積を既存サイズに限定し、opacity と border で視認性を確保する
- [依存公開直後に registry index が遅延する] -> `cargo info` と checksummed lockfile の両方が確認できるまで downstream release を開始しない
