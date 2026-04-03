## Definition of Ready (DoR)
- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.20.0 の変更 ID とスコープが確認されていること
- [ ] `v0.19.0` の command inventory が利用可能であること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Shortcut Schema and Defaults

- [ ] 1.1 command inventory key と default shortcut set を定義する
- [ ] 1.2 settings schema と persistence に shortcut bindings を追加する
- [ ] 1.3 existing hard-coded shortcuts から migration する

### Definition of Done (DoD)

- [ ] shortcut schema が永続化可能であり、default set が固定されていること
- [ ] migration 方針が明文化されていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Runtime Shortcut Dispatcher

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 runtime shortcut dispatcher を inventory-driven に置き換える
- [ ] 2.2 platform-aware modifier handling を整理する
- [ ] 2.3 app-local duplicate detection を実装する

### Definition of Done (DoD)

- [ ] custom binding が runtime で正しく dispatch されること
- [ ] duplicate binding が runtime ambiguity を起こさないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Settings UI and Conflict Popup

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 shortcut settings UI を追加する
- [ ] 3.2 conflict 時に既存割当先を表示する popup を追加する
- [ ] 3.3 restore defaults を実装する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] shortcut の確認、変更、restore defaults が UI から行えること
- [ ] conflict popup が既存割当先を user-facing label で表示すること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.20.0`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
