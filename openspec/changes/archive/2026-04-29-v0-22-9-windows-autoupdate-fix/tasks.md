## 0. Definition of Ready (DoR)

- [ ] 本ドキュメントがレビューされ、スコープが合意されている
- [ ] `master` ブランチが最新状態である

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `master`
- **作業ブランチ**: `feature/v0.22.9-task-x`（xはタスク番号）

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. 設定タブ更新アクションのハンドラ追加

### 実装内容

- `AppAction::StartUpdateDownload` のハンドラを追加（`handle_action_install_update()` へ委譲）
- `AppAction::InstallUpdateAndRestart` のハンドラを追加（`ConfirmRelaunch` と同等の処理）
- 追加場所: `dispatch_secondary.rs` または `dispatch_tertiary.rs`

### 対象ファイル

- `crates/katana-ui/src/app/action/dispatch_secondary.rs`
- `crates/katana-ui/src/app/action/dispatch_tertiary.rs`
- `crates/katana-ui/src/settings/tabs/updates.rs`（動作確認用）

### Definition of Done (DoD)

- [x] 1.1 `StartUpdateDownload` ハンドラ追加（`handle_action_install_update()` 委譲）
- [x] 1.2 `InstallUpdateAndRestart` ハンドラ追加（`pending_relaunch`経由でrelauncher実行）
- [x] 1.3 設定タブの「Download Update」「Install Update」ボタンが正常動作することを手動確認（またはテスト追加）
- [ ] Execute `/openspec-delivery` workflow to run the comprehensive delivery routine.

---

## 2. PowerShellスクリプトのファイルロック対策

### Definition of Ready (DoR)

- [ ] Task 1の完全デリバリーが完了している
- [ ] `master` が最新、新しいブランチを明示的に作成済み

### 実装内容

PowerShellスクリプト（`scripts.rs`のWindows向け生成部分）を改善:

1. 親プロセスIDをスクリプトへ引数として渡す
2. `Wait-Process -Id $parentPid -Timeout 30 -ErrorAction SilentlyContinue` で親プロセス終了を待機
3. `Move-Item` にリトライループを追加（最大5回、1秒間隔）
4. リトライ全失敗時はロールバック＋ユーザー通知

### 対象ファイル

- `crates/katana-core/src/update/scripts.rs`（Windowsスクリプト生成部分）
- `crates/katana-core/src/update/installer.rs`（`execute_relauncher` でPID引数追加）
- `crates/katana-core/src/update/types.rs`（必要に応じて型変更）

### Definition of Done (DoD)

- [x] 2.1 親プロセスIDをPowerShellスクリプトに渡す仕組みを実装
- [x] 2.2 `Wait-Process` による親プロセス終了待機を追加
- [x] 2.3 `Move-Item` リトライループを実装（最大5回、1秒間隔）
- [x] 2.4 リトライ全失敗時のロールバック処理を追加
- [x] 2.5 `scripts.rs` の既存テストを更新・新規テストを追加
- [x] 2.6 `make check-light` がパスすることを確認
- [ ] Execute `/openspec-delivery` workflow to run the comprehensive delivery routine.

---

## 3. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [x] 3.1 ユーザーへ実装完了の報告および動作状況を提示する（Windows環境での動作確認結果必須）
- [x] 3.2 ユーザーから受けたフィードバックを本ドキュメントに追記し、すべて対応・解決する

---

## 4. Final Verification & Release Work

### Definition of Ready (DoR)

- [ ] Task 3（User Review）が完了している

### Definition of Done (DoD)

- [ ] 4.1 `/self-review` スキルを実行し、セルフレビューを完了する
- [ ] 4.2 Markdownドキュメントのフォーマット整形を行う
- [ ] 4.3 `git push` でpre-pushフックを正式ゲートとして通す（`--no-verify` 原則禁止）
- [ ] 4.4 `master` へのPRを作成する（feature/v0.22.9-taskX → master）
- [ ] 4.5 CI確認（Lint / Coverage / CodeQL）
- [ ] 4.6 `master` へマージ
- [ ] 4.7 `release/v0.22.9` ブランチを作成する
- [ ] 4.8 `make release VERSION=0.22.9` を実行し、CHANGELOG を更新する
- [ ] 4.9 Release PRを `master` へ作成し、Release Readiness CI確認
- [ ] 4.10 Release PRを `master` へマージ
- [ ] 4.11 GitHub Releaseの完了を確認し、`/openspec-archive` でアーカイブ
