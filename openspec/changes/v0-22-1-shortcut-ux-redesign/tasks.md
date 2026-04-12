## Definition of Ready (DoR)

- [x] `proposal.md` と `design.md` がレビュー済みであること
- [x] 現在の `master` ブランチの `make check` がパスすること
- [x] `ShortcutContext` の定義に関して設計方針がユーザー合意済みであること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0.22.1-shortcut-ux-redesign`
- **作業ブランチ**: 標準は `v0.22.1-shortcut-ux-redesign-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. ShortcutContext 基盤実装（コンテキスト境界の設計） ✅ 完了

このタスクはすべての後続タスクの前提となる最も重要な設計作業。
**PR #219 にてマージ済み（2026-04-12）**

- [x] 1.1 `crates/katana-ui/src/state/shortcut_context.rs` を新規作成し、`ShortcutContext` enum を実装する（`Global`, `Editor`, `Preview`, `Explorer`, `Modal`, `Recording`）
- [x] 1.2 `CommandInventoryItem` に `context: ShortcutContext` フィールドを追加し、全コマンドに適切なコンテキストを割り当てる（`Global` または `Editor`）
- [x] 1.3 `ShortcutContextResolver` を `shortcut_context.rs` に実装する（フレームごとのコンテキスト判定ロジック）
- [x] 1.4 `handle_shortcuts()` をコンテキスト認識型に書き換え、`cmd.context != Global && cmd.context != active_context` でスキップするロジックを追加する
- [x] 1.5 既存の `[editor]` サフィックスを `ShortcutContext::Editor` への移行に変換する互換ブリッジを実装する（後方互換ストリップを保持）
- [x] 1.6 UT を記述する: `ShortcutContextResolver` の各コンテキスト判定、コンテキスト優先順位、Recording時のショートカット無効化（17件全テスト通過）

### Definition of Done (DoD) ✅

- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `ShortcutContextResolver` の全分岐に UT（カバレッジ100%）が付いていること
- [x] 既存のショートカット動作が壊れていないこと（特に `edit.bold` と `view.explorer` の `primary+B` 競合が解消されていること）
- [x] `/openspec-delivery` ワークフローを実行し、デリバリー完了（PR #219 マージ済み）

---

## 2. ショートカット設定UI再設計

### Definition of Ready (DoR)

- [x] タスク1のデリバリーサイクルが完全に終了していること（PR #219 マージ済み）
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [ ] 2.1 `shortcuts.rs` から Edit ボタンを廃止し、行全体クリックとペンSVGアイコンによる録音トリガーに変更する
- [ ] 2.2 キー表示部分に OSネイティブキーSVGアイコン（⌘/⇧/⌥/Ctrl/Alt）を表示する `shortcut_token_to_display()` 関数を実装する
- [ ] 2.3 `ShortcutCaptureModal`（録音専用モーダル）を `settings/tabs/shortcut_capture_modal.rs` として新規実装する（Enterで確定、Escでキャンセル）
- [ ] 2.4 録音中（`ShortcutContext::Recording`）は `egui` の `set_key_filter` を用いて Esc/Enter 以外の入力をブロックする仕組みを実装する
- [ ] 2.5 OS差異の吸収: macOSの ⌘+Enter / Windows の Ctrl+Enter が "Enter" として扱われるよう録音判定を統一する
- [ ] 2.6 i18n キーを追加する（EN + JA 同時）: `settings.shortcuts.capture_prompt`, `settings.shortcuts.confirm_key`, `settings.shortcuts.cancel_key`
- [ ] 2.7 ショートカット設定画面に検索バーを追加する（プレースホルダー: `enter key binding to search...`、↑↓キーで検索履歴を呼び出し）
- [ ] 2.8 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.9 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 編集ボタンが完全に削除され、行クリックとペンアイコンで録音が起動すること
- [ ] 録音中に Esc/Enter 以外のキー（例: Cmd+Q）を押してもアプリが終了しないこと
- [ ] Enterで確定後にショートカットが保存されること
- [ ] OSネイティブキーSVGアイコン（⌘/⇧/⌥/Ctrl）が正しく表示されること
- [ ] 検索バーで絞り込みができること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン（自己レビュー、コミット、PR作成、マージ）を完了すること

---

## 3. デフォルトショートカット割り当て & i18n補完

### Definition of Ready (DoR)

- [x] タスク1のデリバリーサイクルが完全に終了していること（PR #219 マージ済み）
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

**Note:** タスク2と並行実施可。タスク2完了を待たずに着手できる。

