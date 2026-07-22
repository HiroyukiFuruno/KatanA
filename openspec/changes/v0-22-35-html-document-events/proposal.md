## Why

公開済み v0.22.34 の Rust/V8 HTML runtime は element interaction を処理できるが、一般的な初期化経路である `document.addEventListener("DOMContentLoaded", ...)` を実装していない。そのため HTML として妥当な外部 JavaScript が初期化時に例外となり、KatanA から browser-equivalent viewer を評価できない。

## What Changes

- KRR に `document` / `window` EventTarget と browser lifecycle 順序を追加する
- JavaScript 例外へ stack または script/line/column/source を付与する
- KRR runtime source が crates.io package に含まれることを release gate で検査する
- KDV v0.3.3 と KatanA v0.22.35 が公開済み KRR v0.4.5 を registry-only で解決する
- KatanA headless fixture が `DOMContentLoaded` から accordion、click、input、link/navigation を初期化して操作証跡を生成する
- strict coverage 100% / uncovered 0 と Chromium/WebView 禁止を維持する

## Impact

- KRR: in-process Rust/V8 DOM bridge、lifecycle、diagnostics、package gate
- KDV: registry dependency と browser-session adapter contract のみ
- KatanA: registry dependency、headless acceptance、release/SemVer contract

