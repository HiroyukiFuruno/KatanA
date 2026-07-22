## Context

HTML script は parse 中に `document.readyState === "loading"` で listener を登録し、parse 完了後に `DOMContentLoaded`、microtask、`load` の順で初期化を進める。KRR は element listener を V8 bridge へ接続していたが、Document/Window EventTarget と lifecycle が欠落していた。

## Decisions

1. EventTarget と lifecycle は HTML semantics の owner である KRR に実装する。KDV は session adapter、KatanA は frame/input/navigation consumer のまま維持する。
2. `readyState` は `loading -> interactive -> complete`、イベントは `readystatechange -> DOMContentLoaded -> microtask checkpoint -> readystatechange -> load` の順にする。
3. callback、EventListener object、once、remove、handler property、preventDefault を同じ EventTarget 契約で扱う。
4. lifecycle listener の例外は握り潰さず、V8 stack または source location を KDV/KatanA の既存 typed error 経路へ渡す。
5. headless acceptance は listener 経由で既存の accordion、button、IME input、fragment、別文書 navigation を登録し、単なる静的描画では成功できないようにする。
6. 公開順序は KRR v0.4.5 -> KDV v0.3.3 -> KatanA v0.22.35 とし、最終 lockfile に path/git override を残さない。

## Non-Goals

- Chromium、WebView、外部 helper process の導入
- KDV/KatanA への HTML parser、CSS cascade/layout、JavaScript interpreter、hit-test の追加
- iframe、credential URL、mixed content の許可
- coverage 閾値または SemVer guard の緩和

