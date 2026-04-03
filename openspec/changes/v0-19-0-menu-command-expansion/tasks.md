## Definition of Ready (DoR)
- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.19.0 の変更 ID とスコープが確認されていること
- [ ] 現行の native menu、command palette、`AppAction` 一覧を再確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Shared Command Inventory

- [ ] 1.1 user-facing commands の棚卸しを行う
- [ ] 1.2 label、group、availability を持つ shared command inventory を導入する
- [ ] 1.3 menu、palette、future shortcut editor が inventory を参照できる shape を定義する

### Definition of Done (DoD)

- [ ] command inventory が単一の source of truth として成立していること
- [ ] `AppAction` と inventory の責務分離が明確であること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. File and View Menu Expansion

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 File menu に workspace / document commands を追加する
- [ ] 2.2 View menu に navigation / visibility commands を追加する
- [ ] 2.3 disabled state を inventory の availability と揃える
- [ ] 2.4 non-macOS command surface でも同等 coverage を提供する
- [ ] 2.5 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 2.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] File / View menu の command coverage が設計通りに増えていること
- [ ] surface 間で availability judgment が一致していること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Help Menu and Palette Alignment

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 Help menu に docs / GitHub / release notes / update commands を整理して追加する
- [ ] 3.2 command palette が inventory 由来の labels と groups を使うようにする
- [ ] 3.3 docs と i18n copy を更新する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] Help menu と palette が inventory contract に揃っていること
- [ ] i18n copy と docs が新しい command surface を反映していること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.19.0`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
