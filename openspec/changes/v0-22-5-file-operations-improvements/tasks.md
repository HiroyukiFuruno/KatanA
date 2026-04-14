## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md` がレビュー済みであること
- [ ] 対象バージョン 0.22.5 の変更 ID とスコープが確認されていること
- [ ] v0.22.4 のリリースが完了していること
- [ ] ワークスペース管理システムとの整合性を確認済みであること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-5-file-operations-improvements`
- **作業ブランチ**: 標準は `v0-22-5-file-operations-improvements-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

---

## 1. システムファイルダイアログ実装

### 概要

システムネイティブのファイル選択ダイアログを実装し、2 種類のオープンモード（新規ワークスペース / 現在ワークスペース）をサポートする。

- [ ] 1.1 `crates/katana-ui/src/views/dialogs/file_picker.rs` にシステムファイルダイアログを実装
- [ ] 1.2 単一ファイル選択をサポート
- [ ] 1.3 「新規ワークスペースで開く」と「現在ワークスペースで開く」の 2 モードを実装
- [ ] 1.4 一時的なワークスペースであることを示すシステム SVG アイコンを追加
- [ ] 1.5 ファイルオープン後のエディタ状態管理（タブ、ワークスペース）

### Definition of Done (DoD)

- [ ] システムダイアログからファイル選択が可能であること
- [ ] 2 種類のオープンモードが正しく動作すること
- [ ] 一時的なワークスペースが視覚的に識別できること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. 外部ファイルのドラッグ＆ドロップサポート

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除）を完全に終えていること
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [ ] 2.1 アプリケーションウィンドウへのファイルドラッグ＆ドロップを検知するリストナーを実装
- [ ] 2.2 ドロップされたファイルを現在ワークスペースで開くデフォルト動作を実装
- [ ] 2.3 既存のタブ構造を保証するロジックを追加
- [ ] 2.4 複数ファイルのドラッグ＆ドロップをサポート
- [ ] 2.5 開けるファイル形式のバリデーション

### Definition of Done (DoD)

- [ ] 外部からファイルをドラッグ＆ドロップして開けること
- [ ] デフォルトで現在ワークスペースに追加されること
- [ ] 複数ファイルのドラッグ＆ドロップが動作すること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. タブ管理機能の強化

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [ ] 3.1 タブのドラッグ＆ドロップでアクティブ切り替えを実装
- [ ] 3.2 一時的タブの位置精度管理（特定位置への追加）
- [ ] 3.3 既存のタブグループへの追加をサポート
- [ ] 3.4 デフォルト動作：末尾への追加、精密操作による特定位置への配置
- [ ] 3.5 タブグループ間の移動をサポート

### Definition of Done (DoD)

- [ ] タブのドラッグ＆ドロップで切り替えが可能であること
- [ ] 特定位置へのドラッグ＆ドロップが動作すること
- [ ] タブグループ間移動が可能であること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. ファイル移動機能と確認ダイアログ

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [ ] 4.1 Explorer 内のドラッグ＆ドロップでファイル移動をサポート
- [ ] 4.2 ファイル移動時の確認ダイアログを実装（デフォルト：確認必須）
- [ ] 4.3 移動操作の明確な通知（例：「xx から yyy/zzz へファイルを移動」）
- [ ] 4.4 設定で確認ダイアログのオン/オフを切り替え可能にする
- [ ] 4.5 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 4.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] Explorer でドラッグ＆ドロップによるファイル移動が可能であること
- [ ] 確認ダイアログが表示され、移動操作が明確であること
- [ ] 設定で確認ダイアログをカスタマイズできること
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
- [ ] 3.6 Create `release/v0.22.5` branch from master
- [ ] 3.7 Run `make release VERSION=0.22.5` and update CHANGELOG (`changelog-writing` skill)
- [ ] 3.8 Create PR from `release/v0.22.5` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 3.9 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.10 Verify GitHub Release completion and archive this change using `/opsx-archive`
