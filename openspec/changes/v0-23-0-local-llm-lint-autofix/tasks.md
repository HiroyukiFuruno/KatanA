## Definition of Ready (DoR)
- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.23.0 の変更 ID とスコープが確認されていること
- [ ] `v0.19.0` の markdownlint diagnostics payload が stable であること
- [ ] local endpoint を使う前提が確定していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Provider Settings and Registry Extensions

- [ ] 1.1 local provider 設定 schema に provider kind、endpoint、model、capability を追加する
- [ ] 1.2 `Ollama`、`LM Studio`、OpenAI 互換 local endpoint の provider adapter / preset を実装する
- [ ] 1.3 active provider 切替と永続化を registry に接続する
- [ ] 1.4 availability check と lightweight model 推奨導線を追加する

### Definition of Done (DoD)

- [ ] user が local provider を設定、保存、再選択できること
- [ ] provider 未設定時でも app の通常機能が維持されること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Autofix Request and Apply Pipeline

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 markdownlint diagnostics payload から autofix request を組み立てる
- [ ] 2.2 local provider への request / response を normalized shape で扱う
- [ ] 2.3 autofix 候補の preview / confirm / apply flow を実装する
- [ ] 2.4 apply 後に save、re-lint、error recovery が成立することを確認する

### Definition of Done (DoD)

- [ ] autofix が diagnostic 起点で実行できること
- [ ] user confirmation なしに修正が適用されないこと
- [ ] apply 後に再 lint で結果を確認できること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Settings and Diagnostics UI Integration

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 provider settings UI と test connection 導線を追加する
- [ ] 3.2 diagnostics UI に autofix entry point を追加する
- [ ] 3.3 provider unavailable 時の disabled state と recovery 導線を追加する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] provider 設定から autofix 実行まで UI 上で辿れること
- [ ] unavailable 状態の理由と復旧導線が user に分かること
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
