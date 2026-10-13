## Context

旧 renderer は `mmdc` を起動し、Mermaid CLI と Puppeteer に PNG 出力を任せていた。現行 renderer は `~/.local/katana/mermaid.min.js` を読み込み、ヘッドレスブラウザ上で Mermaid.js を実行して `#diagram` を screenshot する。

この移行で `mmdc` 依存は消えた。一方で、`mmdc` が持っていた「ページ幅」「背景色」「テーマ」「PNG 出力」「Puppeteer capture」といった出力前提は、KatanA 側で明示的に持たないと図形ごとに見た目が揺れる。

また、現在の headless browser（画面を出さないブラウザ）経路は、実体として OS にインストールされた Chrome / Chromium アプリを起動する。これは見えなくても OS アプリ依存であり、通常 preview と HTML export（HTML出力）の既定経路としては扱わない。

移行順序は、まず Rust 管理 JS（Rust 側が所有する JavaScript 実行環境）で公式 Mermaid.js / Drawio.js を動かせるか試す。DOM / SVG / layout API の不足で描画互換性や速度を満たせない場合、KatanA 管理下の高速な headless browser（画面なしブラウザ）/ WebView（アプリ内ブラウザ部品）/ Chromium（Chrome 系ブラウザエンジン）から単一の採用経路を選ぶ。Rust 製または Rust 管理で高速な headless browser が今回の用途を満たす場合は、preview と export の共通 runtime 候補として優先的に評価する。実行時の退避経路（fallback）はロジックを複雑にするため持たない。

## Goals

- `mmdc` を参照実装として扱い、実行時依存として再導入しない。
- 通常 preview と HTML export から OS Chrome / Chromium 依存を外す。
- まず Rust 管理 JS で公式 Mermaid.js / Drawio.js を動かす spike を行う。
- Rust 管理 JS が崩れなく速い描画を満たせない場合、Mermaid と Draw.io の通常 preview / HTML export は KatanA 管理下の高速な headless browser / WebView / Chromium から単一の採用経路を選ぶ。
- `mmdc` がきれいに出力していた条件を、KatanA の Mermaid renderer に移植する。
- 図形ごとの fixture と証跡で、サイズ、余白、中央寄せ、テーマ色、フォント、特殊マーカー、エラー表示の差分を確認できるようにする。
- 修正対象を renderer の責務、fixture / test の責務、後続 change の責務に分ける。

## Non-Goals

- 実行時に `mmdc` を必須に戻さない。
- OS にインストールされた Chrome / Chromium を通常 preview の必須条件にしない。
- Mermaid.js / Drawio.js を使わない Rust-native renderer へ切り替えない。
- Rust 側で試す場合も、公式 Mermaid.js / Drawio.js を Rust 管理の JS 実行環境で動かす方式に限定する。
- 実行時の退避経路（fallback）を持ち込まない。
- Mermaid.js 以外の Draw.io / PlantUML を主対象にしない。
- この change ではユーザー設定を増やさない。
- ピクセル完全一致を目標にしない。
- Mermaid renderer の汎用 interface 化、Mermaid.js の version 固定、`katana-renderer` 分離設計は v0.22.11 へ移管する。
- Draw.io 描画 runtime の所有境界は v0.22.11 の分離設計で整理する。ただし HTML 生成不正と PDF / PNG / JPEG export の Chromium 依存除去は v0.22.10 の残課題として扱う。
- 公式比較画像との採点評価は v0.22.10 では手動検証の補助に留める。直近の合格ラインは公式ドキュメント由来の Draw.io fixture 85 点以上とし、日本語版（ja）評価、保存時チェック（pre-commit）や CI/CD の必須検証に組み込む設計は、`katana-renderer` 分離後に扱う。
- Linux Homebrew cask 対応は、KatanA の GUI アプリ配布経路として扱う。CLI 用または互換目的の Formula が残る場合でも、GUI アプリ導線は `brew install --cask` に寄せる。

## Approach

1. 旧 `mmdc` 経路の出力契約を抽出する。
   - `mmdc` に渡していた引数
   - `mmdc -h` で確認できる既定 width / height / background / scale
   - テーマ、背景色、フォント、PNG 出力の扱い
1. Browser dependency の方針を決める。
   - Rust 管理 JS で公式 Mermaid.js / Drawio.js を動かす spike を先に行う
   - Rust 管理 JS が不採用の場合に高速な headless browser / WebView / Chromium から単一の採用経路を選ぶ
   - OS Chrome / Chromium アプリ起動を禁止する境界を preview / HTML export / PDF / PNG / JPEG export ごとに決める
   - platform ごとの runtime と配布方法を決める
   - v0.22.10 で移行しきれない Draw.io 経路や特殊ケースは後続 versioned change に分離する
