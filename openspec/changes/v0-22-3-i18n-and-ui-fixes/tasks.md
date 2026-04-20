## Definition of Ready (DoR)

- [x] v0.22.1 のリリースに向けた致命的不具合（フィルター、フォーカス）の修正がデリバリー済みであること
- [ ] `proposal.md` と `design.md` がレビュー済みであること
- [ ] 劣後させたタスクの要件が本 tasks.md に正しく統合されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-3-i18n-and-ui-fixes`
- **作業ブランチ**: 標準は `v0-22-3-i18n-and-ui-fixes-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. コマンドパレットのスクロール挙動修正 (TDD)

### 概要

検索結果が多い場合の縦スクロールが正しく機能しない不具合を TDD (RED) プロセスで修正します。

- [ ] 1.1 **TDD (RED)**: `crates/katana-ui/src/views/modals/command_palette_scroll_tests.rs` を新規作成。100件以上のダミーコマンドを表示した際、スクロール領域の `content_size` がウィンドウ制限を超え、スクロールバーが表示される（または正しく追従する）ことを IT で検証し、不具合を再現させる。
- [ ] 1.2 `crates/katana-ui/src/views/modals/command_palette.rs` のレイアウト実装（`egui::ScrollArea`）を修正。`max_height` やレイアウト制約の設定ミスを解消する。
- [ ] 1.3 **TDD (GREEN)**: 1.1 のテストを実行し、パスすることを確認。

### Definition of Done (DoD)

- [ ] 100件以上の項目があってもリストがウィンドウを突き抜けず、スクロールが可能であること
- [ ] 全テスト（IT）がパスすること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. ショートカット設定 UI の再設計 (v0.22.1 からの引継ぎ)

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `shortcuts.rs` から Edit ボタンを廃止し、行全体クリックとペンSVGアイコンによる録音トリガーに変更する
- [ ] 2.2 キー表示部分に OSネイティブキーSVGアイコン（⌘/⇧/⌥/Ctrl/Alt）を表示するロジックを実装する
- [ ] 2.3 `ShortcutCaptureModal`（録音専用モーダル）を新規実装する（Enterで確定、Escでキャンセル）
- [ ] 2.4 録音中は `egui` の `set_key_filter` を用いて Esc/Enter 以外の入力をブロックする
- [ ] 2.5 i18n キーを追加する（EN + JA 同時）: `settings.shortcuts.capture_prompt`, `settings.shortcuts.confirm_key`, `settings.shortcuts.cancel_key`
- [ ] 2.6 ショートカット設定画面に検索バーを追加する
- [ ] 2.7 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.8 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 編集ボタンが削除され、録音モーダルが起動すること
- [ ] OSネイティブキーアイコンが正しく表示されること
- [ ] `make check` がパスすること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. i18n 翻訳の完全補完 (v0.22.1 からの引継ぎ)

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle.
- [ ] Base branch is synced.

- [ ] 3.1 画像で報告された英語のままのラベルを、日本語（`ja.json`）および全11言語に翻訳追加する
  - `Close Document`, `Restore Closed Document`, `Global Search`, `Document Search`, `Toggle Explorer Filter` 等
- [ ] 3.2 デフォルトショートカットの割り当て（Task 3.1〜3.15 の内容）
  - `file.close_workspace` → `primary+Shift+W`
  - `view.refresh_explorer` → `primary+Shift+R`
  - その他 `edit.*` 系ショートカットの追加
- [ ] 3.3 割り当て後の重複チェック: `debug_assert!` による実行時重複検知の確認。

### Definition of Done (DoD)

- [ ] コマンドパレットや設定画面で英語フォールバックが発生していないこと
- [ ] `make check` がパスすること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. ShortcutAST Linter (静的重複検知) (v0.22.1 からの引継ぎ)

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle.

- [ ] 4.1 `katana-linter/src/rules/domains/shortcut/` を作成し、コマンド定義ファイルからショートカット重複を検知するルールを実装。
- [ ] 4.2 Makefile の `ast-lint` ターゲットに統合。

### Definition of Done (DoD)

- [ ] `make ast-lint` でショートカットの重複が検知可能であること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Final Verification & Release Work

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 5.2 Ensure `make check` passes with exit code 0
- [ ] 5.3 Create PR from Base Feature Branch targeting `master`
- [ ] 5.4 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 5.5 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 5.6 Create `release/v0.22.2` branch from master
- [ ] 5.7 Run `make release VERSION=0.22.2` and update CHANGELOG (`changelog-writing` skill)
- [ ] 5.8 Create PR from `release/v0.22.2` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 5.9 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 5.10 Verify GitHub Release completion and archive this change using `/opsx-archive`
