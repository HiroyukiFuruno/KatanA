# Tasks: establish-kme-markdown-platform

## 1. Platform Contract

### Definition of Ready

- [ ] `assets/fixtures/sample.md` とREADME badgeをKME v0の仕様fixtureとして扱う合意がある
- [ ] Floemをpreview/editor/widgetの前提にする方針が合意済みである
- [ ] P0 `katana-ast-lint`、P1 `katana-markdown-engine`、P2 `katana-ui-widget`、P3 その他の順序が合意済みである

### Tasks

- [ ] 1.1 KME v0の対象を「現在KatanAで実現できているMarkdown挙動の踏襲」として固定する
- [ ] 1.2 `sample.md`、README badge、alert、description listのfixture責務を明文化する
- [ ] 1.3 CommonMark完全準拠をv0の完了条件にしないことを明記する
- [ ] 1.4 KME、editor、preview、export、KatanA統合、widget分離のrepository責務を固定する
- [ ] 1.5 AST lint共通化をP0として固定する

### Definition of Done

- [ ] fixture基準が親OpenSpecから参照できる
- [ ] 各repositoryのDoR/DoDが親OpenSpecにある
- [ ] 依存方向がKME中心の一方向として説明されている

## 2. Metadata and Editor Contract

### Definition of Ready

- [ ] KMEがmetadata schemaと位置解決を持つ方針が合意済みである

### Tasks

- [ ] 2.1 metadataをMarkdown本文に埋め込まない方針を固定する
- [ ] 2.2 `README.md.metadata.json` を標準命名として定義する
- [ ] 2.3 metadata targetにfile path、node id、byte range、line-column、fingerprint、前後文脈を含める
- [ ] 2.4 editor保存直後にmetadataを更新する責務を `katana-language-editor` へ割り当てる
- [ ] 2.5 unresolved metadataを削除せず保持する方針を固定する

### Definition of Done

- [ ] metadata contractがKME、editor、preview、exportの各OpenSpecへ展開できる
- [ ] PDFページングとLLM注釈の両方がmetadataで扱える

## 3. Repository OpenSpec Expansion

### Definition of Ready

- [ ] 各repositoryの作業境界が確定している

### Tasks

- [ ] 3.1 `katana-ast-lint` に `bootstrap-shared-ast-lint` を作る
- [ ] 3.2 `katana` に `extract-katana-ast-lint` を作る
- [ ] 3.3 `katana-markdown-engine` に `bootstrap-kme-document-model` を作る
- [ ] 3.4 `katana-language-editor` に `sync-kme-metadata-on-save` を作る
- [ ] 3.5 `katana-document-preview` に `adopt-kme-preview-model` を作る
- [ ] 3.6 `katana-canvas-forge` の `v0-1-2-export-css-debug` を品質ゲート範囲として維持する
- [ ] 3.7 `katana` に `adopt-kme-in-katana` を作る
- [ ] 3.8 `katana` に `extract-katana-ui-widget` を作る

### Definition of Done

- [ ] 各OpenSpecにDoR/DoDがある
- [ ] KME本体実装と品質ゲート構築が混ざっていない

## 4. Final Verification

- [ ] 4.1 `npx -y @fission-ai/openspec validate "establish-kme-markdown-platform" --strict` を実行する
- [ ] 4.2 関連repositoryのOpenSpec validation結果を記録する
- [ ] 4.3 KME repository作成とOpenSpec展開の差分を確認する
- [ ] 4.4 `katana-ast-lint` repository作成とOpenSpec展開の差分を確認する
