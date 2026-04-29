## 0. Definition of Ready (DoR)

- [ ] この change は特定バージョンに割り当てない調査バックログとして扱う
- [ ] proposal.md / design.md / spec.md が合意されている
- [ ] v0.22.7 のガントチャート修正とは分離済みである

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-10-mermaid-rendering-compatibility-investigation`（チェンジディレクトリ名）
- **作業ブランチ**: `feature/mermaid-compat-investigation-task-x`（xはタスク番号）

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. task.mdの更新

### 実装内容

openspec-tasks-template を利用して tasks.md を template 形式に統一する。
この change の第一タスク。

### Definition of Done (DoD)

- [x] 1.1 tasks.md の構造を DoR → Branch Rule → Task 群 → User Review → Final Verification に整理する
- [x] 1.2 各 Task Group に DoR（Task 1除く）/ DoD（/openspec-delivery 含む）を追加する
- [ ] Execute `/openspec-delivery` workflow to run the comprehensive delivery routine.

---

## 2. mmdc baseline 調査

### Definition of Ready (DoR)

- [ ] Task 1 の完全デリバリーが完了している
- [ ] Base ブランチが最新状態である

### 実装内容

旧 KatanA が使用していた `mmdc` 呼び出し方式、出力形式、出力特性を確認する。
Mermaid.js 経路との比較基準を確立するための調査。

### 対象ファイル / リソース

- 旧 KatanA ソースコード内 mmdc 呼び出し履歴
- mmdc コマンドラインドキュメント（バージョン特定）
- Mermaid CLI / Puppeteer / Mermaid.js 各バージョンの互換性記録

### Definition of Done (DoD)

- [ ] 2.1 旧 KatanA が `mmdc` に渡していた引数（viewport, bg color, scale, theme）を確認する
- [ ] 2.2 `mmdc` の既定 output size / viewport が固定値か入力依存かを実測する
- [ ] 2.3 ガントチャートの赤い「今日」線が `mmdc` 出力でどのように扱われていたかを記録する
- [ ] 2.4 mmdc / Puppeteer / Mermaid.js のバージョン依存性がどの程度影響するかを文書化する
- [ ] Execute `/openspec-delivery` workflow to run the comprehensive delivery routine.

---

## 3. Mermaid.js renderer 調査

### Definition of Ready (DoR)

- [ ] Task 2 の完全デリバリーが完了している
- [ ] Base ブランチが最新状態である

### 実装内容

現行 Mermaid.js 描画経路の特性を確認する。viewport, container 幅, SVG 取得方法, キャッシュキー設計などを整理する。

### 対象ファイル

- `crates/katana-core/src/markdown/mermaid_renderer/`
- `crates/katana-core/src/system/renderer/mermaid_renderer_impl.rs` など Mermaid.js インテグレーション部分

### Definition of Done (DoD)

- [ ] 3.1 ヘッドレスブラウザの viewport、render 対象 container の幅、SVG `viewBox` / `getBBox()` を確認する
- [ ] 3.2 キャッシュキーに含めるべき描画条件（viewport, container width, theme, custom CSS）を確認する
- [ ] 3.3 図形ごとに、親要素の幅を拾うか、内容幅で描画されるかを確認する
- [ ] 3.4 Mermaid 初期化設定で解決できる差分（テーマ色, フォント, padding）と SVG 後処理が必要な差分を分離する
- [ ] Execute `/openspec-delivery` workflow to run the comprehensive delivery routine.

---

## 4. Fixture 作成と証跡生成

### Definition of Ready (DoR)

- [ ] Task 3 の完全デリバリーが完了している
- [ ] Base ブランチが最新状態である

### 実装内容

Mermaid 図形種類ごとの fixture を作成し、`mmdc` 出力と Mermaid.js 出力の比較証跡を生成する。

### 対象図形

flowchart / sequence / class / state / entity relationship / gantt / pie / journey / mindmap / timeline

### 対象ファイル

- `crates/katana-core/tests/markdown_mermaid.rs`（テスト fixture）
- `scripts/screenshot/`（スクリーンショット生成シナリオ）
- `tmp/mermaid-compat-evidence/`（生成済み証跡、`.gitignore` 対象）

### Definition of Done (DoD)

- [ ] 4.1 各図形ごとの fixture を用意する（labels, edges, theme-sensitive elements を含む）
- [ ] 4.2 各 fixture について `mmdc` baseline 出力を取得する
- [ ] 4.3 現行 Mermaid.js で同じ fixture を render し、PNG 出力を得る
- [ ] 4.4 出力先を `.gitignore` 対象に置き、fixture と生成手順だけを git 管理対象にする
- [ ] 4.5 `scripts/screenshot` 環境で確認できるものは、手動操作不要なシナリオにする
- [ ] Execute `/openspec-delivery` workflow to run the comprehensive delivery routine.

---

## 5. 差分分類と後続計画

### Definition of Ready (DoR)

- [ ] Task 4 の完全デリバリーが完了している
- [ ] Base ブランチが最新状態である

### 実装内容

Task 4 で得られた証跡から、差分パターンを分類し、どれを即時修正し、どれを後続 versioned change 化するかを判定する。

### 差分分類軸

- **Layout**: 図形全体の位置・配置
- **Size**: 出力サイズ、内容高さ・幅
- **Theme**: 背景色、線色、文字色
- **Typography**: フォント、文字サイズ
- **Marker**: 特殊マーカー（gantt の今日線、alert アイコンなど）
- **Interaction**: マウスホバーアニメーション（web 環境）
- **Error Handling**: 不正記法のエラー表示
- **Cache Behavior**: キャッシュ有効性

### 対象ファイル

- 本 change 配下に `FINDINGS.md` / `COMPATIBILITY_MATRIX.md` を新規作成

### Definition of Done (DoD)

- [ ] 5.1 差分を layout / size / theme / typography / marker / interaction / error handling / cache behavior に分類する
- [ ] 5.2 各差分について、「即時修正」「versioned change 化」「許容差分（文書化）」のいずれかを判定する
- [ ] 5.3 SVG 後処理が必要な候補は、対象図形と対象 SVG 要素を明確に限定する
- [ ] 5.4 後続 versioned change を作成する場合は、本 change の FINDINGS.md を参照元として記録する
- [ ] 5.5 COMPATIBILITY_MATRIX.md にまとめ、他の開発者が参照しやすくする
- [ ] Execute `/openspec-delivery` workflow to run the comprehensive delivery routine.

---

## 6. 調査結果完成と文書化

### Definition of Ready (DoR)

- [ ] Task 5 の完全デリバリーが完了している
- [ ] Base ブランチが最新状態である

### 実装内容

調査結果を整理し、OpenSpec schema を確認した上で、後続対応への入力バックログとして確定する。

### 対象ファイル

- `openspec/changes/v0-22-10-mermaid-rendering-compatibility-investigation/FINDINGS.md`
- `openspec/changes/v0-22-10-mermaid-rendering-compatibility-investigation/COMPATIBILITY_MATRIX.md`
- `design.md` への補足（必要に応じて）

### Definition of Done (DoD)

- [ ] 6.1 調査結果を FINDINGS.md / COMPATIBILITY_MATRIX.md に整理し、後続 versioned change が参照しやすくする
- [ ] 6.2 既存の proposal.md / design.md / spec.md と矛盾がないことを確認する
- [ ] 6.3 「この change は実装完了ではなく調査バックログ入力」という前提を明記する
- [ ] Execute `/openspec-delivery` workflow to run the comprehensive delivery routine.

---

## 7. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 7.1 ユーザーへ調査完了を報告し、FINDINGS.md / COMPATIBILITY_MATRIX.md を提示する
- [ ] 7.2 ユーザーから受けたフィードバック（修正優先度、許容差分など）を本ドキュメントに追記する
- [ ] 7.3 フィードバック内容を後続 versioned change へ移送する（必要に応じて）

---

## 8. Final Verification & Archive

### Definition of Ready (DoR)

- [ ] Task 7（User Review）が完了している
- [ ] 全調査結果が FINDINGS.md / COMPATIBILITY_MATRIX.md に記録済み

### 注意: 非 Versioned Change のため簡略版

本 change はバージョン非割り当て調査バックログのため、以下を実施しない:

- リリースブランチ (`release/vX.Y.Z`) 作成なし
- `make release VERSION=X.Y.Z` なし
- GitHub Release なし

代わり、後続 versioned change での参照可能な状態で `/openspec-archive` でアーカイブする。

### Definition of Done (DoD)

- [ ] 8.1 `/self-review` スキルを実行し、セルフレビューを完了する
- [ ] 8.2 Markdown / JSON schema の形式チェックを行う
- [ ] 8.3 `git push` でpre-pushフックを正式ゲートとして通す（`--no-verify` 原則禁止）
- [ ] 8.4 `master` へのPRを作成する（feature/mermaid-compat-investigation-taskX → master）
- [ ] 8.5 CI確認（Lint / Type Check）
- [ ] 8.6 `master` へマージ
- [ ] 8.7 `/openspec-archive` で本 change をアーカイブする
- [ ] 8.8 後続 versioned change が FINDINGS.md / COMPATIBILITY_MATRIX.md を参照できることを確認する
