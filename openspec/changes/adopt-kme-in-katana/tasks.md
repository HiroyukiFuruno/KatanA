# Tasks: adopt-kme-in-katana

## 1. Integration Readiness

### Definition of Ready

- [ ] `establish-kme-markdown-platform` が完了している
- [ ] P0 `katana-ast-lint` の共通品質ゲート方針が利用可能である
- [ ] KMM、editor、viewer、export、widgetのOpenSpecが作成済みである

### Tasks

- [ ] 1.1 KatanAが参照するKMM public DTOを整理する
- [ ] 1.2 KatanA stateに持ってよいmetadataと持ってはいけない実装型を明文化する
- [ ] 1.3 description list fixtureを追加する
- [ ] 1.4 KatanA統合で使う共通AST lint入口を固定する

### Definition of Done

- [ ] KatanA本体がparser/vendor internalsを保存しない方針が検証できる
- [ ] metadata schemaはKMMのpublic contractを使い、KatanA本体で独自定義しない
- [ ] editor-viewer同期制御はKatanAが持ち、KatanAがviewerまたはeditorへ命令する
- [ ] 統合前の品質ゲートがrepositoryごとの独自lintに戻っていない

## 2. Viewer and Editor Connection

### Definition of Ready

- [ ] KDVのKMM viewer model contractが利用可能である
- [ ] kleのmetadata sync contractが利用可能である

### Tasks

- [ ] 2.1 viewerをKMM model由来のrender metadataへ接続する
- [ ] 2.2 editor保存時metadata同期をKatanA save flowへ接続する
- [ ] 2.3 unresolved metadataをKatanA UIで表示する入口を接続する
- [ ] 2.4 KMM node id、source range、line-column、fingerprintを使うeditor-viewer同期coordinatorをKatanA側に設計する

### Definition of Done

- [ ] `sample.md` のviewerとmetadata targetが同じsource rangeを参照している
- [ ] 保存後にmetadata targetが更新またはunresolvedとして保持される

## 3. Export and Widget Connection

### Definition of Ready

- [ ] KDVのviewer/export pipeline方針が利用可能である
- [ ] `katana-ui-widget` の分離方針が利用可能である
- [ ] P0/P1/P2の依存順序が維持されている

### Tasks

- [ ] 3.1 KDV exportへKMM modelを渡す経路を設計する
- [ ] 3.2 metadata由来のPDFページングをKDV exportへ渡す経路を設計する
- [ ] 3.3 metadata表示やcopy/editの共通UI部品をwidget分離対象へ移す

### Definition of Done

- [ ] viewerとexportのMarkdown解釈が同じKMM fixtureで検証される
- [ ] 新しい共通UI部品がKatanA本体へ固定化されていない

## 4. Final Verification

- [ ] 4.1 `npx -y @fission-ai/openspec validate "adopt-kme-in-katana" --strict` を実行する
- [ ] 4.2 KMM、KDV、KLE、KCF、widgetの統合順序に循環依存がないことを確認する
- [ ] 4.3 P0 `katana-ast-lint` をKatanA統合の品質ゲートとして参照できることを確認する
