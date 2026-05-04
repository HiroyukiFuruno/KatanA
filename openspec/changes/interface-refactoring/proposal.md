## Why

外部リポジトリ（katana-canvas-forge / katana-document-preview / katana-language-editor / katana-chat-ui）を順次 intake する前提として、KatanA 側に neutral interface（trait + DTO）を先行して定義する。intake 時は KatanA がこの interface 越しに外部 crate を呼ぶだけになり、実装の移管先が変わっても KatanA 本体への影響を最小化できる。

master で直接作業し、version bump・release branch は作成しない。

## What Changes

- **Mermaid / Draw.io renderer interface**: `RendererTrait`、`RenderInput` / `RenderOutput` / `RuntimeVersion` / `RendererProfile` を `katana-core` に定義する
- **Document preview interface**: `PreviewRendererTrait`、`PreviewInput` / `PreviewOutput` を `katana-core` に定義する
- **Language editor interface**: `EditorTrait`、`EditorConfig` / `SyntaxHighlighter` を `katana-core` に定義する
- 既存の内部実装はこれらの trait を impl するように切り替える（挙動は変えない）
- `katana-core/src/ai/mod.rs` の既存 neutral interface（`AiProvider` trait + `AiProviderRegistry`）はそのまま維持する（変更不要）

## Non-Goals

- chat-ui interface の定義は katana-chat-ui 側で行う（KatanA は `katana-chat-ui-egui` を呼ぶだけ）
- 外部 crate の intake・実装の移管は本 change では行わない（各 vX.X.0 の責務）
- UI コンポーネントの分離・Floem 移行は本 change では行わない

## Impact

- `crates/katana-core/src/`: trait / DTO 追加のみ（既存 API の削除・変更なし）
- 外部 intake を受け入れる準備が整う
