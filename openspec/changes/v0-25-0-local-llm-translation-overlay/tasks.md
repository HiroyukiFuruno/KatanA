## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.25.0 の変更 ID とスコープが確認されていること
- [ ] `v0.23.0` の local provider 基盤が translation に再利用可能であること
- [ ] release 時点での dynamic / external English target inventory を洗い出す方針が確認されていること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Translation Target Inventory and Eligibility Rules

- [ ] 1.1 diagnostics、AI result、external text の translation target inventory を作成する
- [ ] 1.2 eligible / ineligible text の判定ルールを定義する
- [ ] 1.3 original English text と translated view の共存方針を定義する
- [ ] 1.4 translation cache key と invalidation rule を定義する
- [ ] 1.5 overlay generated / non-English / translation in progress を除外する rule を定義する

### Definition of Done (DoD)

- [ ] translation を掛ける対象範囲が明文化されていること
- [ ] static i18n と dynamic translation overlay の責務境界が明確であること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Translation Pipeline and Cache

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 local provider を translation request path に接続する
- [ ] 2.2 source text、target language、provider context を基にした cache を実装する
- [ ] 2.3 translation failure、timeout、invalid response の fallback を実装する
- [ ] 2.4 language 切替と provider 切替時の cache behavior を確認する
- [ ] 2.5 overlay text の再翻訳と二重 request が起きないことを確認する

### Definition of Done (DoD)

- [ ] auto translation が local provider enabled 時にのみ動作すること
- [ ] translation failure でも original English text が失われないこと
- [ ] overlay generated text を再翻訳しないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. UI Overlay Integration and Feedback

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 diagnostics、AI result、その他 eligible target に translation overlay を追加する
- [ ] 3.2 original English text を参照する導線を追加する
- [ ] 3.3 auto translation の loading / cached / fallback state を UI に反映する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告（オーバーレイ表示時におけるプレビューのスクロール同期やテキスト選択挙動の変化を含めて入念に検証する）
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] translated view と original English text の両方を user が確認できること
- [ ] auto translation の loading / failure が UI を壊さないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.25.0`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
