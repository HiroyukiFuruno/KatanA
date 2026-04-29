# Tasks: v0.22.10 Mermaid Rendering Compatibility Investigation

## 0. 準備完了条件（Definition of Ready）

- [x] `proposal.md` / `design.md` / `spec.md` が揃っている
- [x] 本 change は `v0.22.10` のリリース対象として扱う
- [x] v0.22.7 のガントチャート即時修正とは分離済みである
- [x] `mmdc` は実行時依存として戻さず、出力条件の参照元として扱う
- [x] 通常の diagram preview と HTML export は OS にインストールされた Chrome / Chromium アプリへ依存しない
- [x] まず Rust 管理 JS で公式 Mermaid.js / Drawio.js を動かせるか試し、不採用なら高速な headless browser / WebView / Chromium から単一の採用経路を選ぶ
- [x] Mermaid.js / Drawio.js を使わない Rust-native renderer へは切り替えない
- [x] Rust 製または Rust 管理で高速な headless browser が用途を満たすなら、preview / export 共通の採用候補に含める
- [x] 実行時の退避経路（fallback）はロジックを複雑化させるため採用しない

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `release/v0.22.10`
- **作業ブランチ**: `feature/v0.22.10-task-x`（xはタスク番号）

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. `mmdc` 由来の出力契約を抽出する

### 実施内容

旧 `mmdc` 経路が暗黙に担っていた viewport、背景、テーマ、拡大率、PNG 出力条件を確認し、KatanA renderer に移植するべき契約として整理する。

### 対象ファイル / リソース

- 旧 KatanA の `mmdc` 呼び出し履歴
- `mmdc -h` / Mermaid CLI 公式ドキュメント
- `openspec/changes/v0-22-10-mermaid-rendering-compatibility-investigation/FINDINGS.md`

### 完了条件（Definition of Done）

- [x] 1.1 旧 KatanA が `mmdc` に渡していた引数（backgroundColor, theme, input, output, quiet）を確認する
- [x] 1.2 `mmdc` の既定 width / height / scale / backgroundColor を確認する
- [x] 1.3 `mmdc` 依存として戻さない条件を `FINDINGS.md` に明記する
- [x] 1.4 KatanA renderer に取り込むべき出力契約を policy として整理する
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Browser runtime 方針を決める

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

現在の Mermaid / Draw.io renderer と HTML export 経路は headless browser（画面を出さないブラウザ）を使うが、実体として `/Applications/Google Chrome.app` など OS 上のブラウザアプリを起動している箇所がある。通常の preview / HTML export ではこの依存を許容しない。

まず Rust 管理 JS（Rust 側が所有する JavaScript 実行環境）で公式 Mermaid.js / Drawio.js を動かせるか試す。DOM / SVG / layout API の不足で表示互換性や速度を満たせない場合、Mermaid と Draw.io は KatanA 管理下の高速な headless browser（画面なしブラウザ）/ WebView（アプリ内ブラウザ部品）/ Chromium（Chrome 系ブラウザエンジン）から単一の採用経路を選ぶ。Rust 製または Rust 管理で高速な headless browser が HTML export まで満たせるなら、置き換え候補に含める。実行時の退避経路（fallback）は持たない。

重要: Task 2 の判定軸は「図形を表示できるか」だけではない。現行実装も表示はできているため、評価トピックは次の2つに絞る。

- OS の環境に依存しない
- 高速かつ正確に表示する

現状は「OS 依存、遅い」。目指す状態は「OS 非依存、高速かつ正確」である。

Rust 管理 JS の spike は、現行の表示品質を保ったまま、初回描画速度、連続描画速度、所有境界、配布安定性、CI 安定性、`mmdc` 由来の出力 policy 反映を改善できるかで判定する。Rust 管理 JS が表示だけできても、余白、サイズ、テーマ、特殊マーカー、HTML export 埋め込み、キャッシュ条件が崩れる場合は採用しない。

### 対象ファイル

