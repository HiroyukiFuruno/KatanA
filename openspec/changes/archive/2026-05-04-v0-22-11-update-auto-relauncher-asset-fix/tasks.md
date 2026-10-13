# Tasks: v0.22.11 Auto-Update Hotfix (Linux 404 / Windows Relauncher)

## 0. 準備完了条件（Definition of Ready）

- [ ] `proposal.md` / `design.md` / `specs/auto-update/spec.md` が揃っている
- [ ] 本 change は `v0.22.11` のリリース対象として扱う
- [ ] Linux アセット名のドリフトと Windows relauncher の成功判定不具合の双方を 1 hotfix としてまとめる
- [ ] アップデート機構の汎用 interface 化、macOS / MSI 経路、UI 文言修正は本 change の範囲外とする
- [ ] `./scripts/openspec validate v0-22-11-update-auto-relauncher-asset-fix --strict` が通る状態にする

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `release/v0.22.11`
- **作業ブランチ**: `feature/v0.22.11-task-x`（x はタスク番号）

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

---

## 1. Linux アセット名整合化

### 実施内容

`crates/katana-core/src/update/version.rs` の Linux 向け `ASSET_NAME` を `KatanA-linux-x86_64.tar.gz` に修正し、`check_for_updates` と `check_for_updates_simple` の双方で同一定義を参照するよう統合する。`installer.rs` / `download.rs` の tar.gz 経路はすでに実装済みのため変更しない。

### 対象ファイル

- `crates/katana-core/src/update/version.rs`
- `crates/katana-core/src/update/tests.rs`（または `version.rs` 内の test mod）
- `.github/workflows/build-and-release.yml`（変更不要だが、アセット名の正本としてコメントを追加する場合のみ）

### 完了条件（Definition of Done）

- [ ] 1.1 `version.rs` の `#[cfg(target_os = "linux")] const ASSET_NAME` を `KatanA-linux-x86_64.tar.gz` に修正する（2 箇所）
- [ ] 1.2 ASSET_NAME を返す関数（例: `platform_asset_name()`）に集約し、`check_for_updates` と `check_for_updates_simple` から同一定義を参照する
- [ ] 1.3 `cargo test -p katana-core` で URL 末尾が `tar.gz` で終わる Linux 向けユニットテストを追加する
- [ ] 1.4 `curl -sI -L "https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.10/KatanA-linux-x86_64.tar.gz"` で 200 / 302 を確認しコマンド結果を記録する
- [ ] 1.5 macOS / Windows のアセット名は変更されていないことを diff で確認する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Windows relauncher の成功判定堅牢化

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

`crates/katana-core/src/update/scripts.rs` の Windows ブランチ（PowerShell relauncher）を、`design.md` の擬似コードに従って書き換える。退避 Move-Item の `-ErrorAction SilentlyContinue` を外し try/catch にする、成功判定を `LastWriteTime` と `extracted` 消滅の二点に切り替える、失敗時に旧版を再起動しない、`update.log` を追記する、を 1 セットで適用する。

### 対象ファイル

- `crates/katana-core/src/update/scripts.rs`
- `crates/katana-core/src/update/installer.rs`（必要なら parentPid 渡し直し / log path 受け渡しの整合）

### 完了条件（Definition of Done）

- [ ] 2.1 退避フェーズを `try { Move-Item -Force $target $bak } catch { ... }` に書き換え、`-ErrorAction SilentlyContinue` を外す
- [ ] 2.2 relauncher 起動直後に `$startedAt = Get-Date` を保持する
- [ ] 2.3 差し替えフェーズの成功判定を `LastWriteTime -gt $startedAt` かつ `-not (Test-Path $extracted)` に変更する
- [ ] 2.4 失敗時の `Start-Process $target` を削除し、ロールバックのみ実行する
- [ ] 2.5 `update.log` への追記関数 `Write-UpdateLog` を実装し、`evacuate` / `replace` / `launch` / `rollback` の 4 フェーズで記録する
- [ ] 2.6 `crates/katana-core/src/update/scripts.rs` の Windows test に `Get-Date` / `LastWriteTime -gt $startedAt` / `-not (Test-Path $extracted)` / `update.log` / 退避から `SilentlyContinue` を外したアサートを追加する
- [ ] 2.7 `cargo test -p katana-core update::scripts` を全 OS で pass する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. 自動アップデート E2E 検証手順の整備

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

Linux と Windows の双方で、自動アップデートの正常パスと失敗パスを次回リリース前に必ず通せる手順を文書化する。`docs/release/update-verification.md`（仮）に手順を新設し、リリースワークフローのチェックリストへ参照を加える。

### 対象ファイル

- `docs/release/update-verification.md`（新規）
- `docs/CHANGELOG.en.md`
- `docs/CHANGELOG.ja.md`
- `.github/workflows/release-readiness.yml`（必要なら 404 監視ステップを追加）

### 完了条件（Definition of Done）

- [ ] 3.1 Linux: `just linux-up` 環境で v0.22.10 → v0.22.11 の自動アップデートが成功し、`KatanA --version` が `0.22.11` を返す手順を記録する
- [ ] 3.2 Windows: VM で v0.22.11 portable zip 起動 → 「Install and Relaunch」→ 新プロセスのバージョン確認の手順を記録する
- [ ] 3.3 Windows: target.exe を読み取り専用化または別プロセスでロックして失敗パスを再現し、ロールバック発動と `update.log` 記録を確認する手順を記録する
- [ ] 3.4 `release-readiness.yml` か別 CI に、最新 release tag のアセット URL を `curl -sI` で叩き 200 / 302 を確認するステップを追加する（404 を CI でブロックする足場）
- [ ] 3.5 `docs/CHANGELOG.en.md` と `docs/CHANGELOG.ja.md` に Linux 404 / Windows old-binary 双方の修正を同期記載する
- [ ] 3.6 リリースノート文面に v0.22.10 ユーザー向けの手動アップグレード手順（Linux: tar.gz 上書き、Windows: portable zip / MSI 再インストール）を含める
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 4.1 ユーザーへ実装完了の報告および動作状況を提示する。Linux は `just linux-up` で取得した端末出力（`curl` 結果と `--version` 出力）を提示し、Windows は VM 上での更新成功・失敗パス双方の動画またはスクリーンショットを `scripts/screenshot/output/v0-22-11-review/` に出力して提示する
- [ ] 4.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 5. Final Verification & Release Work

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 5.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 5.3 `./scripts/openspec validate v0-22-11-update-auto-relauncher-asset-fix --strict` を実行し、OpenSpec の整合性を確認する
- [ ] 5.4 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
- [ ] 5.5 Create PR from Base Feature Branch targeting `master`
- [ ] 5.6 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 5.7 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 5.8 Create `release/v0.22.11` branch from master
- [ ] 5.9 Run `just VERSION=0.22.11 release` and update CHANGELOG (`changelog-writing` skill)
- [ ] 5.10 Create PR from `release/v0.22.11` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 5.11 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 5.12 Verify GitHub Release completion and archive this change using `/opsx-archive`
