## Definition of Ready (DoR)

- The scope is limited to rich-block highlight and split sync stabilization for `0.16.8`
- Proposal, design, and specs are present under this change directory
- Responsibility boundaries are agreed for rich-block source mapping and integration verification

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. v0.16.8 Rich Block Highlight and Split Sync Stabilization

- [ ] 1.1 Extend block-level source-anchor mapping so Mermaid, PlantUML, Draw.io, and GitHub Flavored Markdown alert/admonition sections resolve stable source ranges
- [ ] 1.2 Route preview hover highlighting through that mapping so rich blocks resolve the correct source range
- [ ] 1.3 Update split-scroll mapping and geometry refresh handling to remove drift around diagram and alert/admonition block boundaries, including async render completion for diagram kinds
- [ ] 1.4 Add Mermaid, PlantUML, Draw.io, and alert/admonition integration tests for hover highlight and split sync using response-based assertions
- [ ] 1.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] Mermaid, PlantUML, Draw.io, and alert/admonition preview hover interactions all highlight the matching source range
- [ ] Split sync and source mapping stay aligned before/after diagram and alert/admonition blocks and still converge after async render completion where applicable
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
- [ ] 2.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.16.8`
- [ ] 2.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
