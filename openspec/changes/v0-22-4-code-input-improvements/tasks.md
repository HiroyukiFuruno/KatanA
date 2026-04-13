## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.22.4 の変更 ID とスコープが確認されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-4-code-input-improvements` またはリリース用統合ブランチ（例: `release/vX.Y.Z`）
- **作業ブランチ**: 標準は `v0-22-4-code-input-improvements-task-x`、リリース用は `feature/vX.Y.Z-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. Task 1

- [ ] 1.1 Task 1 description

### Definition of Done (DoD)

- [ ] (Other task-specific verifiable conditions...)
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Task 2

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 Task 2 description

### Definition of Done (DoD)

- [ ] (Other task-specific verifiable conditions...)
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Create PR from Base Feature Branch targeting `master`
- [ ] 3.4 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 3.5 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.6 Create `release/v0.22.4` branch from master
- [ ] 3.7 Run `make release VERSION=0.22.4` and update CHANGELOG (`changelog-writing` skill)
- [ ] 3.8 Create PR from `release/v0.22.4` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 3.9 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.10 Verify GitHub Release completion and archive this change using `/opsx-archive`
