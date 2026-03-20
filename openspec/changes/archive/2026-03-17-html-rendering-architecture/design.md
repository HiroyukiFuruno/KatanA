## Context

現在のKatanAプレビューペインは、Markdown中のHTML要素（`<p align="center">`, `<img>`, `<a>`等）を正規表現で場当たり的にMarkdown構文に変換し、手動で描画している。この実装には以下の制約がある:

1. HTML要素のinline/block区別がなく改行制御が正しくない
2. リンクの種類（外部/内部/アンカー）が未分類で、開き方の制御ができない
3. パースロジックがegui UIコンテキストに依存し、ユニットテスト不可能
4. 新要素の追加が困難で技術的負債が蓄積される

### 技術スタック
- **パース**: `comrak` crate のAST（`NodeValue::HtmlBlock`, `HtmlInline`）を活用 + 内部タグ抽出に `regex`
- **UI**: egui 0.33 + egui_extras 0.33（SVGテキストは `svg_text` feature）
- **既存**: `comrak` でMarkdown→AST変換済み。HTMLブロックは `NodeValue::HtmlBlock` として取得可能

## Goals / Non-Goals

**Goals:**
- HTMLの inline/block 表示モードをタグごとに正しく分類し、改行制御を標準準拠にする
- リンク先を external/internal/anchor に分類し、開き方のポリシーを定義する
- パースとレンダリングを完全に分離し、パーサーをUI非依存でテスト可能にする
- メソッドチェーンでレンダラーの設定を組み合わせ可能にする
- 将来のナビゲーション機能（戻る/進む、タブ管理）を見据えた拡張ポイントを確保する

**Non-Goals:**
- 完全なHTMLレンダリングエンジンの実装（対象はMarkdown内のHTML要素に限定）
- CSSパーサーの実装（`align` 属性など既知の属性のみ対応）
- JavaScript実行
- ナビゲーション機能の実装そのもの（インターフェースの定義まで）

## Decisions

### D1: 3層アーキテクチャ（パーサー → モデル → レンダラー）

**選択**: パース・モデル・レンダリングの3層に分離
**理由**: パーサーとモデルを `katana-core`（UI非依存）に配置することで、ユニットテストでパース結果を検証可能にする。レンダラーのみ `katana-ui` に配置。

**代替案**:
- 全てを `katana-ui` に配置 → テスト不可能、却下
- `html5ever` 依存 → 正確だがオーバースペック。現時点では不要

### D2: `comrak` AST の活用（自前HTMLパースの最小化）

**選択**: `comrak` のAST（`NodeValue::HtmlBlock`, `HtmlInline`）からHTML文字列を取得し、内部タグの属性抽出のみ regex で行う
**理由**: `comrak` は既にMarkdown→ASTの変換を行っており、HTMLブロックの判定も完了している。HTMLブロックの内部タグ（`<img>`, `<a>`, `<p>` 等）の属性抽出は浅いネストに限定されるため、regex で十分。完全な自前HTMLパーサーは車輪の再発明。

**代替案**:
- 完全自前パーサー → comrak と責務が重複、却下
- `html5ever` で内部タグもパース → 依存が大きい。将来の移行パスとして `HtmlNode` ツリー構造を採用しておく

### D3: `HtmlNode` enumによる要素モデル

**選択**: `HtmlNode` enum + `display_mode()` メソッド
**理由**: Rustのenumはバリアントの網羅性チェック（match exhaustive）が効くため、新要素追加時にコンパイラが漏れを検出する。

```rust
pub enum HtmlNode {
    Text(String),
    Image { src: String, alt: String },
    Link { target: LinkTarget, children: Vec<HtmlNode> },
    Heading { level: u8, children: Vec<HtmlNode> },
    Paragraph { align: Option<TextAlign>, children: Vec<HtmlNode> },
    LineBreak,
    Emphasis(Vec<HtmlNode>),
    Strong(Vec<HtmlNode>),
}
```

### D4: `LinkTarget` enumによるリンク分類

**選択**: パース時にリンク先を3種に分類し、`default_action()` でデフォルトの開き方を返す
**理由**: リンクの種類判定を1箇所に集約。将来のナビゲーション機能は `LinkAction` の種類を増やすかリスナー/コールバックを追加するだけで対応可能。

```rust
pub enum LinkTarget {
    External(String),       // http(s)://
    InternalFile(PathBuf),  // 相対パス → 絶対パス
    Anchor(String),         // #section
}

pub enum LinkAction {
    OpenInBrowser,          // 外部リンクのデフォルト
    NavigateCurrentTab,     // 内部リンクのデフォルト
}
```

**注**: `OpenInNewTab` は現時点では定義しない（YAGNI）。タブ管理機能の実装時に追加する。

### D5: `HtmlRenderer` メソッドチェーンパターン

**選択**: Builder-likeなメソッドチェーン
**理由**: レンダリング設定（テキスト色、画像最大幅、SVG拡張子補正など）をコール元が柔軟に組み合わせ可能。

```rust
HtmlRenderer::new(ui, base_dir)
    .text_color(color)
    .max_image_width(width)
    .render(&nodes);
```

### D6: inline グループの中央寄せ戦略

**選択**: 連続するインライン要素を収集し、ブロック要素に遭遇するか末尾に到達したら一括で描画。中央寄せ Paragraph 内のインライングループは `ui.allocate_ui_with_layout()` を使い、コンテンツ幅に応じた最小幅UIで中央配置。
**理由**: `ui.horizontal()` は利用可能幅いっぱいに広がるため、親の `Align::Center` が効かない。`allocate_ui_with_layout` で幅を制限することで中央寄せを実現。

### D7: SVG拡張子補正の責務

**選択**: `HtmlRenderer` レイヤーで画像URL変換（`ensure_svg_extension`）
**理由**: これはegui固有の制約（SVGローダーがURI末尾をチェック）への対応であり、モデル層ではなくレンダリング層の責務。

### D8: `RenderedSection` の汎化

**選択**: `CenteredMarkdown(String)` → `HtmlBlock(Vec<HtmlNode>)` に汎化
**理由**: 現在は中央寄せ HTML のみだが、将来的に `<div>`, `<details>`, `<table>` 等の非中央寄せ HTML も処理する必要がある。align 属性はノード自体（`Paragraph { align }`)が保持するため、セクション型が「中央寄せ」を意識する必要がない。

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| `comrak` AST のHTMLブロック内部構造がバージョンで変わる可能性 | `HtmlNode` アダプター層がバージョン差異を吸収。UT で検出可能 |
| regex での属性抽出の限界（引用符のエスケープ等） | 対象は Markdown 内の HTML に限定。実用上問題になるケースは稀 |
| `HtmlNode` バリアント増加による match 肥大化 | visitor パターンへの移行ポイントを `render()` メソッドに明確化 |
| メソッドチェーンと借用の衝突（`&mut ui` の複数借用） | `HtmlRenderer` はレンダリング呼び出しごとに生成する短命オブジェクト設計 |
| 中央寄せインライングループの幅計算の難しさ | 1フレーム目は推定値で描画し、2フレーム目以降は前フレームの実測値で調整（egui の standard pattern） |
