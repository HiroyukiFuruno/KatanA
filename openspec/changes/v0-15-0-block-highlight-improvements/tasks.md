## ブランチ運用ルール

`##` ごとにグループ化されたタスクは、実装セッション全体を通して `/openspec-branching` ワークフロー（`.agents/workflows/openspec-branching.md`）で定義されたブランチ標準へ無条件で従うこと。

---

## 0. 技術リファレンス

### 0.1 ホバーハイライトのデータフロー

```text
pulldown.rs (hovered_spans: Vec<Range<usize>>)  ← byte range を push
    ↓
section.rs (hovered_lines: Vec<Range<usize>>)   ← byte range → 行番号 range に変換
    ↓
preview.rs (scroll.hovered_preview_lines)       ← ScrollState に格納
    ↓
editor/ui.rs (line_range_to_char_range)         ← 行番号 → 文字インデックスに変換 → rect で描画
```

### 0.2 根本原因の要約

- End handler が複合ブロック内で重複 push している
- `src_span` 末尾改行により行番号変換が 1 行ずれる
- 親ブロックと子要素の責務が混在している

### 0.3 関連ファイル一覧

| ファイル | 役割 |
| ---------- | ------ |
| `vendor/egui_commonmark_upstream/egui_commonmark/src/parsers/pulldown.rs` | イベント処理・hovered_spans push |
| `crates/katana-ui/src/preview_pane/section.rs` | byte span → 行番号変換 |
| `crates/katana-ui/src/widgets/markdown_hooks.rs` | リスト項目のビジュアルハイライト callback |
| `crates/katana-ui/src/views/panels/editor/ui.rs` | コードビュー側のハイライト描画 |
| `crates/katana-ui/src/views/panels/editor/logic.rs` | `line_range_to_char_range` 等のユーティリティ |
| `crates/katana-ui/src/views/panels/preview.rs` | hovered_lines → ScrollState への格納 |

---

## 1. アコーディオン・テーブル要素のプレビュー→コード連動ハイライト対応

### Definition of Ready (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

### タスク

- [ ] 1.1 テーブル要素（`Start(Table)` 〜 `End(Table)`）のホバー時に、対応するソース行範囲を `hovered_spans` に正しく push するロジックを実装する
- [ ] 1.2 アコーディオン要素（`<details>` / `<summary>` HTML ブロック）のホバー時に、対応するソース行範囲を `hovered_spans` に正しく push するロジックを実装する
- [ ] 1.3 テーブル・アコーディオンそれぞれの `src_span` → 行番号変換が末尾改行を含まないことをテストで検証する
- [ ] 1.4 既存のリスト要素・見出し・段落のホバーハイライトに回帰がないことを確認する
- [ ] 1.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] テーブル・アコーディオンのホバー時に、コードビュー側で対応行のみが正確にハイライトされる
- [ ] 既存のリスト・見出し・段落のハイライトに回帰がない
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. 最終検証とリリース作業

### Definition of Ready (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 2.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を用いて self-review を実施する（各ファイルで version 更新漏れがないか確認する）
- [ ] 2.2 `make check` が exit code 0 で通ることを確認する
- [ ] 2.3 中間 base branch（もともと master から派生した branch）を `master` branch に merge する
- [ ] 2.4 `master` を向いた PR を作成する
- [ ] 2.5 master へ merge する（※ `--admin` 使用可）
- [ ] 2.6 `.agents/skills/release_workflow/SKILL.md` を用いて `0.15.0` 向けの release tag 作成と release 作成を実施する
- [ ] 2.7 `/opsx-archive` などの OpenSpec skill を活用して、この change を archive する
