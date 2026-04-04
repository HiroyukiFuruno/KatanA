## Definition of Ready (DoR)

- The scope is limited to the direct pinned-tab toggle behavior for `0.16.6`
- Proposal, design, and specs are present under this change directory
- Responsibility boundaries are agreed for hit testing, action routing, and UI verification

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. v0.16.6 Pinned Tab Direct Toggle

- [ ] 1.1 Separate the hit targets for the pinned-tab body and pin icon, then connect pin-icon clicks to `TogglePinDocument`
- [ ] 1.2 Verify there is no regression in context-menu pin/unpin flows, pinned-tab ordering, or group-removal rules
- [ ] 1.3 Add a UI interaction test that simulates a real pin-icon click and confirms direct unpin behavior
- [ ] 1.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] A pinned tab can be unpinned with one click on the visible pin icon
- [ ] Tab activation and unpin actions are not conflated
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Final Verification & Release Work

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 2.2 Ensure `make check` passes with exit code 0
- [ ] 2.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 2.4 Create a PR targeting `master`
- [ ] 2.5 Merge into master (※ `--admin` is permitted)
- [ ] 2.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.16.6`
- [ ] 2.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
