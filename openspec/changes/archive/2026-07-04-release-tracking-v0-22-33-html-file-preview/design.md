> **Superseded architecture record:** The Chromium/helper decisions below are
> rejected history. The active design is the in-process Rust/V8 contract in
> `openspec/specs/html-file-preview/spec.md` and the current ledger in
> `tasks.md`.

## Context

旧 v0.22.33 preview は KDV の direct HTML normalizer が返す static image surface を KatanA が表示していた。この経路は `<head>` / `<style>` / `<script>` 相当を本文に混在させ、CSS/JavaScript/form/link navigation/browser hit-test を評価できず、action 後 frame も存在しない。表の文字重なりだけを直しても、一般 HTML document の semantics と操作性は改善しない。

KRR `0.4.0` は version-lock した Chromium、browser child、IPC session、complete RGBA frame、pointer/keyboard/text/focus/scroll/resize、browser-confirmed navigation、resource policy、packaged runtime resolution を一つの persistent page contract として実装済みである。KatanA 側の設計はこの contract を end-to-end で保持し、HTML semantics を他層に再実装しない。

## Goals / Non-Goals

**Goals:**

- local `.html` / `.htm` と user-entered `http` / `https` URL を browser-equivalent tab として開く。
- KatanA が主文書を取得し、raw HTML と完全な最終 document URL を KDV 経由で KRR へ渡す。
- KDV は KRR session を worker 上で所有し、latest complete frame、入力、navigation、typed error を中継するだけにする。
- KatanA は native surface に browser frame を表示し、pointer、keyboard、text/IME、focus、scroll、resize、navigation/history、reload を接続する。
- 公開済み KRR `0.4.0` -> KDV `0.3.0` -> KatanA `v0.22.33` の順序と packaged runtime を release gate で強制する。
- packaged native-window evidence で CSS、accordion、JavaScript action、form input、navigation、reload、complete frame、非重なりを検証する。

**Non-Goals:**

- KatanA/KDV に HTML parser、CSS cascade/layout、JavaScript interpreter、browser hit-test、WebView を実装しない。
- KRR の static HTML export/parser を interactive viewer に転用しない。
- KDV の HTML -> PDF/image export 機能を削除しない。ただし interactive path とは型と entry point を分離する。
- arbitrary remote origin、workspace escape、subprocess、unsupported scheme を browser page に許可しない。
- KRR/KDV の unpublished code や local path/git dependency を KatanA package に取り込まない。

## Decisions

### HTML semantics は KRR Chromium page 一箇所に閉じる

KRR browser page が HTML5 parsing、quirks/standards mode、CSS cascade/layout/paint、JavaScript/Web API/event loop、form state、hit-test、default action、navigation decision を所有する。KatanA/KDV は DOM node、CSS property、clickable region、link target を解釈しない。

browser runtime を起動できない場合は typed error を表示する。static parser、direct HTML normalizer、Markdown renderer、export image への fallback は、動かない viewer を正常に見せるため禁止する。

### 主文書取得は KatanA、subresource policy は KRR が所有する

KatanA は local file または URL の主文書を取得し、raw HTML と完全な `file/http/https` document URL を KDV へ渡す。redirect 後は final URL を渡す。KDV は取得・rewrite をせず KRR へ転送する。

KRR は最初の top-level main-document response に raw HTML をそのまま供給し、完全な document origin を基準に stylesheet、script、image を解決する。doctype、`<base>`、navigation script を host 側で挿入しない。workspace escape、許可外 origin、unsupported scheme、subprocess、remote iframe は KRR request policy で拒否する。

### KDV は worker-backed browser-session adapter に限定する

同期的な KRR session は KDV の専用 worker thread が所有する。typed channel は key/text/focus/resize/reload/navigation/error の順序を保持し、pointer move と repaint request は coalesce できる。frame は generation 付き latest complete viewport のみを保持し、未表示 frame を蓄積しない。

