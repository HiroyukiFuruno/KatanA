## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md` がレビュー済みであること
- [ ] 対象バージョン 0.22.6 の変更 ID とスコープが確認されていること
- [ ] v0.22.5 のリリースが完了していること
- [ ] TOC の既存実装を確認済みであること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-6-toc-improvements`
- **作業ブランチ**: 標準は `v0-22-6-toc-improvements-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

---

## 1. TOC 視覚デザインの改善

### 概要

 unnecessarily な背景スタイルを除去し、クリーンでミニマルな外観を実現する。

- [ ] 1.1 `crates/katana-ui/src/views/sidebars/table_of_contents/` の背景スタイルを調査
- [ ] 1.2 TOC ヘッダーからすべての背景スタイルを除去
- [ ] 1.3 クリーンなミニマルな視覚外観を維持
- [ ] 1.4 エクスプローラーインターフェースとのデザイン整合性を確認

### Definition of Done (DoD)

- [ ] TOC ヘッダーに背景スタイルが表示されないこと
- [ ] エクスプローラーとのデザインが統一されていること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. アコーディオン機能実装

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除）を完全に終えていること
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [ ] 2.1 静的 TOC をアコーディオンスタイルのインターフェースに置換
- [ ] 2.2 折りたたみ/展開のアニメーションを実装
- [ ] 2.3 デフォルト状態：全セクション展開
- [ ] 2.4 各セクションの展開/折りたたみ状態を個別に管理
- [ ] 2.5 階層構造のネストしたアコーディオンをサポート

### Definition of Done (DoD)

- [ ] アコーディオン機能でセクションの展開/折りたたみが可能であること
- [ ] デフォルトで全セクションが展開されていること
- [ ] 階層構造が正しく表示されること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. ナビゲーションコントロール追加

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [ ] 3.1 TOC 上部に全開/全閉のトグルコントロールを追加
- [ ] 3.2 全セクションの同時展開/折りたたみ機能を実装
- [ ] 3.3 エクスプローラーインターフェースのデザインと整合させる
- [ ] 3.4 状態永続化：ウィンドウ閉じても前回の状態を保持（設定保存）

### Definition of Done (DoD)

- [ ] TOC 上部から全開/全閉が可能であること
- [ ] エクスプローラーと一貫したデザインであること
- [ ] 設定が永続化されること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. 垂直線表示と設定統合

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [ ] 4.1 アコーディオンセクションに垂直線（階層インジケーター）を追加
- [ ] 4.2 テーマ設定で垂直線の表示オン/オフを切り替え可能にする
- [ ] 4.3 デフォルト設定：垂直線は表示
- [ ] 4.4 ユーザー設定の永続化を実装
- [ ] 4.5 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 4.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] 垂直線がセクション階層を視覚的に示していること
- [ ] テーマ設定から垂直線の表示を切り替えられること
- [ ] 設定が永続化されること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Task 2

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 Task 2 description

### Definition of Done (DoD)

- [ ] (Other task-specific verifiable conditions...)
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Create PR from Base Feature Branch targeting `master`
- [ ] 3.4 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 3.5 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.6 Create `release/v0.22.6` branch from master
- [ ] 3.7 Run `make release VERSION=0.22.6` and update CHANGELOG (`changelog-writing` skill)
- [ ] 3.8 Create PR from `release/v0.22.6` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 3.9 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.10 Verify GitHub Release completion and archive this change using `/opsx-archive`
