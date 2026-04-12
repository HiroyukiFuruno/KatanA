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

## 1. ShortcutContext 基盤実装 & 緊急バグ修正 ✅ 完了

- [x] 1.1 `crates/katana-ui/src/state/shortcut_context.rs` を新規作成し、`ShortcutContext` enum を実装
- [x] 1.2 `CommandInventoryItem` に `context: ShortcutContext` フィールドを追加
- [x] 1.3 `ShortcutContextResolver` を実装
- [x] 1.4 `handle_shortcuts()` をコンテキスト認識型に書き換え
- [x] 1.5 既存の `[editor]` サフィックスの互換ブリッジ実装
- [x] 1.6 **緊急修正**: エクスプローラーのフィルターから隠しディレクトリ（.）を除外するロジックの実装
- [x] 1.7 **緊急修正**: フィルター有効化時に入力フィールドへ自動フォーカスする改善の実装

### Definition of Done (DoD) ✅

- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] 隠しディレクトリの除外が正常に機能すること
- [x] フィルター入力へのフォーカスが動作すること
- [x] `/openspec-delivery` ワークフローを実行し、デリバリー完了

---

## 2. Final Verification & Release Work

- [x] 2.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [x] 2.2 Ensure `make check` passes with exit code 0
- [x] 2.3 Create PR from Base Feature Branch targeting `master`
- [x] 2.4 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL)
- [x] 2.5 Merge into master (`gh pr merge --merge --delete-branch`)
- [x] 2.6 Run `make release VERSION=0.22.1` and update CHANGELOG
- [x] 2.7 Verify GitHub Release completion and archive this change using `/opsx-archive`
