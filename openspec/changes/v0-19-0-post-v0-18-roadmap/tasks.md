## 着手条件 (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.19.0 以降の roadmap change であること、および本 change は planning artifact であって直接 implementation を行わないことが確認されていること
- [ ] 現行の active change（`v0.16.0` / `v0.17.0`）と既存基盤（diagnostics / menu / command palette / settings / editor / AI abstraction）を再確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. v0.19.0 Markdown Diagnostics Plan

- [ ] 1.1 `v0.19.0` の primary concern を「official markdownlint-compatible diagnostics surface」に固定する
- [ ] 1.2 現行 Problems Panel / diagnostics refresh / `katana-linter` rule contract を基準に affected area を整理する
- [ ] 1.3 `v0.19.0` の Definition of Ready として、supported rule subset、official rule code sync policy、message language policy、refresh trigger policy を明記する
- [ ] 1.4 `v0.19.0` の Definition of Done として、rule code visible、Problems Panel jump behavior、English fallback copy、regression check を明記する
- [ ] 1.5 `v0.19.0` の open questions と non-goals を明記し、full parity を初期スコープから外す

### Definition of Done (DoD)

- [ ] `v0.19.0` entry に scope、affected area、DoR、DoD、open questions、non-goals が揃っていること
- [ ] `v0.19.0` が `v0.18.0` 完了後に着手可能な standalone concern として読めること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. v0.20.0 Menu Expansion Plan

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `v0.20.0` の primary concern を File / View / Help の command surface expansion に固定する
- [ ] 2.2 `AppAction`、native menu、command palette の間で共有すべき command inventory を roadmap 上で明記する
- [ ] 2.3 `v0.20.0` の Definition of Ready として、command registry 方針、OS 間 parity 方針、menu grouping 方針を明記する
- [ ] 2.4 `v0.20.0` の Definition of Done として、menu access coverage、empty/disabled state、docs update の完了条件を明記する
- [ ] 2.5 UI 実装計画には、ユーザーへの UI スナップショット提示とフィードバック反映タスクを future change に含める方針を明記する

### Definition of Done (DoD)

- [ ] `v0.20.0` entry に scope、affected area、DoR、DoD、future UI verification obligations が揃っていること
- [ ] `v0.20.0` が `v0.21.0` shortcut system の prerequisite として読めること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. v0.21.0 Shortcut Customization Plan

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 `v0.21.0` の primary concern を user-configurable shortcut system に固定する
- [ ] 3.2 duplicate registration、OS reserved shortcut、current assignee popup、settings persistence の扱いを roadmap 上で明記する
- [ ] 3.3 `v0.21.0` の Definition of Ready として、command registry 完了、shortcut schema、validation policy を明記する
- [ ] 3.4 `v0.21.0` の Definition of Done として、default shortcut migration、conflict detection、editable settings UI、regression checks を明記する
- [ ] 3.5 UI 実装計画には、ユーザーへの UI スナップショット提示とフィードバック反映タスクを future change に含める方針を明記する

### Definition of Done (DoD)

- [ ] `v0.21.0` entry に scope、affected area、DoR、DoD、conflict policy が揃っていること
- [ ] `v0.21.0` が menu expansion 後の command registry を前提にした release であることが明確であること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. v0.22.0 Editor Authoring and Image Workflow Plan

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 `v0.22.0` の primary concern を Markdown source-first authoring enhancements と image asset workflow に固定する
- [ ] 4.2 snippet / formatting command、clipboard paste、asset placement policy、explorer reveal policy を roadmap 上で整理する
- [ ] 4.3 `v0.22.0` の Definition of Ready として、editor model を rewrite しないこと、asset path strategy、naming strategy、dialog policy を明記する
- [ ] 4.4 `v0.22.0` の Definition of Done として、formatting helper、image insert flow、preview consistency、workspace reveal behavior、settings persistence を明記する
- [ ] 4.5 この concern は必要なら future concrete proposal で phase A/B に再分割できることを注記する
- [ ] 4.6 UI 実装計画には、ユーザーへの UI スナップショット提示とフィードバック反映タスクを future change に含める方針を明記する

