# 実装タスク: タブグループとセッション UX 改善 (v0.11.0)

## 着手条件 (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 現行の tab pinning / close actions / workspace-scoped session restore（`views/top_bar.rs`、`app/action.rs`、`app/workspace.rs`）を確認していること

## ブランチ運用ルール

`##` ごとに grouped された task は、`/openspec-branching` workflow（`.agents/workflows/openspec-branching.md`）で定義された branching standard を無条件で守って実装すること。

---

## 1. セッションモデルと永続化の拡張

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 1.1 既存 `workspace_tabs:{workspace_path}` payload を versioned session envelope に置き換える設計を実装する
- [ ] 1.2 session envelope に `tabs`、`active_path`、`expanded_directories`、`groups`、`version` を保持できるようにする
- [ ] 1.3 tab entry に pinned state を保存し、restore 時に `Document.is_pinned` へ反映する
- [ ] 1.4 legacy payload（`tabs`、`active_idx`、`expanded_directories`）を read-time upgrade で受けられるようにする
- [ ] 1.5 restore ON / OFF setting を workspace/session settings に追加し、既存 settings と serde 互換を保つ

### 完了条件 (DoD)

- [ ] workspace-scoped session save / load が grouped / pinned tab を扱えること
- [ ] 旧 payload からの read が default 補完で成立すること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 2. タブグループ UI と runtime state

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 2.1 runtime tab group state（`id`、`name`、`color_hex`、`collapsed`、`members`）を定義する
- [ ] 2.2 tab context menu に group create / add / remove を追加し、1 tab が高々 1 group に所属する制約を守る
- [ ] 2.2.1 pinned tab には group add UI を出さない、または無効化し、grouped tab を pin した場合は membership を外す
- [ ] 2.3 group header の rename / color change / collapse toggle UI を実装する
- [ ] 2.4 `views/top_bar.rs` で group block を描画し、open tab order の最初の member 位置に anchored する projection を実装する
- [ ] 2.5 collapsed group が member tab を非表示にするだけで close しないことを保証する
- [ ] 2.5.1 active tab が collapsed group に属する場合は、その active member だけ visible に保つ
- [ ] 2.6 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 2.7 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] group create / rename / recolor / add / remove / collapse が一通り動作すること
- [ ] grouped tab が workspace 再オープン後に復元されること
- [ ] group / pin の相互作用が design どおりに固定されていること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 3. pinned タブ保護

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 3.1 pinned tab の close button を非表示にし、tooltip 表示は維持する
- [ ] 3.2 `CloseDocument` が pinned tab を通常 close しないようにする
- [ ] 3.3 `CloseAllDocuments` / `CloseOtherDocuments` / `CloseDocumentsToRight` / `CloseDocumentsToLeft` が pinned tab をスキップするようにする
- [ ] 3.4 close shortcut から dispatch される close action でも pinned safeguard が有効であることを確認する
- [ ] 3.5 unpin 後は通常 close path に戻ることを確認する
- [ ] 3.6 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.7 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] pinned tab が通常 UI と batch close から削除されないこと
- [ ] unpin 後は通常 close できること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 4. 検証と復旧経路

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 4.1 legacy session payload の read-time upgrade を test で確認する
- [ ] 4.2 grouped / pinned / restore setting OFF の各 session restore path を test で確認する
- [ ] 4.3 close policy の regression test を追加し、pinned tab が batch close から保護されることを確認する
- [ ] 4.4 実装途中に canonical order や session model の前提が崩れた場合、artifact が先に更新されていることを確認する

### 完了条件 (DoD)

- [ ] session persistence / group rendering / pin safeguards の主要 regression が test 化されていること
- [ ] upgrade path と restore setting OFF path が確認されていること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 5. 最終確認とリリース作業

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を使って self-review を実施する（各 file の version 更新漏れも確認する）
- [ ] 5.2 `make check` が exit code 0 で通過することを確認する
- [ ] 5.3 中間 base branch（もともと master から派生した branch）を `master` へ merge する
- [ ] 5.4 `master` 向け PR を作成する
- [ ] 5.5 `master` へ merge する（`--admin` 許可）
- [ ] 5.6 `.agents/skills/release_workflow/SKILL.md` を使って `0.11.0` の release tagging と release 作成を実施する
- [ ] 5.7 `/opsx-archive` など OpenSpec skill を使ってこの change を archive する
