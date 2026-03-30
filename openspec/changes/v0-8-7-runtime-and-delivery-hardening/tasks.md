## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Failure Contract Alignment

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 1.1 `shell.rs` と `settings_window.rs` の `settings.save()` 呼び出し箇所を棚卸しし、silent failure をなくす対象を確定する
- [x] 1.2 settings / update / release の failure contract（何を守り、どこで止め、何を表示するか）をコード単位で確定する
- [x] 1.3 失敗系を含む追加テストケース（settings save failure, corrupted settings, staged update swap, release preflight）を洗い出す

### Definition of Done (DoD)

- [x] 対象 call site、failure contract、必要テストが implementation 可能な粒度まで整理されている
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. Settings Persistence Hardening

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 2.1 `JsonFileRepository` に temp file + rename ベースの atomic save を実装する
- [x] 2.2 破損した `settings.json` を backup 名で退避し、default への fallback と load diagnostics を追加する
- [x] 2.3 `let _ = settings.save()` を共通の recoverable error handling に置き換える
- [x] 2.4 settings 保存失敗と破損復旧に対する unit / integration test を追加する

### Definition of Done (DoD)

- [x] settings 保存は atomic write になり、破損時と保存失敗時の UI/log 振る舞いが spec を満たす
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. Update Install Hardening

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 3.1 update relaunch script を staged swap + rollback 方式に置き換える
- [x] 3.2 `prepare_update` と relaunch path に bundle 検証、swap failure handling、actionable error message を追加する
- [x] 3.3 更新準備失敗と swap failure を検証する自動テストを追加する

### Definition of Done (DoD)

- [x] destructive replacement を廃止し、更新失敗時でも既存アプリの可用性が保たれる
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 4. Release Pipeline Consistency

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 4.1 ローカル release と GitHub Actions release が共有する preflight entrypoint を抽出する
- [x] 4.2 changelog / OpenSpec / artifact naming の検証を共通 preflight に集約する
- [x] 4.3 `.github/workflows/release.yml` を共通 preflight と artifact verification に合わせて更新する
- [x] 4.4 publish なしで実行できる release helper smoke test を追加する

### Definition of Done (DoD)

- [x] ローカルと CI の release 条件差分がなくなり、publish 前に helper regression を検出できる
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Final Verification & Release Work

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 5.2 Ensure `make check` passes with exit code 0
- [ ] 5.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 5.4 Create a PR targeting `master`
- [ ] 5.5 Merge into master (※ `--admin` is permitted)
- [ ] 5.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.8.7`
- [ ] 5.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
