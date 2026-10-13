# Tasks: v0.22.26 replace kcf with kdv v0.1.0 - KatanA

## 要件(Verbatim Requirements)

> 終わったらkcfの依存を排除して、次期v0.1.0のkdvに完全載せ替えをv0.22.26としてcreate openspec changeしてください。

> ※kdrはcrates.io経由で依存に追加すること

### この要件から導出される制約(MUST)

- KatanA から `katana-canvas-forge`（kcf）依存を排除する。
- 次期 `katana-document-viewer`（kdv）v0.1.0 へ完全載せ替えする。
- 対象リリースは KatanA v0.22.26 とする。
- `katana-diagram-renderer`（kdr）は crates.io 経由の dependency として追加・維持する。
- kdr を git dependency、path dependency、sibling repository 参照にしない。

### 設計判断時の参照義務

- 設計方針を提案・変更する際は、本セクションを直接参照し、各 MUST 制約を満たすことを設計説明の冒頭で確認する。
- 設計変更が verbatim と矛盾する場合、設計を変更するのではなくユーザーへ要件の更新を確認する。

## Definition of Ready

- [x] `katana-document-viewer` v0.1.0 が crates.io で利用可能であることを確認する。
- [x] `katana-diagram-renderer` の採用 version を crates.io semver dependency として確定する。
- [x] KatanA v0.22.26 の対象 branch と release 方針を確認する。
- [x] kcf 依存が現在どの crate / module / test / docs に残っているかを棚卸しする。

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-22-26-replace-kcf-with-kdv-v010` またはリリース用統合ブランチ（例: `release/v0.22.26`）
- **作業ブランチ**: 標準は `v0-22-26-replace-kcf-with-kdv-v010-task-x`、リリース用は `feature/v0.22.26-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## Release Process

- 本 capability は v0.22.26 の patch リリースに含める。
- kdv v0.1.0 と kdr は crates.io dependency として取り込む。
- push 後の GitHub Release 実行はユーザーが行う。
- DoD は `just check-local` 全通過とユーザーの動作確認 OK とする。
- ユーザー動作確認の前に `just check-local` を実行する。動作確認後の lint 修正でデグレードを起こさないため。

## 1. Dependency Boundary

- [x] 1.1 `Cargo.toml` / crate manifests / `Cargo.lock` から `katana-canvas-forge` dependency を削除する。
- [x] 1.2 `katana-document-viewer = "0.1.0"` を crates.io workspace dependency として追加する。
- [x] 1.3 `katana-diagram-renderer` が crates.io semver dependency として解決され、git / path dependency ではないことを固定する。
- [x] 1.4 `cargo tree` または `cargo metadata` で `katana-canvas-forge` が dependency graph に残っていないことを検証する。
- [x] 1.5 kcf DTO / adapter / feature flag / test helper の参照を棚卸しし、削除対象と移行対象を分ける。

### Definition of Done (DoD)

- [x] workspace dependency graph に `katana-canvas-forge` が含まれないこと。
- [x] kdv v0.1.0 と kdr が crates.io から解決されること。
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. Preview Migration

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 2.1 Markdown preview の kcf 呼び出しを kdv adapter 呼び出しへ差し替える。
- [x] 2.2 Mermaid / Draw.io / PlantUML の theme snapshot を kdv adapter 経由で kdr `RenderInput` へ渡す。
- [x] 2.3 図形 cache key が kdv / kdr の runtime、profile、theme fingerprint で変化することを維持する。
- [x] 2.4 tab switch / scroll / zoom だけで checksum 判定や再描画が走らないことを維持する。
- [x] 2.5 Mermaid / Draw.io / PlantUML preview の回帰テストを kdv 経由に更新する。

### Definition of Done (DoD)

- [x] Preview 経路に kcf adapter 呼び出しが残っていないこと。
- [x] Light theme の Mermaid / Draw.io が dark 配色へ戻らないこと。
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. Export Migration

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 3.1 HTML export を kdv v0.1.0 経由へ差し替える。
- [x] 3.2 PDF / PNG / JPEG export を kdv v0.1.0 経由へ差し替える。
- [x] 3.3 export thread に渡す theme snapshot を kdv / kdr 境界で維持する。
- [x] 3.4 export の Mermaid / Draw.io 図形が現在テーマで描画される回帰テストを更新する。
- [x] 3.5 HTML semantics と PDF / PNG / JPEG surface の既存 parity 検証を kdv 経由で通す。

### Definition of Done (DoD)

- [x] Export 経路に kcf API / DTO / adapter 呼び出しが残っていないこと。
- [x] HTML / PDF / PNG / JPEG export が kdv v0.1.0 経由で通ること。
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 4. Active Spec and Documentation Cleanup

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 4.1 active spec の kcf 前提を kdv / kdr 境界へ更新し、archive 内の履歴は変更対象外として残す。
- [x] 4.2 docs / comments / diagnostics / test names に残る current-context の kcf 表現を棚卸しし、必要箇所だけ更新する。
- [x] 4.3 `CHANGELOG.md` に v0.22.26 のユーザー向け変更を記録する。
- [x] 4.4 `./scripts/openspec validate v0-22-26-replace-kcf-with-kdv-v010 --strict` を実行する。

### Definition of Done (DoD)

- [x] active OpenSpec と docs が kcf を current dependency として扱っていないこと。
- [x] OpenSpec strict validation が通ること。
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 5.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする。
- [ ] 5.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）。
- [/] 5.3 README の JDK / `plantuml.jar` セットアップ記述を削除し、PlantUML JAR の取得・更新 UI / 実装を KDV / KDR 境界移行後の不要機能として除去する。

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 6. Final Verification & Release Work

- [x] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `$self-review` skill.
- [x] 6.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md).
- [ ] 6.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない。
- [ ] 6.4 Create PR from Base Feature Branch targeting `master`.
- [ ] 6.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail.
- [ ] 6.6 Merge into master (`gh pr merge --merge --delete-branch`).
- [ ] 6.7 Create `release/v0.22.26` branch from master.
- [ ] 6.8 Run `just VERSION=0.22.26 release` and update CHANGELOG (`changelog-writing` skill).
- [ ] 6.9 Create PR from `release/v0.22.26` targeting `master` — Ensure `Release Readiness` CI passes.
- [ ] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`).
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`.