- `crates/katana-core/src/markdown/mermaid_renderer/`
- `crates/katana-core/src/markdown/drawio_renderer/`
- `crates/katana-core/src/markdown/export/`
- `crates/katana-core/tests/markdown_mermaid.rs`
- `crates/katana-core/tests/export_regression.rs`
- `crates/katana-ui/src/app/export.rs`
- `crates/katana-ui/src/app/export_poll.rs`
- 必要に応じて `crates/katana-ui/tests/integration/preview_pane/diagrams.rs`

### 完了条件（Definition of Done）

- [ ] 2.1 現行実装が OS の Chrome / Chromium アプリを起動している箇所を Mermaid / Draw.io / HTML export / PDF / PNG / JPEG export ごとに棚卸しする
- [ ] 2.2 Rust 管理 JS で公式 Mermaid.js / Drawio.js を動かす spike を先に行う。候補は `rquickjs`、`boa_engine` / `boa_runtime`、`deno_core` を比較し、Mermaid 専用 Rust renderer は OpenSpec 条件と衝突する代替案として別枠にする
  - [x] 2.2.1 Rust 管理 V8 で `mermaid.min.js` を読み込み、`mermaid.render(...)` が `ReferenceError: document is not defined` で止まることを確認する
  - [x] 2.2.2 Rust 管理 V8 に最小 DOM / SVG shim を足し、flowchart / sequence / class / state / gantt / pie の SVG 生成と label 保持に成功することを確認する
  - [x] 2.2.3 Rust 管理 JS を本体 Mermaid renderer の既定経路へ接続し、`make run-release` から試せる状態にする
  - [x] 2.2.4 Mermaid 描画ごとの V8 isolate を worker 上で独立実行し、全体を塞ぐ直列 lock を外す
  - [x] 2.2.5 `assets/fixtures/sample_mermaid_all.md` に Mermaid 図形種別の確認用 fixture を追加する
  - [x] 2.2.6 全パターン fixture を実行評価し、26 block すべてが SVG 生成と rasterize まで通ることを確認する
  - [x] 2.2.7 不足していた DOM / SVG / layout API を追加し、異常に大きい SVG が GPU texture 上限で preview を落とさない安全弁を入れる
  - [x] 2.2.8 ガントチャートの期間外 today marker と SVG 最大幅 policy を Rust 管理 JS 経路へ移植する
  - [x] 2.2.9 `sample_mermaid_all.md` は維持しつつ、26 種別を個別 Markdown fixture へ分解する
  - [x] 2.2.10 `scripts/screenshot` に26個の個別 Markdown を順に開いてスクリーンショットを取るシナリオを追加する
  - [x] 2.2.11 26種別の描画チェックシートを作り、サイズ・余白・ラベル・配色・欠落を1件ずつ記録して修正する
  - [x] 2.2.12 公式 Mermaid.js を実ブラウザーで描画した参照画像を26個生成し、各個別 Markdown fixture へ比較用画像として埋め込む
  - [x] 2.2.13 `make mermaid-diagram-update` で、Mermaid.js 更新後も公式参照画像と Markdown 内の画像参照を再生成できるようにする
  - [ ] 2.2.14 P1: 公式 Mermaid.js の実ブラウザー描画と KatanA 描画を比較する評価証跡を作成し、差分を種別ごとに記録する
  - [ ] 2.2.15 P1: 公式描画との差分から、今回補正するものと後続へ送るものを分け、今回対象の補正を実装する
  - [ ] 2.2.16 P1: 公式比較で見つかった表示差分を、サイズ / 余白 / 配色 / SVG 互換 / エラー表示に分類し、v0.22.10 で補正する対象を実装する
- [ ] 2.3 Rust 管理 JS で DOM / SVG / layout API の不足、表示崩れ、速度劣化がないかを確認する。判定は「表示できるか」ではなく、「OS の環境に依存しない」「高速かつ正確に表示する」を満たすかで行う
  - [x] 2.3.1 JavaScript engine 単体では公式 Mermaid.js の描画条件を満たせず、`document` / SVG DOM / layout API の所有可否が主論点になることを記録する
  - [x] 2.3.2 最小 DOM / SVG shim の `getBBox()` / text measurement / selector / DOMPurify 周辺 API が、表示品質として許容できる精度か確認する
  - [x] 2.3.3 preview cache key に Rust 管理 JS SVG profile を含め、旧 PNG キャッシュと混在させない
