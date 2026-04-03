## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.17.0 の変更 ID とスコープが確認されていること
- [ ] 現行 icon registry、SVG loader、SVG linter、settings schema を再確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Icon Pack Contract and Asset Layout

- [ ] 1.1 existing `assets/icons` を `assets/icons/katana/...` 配下へ再編し、`katana-icon` pack の asset root を固定する
- [ ] 1.2 icon pack manifest または同等 metadata を追加し、pack id、display name、render policy、license metadata を表現できるようにする
- [ ] 1.3 `Icon` enum 全件を pack coverage table で確認できる contract を作る
- [ ] 1.4 curated external pack 5 種類の採用候補と source / license を固定する
- [ ] 1.5 built-in pack の directory 命名規則を `assets/icons/<pack-dir>/...` に統一する

### Definition of Done (DoD)

- [ ] `katana-icon` が existing default として定義されていること
- [ ] built-in pack が `assets/icons/katana`、`assets/icons/<external-pack>` のように pack 単位の階層へ整理されていること
- [ ] shipping pack の metadata source of truth が 1 箇所に固定されていること
- [ ] pack coverage を確認できる一覧または検証手段があること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Runtime Registry and Color-aware Rendering

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 icon registry を pack-aware に変更し、active pack から asset 解決できるようにする
- [ ] 2.2 selected pack の direct asset、pack override、`katana-icon` fallback の順で解決する safety net を追加する
- [ ] 2.3 `TintedMonochrome` と `NativeColor` の rendering policy を追加する
- [ ] 2.4 current white-only icon validation を pack policy aware に更新する

### Definition of Done (DoD)

- [ ] active pack を切り替えると icon asset 解決先が変わること
- [ ] colorful pack が tint で潰れずに表示されること
- [ ] asset 欠落時も recoverable fallback で UI が壊れないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Settings UI and Live Preview

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 settings に icon pack selection UI を追加する
- [ ] 3.2 pack 一覧に preview、display name、必要なら render policy の説明を表示する
- [ ] 3.3 selected pack を settings に保存し、次回起動時に復元する
- [ ] 3.4 pack 切り替えを再起動なしで即時反映する
- [ ] 3.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] settings だけで icon pack を切り替えられること
- [ ] selected pack が restart 後も復元されること
- [ ] live preview と実際の UI icon 表示が一致すること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Curated Pack Import, Overrides, and License Inventory

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 curated external pack 5 種類の SVG を repository へ追加し、pack ごとに整理する
- [ ] 4.2 `Icon` enum 全件について third-party source / KatanA authored override / fallback の対応表を作る
- [ ] 4.3 直接互換しない icon は selected pack の visual language に寄せた KatanA authored override を作る
- [ ] 4.4 `docs/licenses/icon-packs.md` または同等文書に source、license、override rationale を記録する
- [ ] 4.5 commercial use を阻害しない curated pack のみを shipping target に含めることを確認する

### Definition of Done (DoD)

- [ ] shipping pack ごとに required icon contract を満たしていること
- [ ] third-party source と KatanA authored override の境界が文書化されていること
- [ ] bundled pack の provenance と license が repository から追跡できること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Final Verification & Release Work

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 5.2 Ensure `make check` passes with exit code 0
- [ ] 5.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 5.4 Create a PR targeting `master`
- [ ] 5.5 Merge into master (※ `--admin` is permitted)
- [ ] 5.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.17.0`
- [ ] 5.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
