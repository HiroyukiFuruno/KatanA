# Tasks: extract-katana-ast-lint

## 1. Source Inventory

### Definition of Ready

- [ ] P0として `katana-ast-lint` を先に分離する合意がある
- [ ] KatanA本体にあるAST lint相当の検査入口を確認できる

### Tasks

- [ ] 1.1 既存AST lint相当の検査、対象file、違反形式を棚卸しする
- [ ] 1.2 共通化するruleとrepository固有adapterへ残す処理を分ける
- [ ] 1.3 KMM、kdp、kle、krr、kcf、kuwで必要になる最低ruleを整理する

### Definition of Done

- [ ] 共通ruleへ移す対象が明確である
- [ ] KatanA固有pathを共通ruleへ直書きしない方針が明確である

## 2. Repository Contract

### Definition of Ready

- [ ] `katana-ast-lint/openspec/changes/bootstrap-shared-ast-lint` が作成済みである

### Tasks

- [ ] 2.1 共通実行入口を定義する
- [ ] 2.2 共通違反形式を定義する
- [ ] 2.3 各repositoryが持つadapter責務を定義する
- [ ] 2.4 lint除外で失敗を隠さない運用を明文化する

### Definition of Done

- [ ] P1以降のrepositoryが同じAST lint品質ゲートを参照できる
- [ ] repositoryごとの独自lint driftを検出できる

## 3. Downstream Gate

### Definition of Ready

- [ ] Task 2のcontractが確定している

### Tasks

- [ ] 3.1 `katana-markdown-model` の着手条件へP0完了を入れる
- [ ] 3.2 `katana-ui-widget` の着手条件へP0完了を入れる
- [ ] 3.3 `katana-document-preview`、`katana-language-editor`、`katana-render-runtime`、`katana-canvas-forge` の後続計画へP0利用を入れる
- [ ] 3.4 KatanA本体の `crates/katana-linter` と `katana-ast-lint` の重複ruleを照合する
- [ ] 3.5 KatanA本体の `Cargo.toml` workspace dependencyへ `katana-ast-lint` を追加し、`just ast-lint` の実行入口を外部crate利用へ移す
- [ ] 3.6 KatanA本体に残すべきrepository adapter責務と、削除または縮小する内部linter責務を分ける
- [ ] 3.7 KatanA本体の `crates/katana-linter` を残す場合は、共通ruleを再実装せずadapter/test runnerだけを持つ境界にする

### Definition of Done

- [ ] KMM以降の分離repositoryが共通AST lintを前提にしている
- [ ] KatanA本体が内部 `katana-linter` のコピー実装へ戻っていない

## 4. Final Verification

- [ ] 4.1 `scripts/openspec validate "extract-katana-ast-lint" --strict` を実行する
- [ ] 4.2 親OpenSpecとP1/P2計画にP0依存が反映されていることを確認する
- [ ] 4.3 KMM、kdp、kle、krr、kcf、kuw、KatanA本体の各repositoryで、test/CIから `katana_ast_lint` APIを呼べる計画になっていることを確認する
