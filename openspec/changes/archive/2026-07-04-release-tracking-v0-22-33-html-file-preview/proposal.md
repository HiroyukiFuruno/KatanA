> **Superseded architecture record:** This proposal preserves the rejected
> Chromium design for historical context only. The current release contract is
> `openspec/specs/html-file-preview/spec.md`: in-process Rust/V8 KRR, KDV
> adapter-only, KatanA host-only, and headless acceptance evidence.

## Why

KatanA の HTML viewer は、HTML を「それらしく描いた静止画像」ではなく、ブラウザと同じ HTML/CSS/JavaScript semantics と操作を評価できなければならない。旧 direct HTML normalizer / static preview surface は metadata や source text を本文に混在させ、CSS、JavaScript、form、link navigation、browser hit-test を評価できなかった。個別の見た目を補修しても一般 HTML document の正しさは保証できない。

この変更は HTML semantics を KRR の persistent Chromium page 一箇所へ集約する。KatanA は主文書取得と native document tab、KDV は browser-session adapter、KRR は parsing、layout/paint、JavaScript/Web API/event loop、input dispatch、navigation detection、resource policy を所有する。

## What Changes

- `.html` / `.htm` を workspace tree、file open dialog、drag-and-drop の対象にし、`http` / `https` URL input を HTML browser tab として扱う。
- KatanA が local file または URL の主文書を取得し、raw HTML と完全な最終 document URL を KDV へ渡す。
- KDV `0.3.0` は KRR `0.4.0` browser session の lifecycle、latest complete frame、input、navigation、typed error を中継する adapter のみとする。
- KRR `0.4.0` の persistent Chromium page を HTML5 parsing、CSS layout/paint、JavaScript/Web API、event loop、form、hit-test、navigation の唯一の source of truth とする。
- KatanA は latest complete viewport frame を表示し、pointer、keyboard、text/IME、focus、scroll、resize を転送する。link navigation は KRR が browser-confirmed event として返した場合だけ主文書取得へ接続する。
- browser runtime failure 時に static parser、Markdown renderer、direct HTML normalizer、export image へ fallback しない。
- Markdown lint、format、export、diagram wrapping を HTML tab に誤適用しない。
- 公開順序を KRR `0.4.0`、KDV `0.3.0`、KatanA `v0.22.33` に固定し、KDV/KRR の path/git dependency を禁止する。
- KatanA package は公開済み KRR runtime archive と checksum を取得し、helper と Chromium bundle を executable 隣接へ配置する。
- native-window evidence は CSS layout、accordion、JavaScript action、text input、link navigation、reload、完全 action frame、文字の非重なりを操作前後で示す。
- `v0.29.0` は取り下げ済みとして release target から除外し、`v0.22.33` はユーザーの明示 OK まで release しない。

## Capabilities

### New Capabilities

- `html-file-preview`: local HTML file と user-entered URL を browser-equivalent persistent session として native document surface に表示・操作する。

### Modified Capabilities

- `workspace-file-filter`: 標準表示・open 対象に HTML file を追加する。
- `release-readiness`: 公開済み KRR/KDV browser chain、runtime bundle、interactive evidence、正しい v0.22.33 隣接更新を必須化する。

## Impact

- `crates/katana-core`: HTML file / URL document source と完全な document URL を保持する。
- `crates/katana-ui`: KDV browser-session handle の lifecycle、frame painting、input forwarding、navigation/history、reload、typed error を native tab に接続する。
- `scripts/screenshot`: native window で操作前後の browser session を capture し、静的 fixture ではなく interactive acceptance evidence を生成する。
- `scripts/release`: KDV `0.3.x`、KRR `0.4.x` の crates.io source、KRR runtime archive、OpenSpec browser contract、v0.22.33 target を機械検証する。
- `katana-document-viewer`: 公開済み KRR `^0.4.0` を利用する browser-session adapter を `0.3.0` として公開する。interactive viewer に parser/CSS cascade/export image fallback を持たない。
- `katana-render-runtime`: Chromium browser process、IPC session、complete frame、input/navigation、resource policy、4-platform runtime archive を `0.4.0` として公開する。
