## Definition of Ready (DoR)

- [x] `proposal.md`、`design.md`、`specs/markdown-authoring/spec.md`、`specs/markdown-asset-ingest/spec.md` が active change として揃っていること
- [x] 対象バージョンが `0.22.5`、change-id が `v0-22-5-code-input-improvements`、作業ベースが `release/v0.22.5` であることを確認する
- [x] 既存 archive `openspec/changes/archive/2026-04-24-v0-22-5-code-input-improvements` は参照のみとし、編集・移動しないことを確認する
- [x] ユーザーFBの優先順位を確認する: Markdown 入力、toolbar 表示、上方向 scroll 復帰をデグレード優先修正として扱う

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `release/v0.22.5`
- **作業ブランチ**: `feature/v0.22.5-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

> Implementation note: 以前の v0.22.5 worktree は破棄して作り直したため、タスク 1-5 は `feature/v0.22.5-task1-code-input-recovery` でまとめて recovery 実装する。

---

## 1. User Review Feedback Regression Baseline

- [x] 1.1 `release/v0.22.5` 上で、Markdown が入力できること、入力サポート UI/toolbar が表示されること、editor を上に戻せることを手動または既存 harness で再現確認する
- [x] 1.2 editor 入力中に競合しうる shortcut を棚卸しする: normal paste、copy/cut、undo/redo、select all、IME composition、cursor/selection movement、newline、indent、delete/backspace
- [x] 1.3 画像添付の既存導線を確認し、command palette から image attach を検索・実行できるかを記録する
- [x] 1.4 clipboard に画像だけがある場合と text がある場合の normal paste 挙動を分けて記録する
- [x] 1.5 上記 4 つのユーザーFBを回帰テストまたは明示的な検証手順に落とし込む

### Definition of Done (DoD)

- [x] v0.22.5 で修正すべき入力回帰が、再現条件と期待結果つきで整理されていること
- [x] 以降のタスクが参照できる検証項目として、ユーザーFB 1-4 がすべて追跡可能になっていること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Editor Input Shortcut Conflict Isolation

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 2.1 `crates/katana-ui/src/state/shortcut_context.rs` と `crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs` を修正し、editor focus 中の protected text-entry shortcut を app command dispatch が consume しないようにする
- [x] 2.2 `crates/katana-ui/src/state/command_inventory/edit_commands.rs` の default shortcut を見直し、editor/native input と競合する authoring command は default shortcut を外すか非競合の導線へ移す
- [x] 2.3 editor focus 中に Global shortcut が protected input を横取りしない unit test を追加する
- [x] 2.4 non-conflicting editor command は toolbar または command palette から実行できることを確認する

### Definition of Done (DoD)

- [x] editor 入力モードで、入力機能と競合する shortcut が無効化されていること
- [x] Markdown typing、IME composition、selection movement、normal paste、undo/redo が app command と競合しないこと
- [x] shortcut 関連 unit test が pass すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Markdown Input, Toolbar, and Scroll Regression Fixes

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 3.1 `crates/katana-ui/src/views/panels/editor/ui.rs` の `TextEdit` が editable Markdown で入力可能な状態を維持していることを確認し、typing が `AppAction::UpdateBuffer` へ到達するよう修正する
- [x] 3.2 `crates/katana-ui/src/views/panels/editor/toolbar.rs` の authoring toolbar が editable Markdown document で常に表示され、read-only/reference/virtual document では mutating controls を出さないようにする
- [x] 3.3 toolbar button からの authoring action が Markdown source buffer、dirty state、preview refresh と同じ更新経路に乗ることを確認する
- [x] 3.4 `crates/katana-ui/src/views/panels/editor/logic.rs` と `crates/katana-ui/src/state/scroll.rs` 周辺を修正し、scroll down 後に上方向へ戻せること、scroll sync が manual upward scroll を妨げないことを保証する
- [x] 3.5 UI スナップショットまたは integration harness で、Markdown 入力、toolbar 表示、上方向 scroll 復帰をユーザーに提示できる形で確認する
- [x] 3.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [x] Markdown が通常入力できること
- [x] 入力サポート UI/toolbar が editable Markdown document で表示されること
- [x] 長い Markdown document で下へ scroll した後、先頭まで上に戻せること
- [x] editor/toolbar/scroll 関連の focused tests が pass すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Image Attach Command Palette Route and Normal Paste

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 4.1 `AppAction::IngestImageFile` を shared command inventory 経由で command palette result として検索・実行できることを確認し、不足があれば provider/result execution を修正する
- [x] 4.2 command palette で「Attach Image File」「画像添付」相当の query から画像添付 flow に到達できる integration test または UI 検証を追加する
- [x] 4.3 normal paste gesture で clipboard image を検出し、`AppAction::IngestClipboardImage` または同等の ingest path に接続する
- [x] 4.4 clipboard text がある場合は normal paste が text paste として動作し、image ingest が横取りしないことを保証する
- [x] 4.5 画像保存先、directory auto-create、Markdown image reference 挿入が existing ingest settings と一致することを unit test で確認する

### Definition of Done (DoD)

- [x] 画像添付を command palette から開始できること
- [x] クリップボード画像を通常 paste で Markdown document に挿入できること
- [x] text clipboard の通常 paste が壊れていないこと
- [x] image ingest 関連の focused tests が pass すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Final Verification and Release Readiness

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する
- [x] 5.2 ユーザーFB 4 件を release checklist として再検証する: shortcut 無効化、command palette 画像添付、normal paste 画像、Markdown 入力/toolbar/scroll 復帰
- [x] 5.3 `cargo test -p katana-ui` など focused tests を実行し、editor shortcut、toolbar、scroll、image ingest の対象テストが pass することを確認する
- [x] 5.4 `make check` が exit code 0 で通過することを確認する
- [x] 5.5 `release/v0.22.5` 上で impl-release workflow がこの active change を参照できることを確認する
- [ ] 5.6 Release Readiness CI と GitHub Release 完了後、この change を `/opsx-archive` で archive する

---

## 6. User Review Feedback: Image Ingest Completion

- [x] 6.1 クリップボード画像 paste が実際に画像データを取得できない経路を調査し、通常 paste で Markdown 画像参照が挿入されるよう修正する
- [x] 6.2 Finder など OS ファイルエクスプローラでコピーした画像ファイルも clipboard image paste として取り込めるようにする
- [x] 6.3 editor toolbar/control icon から画像ファイル挿入を開始できるようにし、OS file picker で選択した画像を既存 ingest path に接続する
- [x] 6.4 editor context menu から toolbar/control 相当の Markdown authoring と画像挿入操作を grouped submenu として実行できるようにする
- [x] 6.5 `markdown-asset-ingest` spec に toolbar control、grouped context menu、clipboard image file paste、`Command+V` の受け入れ条件を追記する
- [x] 6.6 focused tests と `openspec validate v0-22-5-code-input-improvements` を通す
- [x] 6.7 toolbar group separator を `|` 表示にし、separator と icon group の上下中央を揃える

---

## 7. User Review Feedback: Preview-first and Contextual Input Controls

- [x] 7.1 未指定の active view mode は editable/reference/virtual document を問わず `PreviewOnly` にし、明示的な user-selected mode は維持する
- [x] 7.2 入力サポート UI を常時表示 toolbar から、editable editor focus と cursor range に連動する cursor-adjacent popup に変更する
- [x] 7.3 popup 内の authoring group は `|` separator で区切り、separator と icon 群の上下中央を揃えたまま維持する
- [x] 7.4 egui の特殊文字・OS絵文字制約を避ける独自入力ウィンドウ案は、v0.22.5 の実装範囲外の設計 follow-up として `design.md` に明記する
- [x] 7.5 focused tests、`openspec validate v0-22-5-code-input-improvements`、必要な full check を通す