- [ ] 2.4 Rust 製または Rust 管理で高速な headless browser が preview / HTML export の用途を満たすか確認する
- [ ] 2.5 Rust 管理 JS が不採用の場合、高速な headless browser / WebView / Chromium を表示互換性、速度、配布サイズ、platform 差分、sandbox、CI 安定性で比較し、単一の採用経路を決める
- [x] 2.6 `headless_chrome` 依存を削除し、通常 preview / HTML export / PDF / PNG / JPEG export から OS Chrome アプリを起動する経路を外す
- [ ] 2.7 採用した runtime と、不採用候補を退避経路（fallback）として残さない理由を `design.md` に記録する
- [ ] 2.8 HTML から PDF / PNG / JPEG へ変換する export runtime も、同じ採用経路へ寄せられるか確認する
  - [x] 2.8.1 `headless_chrome` で PDF / PNG / JPEG を出す経路は削除し、管理下 Chromium runtime 未接続の明示エラーへ変更する
- [ ] 2.9 v0.22.10 で移行しきれない export 経路や特殊ケースは、採用経路へ混ぜず後続 versioned change として扱う
- [ ] 2.10 runtime 採用判断の前に、spike 結果と比較表をユーザーへ提示して確認を得る
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. 採用した単一 Mermaid renderer に出力 policy を実装する

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

KatanA 管理下で採用した単一 Mermaid renderer に、`mmdc` 由来のきれいな出力条件を移植する。主対象は viewport / container 幅 / SVG 計測 / PNG capture / HTML export 埋め込み / 余白 / 最大幅 / 背景 / テーマである。

### 対象ファイル

- `crates/katana-core/src/markdown/mermaid_renderer/`
- `crates/katana-core/src/markdown/export/`
- `crates/katana-core/tests/markdown_mermaid.rs`
- `crates/katana-core/tests/export_regression.rs`

### 完了条件（Definition of Done）

- [ ] 3.1 採用した単一 Mermaid renderer の render width、capture width、padding、scale を明示的な policy として扱う
- [ ] 3.2 SVG `getBBox()` / `viewBox` / screenshot 対象の扱いが、過度な横長化や余白過多を生まないことを確認する
- [ ] 3.3 background / transparent background / theme variables が PNG 出力に反映されることを確認する
- [ ] 3.4 HTML export に埋め込まれる Mermaid / Draw.io 出力も同じ policy で生成されることを確認する
- [ ] 3.5 gantt の今日線など、出力サイズを壊す特殊マーカーの扱いを限定的な後処理として維持または改善する
- [ ] 3.6 出力 policy を変えた場合は Mermaid cache key の version を更新する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Fixture と回帰テストを追加する

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

代表的な Mermaid 図形で、KatanA renderer の出力が崩れていないことを確認できる fixture と回帰テストを追加する。

### 対象図形

flowchart / sequence / class / state / entity relationship / gantt / pie / journey / mindmap / timeline

### 対象ファイル

- `crates/katana-core/tests/markdown_mermaid.rs`
- `scripts/screenshot/examples/`
- `tmp/mermaid-compat-evidence/`（生成済み証跡、`.gitignore` 対象）
- `openspec/changes/v0-22-10-mermaid-rendering-compatibility-investigation/COMPATIBILITY_MATRIX.md`

### 完了条件（Definition of Done）

- [ ] 4.1 各図形ごとの fixture を用意する（labels, edges, theme-sensitive elements を含む）
- [ ] 4.2 採用した単一 KatanA renderer で PNG または renderer-neutral output の生成に成功することを確認する
- [ ] 4.3 画像サイズが極端に横長・縦長・余白過多にならない最小回帰テストを追加する
- [ ] 4.4 `scripts/screenshot` で確認できるものは、手動操作不要なシナリオにする
- [ ] 4.5 比較結果を `COMPATIBILITY_MATRIX.md` に記録する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. 差分分類と後続計画

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

fixture と証跡から、今回 KatanA renderer に取り込む差分、後続 versioned change に送る差分、許容差分を分類する。

### 対象ファイル

