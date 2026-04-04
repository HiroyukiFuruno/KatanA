## Definition of Ready (DoR)

- The scope is limited to Help demo bundle opening and reference-mode enforcement for `0.16.10`
- Proposal, design, and specs are present under this change directory
- Responsibility boundaries are agreed for menu dispatch, localized asset resolution, document access policy, and integration verification

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. v0.16.10 Help Demo Bundle

- [ ] 1.1 Add the initial `assets/feature` bundle structure with base English Markdown docs, Japanese `.ja.md` variants where needed, and textual reference code assets that the demo will open
- [ ] 1.2 Add localized Help-menu strings and native-menu/app actions for `Help -> Demo`
- [ ] 1.3 Implement the `assets/feature` resolver so Markdown assets follow the `ja`-only override and base-English fallback contract
- [ ] 1.4 Open the resolved demo bundle into the existing tab strip, create or reuse a stable `demo` tab group, and avoid duplicate demo tabs on repeated launches
- [ ] 1.5 Add integration tests for Help-menu dispatch, missing-bundle recovery, locale resolution, and demo-group reuse
- [ ] 1.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] `Help -> Demo` opens the localized `assets/feature` bundle inside the existing tab surface and groups it under `demo`
- [ ] Re-running the Demo action does not replace unrelated tabs or create duplicate demo groups
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. v0.16.10 Reference Mode

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 Extend the document/open state with a per-document access policy so demo-opened code assets can be marked as reference documents
- [ ] 2.2 Route non-Markdown textual demo assets into `CodeOnly` presentation while preserving normal editable behavior for Markdown demo docs
- [ ] 2.3 Make the reference-mode editor non-interactive and block save/update/replace mutation paths so reference documents never become dirty
- [ ] 2.4 Add tests covering read-only rendering, ignored mutation attempts, and save no-op behavior for reference documents
- [ ] 2.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] Demo-opened code assets are viewable in the existing code pane but cannot be edited
- [ ] Reference documents never become dirty and are never written back to disk through normal save or mutation flows
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 3.4 Create a PR targeting `master`
- [ ] 3.5 Merge into master (※ `--admin` is permitted)
- [ ] 3.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.16.10`
- [ ] 3.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
