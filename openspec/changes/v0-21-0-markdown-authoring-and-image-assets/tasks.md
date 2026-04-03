## Definition of Ready (DoR)
- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.21.0 の変更 ID とスコープが確認されていること
- [ ] `v0.19.0` の command inventory と `v0.20.0` の shortcut schema が利用可能であること
- [ ] active Markdown file 起点の asset path 方針が `./asset/img` で確定していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Source-First Authoring Commands

- [ ] 1.1 見出し、装飾、リスト、表などの authoring command 一覧を確定する
- [ ] 1.2 cursor / selection に対する Markdown 挿入・変換ロジックを実装する
- [ ] 1.3 command inventory と shortcut から authoring command を呼び出せるようにする
- [ ] 1.4 save / dirty buffer / preview sync 契約が壊れていないことを確認する

### Definition of Done (DoD)

- [ ] editor 上で Markdown source-first のまま authoring command を利用できること
- [ ] selection 有無で不正な Markdown 破壊が起きないこと
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Image Ingest Pipeline and Settings

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 active Markdown file 起点の `./asset/img` 解決ロジックを実装する
- [ ] 2.2 local file attach を image ingest pipeline に接続する
- [ ] 2.3 clipboard image paste を同じ ingest pipeline に接続する
- [ ] 2.4 保存先、命名、ダイアログ表示ポリシーを settings schema に追加する
- [ ] 2.5 relative path 挿入と asset directory 自動作成のテストを追加する

### Definition of Done (DoD)

- [ ] file attach と clipboard paste の両方が同じ保存規則で動作すること
- [ ] default の保存先が active Markdown file 起点の `./asset/img` になっていること
- [ ] settings 変更が subsequent ingest に反映されること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. UI Integration and Asset Navigation

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 attach image / paste image の UI 導線を editor / menu / shortcut に追加する
- [ ] 3.2 local image reference から対象ディレクトリまたはファイルへ辿る導線を追加する
- [ ] 3.3 missing local image と remote image の扱いを UI で区別する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] 画像挿入から asset 参照先確認まで UI 上で完結すること
- [ ] missing / non-local image で誤った導線が出ないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.21.0`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