- `openspec/changes/v0-22-10-mermaid-rendering-compatibility-investigation/FINDINGS.md`
- `openspec/changes/v0-22-10-mermaid-rendering-compatibility-investigation/COMPATIBILITY_MATRIX.md`
- `openspec/changes/v0-22-10-mermaid-rendering-compatibility-investigation/design.md`（必要な補足がある場合のみ）

### 完了条件（Definition of Done）

- [ ] 5.1 差分を layout / size / theme / typography / marker / interaction / error handling / cache behavior に分類する
- [ ] 5.2 各差分について「今回実装」「後続 versioned change 化」「許容差分（文書化）」のいずれかを判定する
- [ ] 5.3 SVG 後処理が必要な候補は、対象図形と対象 SVG 要素を明確に限定する
- [ ] 5.4 後続 versioned change を作成する場合は、本 change の `FINDINGS.md` を参照元として記録する
- [ ] 5.5 既存の `proposal.md` / `design.md` / `spec.md` と矛盾がないことを確認する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 6.1 ユーザーへ実装完了の報告および比較証跡を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする
- [ ] 6.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）
- [ ] 6.3 フィードバック: 現状は「OS 依存、遅い」。通常 preview が OS の Chrome / Chromium アプリへ依存する状態は NG。Mermaid / Draw.io は KatanA 管理下の単一経路へ移し、高速かつ正確な表示を判断基準にする
- [ ] 6.4 フィードバック: HTML 生成時の SVG 読み込み表示文言で `{{lucide:hourglass}}` のようなテンプレート文字列が展開されずに見えている。Rust 管理 JS runtime の評価後、別バグ対応タスクとして修正する
- [/] 6.5 フィードバック: Rust 管理 JS runtime の不足 API を追加して対応できないか。text content、append、layout size、`getComputedStyle()` 初期値を追加し、Mermaid 全パターン fixture を 26/26 に更新する
- [/] 6.6 フィードバック: `sample_mermaid_all.md` は維持したまま、26個の個別 Markdown、スクリーンショットシナリオ、チェックシートを追加し、26個すべての正確な描画を目指す
- [/] 6.7 フィードバック: 26種別を人間が目視で正解判定し続けるのは難しいため、公式 Mermaid.js を実ブラウザーで描画した参照画像を基準に比較できるようにする
- [/] 6.8 P1 フィードバック: Mermaid preview 表示機構と検証機構は、HTML / PDF / PNG / JPEG export の境界も含めて `katana-renderer` として早期に別 repository 化する。KatanA は描画専門性を負わない。interface 汎用化、Mermaid.js version 固定、分離設計は v0.22.11 `v0-22-11-renderer-runtime-interface-and-versioning` へ移管し、v0.22.10 は表示最適化へ集中する
- [ ] 6.9 最優先フィードバック: 公式 Mermaid.js の実ブラウザー描画との比較評価を行い、今回補正できる差分は補正する
- [/] 6.10 フィードバック: Mermaid.js に渡せる値との互換性を保つ。`theme` / `themeVariables` / `securityLevel` / diagram-specific config を Mermaid.js config として扱い、KatanA 独自 policy を外側へ分離する interface 整理は v0.22.11 へ移管する。v0.22.10 では表示補正のための最小限の既存設定変更に留める

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 7. Final Verification & Release Work

### 準備完了条件（Definition of Ready）

- [ ] Task 6（User Review）が完了している
- [ ] `FINDINGS.md` / `COMPATIBILITY_MATRIX.md` に調査結果と後続判断が記録済みである
  - [x] `RUST_MANAGED_JS_SPIKE.md` に V8 直接実行の結果を記録済みである

### 完了条件（Definition of Done）

- [ ] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 7.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 7.3 `./scripts/openspec validate v0-22-10-mermaid-rendering-compatibility-investigation --strict` を実行し、OpenSpec の整合性を確認する
- [ ] 7.4 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `make check` / `make check-light` を二重実行しない
- [ ] 7.5 Create PR from `release/v0.22.10` targeting `master`
- [ ] 7.6 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL / Release Readiness) — blocking merge if any fail
- [ ] 7.7 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.8 Verify GitHub Release completion and archive this change using `/opsx-archive`
