# Tasks: extract-katana-ui-widget

## 1. Widget Boundary

### Definition of Ready

- [ ] Floem移行方針が親OpenSpecで確定している
- [ ] P0 `katana-ast-lint` の共通品質ゲート方針が利用可能である
- [ ] P1 KMEのmetadata/display DTO方針が利用可能である
- [ ] KME/kdp/kleで必要になる共通UI部品候補が列挙されている

### Tasks

- [ ] 1.1 `katana-ui/src/widgets` の分離候補を棚卸しする
- [ ] 1.2 KME metadata表示、unresolved表示、copy/edit actionを初期候補にする
- [ ] 1.3 KatanA shell/chrome固有部品を対象外にする
- [ ] 1.4 `kcu` で見えている課題を責務境界の入力として整理する

### Definition of Done

- [ ] `katana-ui-widget` が所有するものと所有しないものが明確である
- [ ] KME文書モデルやmetadata schemaをwidget repoへ持たせない方針が明確である
- [ ] KMEより先にUI表示型を固定しない方針が明確である

## 2. Repository Bootstrap

### Definition of Ready

- [ ] Widget boundaryが確定している
- [ ] 共通AST lintを品質ゲートとして使う方針が確定している

### Tasks

- [x] 2.1 `katana-ui-widget` repositoryを作成する
- [ ] 2.2 Floem前提のcrate構成を定義する
- [ ] 2.3 metadata/unresolved表示の最小DTOを定義する
- [ ] 2.4 P0 AST lintの実行入口を接続する

### Definition of Done

- [ ] egui依存がない
- [ ] kdp/kle/KatanAから共通UI部品として参照できる
- [ ] 共通AST lintを避けるための独自除外を持たない

## 3. Final Verification

- [ ] 3.1 `npx -y @fission-ai/openspec validate "extract-katana-ui-widget" --strict` を実行する