### i18n 補完

- [ ] 3.0 `file.restore_closed` のラベル（`menu.restore_closed`）が英語フォールバック（"Restore Closed Document"）になっている全localeに翻訳を追加する（en以外の全locale: ja, de, fr, es, zh-CN, zh-TW 等）

### ショートカット割り当て

- [ ] 3.1 `file.close_workspace` → `primary+Shift+W`
- [ ] 3.2 `view.refresh_explorer` → `primary+Shift+R`
- [ ] 3.3 `view.close_all` → `primary+Shift+K`
- [ ] 3.4 `edit.strikethrough` → `primary+Shift+S`（Editor context）
- [ ] 3.5 `edit.heading1` → `primary+Shift+1`（Editor context）
- [ ] 3.6 `edit.heading2` → `primary+Shift+2`（Editor context）
- [ ] 3.7 `edit.heading3` → `primary+Shift+3`（Editor context）
- [ ] 3.8 `edit.bullet_list` → `primary+Shift+8`（Editor context）
- [ ] 3.9 `edit.numbered_list` → `primary+Shift+7`（Editor context）
- [ ] 3.10 `edit.blockquote` → `primary+Shift+.`（Editor context）
- [ ] 3.11 `edit.code_block` → `primary+alt+C`（Editor context）
- [ ] 3.12 `edit.horizontal_rule` → `primary+Shift+-`（Editor context、`egui::Key` 対応要確認）
- [ ] 3.13 `edit.insert_table` → `primary+alt+T`（Editor context）
- [ ] 3.14 `edit.ingest_image_file` → `primary+Shift+I`（Editor context）
- [ ] 3.15 `edit.ingest_clipboard_image` → `primary+alt+V`（Editor context）
- [ ] 3.16 割り当て後の重複チェック: `debug_assert!` をデバッグビルド起動時に実行し、コンテキスト内重複がないことを確認する

### Definition of Done (DoD)

- [ ] `menu.restore_closed` が全locale（11言語）に翻訳済みであること
- [ ] すべての割り当てが `ShortcutContext` と整合しており `[editor]` サフィックス表記が残っていないこと
- [ ] 割り当て後にショートカット競合が発生していないこと（`make check` 含む）
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン（自己レビュー、コミット、PR作成、マージ）を完了すること

---

## 4. ShortcutAST Linter（静的重複検知）

### Definition of Ready (DoR)

- [x] タスク1のデリバリーサイクルが完全に終了していること（PR #219 マージ済み）
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

**Note:** タスク2・3と並行実施可。タスク3の割り当て完了後にこのlinterで重複を静的検知することが推奨。

- [ ] 4.1 `katana-linter/src/rules/domains/shortcut/` ディレクトリを作成し、モジュール構造（`mod.rs`, `discovery.rs`, `conflict.rs`）を設定する
- [ ] 4.2 `discovery.rs`: `*_commands.rs` ファイルをパースして `{id, context, default_shortcuts}` を抽出する `CommandInventoryParser` を実装する
- [ ] 4.3 `conflict.rs`: `{context, os, shortcut}` が同一のエントリを重複として検知するルールを実装する（Global同士は完全重複禁止、Editor+Global は許可）
- [ ] 4.4 `ShortcutLinterOps::lint()` を `lib.rs` の lint エントリポイントに登録する
- [ ] 4.5 `tests/ast_linter.rs` に shortcut linter のテストケースを追加する（意図的な重複でエラー、正常ケースでパス）
- [ ] 4.6 Makefile の `ast-lint` ターゲットに shortcut linter が実行されることを確認する

### Definition of Done (DoD)

- [ ] `make ast-lint` で intentional な shortcut 重複が検知されること
- [ ] 正常なショートカット定義に対してfalse positiveが出ないこと
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン（自己レビュー、コミット、PR作成、マージ）を完了すること

---

## 5. Final Verification & Release Work

- [ ] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する
- [ ] 5.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 5.3 Base Feature Branchから `release/v0.22.1` へPRを作成する
- [ ] 5.4 PR上のCIチェック（Lint / Coverage / CodeQL）がパスすることを確認する
- [ ] 5.5 `release/v0.22.1` へマージする (`gh pr merge --merge --delete-branch`)
- [ ] 5.6 リリースバージョンに応じて `make release VERSION=0.22.1` を実行しCHANGELOGを更新する（`changelog-writing` スキル使用）
- [ ] 5.7 GitHub Release完了後に `/opsx-archive` でこのChangeをアーカイブする
