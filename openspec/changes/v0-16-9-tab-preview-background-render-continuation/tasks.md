## Definition of Ready (DoR)

- The target scope is limited to preview lifecycle continuity for tab switching in `v0.16.9`
- Proposal, design, and specs are present under this change directory
- Responsibility boundaries are agreed for preview session state, diagram continuation, and image hydration

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. v0.16.9 Preview Session Lifecycle Foundation

- [ ] 1.1 Replace focus-loss-driven preview cancellation with invalidation-driven cancellation rules tied to source change, explicit refresh, tab close, and workspace teardown
- [ ] 1.2 Add tab-scoped preview session state that tracks lifecycle readiness per section, including `is_loaded` and `is_drawn` semantics, and derive any tab-level summary from those section states
- [ ] 1.3 Stamp preview work with tab/source generation identity and stable section ordinal keys so stale completions are rejected safely
- [ ] 1.4 Add unit tests for lifecycle transitions, generation invalidation, stable section matching, and lazy completion observation after tab reactivation

### Definition of Done (DoD)

- [ ] Non-active tab preview work is no longer canceled only because focus moved to another tab
- [ ] Loaded-but-not-drawn state is representable and usable as a rehydration trigger
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. v0.16.9 Diagram Background Continuation

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 Move Mermaid, PlantUML, and Draw.io tab render jobs onto the new lifecycle so they continue in the background after tab switches
- [ ] 2.2 Reattach completed diagram results on the next tab activation without duplicate renders or stale loading placeholders
- [ ] 2.3 Add tests that switch tabs while diagram work is pending and verify continued progress plus correct hydration on return
- [ ] 2.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] Diagram work for Mermaid, PlantUML, and Draw.io continues across tab switches and reuses completed results on revisit
- [ ] Stale diagram completions are ignored after source invalidation or tab close
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. v0.16.9 Image-Backed Preview Continuation

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 Move extracted local-image readiness out of the current active-draw-only `show_local_image()` path into the tab preview lifecycle, and explicitly validate the CommonMark/HTTP image path under the same reproduction
- [ ] 3.2 Ensure `is_loaded = true` and `is_drawn = false` local-image state hydrates into the visible preview on next activation, and enroll the CommonMark/HTTP path too if the same regression reproduces there
- [ ] 3.3 Add tests for tab switching before local-image sections are ready, for lazy hydration on revisit, and for any reproduced CommonMark/HTTP image regression that was enrolled
- [ ] 3.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] Extracted local-image preview sections do not get stuck in an old loading state after tab switches
- [ ] Loaded local-image results draw on revisit without duplicate loading for the same valid asset
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.16.9`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
