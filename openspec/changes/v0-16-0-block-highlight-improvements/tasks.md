## ブランチ運用ルール

`##` ごとにグループ化されたタスクは、実装セッション全体を通して `/openspec-branching` ワークフロー（`.agents/workflows/openspec-branching.md`）で定義されたブランチ標準へ無条件で従うこと。

---

## 0. 技術リファレンス（v0.8.10 セッションで判明した知見）

> **目的**: v0.16.0 実装時に再調査を不要にするため、根本原因分析の結果と確立した修正パターンをここに集約する。

### 0.1 ホバーハイライトのデータフロー

```
pulldown.rs (hovered_spans: Vec<Range<usize>>)  ← byte range を push
    ↓
section.rs (hovered_lines: Vec<Range<usize>>)   ← byte range → 行番号 range に変換
    ↓
preview.rs (scroll.hovered_preview_lines)       ← ScrollState に格納
    ↓
editor/ui.rs (line_range_to_char_range)         ← 行番号 → 文字インデックスに変換 → rect で描画
```

### 0.2 リストで発見・修正した2つの根本原因

#### 原因1: End handler の無制限 push

**場所**: `vendor/egui_commonmark_upstream/egui_commonmark/src/parsers/pulldown.rs` の `process_event` 内 `Event::End` ハンドラ

**問題**: `End(tag)` が `is_container`（Item, List, BlockQuote）以外の **全タグ** で `hovered_spans.push()` を実行していた。リスト内の `- [x] **bold** text` をホバーすると:
1. `End(Strong)` → rect contains pos → push（bold の span）
2. `End(Paragraph)` → rect contains pos → push（段落全体の span）

→ **必ず2つ以上** の span が push される。Paragraph rect は Strong rect を包含するため、100%再現。

**修正パターン**: リスト内では End handler を全面抑制し、`item_list_wrapping` でアイテム単位で1回だけ push する。

```rust
// End handler 内
let inside_list_item = !self.list.items.is_empty();
if !is_container && !inside_list_item {
    // push logic (リスト外のみ)
}
```

```rust
// item_list_wrapping 内（唯一の push 箇所）
if let Some(hovered) = &mut self.hovered_spans {
    if let Some(pos) = ui.ctx().pointer_hover_pos() {
        if rect.contains(pos) && pos.y < rect.max.y {  // 下端 exclusive
            hovered.push(span);
        }
    }
}
```

#### 原因2: src_span 末尾改行による off-by-one

**場所**: `crates/katana-ui/src/preview_pane/section.rs` の行番号変換

**問題**: pulldown_cmark の `src_span` は末尾の `\n` を含む。例: `- item one\n` → span `0..11`（11文字目が `\n`）。`md[..11]` の改行カウントが1多くなり、`end_line` が次の行を指す。

```
hovered.push(0..1)  // Range は exclusive end だが、
                     // line_range_to_char_range(buf, 0, 1) は line 0 と line 1 の両方をカバー
                     // → 2行分ハイライトされる
```

**修正**: `end_pos = local_span.end.saturating_sub(1).max(local_span.start)` で末尾改行を除外。

### 0.3 確立した設計原則

| 原則 | 説明 |
|------|------|
| **1ブロック1push** | 各ブロック要素（リストアイテム、テーブル、アコーディオン等）につき、`hovered_spans` への push は **1箇所** で **1回** のみ |
| **親ブロック優先** | 親ブロック（`item_list_wrapping` 等）が push を担当し、内部の End handler は抑制する |
| **末尾改行除外** | `src_span` → 行番号変換時は `span.end.saturating_sub(1)` で trailing newline を除外 |
| **下端 exclusive** | 隣接ブロックの共有境界で2つが同時 match しないよう `pos.y < rect.max.y` |

### 0.4 アコーディオン・テーブルへの適用指針

#### テーブル

- **現象**: ホバー時にハイライト自体が発生しない
- **原因仮説**: テーブルは `is_container` に含まれていないが、テーブル内部の描画が `process_event` を経由せず独自レンダリングしている可能性が高い。`hovered_spans` への push ロジックが存在しない
- **修正方針**: テーブル描画完了後に、テーブル全体の rect × span で1回 push する処理を追加

#### アコーディオン

- **現象**: ホバー時に隣接する次のブロック要素（例: 数式セクション）までハイライトが伸びる
- **原因仮説**: アコーディオンは HTML ブロックとして処理されるため、内部のリスト要素が End handler 経由で push される。リストの `inside_list_item` 抑制は効くが、アコーディオン自体の `src_span` が次セクションまで及んでいるか、HTML ブロックの End handler が別経路で push している
- **修正方針**: リストと同じパターン — アコーディオン描画を囲む親ブロック処理で1回 push + 内部 End handler を抑制。`inside_accordion` フラグまたは汎用的な `inside_custom_block` フラグの導入を検討

### 0.5 関連ファイル一覧

| ファイル | 役割 |
|----------|------|
| `vendor/egui_commonmark_upstream/egui_commonmark/src/parsers/pulldown.rs` | イベント処理・hovered_spans push（修正済） |
| `crates/katana-ui/src/preview_pane/section.rs` | byte span → 行番号変換（修正済） |
| `crates/katana-ui/src/widgets/markdown_hooks.rs` | リスト項目のビジュアルハイライト callback |
| `crates/katana-ui/src/views/panels/editor/ui.rs` | コードビュー側のハイライト描画 |
| `crates/katana-ui/src/views/panels/editor/logic.rs` | `line_range_to_char_range` 等のユーティリティ |
| `crates/katana-ui/src/views/panels/preview.rs` | hovered_lines → ScrollState への格納 |

---

## 1. アコーディオン・テーブル要素のプレビュー→コード連動ハイライト対応

### タスク

- [ ] 1.1 テーブル要素（`Start(Table)` 〜 `End(Table)`）のホバー時に、対応するソース行範囲を `hovered_spans` に正しく push するロジックを実装する
- [ ] 1.2 アコーディオン要素（`<details>` / `<summary>` HTML ブロック）のホバー時に、対応するソース行範囲を `hovered_spans` に正しく push するロジックを実装する
- [ ] 1.3 テーブル・アコーディオンそれぞれの `src_span` → 行番号変換が末尾改行を含まない（off-by-one が発生しない）ことをテストで検証する
- [ ] 1.4 既存のリスト要素・見出し・段落のホバーハイライトに回帰がないことを確認する
- [ ] 1.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### 完了条件 (DoD)

- [ ] テーブル・アコーディオンのホバー時に、コードビュー側で対応行のみが正確にハイライトされる
- [ ] 既存のリスト・見出し・段落のハイライトに回帰がない
- [ ] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、Self-review、Commit、PR 作成、Merge を含む包括的なデリバリー手順を完了する。

---

## 2. 最終検証とリリース作業

- [ ] 2.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を用いて self-review を実施する（各ファイルで version 更新漏れがないか確認する）
- [ ] 2.2 `make check` が exit code 0 で通ることを確認する
- [ ] 2.3 中間 base branch（もともと master から派生した branch）を `master` branch に merge する
- [ ] 2.4 `master` を向いた PR を作成する
- [ ] 2.5 master へ merge する（※ `--admin` 使用可）
- [ ] 2.6 `.agents/skills/release_workflow/SKILL.md` を用いて `0.16.0` 向けの release tag 作成と release 作成を実施する
- [ ] 2.7 `/opsx-archive` などの OpenSpec skill を活用して、この change を archive する
