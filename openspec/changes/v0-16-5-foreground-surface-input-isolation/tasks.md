## Definition of Ready (DoR)

- The scope is limited to foreground-surface input isolation for `0.16.5`
- Proposal, design, and specs are present under this change directory
- Responsibility boundaries are agreed for shell blocker registration and UI verification

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. v0.16.5 Foreground Surface Input Isolation

- [ ] 1.1 Define a blocker contract and blocker-source inventory so the shell can determine foreground window / popup / overlay / detached-surface active state in one place
- [ ] 1.2 Wire blocker control into the settings window, command palette, file search modal, file-operation/about/meta/update/terms windows, tab/workspace context menus, history/breadcrumb/group popups, settings-local popups, splash overlay, and fullscreen / slideshow / detached surfaces
- [ ] 1.3 Ensure workspace tree, editor, preview, and tab-strip hover/click/context-menu reactions do not update while the blocker is active
- [ ] 1.4 Add UI integration tests that cover the settings window, command palette, file search modal, representative context/popup menus, settings-local popups, and fullscreen/slideshow surfaces without using visual snapshots
- [ ] 1.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] Background-pane hover/click state does not change while a settings window, command palette, file search modal, context/popup menu, settings-local popup, or detached/overlay surface is open
- [ ] Close, scroll, and click operations on the foreground surface itself remain functional
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
- [ ] 2.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.16.5`
- [ ] 2.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
