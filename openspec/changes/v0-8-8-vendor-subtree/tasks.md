### 全体の着手条件 (DoR)

- [ ] `v0-8-7-preview-refresh-and-tasklist-fixes` が merge 済みで安定しており、同時に `vendor/*egui_commonmark*` を編集している branch が存在しない
- [ ] このメンテナンス枠で使う target release version が確定している
- [ ] 実装開始前に、`0.22.x` 系の互換 upstream revision または明示的な互換 fork revision が特定されている

## ブランチ運用ルール

`##` ごとに grouped された task は、`/openspec-branching` workflow（`.agents/workflows/openspec-branching.md`）で定義された branching standard を無条件で守って実装すること。

## 1. 現在の patch 棚卸しと前提固定

- [x] 1.1 `vendor/egui_commonmark` と `vendor/egui_commonmark_backend` を、選定した互換 upstream revision と diff する
- [x] 1.2 各 local delta を「保存すべき Katana patch」「除去可能な drift」「path / layout migration 対応」のいずれかへ分類する
- [x] 1.3 `katana-core` 統合、parser / rendering override、backend UI / layout 変更、vendored SVG asset を含む必須 patch inventory を明文化する
- [x] 1.4 upstream layout、互換 revision、local patch inventory のいずれかがこの change の前提と大きく異なる場合、続行前に `proposal.md`、`design.md`、`specs/`、`tasks.md` を更新する

### 完了条件 (DoD)

- [x] 現在の vendor-local delta が subtree import commit 前にすべて分類されている
- [x] 互換 upstream revision が明示的に pin されている
- [x] 中核前提が崩れた場合は artifact が先に修正されている
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 2. Subtree 取り込みと runtime path 張り替え

### 着手条件 (DoR)

- [x] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [x] 2.1 upstream `lampsitter/egui_commonmark` repository root を `vendor/egui_commonmark_upstream/` 配下へ `git subtree` で取り込む
- [x] 2.2 raw subtree import と Katana-specific change を別 commit に分ける
- [x] 2.3 `Cargo.toml` の `[patch.crates-io]` を更新し、`egui_commonmark` と `egui_commonmark_backend` が subtree root 内の crate subdirectory から解決されるようにする
- [x] 2.4 `crates/katana-ui/src/svg_loader/mod.rs` など direct file consumer を新しい subtree asset path へ更新する
- [x] 2.5 legacy `vendor/egui_commonmark` / `vendor/egui_commonmark_backend` directory への build / runtime 参照を除去する

### 完了条件 (DoD)

- [x] Katana は legacy copied directory ではなく subtree root から両 vendored crate を解決している
- [x] raw subtree import commit は Katana patch logic と混ざらず review 可能である
- [x] 削除済み legacy vendor layout に依存する runtime / build path が残っていない
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 3. Katana patch 再適用と regression 保護

### 着手条件 (DoR)

- [x] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [x] 3.1 Task 1 で確定した Katana-specific manifest、parser、rendering、asset、UI patch を再適用する
- [x] 3.2 Katana が既に依存している vendor-dependent behavior について、regression coverage を維持または追加する
- [x] 3.3 subtree migration により KatanA が暗黙に upstream `0.23.x` / `egui 0.34.x` へ上がっていないことを確認する
- [x] 3.4 `v0-8-7-preview-refresh-and-tasklist-fixes` の vendor-dependent fix が migration 後も維持されていることを確認する

### 完了条件 (DoD)

- [x] 最終 commit stack が subtree base と Katana patch layer を明確に分離している
- [x] vendor-dependent な runtime behavior が migration 前の契約を保っている
- [x] patch layer が守るべき挙動に対して必要な regression coverage が存在する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 4. Sync runbook と保守 handoff

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 4.1 `vendor/README.md` として maintainer runbook を追加し、subtree remote、prefix、互換 revision 方針、patch layer ルールを記述する
- [ ] 4.2 今後の subtree pull に必要な command と verification 手順を文書化する
- [ ] 4.3 互換性前提が変わった場合は OpenSpec artifact を先に更新する stop-and-correct rule を文書化する

### 完了条件 (DoD)

- [ ] 別の AI agent や maintainer が、この会話に頼らず subtree 更新と再検証を実行できる
- [ ] runbook (`vendor/README.md`) に steady-state sync 手順と artifact 修正への escalation path の両方が記載されている
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 5. 最終確認とリリース作業

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を使って self-review を実施する（各 file の version 更新漏れも確認する）
- [ ] 5.2 `make check` が exit code 0 で通過することを確認する
- [ ] 5.3 legacy `vendor/egui_commonmark*` directory を参照する code path が残っていないことを確認する
- [ ] 5.4 中間 base branch（もともと master から派生した branch）を `master` へ merge する
- [ ] 5.5 `master` 向け PR を作成する
- [ ] 5.6 `master` へ merge する（`--admin` 許可）
- [ ] 5.7 全体の着手条件で確定した target version に対して `.agents/skills/release_workflow/SKILL.md` を使い release tagging と release 作成を実施する
- [ ] 5.8 `/opsx-archive` など OpenSpec skill を使ってこの change を archive する
