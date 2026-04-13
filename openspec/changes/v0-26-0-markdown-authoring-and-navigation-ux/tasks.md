## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` がレビュー可能な粒度で揃っていること
- [ ] 対象バージョン 0.26.0 の変更 ID とスコープが確認されていること
- [ ] `v0.23.0` の local LLM autofix scope と、本 change の deterministic diagnostics UX scope が分離されていること
- [ ] image ingest の default 保存先として `./asset/img` を継続利用する方針が確認されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-26-0-markdown-authoring-and-navigation-ux`
- **作業ブランチ**: 標準は `v0-26-0-markdown-authoring-and-navigation-ux-task-x`、リリース用は `feature/v0.26.0-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. markdownlint diagnostics の coverage と editor surface を再設計する

- [ ] 1.1 現在の shipped diagnostics rule を棚卸しし、MD001 以外で未出荷・未装飾になっている rule gap を inventory 化する
- [ ] 1.2 shipped supported rule set を横断して diagnostics payload を出力できるように evaluator と metadata binding を拡張する
- [ ] 1.3 diagnostics location を editor buffer range に解決し、該当箇所へ inline warning decoration を描画する
- [ ] 1.4 inline warning decoration の hover / focus から official rule code、message、docs link を表示する popup を実装する
- [ ] 1.5 safe fix provider を持つ rule のみ popup 内に quick-fix button を表示し、対象範囲限定の修正を適用できるようにする
- [ ] 1.6 theme settings から diagnostics decoration color を変更できるようにし、editor 装飾へ即時反映する
- [ ] 1.7 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.8 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] MD001 以外の shipped supported markdownlint rule も Problems Panel と editor inline surface で同一 contract で表示されること
- [ ] inline warning decoration から popup detail を開け、safe fix provider がある rule にのみ quick-fix button が表示されること
- [ ] diagnostics decoration color を theme settings から変更でき、再起動なしで editor 装飾へ反映されること
- [ ] `make check` がパスすること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Markdown authoring toolbar と unified image ingest pipeline を実装する

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 editable な active Markdown document に対して、system SVG icon で構成された authoring toolbar を表示する
- [ ] 2.2 toolbar action が既存 command / shortcut と同じ Markdown transform contract を使うように統合する
- [ ] 2.3 file attach、clipboard paste、external image drag-and-drop を同一 image ingest pipeline へ統合する
- [ ] 2.4 image ingest の default 保存先 `./asset/img`、命名 format、確認ダイアログ設定、未保存 document 時の guard を整理する
- [ ] 2.5 image reference の挿入位置を cursor / selection 優先、未解決時は文書末尾 fallback で統一する
- [ ] 2.6 image ingest 成功後に preview と explorer の asset state が refresh されることを確認する
- [ ] 2.7 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.8 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] authoring toolbar から見出し、装飾、リスト、表、画像挿入を GUI で実行できること
- [ ] file attach、clipboard paste、external image drag-and-drop が同じ保存先・命名・挿入 contract で動作すること
- [ ] image reference が cursor 位置または文書末尾へ期待通りに挿入され、preview と explorer の asset state が更新されること
- [ ] `make check` がパスすること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. explorer thumbnail と single-file open flow を追加する

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 Markdown から参照されている local image asset を判定する reference-aware inventory を explorer 用に整備する
- [ ] 3.2 explorer 初回描画を阻害しない lazy thumbnail queue と cache を実装する
- [ ] 3.3 File menu / command surface に single file open を追加し、「temporary workspace」と「current workspace session」の 2 モードを提供する
- [ ] 3.4 system icon と label を持つ temporary workspace を定義し、persisted workspace history に保存しない contract を実装する
- [ ] 3.5 外部から drop された file を current workspace session で開き、session 不在時は temporary workspace へ fallback する
- [ ] 3.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] referenced local image asset の explorer row で lazy thumbnail が表示され、初回 load の text row 表示を阻害しないこと
- [ ] single file open で temporary workspace と current workspace session の 2 モードを選べること
- [ ] 外部から drop した file が current workspace session で開かれ、session が無い場合は temporary workspace へ fallback すること
- [ ] `make check` がパスすること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. explorer からの drag-and-drop file operation を完成させる

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 explorer item を tab strip へ drop したときの append / positioned insert / tab group add の意図ゾーンを定義する
- [ ] 4.2 casual drop では tab 末尾へ追加して active、precise drop では指定位置へ temporary tab 挿入する挙動を実装する
- [ ] 4.3 既存 tab group への drop を許可し、target group へ document を追加できるようにする
- [ ] 4.4 explorer 内の file / directory drag-and-drop move を実装し、supported target directory にのみ move できるようにする
- [ ] 4.5 move confirmation setting を default enabled で追加し、確認メッセージと no-confirm path の両方を整備する
- [ ] 4.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 4.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] explorer から tab strip への casual drop と precise drop が期待どおり区別されること
- [ ] explorer item を既存 tab group に追加できること
- [ ] explorer 内 move drag-and-drop で confirmation enabled / disabled の両方が期待どおり動作すること
- [ ] `make check` がパスすること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. TOC を accordion 化し presentation setting を追加する

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 TOC row の filled background を除去し、all-open default の accordion presentation へ置き換える
- [ ] 5.2 TOC panel header 左上に expand-all / collapse-all icon を追加する
- [ ] 5.3 TOC accordion の vertical guide line を実装し、layout setting で表示有無を切り替えられるようにする
- [ ] 5.4 accordion 化後も見出しジャンプと preview scroll sync が破綻しないことを確認する
- [ ] 5.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 5.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] TOC row が filled background なしの accordion presentation で表示され、default では全展開であること
- [ ] TOC panel header から全開 / 全閉を実行できること
- [ ] vertical guide line の表示有無が layout setting へ保存され、次回起動時も復元されること
- [ ] `make check` がパスすること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. Final Verification & Release Work

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 6.2 Ensure `make check` passes with exit code 0
- [ ] 6.3 Create PR from Base Feature Branch targeting `master`
- [ ] 6.4 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 6.5 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.6 Create `release/v0.26.0` branch from master
- [ ] 6.7 Run `make release VERSION=0.26.0` and update CHANGELOG (`changelog-writing` skill)
- [ ] 6.8 Create PR from `release/v0.26.0` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 6.9 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.10 Verify GitHub Release completion and archive this change using `/opsx-archive`
