# Tasks: v0.26.0 Document Preview 分離 — KatanA intake

> preview 実装・絵文字・ダイアグラム呼び出しはすべて `katana-document-preview` repo 側で行う。
> katana-document-preview 側の実装タスクは [katana-document-preview openspec](https://github.com/HiroyukiFuruno/katana-document-preview) を参照。
> 本 tasks.md は KatanA 側の intake（git dependency 追加 + vendor/ 除去 + PreviewWidget 呼び出しへの差し替え）のみを扱う。

## Branch Rule

interface 整理リファクタリングとして `master` で直接作業する（バージョンブランチ不要）。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-document-preview` `v0.1.0` release tag が切られていること
- [ ] `PreviewWidget::show(ui, source, config)` API が確定していること
- [ ] `katana-document-preview`（neutral interface）が egui を含まないことを確認していること

---

## 1. katana-document-preview を git dependency として追加する

- [ ] 1.1 root `Cargo.toml` の workspace dependencies に追加する
  ```toml
  katana-document-preview = { git = "https://github.com/HiroyukiFuruno/katana-document-preview", tag = "v0.1.0" }
  katana-document-preview-egui = { git = "https://github.com/HiroyukiFuruno/katana-document-preview", tag = "v0.1.0" }
  ```
- [ ] 1.2 `cargo build` が通ることを確認する

---

## 2. KatanA 側 preview 描画を PreviewWidget 経由に切り替える

### 準備完了条件

- [ ] Task 1 完了

- [ ] 2.1 `katana-ui` の preview_pane を `PreviewWidget::show()` 呼び出しに差し替える
- [ ] 2.2 `PreviewConfig`（テーマ・フォントサイズ等）を `katana-ui` 側の settings から組み立てて渡す
- [ ] 2.3 絵文字描画が `katana-document-preview-egui` 側で完結していることを目視確認する
- [ ] 2.4 ダイアグラム（Mermaid / Draw.io）が kcf 経由で描画されることを確認する

---

## 3. vendor/ と [patch.crates-io] を除去する

### 準備完了条件

- [ ] Task 2 完了

- [ ] 3.1 `vendor/egui_commonmark_upstream/` を KatanA から除去する
- [ ] 3.2 root `Cargo.toml` の `[patch.crates-io]` から preview 関連エントリを除去する
- [ ] 3.3 `git grep egui_commonmark` で KatanA 内に直接参照が残っていないことを確認する

---

## 4. 検証と commit

- [ ] 4.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 4.2 `./scripts/openspec validate v0-26-0-decouple-preview --strict` を実行し OpenSpec の整合性を確認する
- [ ] 4.3 commit & push（`master` 直接）