### Definition of Done (DoD)

- [ ] `v0.22.0` entry に scope、affected area、DoR、DoD、phase split option が揃っていること
- [ ] `v0.22.0` が full WYSIWYG ではなく Markdown source-first であることが明確であること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. v0.23.0 Local LLM Lint Autofix Plan

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 `v0.23.0` の primary concern を local LLM foundation と lint autofix に固定する
- [ ] 5.2 provider abstraction、local runtime choice、settings、availability detection、manual apply UX を roadmap 上で整理する
- [ ] 5.3 `v0.23.0` の Definition of Ready として、runtime choice、prompt input contract、safety boundary、fallback behavior を明記する
- [ ] 5.4 `v0.23.0` の Definition of Done として、configured provider detection、lint autofix suggestion/apply flow、failure fallback、auditability を明記する
- [ ] 5.5 UI 実装計画には、ユーザーへの UI スナップショット提示とフィードバック反映タスクを future change に含める方針を明記する

### Definition of Done (DoD)

- [ ] `v0.23.0` entry に scope、affected area、DoR、DoD、runtime assumption が揃っていること
- [ ] `v0.23.0` が deterministic diagnostics の後段として位置付けられていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. v0.24.0 Local LLM Document Generation Plan

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 6.1 `v0.24.0` の primary concern を local LLM document generation に固定する
- [ ] 6.2 active document insertion、新規 document generation、template-based generation の優先順位を roadmap 上で整理する
- [ ] 6.3 `v0.24.0` の Definition of Ready として、provider runtime stabilization、output target、overwrite policy、preview validation を明記する
- [ ] 6.4 `v0.24.0` の Definition of Done として、generation entry point、target document handling、undo/safety、failure fallback を明記する
- [ ] 6.5 UI 実装計画には、ユーザーへの UI スナップショット提示とフィードバック反映タスクを future change に含める方針を明記する

### Definition of Done (DoD)

- [ ] `v0.24.0` entry に scope、affected area、DoR、DoD、output policy が揃っていること
- [ ] `v0.24.0` が `v0.23.0` local LLM foundation を prerequisite にしていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 7. v0.25.0 Translation Overlay Plan

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 7.1 `v0.25.0` の primary concern を dynamic/external English text 向け translation overlay に固定する
- [ ] 7.2 linter message、AI-generated text、docs excerpt のどこまでを translation target にするかを roadmap 上で整理する
- [ ] 7.3 `v0.25.0` の Definition of Ready として、opt-in policy、cache policy、raw English fallback、non-target UI exclusion を明記する
- [ ] 7.4 `v0.25.0` の Definition of Done として、overlay behavior、failure fallback、display latency expectation、settings integration を明記する
- [ ] 7.5 UI 実装計画には、ユーザーへの UI スナップショット提示とフィードバック反映タスクを future change に含める方針を明記する

### Definition of Done (DoD)

- [ ] `v0.25.0` entry に scope、affected area、DoR、DoD、translation target boundary が揃っていること
- [ ] `v0.25.0` が locale JSON ベースの app i18n を置き換えないことが明確であること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 8. Roadmap Validation and Question Backlog

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 8.1 `v0.19.0` から `v0.25.0` までの dependency order に循環がないことを確認する
- [ ] 8.2 各 version entry に共通フォーマット（goal / affected area / DoR / DoD / open questions / non-goals）が揃っていることを確認する
- [ ] 8.3 user decision が必要な open questions を version ごとに抽出する
- [ ] 8.4 実装前に dedicated change へ分割する運用方針を明記する

### Definition of Done (DoD)

- [ ] roadmap 全体が version map として一貫して読めること
- [ ] user へ確認すべき open questions backlog が version ごとに整理されていること
- [ ] follow-up で dedicated OpenSpec change を切る前提条件が明記されていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 9. Final Verification & Release Work

- [ ] 9.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 9.2 Ensure `make check` passes with exit code 0
- [ ] 9.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 9.4 Create a PR targeting `master`
- [ ] 9.5 Merge into master (※ `--admin` is permitted)
- [ ] 9.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.19.0`
- [ ] 9.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
