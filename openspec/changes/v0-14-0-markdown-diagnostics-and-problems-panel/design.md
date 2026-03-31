## Context

KatanA には既に `katana-linter` があり、Markdown pair heading structure などの rule も実装されている。しかし現状の surface は test/CI 寄りで、執筆者が app の中で問題を読んで直す導線がない。
一方、UI 側には status bar、editor pane、preview pane、search modal はあるが、diagnostics を継続的に表示する Problems Panel はない。

`v1.0.0` 以前に local LLM を入れない前提なら、まず deterministic な diagnostics を app 内で見える化する方が価値が高い。

## Goals / Non-Goals

**Goals:**

- 既存 linter foundation を使って Markdown diagnostics を app 内で見えるようにする
- Problems Panel で diagnostics を継続的に確認できるようにする
- diagnostics から editor / preview の該当箇所へ jump できるようにする
- per-keystroke analysis を避け、manual refresh と save-triggered refresh で安定運用する
- 他の AI エージェントが会話なしで diagnostic contract を組み立てられるようにする

**Non-Goals:**

- local LLM による自動修正提案
- Rust AST lint 全体を app UI に完全統合すること
- keystroke ごとのリアルタイム lint 実行
- 一般的な language server の全面導入

## Decisions

### 1. Diagnostics engine は `katana-linter` を再利用する

新しい parser や separate diagnostics engine を UI 側に作るのではなく、既存の `katana-linter` rule 群を再利用する。Markdown diagnostics は linter crate 側に寄せ、UI は結果表示と navigation に集中する。

- 採用理由:
  - 既存 rule と CI rule を再利用できる
  - rule の source of truth が分裂しない
  - contributor 向けにも「CI と app で同じ問題を見る」形にしやすい
- 代替案:
  - UI 側で独自に Markdown diagnostics 実装: rule 二重管理になるため不採用

### 2. Diagnostics contract は source line/column range を持つ

Problems Panel から editor / preview に jump するため、diagnostic result は少なくとも file path、severity、message、rule id、source line/column range を持つ。
既存の editor / preview 周辺は line-based navigation と相性がよいため、location も source range に揃える。

- 採用理由:
  - jump 先が一意になる
  - editor highlight と preview reveal を揃えやすい
  - 他の AI エージェントが diagnostics payload を誤解しにくい
- 代替案:
  - line number only: range highlight が弱い
  - byte offset only: UI navigation contract が読みにくいため不採用

### 3. Problems Panel は modal ではなく persistent panel にする

diagnostics は編集しながら何度も見返す情報なので、検索 modal のような一時 UI より persistent panel の方が適している。
位置は editor / preview の読書導線を壊しにくい bottom panel を第一候補とする。

- 採用理由:
  - 問題一覧を見ながら修正しやすい
  - empty state / counts / grouping を維持しやすい
  - search modal と責務が混ざらない
- 代替案:
  - modal: 一覧確認と修正の往復に不向き
  - status bar only: 件数は見えても具体的修正導線にならない

### 4. Refresh policy は manual + on-save を初期スコープにする

per-keystroke lint は editor 体験を不安定にしやすく、rule 拡張時のコストも読みにくい。そのため初期スコープでは manual refresh と save-triggered refresh を採用する。

- 採用理由:
  - 性能と predictability を守りやすい
  - rule 実行の責務が説明しやすい
  - contributor にも「保存後/明示実行で更新」と伝えやすい
- 代替案:
  - live lint: 初期スコープとしては重く、誤検知時のノイズも大きいため不採用

### 5. 初期 rule set は Markdown workspace 向けの deterministic checks に絞る

初期対象は KatanA の価値に直結する文書品質問題へ絞る。

- heading hierarchy / structure
- `*.md` と `*.ja.md` の heading sync
- broken relative links
- missing local assets

Rust AST lint 全体の surface は将来拡張として残し、まずは Markdown workspace product としての価値を優先する。

## Risks / Trade-offs

- [Risk] CI lint と app diagnostics の結果がズレる -> Mitigation: rule 実装は `katana-linter` 側に集約する
- [Risk] diagnostics refresh が重くなる -> Mitigation: initial scope は manual + on-save に限定する
- [Risk] Problems Panel が画面を圧迫する -> Mitigation: bottom panel として折りたたみ可能な設計にする
- [Risk] broken links や assets の rule が false positive を出す -> Mitigation: workspace-root-relative resolution contract を先に固定する
