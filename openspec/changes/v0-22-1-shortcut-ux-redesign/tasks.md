## Definition of Ready (DoR)

- [ ] `proposal.md` と `design.md` がレビュー済みであること
- [ ] 現在の `master` ブランチの `make check` がパスすること
- [ ] `ShortcutContext` の定義に関して設計方針がユーザー合意済みであること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-1-shortcut-ux-redesign`
- **作業ブランチ**: 標準は `v0-22-1-shortcut-ux-redesign-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. ShortcutContext 基盤実装（コンテキスト境界の設計）

このタスクはすべての後続タスクの前提となる最も重要な設計作業。

- [x] 1.1 `crates/katana-ui/src/state/shortcut_context.rs` を新規作成し、`ShortcutContext` enum を実装する（`Global`, `Editor`, `Preview`, `Explorer`, `Modal`, `Recording`）
- [x] 1.2 `CommandInventoryItem` に `context: ShortcutContext` フィールドを追加し、全コマンドに適切なコンテキストを割り当てる（`Global` または `Editor`）
- [x] 1.3 `ShortcutContextResolver` を `shortcut_context.rs` に実装する（フレームごとのコンテキスト判定ロジック）
- [x] 1.4 `handle_shortcuts()` をコンテキスト認識型に書き換え、`cmd.context != Global && cmd.context != active_context` でスキップするロジックを追加する
- [x] 1.5 既存の `[editor]` サフィックスを `ShortcutContext::Editor` への移行に変換する互換ブリッジを実装する（サフィックス文字列自体はまだ残す）
- [x] 1.6 UT を記述する: `ShortcutContextResolver` の各コンテキスト判定、コンテキスト優先順位、Recording時のショートカット無効化
- [ ] 1.7 ユーザーへのUIスナップショット（動作確認）の提示および報告
- [ ] 1.8 ユーザーからのフィードバックに基づく調整

### Definition of Done (DoD)

- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `ShortcutContextResolver` の全分岐に UT（カバレッジ100%）が付いていること
- [ ] 既存のショートカット動作が壊れていないこと（特に `edit.bold` と `view.explorer` の `primary+B` 競合が解消されていること）
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン（自己レビュー、コミット、PR作成、マージ）を完了すること

---

## 2. ショートカット設定UI再設計

### Definition of Ready (DoR)

- [ ] タスク1のデリバリーサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）が完全に終了していること
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [ ] 2.1 `shortcuts.rs` から Edit ボタンを廃止し、行全体クリックとペンSVGアイコンによる録音トリガーに変更する
- [ ] 2.2 キー表示部分に OSネイティブキーSVGアイコン（⌘/⇧/⌥/Ctrl/Alt）を表示する `shortcut_token_to_display()` 関数を実装する
- [ ] 2.3 `ShortcutCaptureModal`（録音専用モーダル）を `settings/tabs/shortcut_capture_modal.rs` として新規実装する（Enterで確定、Escでキャンセル）
- [ ] 2.4 録音中（`ShortcutContext::Recording`）は `egui` の `set_key_filter` を用いて Esc/Enter 以外の入力をブロックする仕組みを実装する
- [ ] 2.5 OS差異の吸収: macOSの ⌘+Enter / Windows の Ctrl+Enter が "Enter" として扱われるよう録音判定を統一する
- [ ] 2.6 i18n キーを追加する（EN + JA 同時）: `settings.shortcuts.capture_prompt`, `settings.shortcuts.confirm_key`, `settings.shortcuts.cancel_key`
- [ ] 2.7 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.8 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 編集ボタンが完全に削除され、行クリックとペンアイコンで録音が起動すること
- [ ] 録音中に Esc 以外のキー（例: Cmd+Q）を押してもアプリが終了しないこと
- [ ] Enterで確定後にショートカットが保存されること
- [ ] OFileSVGアイコン（⌘/⇧/⌥/Ctrl）が正しく表示されること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン（自己レビュー、コミット、PR作成、マージ）を完了すること

---

## 3. デフォルトショートカット割り当て

### Definition of Ready (DoR)

- [ ] タスク1・2のデリバリーサイクルが完全に終了していること
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [ ] 3.1 `file.close_workspace` → `primary+Shift+W` を割り当てる
- [ ] 3.2 `view.refresh_explorer` → `primary+Shift+R` を割り当てる
- [ ] 3.3 `view.close_all` → `primary+Shift+K` を割り当てる
- [ ] 3.4 `edit.strikethrough` → `primary+Shift+S[editor]` を割り当てる
- [ ] 3.5 `edit.heading1〜3` → `primary+Shift+1/2/3[editor]` を割り当てる
- [ ] 3.6 `edit.bullet_list` → `primary+Shift+8[editor]` を割り当てる
- [ ] 3.7 `edit.numbered_list` → `primary+Shift+7[editor]` を割り当てる
- [ ] 3.8 `edit.blockquote` → `primary+Shift+.[editor]` を割り当てる
- [ ] 3.9 `edit.code_block` → `primary+alt+C[editor]` を割り当てる
- [ ] 3.10 `edit.horizontal_rule` → `primary+Shift+-[editor]` を割り当てる（Note: `-` が `egui::Key`として対応するか要確認）
- [ ] 3.11 `edit.insert_table` → `primary+alt+T[editor]` を割り当てる
- [ ] 3.12 `edit.ingest_image_file` → `primary+Shift+I[editor]` を割り当てる
- [ ] 3.13 `edit.ingest_clipboard_image` → `primary+alt+V[editor]` を割り当てる
- [ ] 3.14 重複がないことをランタイム起動時（デバッグビルド）にアサートする `debug_assert!` を追加する

### Definition of Done (DoD)

- [ ] すべての割り当てがShortcutContextと整合していること
- [ ] 割り当て後にショートカット競合が発生していないこと（`make check` 含む）
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン（自己レビュー、コミット、PR作成、マージ）を完了すること

---

## 4. ShortcutAST Linter（静的重複検知）

### Definition of Ready (DoR)

- [ ] タスク1〜3のデリバリーサイクルが完全に終了していること
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

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
- [ ] 5.3 Base Feature Branchから `master` へPRを作成する
- [ ] 5.4 PR上のCIチェック（Lint / Coverage / CodeQL）がパスすることを確認する
- [ ] 5.5 `master` へマージする (`gh pr merge --merge --delete-branch`)
- [ ] 5.6 リリースバージョンに応じて `make release VERSION=x.y.z` を実行しCHANGELOGを更新する（`changelog-writing` スキル使用）
- [ ] 5.7 GitHub Release完了後に `/opsx-archive` でこのChangeをアーカイブする
