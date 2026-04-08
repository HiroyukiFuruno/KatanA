## Context

Katana v0.17.1ではアイコン基盤を整備しましたが、現在はアプリケーション全体でテーマカラーに応じた単色（モノクロ）で描画され、ユーザーはベンダーごとの色違いや個別のカスタマイズを設定できません。v0.17.2の目標は、アイコンベンダーごとの自動着色、UI上でのベンダー別プレビュー、高度な個別アイコンの設定（ベンダーと色のオーバーライド）、およびプリセット機能を提供することです。

## Goals / Non-Goals

**Goals:**

- 設定画面でのアイコンプレビューをディレクトリ（ベンダー）単位にグループ化する。
- Katanaアイコンはモノクロ（テーマ追従）のまま、他ベンダーのアイコンにはベンダーごとのデフォルトテーマカラーを自動適用する。
- 高度な設定（Advanced Settings）を開くことで、アイコンごとのベンダー（Feather, Heroicons, Katana等）と色（Color32）を個別オーバーライド可能にする。
- ユーザーが行ったカスタマイズを名前をつけてプリセットとして保存、読み込みができ、初期化も可能な仕組みを構成する。

**Non-Goals:**

- ユーザー独自のカスタムSVGファイルをOSからアップロードする機能（組込SVGパックに限定する）。
- ランタイムでのSVG DOM動的書き換え以外の複雑なアニメーションの追加。

## Decisions

**1. 設定データ構造 (Config Schema)**
`crates/katana-core/src/config/theme.rs`（あるいは新規の `icon_settings.rs`）に、各アイコンごとのオーバーライドマップとプリセット定義を導入します。

```rust
pub struct IconOverride {
    pub vendor: Option<String>,
    pub color: Option<String>, // Hex code
}
pub struct IconConfig {
    pub overrides: HashMap<String, IconOverride>, // e.g. "ui/copy" -> Override
}
pub struct Config { ... pub icon_presets: HashMap<String, IconConfig>, pub active_icon_preset: Option<String>, ... }
```

**2. ベンダーごとのデフォルト着色**
UI描画時（`Icon::draw` またはその周辺）で、対象アイコンが所属するベンダーパスを判定します。`katana/` 以下なら currentColor、`feather/` なら青系、`heroicons/` なら紫系といった静的マッピングを用意します。

**3. Settings UI のベンダー別グループ化**
`crates/katana-ui/src/views/settings.rs` のアイコンプレビュー部分で、`IconRegistry` から全アイコンを取得する際、そのパスプレフィックス（`feather`, `katana`, `heroicons`, `lucide` 等）ごとにグルーピングし、それぞれを折りたたみ可能な `egui::CollapsingHeader` に格納して表示します。

## Risks / Trade-offs

- **Risk**: プリセット保存によって設定JSONファイルが肥大化する懸念。
  - **Mitigation**: 変更があった（オーバーライドされた）アイコンのみを保存（Sparseなマップ）するようにし、デフォルト状態は保存しない。
- **Risk**: 個別に色を変えまくると、ライトテーマ/ダークテーマの切り替え時に視認性が失われる。
  - **Mitigation**: ユーザー指定の色オーバーライドは絶対色指定になるため、テーマ切り替え時に注意喚起を出すか、明るさに応じて自動補正（将来課題）するか、現状は「ユーザー責任のAdvanced設定」と位置づける。