KDV `0.2.x` の direct HTML normalizer / static preview surface は export-style path として扱い、KatanA interactive viewer から呼ばない。KDV `0.3.0` browser adapter は export API と別 entry point・別型で公開する。

### KatanA は native browser surface host に限定する

KatanA は tab ごとに source、完全な document URL、KDV session handle、latest frame generation、history、loading/error state を保持する。native UI は frame を exact viewport に paint し、raw pointer/keyboard/text/focus/scroll/resize event を adapter へ渡す。KatanA は hover-only image control、HTML element section、browser geometry tableを持たない。

KRR が browser-confirmed top-level navigation event を返した場合だけ、KatanA が policy/history を適用して次の主文書を取得する。`preventDefault()`、same-document behavior、new-window policy は KRR/Chromium の結果に従い、KatanA は markup や座標から navigation を推測しない。

### Frame は毎回 complete viewport として扱う

KRR action frame は damage image ではなく、変更領域と不変領域を含む完全な viewport RGBA frame である。KDV/KatanA は partial frame を合成しない。accordion、button、text input の連続操作でも heading、table、本文などの不変領域が同じ frame に残ることを integration test と packaged evidence で固定する。

### 公開済み artifacts だけを直列に統合する

1. KRR `0.4.0` の crates.io crate と 4-platform runtime archive/checksum を公開・確認する。
2. その後に KDV を crates.io KRR `^0.4.0` へ更新し、browser-session adapter を `0.3.0` として公開・確認する。
3. その後に KatanA を crates.io KDV `0.3.0` / KRR `0.4.x` へ更新し、platform package に matching runtime archive を配置する。
4. native-window interactive evidence と release gate を通し、ユーザーの明示 OK 後にだけ `v0.22.33` を release する。

KDV/KRR の path/git dependency、開発機 Chrome、Cargo build directory の helper は配布成果物に使わない。

### Release evidence は操作契約を証明する

fixture は metadata/style/script、responsive CSS、table、link、accordion、JavaScript button、form input、reload 後 state を含む。capture は initial、accordion、button、typed input、navigation、reload の各状態を識別でき、全 frame で不変領域と変更領域、文字の非重なり・非 clipping を確認できるものとする。旧 static screenshot は failure evidence であり acceptance evidence に数えない。

## Risks / Trade-offs

- **Browser bundle が欠損する** -> KRR runtime archive/checksum を KatanA packaging 前に検証し、helper 隣接配置の packaged smoke を実行する。
- **HTML semantics が KDV/KatanA に漏れる** -> source-boundary test と release gate で parser/CSS/hit-test/static fallback を拒否する。
- **部分 repaint で既存 content が消える** -> action ごとの exact complete-frame regression と native evidence を必須化する。
- **navigation が policy 前に通信する** -> KRR browser-level interception/auto-attach の結果だけを KDV/KatanA が消費する。
- **UI thread が browser RPC で停止する** -> KDV worker + typed channel + latest-frame coalescing を adapter contract にする。
- **静的画像で release 判定してしまう** -> click/input/navigation/reload を含まない evidence を release gate で不合格にする。
- **誤 version line が復活する** -> SemVer guard は最新有効 release `v0.22.32` の隣接 patch `v0.22.33` だけを受理し、取り下げ済み `v0.29.0` を拒否する。

## Migration Plan

1. KRR `0.4.0` local release gate、strict coverage、runtime archive、packaged smoke を完了する。
2. KRR publication を確認後、KDV `0.3.0` browser-session adapter を実装・検証・公開する。
3. KDV publication を確認後、KatanA の static preview surface を browser-session host に置換する。
4. KatanA package へ matching KRR runtime を配置し、native-window interactive evidence を生成する。
5. self-review、OpenSpec strict validation、release guard、platform package smoke を通す。
6. ユーザーへ evidence を提示し、明示 OK 後にだけ commit/push/PR/release を進める。
