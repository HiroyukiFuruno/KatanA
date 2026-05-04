# Tasks: Interface Refactoring（master 直接）

> KatanA 側に neutral interface（trait + DTO）を先行定義する。  
> 外部 crate の実装移管・intake は各 vX.X.0 の責務であり本 tasks では扱わない。  
> version branch 不要。master で直接作業・commit する。

---

## 1. Mermaid / Draw.io renderer interface

- [ ] 1.1 `crates/katana-core/src/renderer/mod.rs` を新設し、`RendererTrait`、`RenderInput`、`RenderOutput`、`RuntimeVersion`、`RendererProfile`、`RenderDiagnostics` を定義する
- [ ] 1.2 既存の `mermaid_renderer` / `drawio_renderer` 実装が `RendererTrait` を impl するように切り替える（挙動は変えない）
- [ ] 1.3 `cargo test` がエラーなしで通ること

---

## 2. Document preview interface

- [ ] 2.1 `crates/katana-core/src/preview/mod.rs` に `PreviewRendererTrait`、`PreviewInput`、`PreviewOutput` を定義する
- [ ] 2.2 既存 preview 実装が `PreviewRendererTrait` を impl するように切り替える（挙動は変えない）
- [ ] 2.3 `cargo test` がエラーなしで通ること

---

## 3. Language editor interface

- [ ] 3.1 `crates/katana-core/src/editor/mod.rs` に `EditorTrait`、`EditorConfig`、`SyntaxHighlighter` を定義する
- [ ] 3.2 既存 editor 実装が `EditorTrait` を impl するように切り替える（挙動は変えない）
- [ ] 3.3 `cargo test` がエラーなしで通ること

---

## 4. 確認と commit

- [ ] 4.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 4.2 `katana-core/src/ai/mod.rs` が変更されていないことを確認する（`git diff crates/katana-core/src/ai/`）
- [ ] 4.3 commit & push（master 直接）
