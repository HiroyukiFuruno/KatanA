## Definition of Ready (DoR)
- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.23.0 の変更 ID とスコープが確認されていること
- [ ] `v0.22.0` の local provider 設定と availability 判定が利用可能であること
- [ ] current document / new file / template scaffold の 3 出力先を同時に扱う前提が確認されていること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Generation Job Model and Context Assembly

- [ ] 1.1 current document、new file、template scaffold を表現する generation job model を定義する
- [ ] 1.2 active document、selection、workspace から context を構築する
- [ ] 1.3 generation input の size 制御と target metadata を整理する
- [ ] 1.4 write 前 preview に必要な normalized response shape を定義する

### Definition of Done (DoD)

- [ ] 3 種類の出力先が 1 つの generation contract で表現できること
- [ ] prompt に渡す context 範囲が明示されていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Write and Insert Execution Pipeline

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 current document insert の preview / apply / undo-friendly flow を実装する
- [ ] 2.2 new file creation の preview / save flow を実装する
- [ ] 2.3 template scaffold の preset / destination / save flow を実装する
- [ ] 2.4 write 後の refresh、dirty state、file collision handling を統一する

### Definition of Done (DoD)

- [ ] current document、new file、template scaffold の 3 方式で write が成立すること
- [ ] いずれも user confirmation 前に書き込みが走らないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. UI Integration and Review Flow

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 current document、new file、template scaffold の entry point を UI に追加する
- [ ] 3.2 generation preview、target selection、confirmation 導線を追加する
- [ ] 3.3 provider unavailable、file collision、empty result の error state を追加する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] 3 つの generation flow を UI 上で区別して使えること
- [ ] review してから反映する導線が user に分かること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.23.0`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
