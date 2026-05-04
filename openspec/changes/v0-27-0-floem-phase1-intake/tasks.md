# Tasks: v0.27.0 Language Editor 分離 — KatanA intake

> editor 実装・シンタックスハイライト・フォント管理はすべて `katana-language-editor` repo 側で行う。
> katana-language-editor 側の実装タスクは [katana-language-editor openspec](https://github.com/HiroyukiFuruno/katana-language-editor) を参照。
> 本 tasks.md は KatanA 側の intake（git dependency 追加 + EditorWidget 呼び出しへの差し替え + SyntaxHighlighter 注入）のみを扱う。

## Branch Rule

interface 整理リファクタリングとして `master` で直接作業する（バージョンブランチ不要）。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-language-editor` `v0.1.0` release tag が切られていること
- [ ] `EditorWidget::show(ui, buffer, config)` API が確定していること
- [ ] `EditorConfig { syntax_highlighter: Box<dyn SyntaxHighlighter>, ... }` が確定していること
- [ ] `katana-language-editor`（neutral interface）が egui を含まないことを確認していること

---

## 1. katana-language-editor を git dependency として追加する

- [ ] 1.1 root `Cargo.toml` の workspace dependencies に追加する
  ```toml
  katana-language-editor = { git = "https://github.com/HiroyukiFuruno/katana-language-editor", tag = "v0.1.0" }
  katana-language-editor-egui = { git = "https://github.com/HiroyukiFuruno/katana-language-editor", tag = "v0.1.0" }
  ```
- [ ] 1.2 `cargo build` が通ることを確認する

---

## 2. KatanA 側 editor を EditorWidget 経由に切り替える

### 準備完了条件

- [ ] Task 1 完了

- [ ] 2.1 `katana-ui` の editor view を `EditorWidget::show()` 呼び出しに差し替える
- [ ] 2.2 `EditorConfig` に KatanA の `MarkdownSyntaxHighlighter` を注入する
  - KatanA は Markdown に特化した `SyntaxHighlighter` 実装を `katana-language-editor` に渡すだけ
  - `katana-language-editor` 自体はどの言語かを知らない
- [ ] 2.3 フォント・テーマ設定を `EditorConfig` 経由で渡し、katana-ui 側のグローバル設定直参照を排除する
- [ ] 2.4 絵文字・IME 関連の workaround が `katana-language-editor-egui` 側に閉じていることを確認する

---

## 3. KatanA 側の editor コードを除去する

### 準備完了条件

- [ ] Task 2 完了

- [ ] 3.1 `katana-ui` から TextEdit ラップ・行番号・シンタックスハイライトの実装を除去する
- [ ] 3.2 `git grep` で `syntect` / `tree-sitter` 等が KatanA 直接依存として残っていないことを確認する

---

## 4. 検証と commit

- [ ] 4.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 4.2 `./scripts/openspec validate v0-27-0-decouple-editor --strict` を実行し OpenSpec の整合性を確認する
- [ ] 4.3 commit & push（`master` 直接）
