## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.19.0 の変更 ID とスコープが確認されていること
- [ ] 現行の diagnostics 実装、Problems Panel、archived `markdown-diagnostics` change を再確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Supported Rule Catalog and Parity Contract

- [ ] 1.1 現行 internal diagnostics rule を棚卸しし、official markdownlint rule へ対応付ける
- [ ] 1.2 user-facing diagnostics から internal rule 名を排除する contract を定義する
- [ ] 1.3 official rule code、title、English description、docs URL を持つ catalog を追加する
- [ ] 1.4 parity 未達 rule の hidden / experimental 扱いを定義する

### Definition of Done (DoD)

- [ ] user-facing diagnostics に internal rule 名が残っていないこと
- [ ] parity 未達 rule の扱いがコードと docs の両方で明確であること
- [ ] official metadata の source of truth が 1 箇所に固定されていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Diagnostics Engine Parity

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 user-facing official rule ごとに detection behavior を実装または修正する
- [ ] 2.2 diagnostics payload に official rule metadata を追加する
- [ ] 2.3 violation / valid fixture に基づく parity regression test を追加する
- [ ] 2.4 false positive / false negative が既知ケースで抑えられていることを確認する

### Definition of Done (DoD)

- [ ] user-facing rule について official behavior と整合する diagnostics が返ること
- [ ] future autofix で再利用できる payload shape が固定されていること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Problems Panel UX

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 Problems Panel に official rule code、English description、location、severity を表示する
- [ ] 3.2 diagnostic item から editor / preview の該当 location へ jump できることを確認する
- [ ] 3.3 docs link または同等の公式参照導線を追加する
- [ ] 3.4 parity 未達 rule が user-facing official result と混同されないことを UI 上で確認する
- [ ] 3.5 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] Problems Panel だけで official rule の識別と jump ができること
- [ ] parity 未達 rule と official parity rule が UI 上で混同されないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Refresh and Compatibility Hardening

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 manual refresh と save-triggered refresh の両方で parity diagnostics が更新されるようにする
- [ ] 4.2 parity / experimental rule contract の回帰テストを追加する
- [ ] 4.3 docs と status copy を v0.19.0 の contract に合わせて更新する

### Definition of Done (DoD)

- [ ] diagnostics refresh policy が deterministic に動作すること
- [ ] official metadata drift を検出できるテストまたは検証手段があること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Final Verification & Release Work

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 5.2 Ensure `make check` passes with exit code 0
- [ ] 5.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 5.4 Create a PR targeting `master`
- [ ] 5.5 Merge into master (※ `--admin` is permitted)
- [ ] 5.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.19.0`
- [ ] 5.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
