## 着手条件 (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.10.1 の bugfix scope が確認されていること
- [ ] `shell_ui.rs` / `preview_pane/renderer.rs` / `mermaid_renderer` / `plantuml_renderer` の現行 theme lookup 経路を確認していること

## ブランチ運用ルール

`##` ごとに grouped された task は、`/openspec-branching` workflow（`.agents/workflows/openspec-branching.md`）で定義された branching standard を無条件で守って実装すること。

---

## 1. ダイアグラムテーマ文脈の導入

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 1.1 `ThemeColors` から diagram 用 `DiagramRenderTheme` を生成する helper を定義する
- [ ] 1.2 `preview_pane/core_render.rs` の render job に request-scoped theme snapshot を載せる
- [ ] 1.3 `preview_pane/renderer.rs` の dispatch 経路を更新し、diagram backend が explicit theme parameter を受けるようにする
- [ ] 1.4 `mermaid_renderer` と `plantuml_renderer` から render path 上の `DiagramColorPreset::current()` 直接参照を外す
- [ ] 1.5 同じ helper に依存する diagram backend がある場合は theme source を統一する
- [ ] 1.6 theme mapping に追加ルールが必要と分かった場合は、コード継続前に `design.md` / `specs` / `tasks.md` を更新する

### 完了条件 (DoD)

- [ ] diagram renderer は request-scoped theme snapshot を使って描画されること
- [ ] render path の global preset lookup が主要経路から外れていること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 2. テーマ追従型キャッシュキーと更新経路の修正

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 2.1 `preview_pane/renderer.rs` の diagram cache key を dark / light bool 依存から `theme_fingerprint` 依存へ置き換える
- [ ] 2.2 同じ dark mode 内で `preview.text` 等が変わった場合も cache miss になることを確認する
- [ ] 2.3 `shell_ui.rs` の theme change → `RefreshDiagrams` 経路で、current theme snapshot に基づく再描画になるよう整合を取る
- [ ] 2.4 fingerprint が過剰 invalidation または不足 invalidation を起こすと分かった場合は artifact を先に更新する

### 完了条件 (DoD)

- [ ] theme change 後の diagram cache が stale 色を再利用しないこと
- [ ] `RefreshDiagrams` 後の active / inactive preview が current theme と一致すること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 3. 回帰テストと検証

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 3.1 unit test を追加し、theme fingerprint が変わると diagram cache key も変わることを確認する
- [ ] 3.2 Mermaid と PlantUML の少なくとも 2 経路で、theme / preview text color 変更後に renderer 入力が current theme と一致することを確認する
- [ ] 3.3 custom `preview.text` を同一 dark mode 内で変更した場合も diagram が再描画されることを確認する
- [ ] 3.4 static preset は fallback 用に残してよいが、preview render path の correctness 判定に使っていないことを確認する
- [ ] 3.5 実装途中に design 前提と乖離した点があれば、関連 artifact が先に更新されていることを確認する

### 完了条件 (DoD)

- [ ] Mermaid / PlantUML の regression が test で再発防止されていること
- [ ] custom preview color 変更で diagram text color が追従すること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 4. 最終確認とリリース作業

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 4.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を使って self-review を実施する（各 file の version 更新漏れも確認する）
- [ ] 4.2 `make check` が exit code 0 で通過することを確認する
- [ ] 4.3 中間 base branch（もともと master から派生した branch）を `master` へ merge する
- [ ] 4.4 `master` 向け PR を作成する
- [ ] 4.5 `master` へ merge する（`--admin` 許可）
- [ ] 4.6 `.agents/skills/release_workflow/SKILL.md` を使って `0.10.1` の release tagging と release 作成を実施する
- [ ] 4.7 `/opsx-archive` など OpenSpec skill を使ってこの change を archive する