1. 採用した単一 Mermaid renderer の描画条件を確認する。
   - ヘッドレスブラウザの window size
   - HTML / body / container の幅
   - Mermaid 初期化設定
   - SVG `getBBox()` と `viewBox` の再設定
   - screenshot 対象
   - キャッシュキーに含める条件
1. `mmdc` 由来の条件を KatanA 側の renderer policy として実装する。
   - 標準 render width
   - content-based cropping
   - 最小余白
   - 最大 capture width
   - 背景色と透明背景
   - theme variables
   - 図形固有マーカーの扱い
1. 代表 fixture で回帰検知する。
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
1. 修正方針を分類する。
   - renderer policy で解決するもの
   - Mermaid 初期化設定で解決するもの
   - SVG 後処理が必要なもの
   - Mermaid.js 側の仕様差として許容または後続 versioned change に送るもの
1. Linux Homebrew cask の成立条件を固定する。
   - Homebrew の Linux cask 対応範囲を確認する
   - `homebrew-katana` の macOS cask / Linux formula / Linux cask の責務を分離する
   - Linux release asset の URL、sha256、実行ファイル配置、desktop entry、icon 配置を cask で表現できるか確認する
   - `just linux-up` 環境で tap、install、launch、uninstall を検証する
   - 成立しない場合は Formula へ曖昧に戻さず、制約と後続配布方式を記録する

## Risks

- `mmdc` と Mermaid.js のバージョン差をピクセル一致で追うと、将来の Mermaid 更新に追従しづらくなる。
- headless browser / WebView / Chromium runtime は platform ごとに配布・sandbox・CI の差が出る可能性がある。
- Rust 管理の JS 実行環境は、Mermaid.js / Drawio.js が期待する DOM / SVG / layout API を再現できない可能性がある。
- 図形ごとに最適な container 幅が異なる場合、単一の固定幅では別の図形を崩す可能性がある。
- SVG 後処理を増やしすぎると、Mermaid.js 公式更新への追従が難しくなる。
- Linux cask は Homebrew 側の対応範囲、cask DSL の Linux artifact 表現、GUI 起動統合に制約がある。特に desktop entry や icon 配置が Homebrew の期待から外れる場合、Formula ではなく別配布方式へ切り出す判断が必要になる。

## Runtime Selection Criteria

Rust 管理 JS が不採用になった場合、高速な headless browser / WebView / Chromium の比較は「表示が崩れず速い方」を主基準にする。ただし、速度だけで決めない。採用後は単一経路として実装し、実行時の退避経路（fallback）は作らない。

- 表示互換性: Mermaid.js / Drawio.js の公式出力に近く、既存 fixture で崩れないこと
- 速度: 初回起動、連続描画、cache hit / miss の体感待ち時間が短いこと
- 所有境界: OS にインストールされた Chrome / Chromium アプリを起動せず、KatanA 管理下の runtime で完結すること
- 共通性: preview、HTML export の diagram rendering、HTML から PDF / PNG / JPEG への変換を同じ runtime 境界で扱えること
- 配布: アプリサイズ、追加 runtime の有無、更新方法が許容できること
- platform 差分: macOS / Windows / Linux で同じ rendering contract を保てること
- isolation: preview 用 JS がアプリ本体やユーザー環境へ不要な副作用を持たないこと
- CI: headless 検証が安定し、専用 OS アプリのインストールに依存しないこと

## Open Questions

- `mmdc` 互換の許容範囲を、ピクセル一致ではなく「ユーザーが見て崩れていない」基準にするか。
- Rust 管理 JS で DOM / SVG / layout API をどこまで現実的に満たせるか。
- 高速な headless browser、platform-native WebView、管理下 Chromium runtime のどれを採用するか。
- 図形ごとの固定幅を持つべきか、全 Mermaid 図形で共通の既定幅を維持するか。
- 分離後の `katana-renderer` で、公式比較画像との採点評価を保存時チェック（pre-commit）や CI/CD にどの粒度で入れるか。
- Linux cask の token は既存 macOS cask と同じ `katana-desktop` に統合するか、Linux 専用 token を分けるか。
- Linux GUI アプリの desktop entry / icon / executable link を cask 側でどこまで管理し、どこから release asset 側の責務にするか。
