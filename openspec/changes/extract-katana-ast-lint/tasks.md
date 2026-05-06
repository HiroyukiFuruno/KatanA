# Tasks: extract-katana-ast-lint

## 1. Source Inventory

### Definition of Ready

- [ ] P0として `katana-ast-lint` を先に分離する合意がある
- [ ] KatanA本体にあるAST lint相当の検査入口を確認できる

### Tasks

- [ ] 1.1 既存AST lint相当の検査、対象file、違反形式を棚卸しする
- [ ] 1.2 共通化するruleとrepository固有adapterへ残す処理を分ける
- [ ] 1.3 KME、kdp、kle、kcf、kuwで必要になる最低ruleを整理する

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

- [ ] 3.1 `katana-markdown-engine` の着手条件へP0完了を入れる
- [ ] 3.2 `katana-ui-widget` の着手条件へP0完了を入れる
- [ ] 3.3 `katana-document-preview`、`katana-language-editor`、`katana-canvas-forge` の後続計画へP0利用を入れる

### Definition of Done

- [ ] KME以降の分離repositoryが共通AST lintを前提にしている

## 4. Final Verification

- [ ] 4.1 `npx -y @fission-ai/openspec validate "extract-katana-ast-lint" --strict` を実行する
- [ ] 4.2 親OpenSpecとP1/P2計画にP0依存が反映されていることを確認する
