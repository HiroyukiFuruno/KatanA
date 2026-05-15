# Tasks: establish-kme-markdown-platform

## 0. 全体見直し

### Definition of Ready

- [x] P0 `katana-ast-lint`、P1 `katana-markdown-model`、P2 `katana-ui-widget`、P3 downstream統合の順序が合意済みである
- [x] Floemをviewer / editor / widgetの前提にする方針が合意済みである
- [x] KCFは外部描画へ縮小し、既存exportはKDV移譲まで維持する方針が合意済みである

### Tasks

- [x] 0.1 KMMをMarkdown文書モデル、source mapping、metadata target解決の正本として固定する
- [x] 0.2 KALを分離repo共通のAST lint品質ゲートとして固定する
- [x] 0.3 KUWが未作成であることを、P2のblocking riskとして明記する
- [x] 0.4 KCFを外部描画へ縮小し、既存exportをKDV実装まで維持する条件を明記する
- [x] 0.5 KMMのpublic DTOとmetadata APIが固定されるまで、downstreamが独自document modelやmetadata schemaを作らないことを各repo OpenSpecへ反映する

### Definition of Done

- [x] 別セッションが親OpenSpecからrepo別の開始条件、完了条件、停止条件を判断できる
- [x] downstream repoがKMMより先に仕様を確定してよい範囲と、待つべき範囲を判断できる

## 1. Platform Contract

### Definition of Ready

- [x] `assets/fixtures/sample.md` とREADME badgeをKMM v0の仕様fixtureとして扱う合意がある
- [x] alert、description list、table/grid、footnote、emoji、diagram、math、HTML blockをKatanA現行仕様の棚卸し対象にする
- [x] CommonMark完全準拠をKMM v0の完了条件にしない方針が合意済みである

### Tasks

- [x] 1.1 KMM v0の対象を「現在KatanAで実現できているMarkdown挙動の踏襲」として固定する
- [x] 1.2 `sample.md`、README badge、alert、description listのfixture責務を明文化する
- [x] 1.3 CommonMark完全準拠をv0の完了条件にしないことを明記する
- [x] 1.4 KMM、editor、viewer、export、KatanA統合、widget分離のrepository責務を固定する
- [x] 1.5 AST lint共通化をP0として固定する

### Definition of Done

- [x] fixture基準が親OpenSpecから参照できる
- [x] 各repositoryのDoR/DoDが親OpenSpecにある
- [x] 依存方向がKMM中心の一方向として説明されている

## 2. Repository DoR / DoD

### Definition of Ready

- [x] `katana-ast-lint` が分離済みで、KMMから利用できる
- [x] `katana-markdown-model` repositoryが作成済みである
- [ ] `katana-ui-widget` repositoryは未作成であり、P2として後続作成が必要である

### Tasks

- [x] 2.1 `katana-ast-lint` のDoR/DoDを、共通AST lint gateとrepository分離後の統制として定義する
- [x] 2.2 `katana-markdown-model` のDoR/DoDを、文書モデル、metadata schema、位置解決、parser境界として定義する
- [x] 2.3 `katana-ui-widget` のDoR/DoDを、KMM metadata/display DTO確定後のFloem共通UI部品として定義する
- [x] 2.4 `katana-document-viewer` のDoR/DoDを、KMM public DTOとKUW境界待ちとして定義する
- [x] 2.5 `katana-language-editor` のDoR/DoDを、KMM metadata schemaとtarget resolution API待ちとして定義する
- [x] 2.6 `katana-diagram-renderer` のDoR/DoDを外部描画専用 crate として定義し、`katana-canvas-forge` のDoR/DoDを既存export維持・KDV移譲後のexport削除条件として定義する
- [x] 2.7 `katana` 統合のDoR/DoDを、各repo public contractだけを使う統合として定義する

### Definition of Done

- [x] 各repoが、自分の責務ではないdocument model、metadata schema、UI widget、export責務を持たないことが明文化されている
- [x] KMM未完了時に進めてよい作業が、repoごとに限定されている

## 3. Metadata and Editor Contract

### Definition of Ready

- [x] KMMがmetadata schemaと位置解決を持つ方針が合意済みである

### Tasks

- [x] 3.1 metadataをMarkdown本文に埋め込まない方針を固定する
- [x] 3.2 `README.md.metadata.json` を標準命名として定義する
- [x] 3.3 metadata targetにfile path、node id、byte range、line-column、fingerprint、前後文脈を含める
- [x] 3.4 editor保存直後にmetadataを更新する責務を `katana-language-editor` へ割り当てる
- [x] 3.5 unresolved metadataを削除せず保持する方針を固定する
- [ ] 3.6 conflicted metadata targetをKMM public DTOとして扱い、editorやviewerが独自状態で代替しない

### Definition of Done

- [ ] metadata contractがKMM、editor、viewer、exportの各OpenSpecへ展開できる
- [ ] PDFページングとLLM注釈の両方がmetadataで扱える

## 4. Repository OpenSpec Expansion

### Definition of Ready

- [x] 各repositoryの作業境界が確定している

### Tasks

- [x] 4.1 `katana-ast-lint` に `bootstrap-shared-ast-lint` を作る
- [x] 4.2 `katana` に `extract-katana-ast-lint` を作る
- [x] 4.3 `katana-markdown-model` に `bootstrap-kme-document-model` を作る
- [x] 4.4 `katana-language-editor` に `sync-kme-metadata-on-save` を作る
- [x] 4.5 `katana-document-viewer` に `adopt-kme-preview-model` を作り、既存repo `katana-document-preview` は改名前提として扱う
- [x] 4.6 `katana-canvas-forge` の `v0-1-3-export-css-debug` をKDV移譲までの既存export保守範囲として維持する
- [x] 4.7 `katana` に `adopt-kme-in-katana` を作る
- [x] 4.8 `katana` に `extract-katana-ui-widget` を作る
- [x] 4.9 各OpenSpecへKMM完了前のpending条件と禁止事項を再反映する

### Definition of Done

- [x] 各OpenSpecにDoR/DoDがある
- [x] KMM本体実装と品質ゲート構築が混ざっていない

## 5. Final Verification

- [x] 5.1 `scripts/openspec validate "establish-kme-markdown-platform" --strict` を実行する
- [x] 5.2 関連repositoryのOpenSpec validation結果を記録する
- [x] 5.3 KMM repository作成とOpenSpec展開の差分を確認する
- [x] 5.4 `katana-ast-lint` repository作成とOpenSpec展開の差分を確認する
- [x] 5.5 KCFの新規viewer/export計画がKDVへ移譲され、KCF側では外部描画へ縮小することを確認する
