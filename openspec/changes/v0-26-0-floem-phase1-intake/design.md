## Context

`canvas-forge-intake` で図形描画と export の責務を kcf へ寄せ、その後 `katana-render-runtime` で図形描画 runtime を分離した後、本設計では入力面の痛みを先に解くため、エディタ機能を `katana-language-editor` として独立した外部リポジトリに切り出す。

`katana-language-editor` は **言語非依存の汎用テキストエディタ widget** であり、KatanA は Markdown に特化した `SyntaxHighlighter` 実装を注入して使う。将来 Markdown 以外の言語にも対応できるが、`katana-language-editor` 自体はどの言語かを知らない。

現在 `katana-ui` はエディタウィジェットのドメイン知識（絵文字フォントの workaround、行番号、シンタックスハイライト）を抱え込んでいる。これらを `katana-language-editor-egui` にカプセル化することで KatanA を薄くする。

egui `TextEdit` の完全置き換え（独自 input surface、IME composition、emoji-safe CoreText レンダリング）は `x-x-x-native-input-surface` に劣後する。v0.26.0 はあくまで **crate 境界の確立** を目的とする。

## Goals / Non-Goals

**Goals:**

- エディタ機能を `katana-language-editor` 外部リポジトリに切り出し、KatanA は git dependency として consume するだけにする。
- `SyntaxHighlighter` trait を neutral interface として定義し、KatanA が `MarkdownSyntaxHighlighter` を注入する設計にする。
- フォント・テーマ設定を `EditorConfig` 経由で注入し、katana-ui のグローバル設定直参照を排除する。
- 将来の独自 UI フレームワーク化（egui 脱却）時に `katana-language-editor-egui` を差し替えるだけで KatanA 側が無影響になる構造を確立する。

**Non-Goals:**

- egui `TextEdit` を完全に置き換える独自入力 surface（`x-x-x-native-input-surface` の責務）。
- Preview コンポーネント全体の Floem 移行（v0.27.0 にて実施）。
- `katana-language-editor` に Markdown 固有のドメイン知識を持たせること。

## Decisions

### 1. katana-language-editor は言語非依存

`katana-language-editor` crate は特定の言語（Markdown 等）を知らない。言語固有の振る舞いは `SyntaxHighlighter` trait として外部から注入する。

```rust
// neutral interface（katana-language-editor crate）
pub trait SyntaxHighlighter: Send + Sync {
    fn highlight(&self, source: &str) -> HighlightedText;
    fn language_id(&self) -> &str;
}

pub struct EditorConfig {
    pub syntax_highlighter: Box<dyn SyntaxHighlighter>,
    pub font_size: f32,
    pub theme: EditorTheme,
    // ...
}
```

KatanA 側は `MarkdownSyntaxHighlighter` を実装して渡すだけ。将来別言語を扱う crate も同じ trait を実装すれば良い。

### 2. egui 実装は -egui crate に閉じる

`katana-language-editor`（neutral interface）は egui に依存しない。egui 固有の描画・TextEdit ラップ・行番号・絵文字 workaround は `katana-language-editor-egui` に閉じる。将来 egui を独自フレームワークに替える際は `katana-language-editor-egui` を差し替えるだけ。

### 3. egui 絵文字制約の記録

egui（epaint）は独自フォントアトラスで管理しており、OS のフォントフォールバックチェーンを無視する。Apple Color Emoji（SBIX/CBTF カラーフォント）をロードしても正しく描画できないため、エディタ上での絵文字表示は MVP 段階では制限がある。根本解決は独自 UI フレームワーク（CoreText / Metal 直接描画）導入時に行う。v0.26.0 の egui 実装ではこの制約を docs に明記するにとどめる。

## katana-render-runtime (KRR) / katana-canvas-forge (KCF) および katana-chat-ui との境界

```text
KatanA shell
  -> katana-language-editor    (本 change で分離)
  -> katana-document-preview   (v0.27.0 で Floem 移行)
  -> katana-document-viewer    (preview / export)
  -> katana-render-runtime     (図形描画 runtime)
katana-language-editor -> KDV / KRR / KCF / katana-chat-ui へは依存しない
```

- `katana-language-editor` は純粋なテキスト編集 widget。図形描画・export・LLM chat は関知しない。
- Mermaid / Draw.io ブロックの**シンタックスハイライト**は editor 側の責務。実際の**描画**は preview 側が KDV adapter 経由で KRR を呼ぶ。
- `katana-chat-ui`（v0.27.0 で migration）とも無関係。editor は LLM agent に依存しない。

## Risks / Trade-offs

- **Trade-off**: KatanA 固有の機能追加時に外部 repo と KatanA の両方を修正する必要が増える。肥大化防止・CI 高速化のための許容コストとする。
- **Risk**: egui 絵文字制約により MVP 段階のエディタ品質に限界がある。独自フレームワーク化まで許容し、`EditorConfig` boundary を今から確立することで移行コストを最小化する。
- **Risk**: `egui` バージョン競合。ワークスペース全体で egui バージョンを統一する。
