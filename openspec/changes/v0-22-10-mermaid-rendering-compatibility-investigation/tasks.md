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
  - [x] 2.2.14 P1: 公式 Mermaid.js の実ブラウザー描画と KatanA 描画を比較する評価証跡を作成し、差分を種別ごとに記録する
  - [x] 2.2.15 P1: 公式描画との差分から、今回補正するものと後続へ送るものを分け、今回対象の補正を実装する
  - [x] 2.2.16 P1: 公式比較で見つかった表示差分を、サイズ / 余白 / 配色 / SVG 互換 / エラー表示に分類し、v0.22.10 で補正する対象を実装する
- [ ] 2.3 Rust 管理 JS で DOM / SVG / layout API の不足、表示崩れ、速度劣化がないかを確認する。判定は「表示できるか」ではなく、「OS の環境に依存しない」「高速かつ正確に表示する」を満たすかで行う
  - [x] 2.3.1 JavaScript engine 単体では公式 Mermaid.js の描画条件を満たせず、`document` / SVG DOM / layout API の所有可否が主論点になることを記録する
  - [x] 2.3.2 最小 DOM / SVG shim の `getBBox()` / text measurement / selector / DOMPurify 周辺 API が、表示品質として許容できる精度か確認する
  - [x] 2.3.3 preview cache key に Rust 管理 JS SVG profile を含め、旧 PNG キャッシュと混在させない
- [ ] 2.4 Rust 製または Rust 管理で高速な headless browser が preview / HTML export の用途を満たすか確認する
- [ ] 2.5 Rust 管理 JS が不採用の場合、高速な headless browser / WebView / Chromium を表示互換性、速度、配布サイズ、platform 差分、sandbox、CI 安定性で比較し、単一の採用経路を決める
- [x] 2.6 `headless_chrome` 依存を削除し、通常 preview / HTML export / PDF / PNG / JPEG export から OS Chrome アプリを起動する経路を外す
- [ ] 2.7 採用した runtime と、不採用候補を退避経路（fallback）として残さない理由を `design.md` に記録する
- [ ] 2.8 HTML 生成と PDF / PNG / JPEG export runtime を、v0.22.10 で OS Chrome / Chromium アプリへ依存しない経路へ修正する
  - [x] 2.8.1 `headless_chrome` で PDF / PNG / JPEG を出す経路は削除し、管理下 Chromium runtime 未接続の明示エラーへ変更する
  - [x] 2.8.2 PDF / PNG / JPEG export が Chromium 依存のまま残っている箇所を修正し、通常操作で OS Chrome / Chromium アプリを要求しないことを確認する
  - [x] 2.8.3 HTML 生成で Mermaid / Draw.io SVG の読み込み表示文言や template 文字列がそのまま出ないことを確認する
  - [ ] 2.8.4 `assets/fixtures/sample_mermaid_all.md` を HTML / PDF / PNG / JPEG export し、図形出力が欠落しないことを回帰確認する
- [ ] 2.9 v0.22.10 で移行しきれない Draw.io 経路や特殊ケースは、採用経路へ混ぜず後続 versioned change として扱う
- [ ] 2.10 runtime 採用判断の前に、spike 結果と比較表をユーザーへ提示して確認を得る
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Linux Homebrew cask 配布契約を追加する

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.
- [ ] Homebrew の Linux cask 対応状況を公式情報で確認し、`brew install --cask` が Linux binary cask を扱える前提を固定する。

### 実施内容

Linux 向け KatanA を、Formula だけでなく `brew install --cask` でも導入できる配布契約として整備する。

Homebrew 4.5 以降では一部の Linux cask がサポート対象になっているため、KatanA の Linux 向け cask は「GUI アプリを Linux binary asset から導入する」経路として扱う。ただし macOS 専用 cask と同じ見た目に寄せるだけではなく、Linux 実行ファイル、desktop entry、アイコン、uninstall / zap 相当、更新時の checksum、release automation の整合性を検証対象に含める。

既存の Linuxbrew Formula を単純に置き換えるのではなく、ユーザー向け導線を `brew install --cask katana-desktop` に寄せ、CLI や互換目的の Formula が残る場合は役割を明確に分離する。

### 対象ファイル / リソース

- `scripts/release/sync-external.sh`
- `scripts/release/update-homebrew.sh`
- `scripts/release/update-linuxbrew.sh`
- sibling repository: `/Users/hiroyuki_furuno/works/private/homebrew-katana`
- `platforms/linux/compose.yml`
- `platforms/linux/provision.sh`
- `platforms/README.md`
- Linux release asset: `KatanA-linux-x86_64.tar.gz`
- Homebrew cask / formula audit commands

### 完了条件（Definition of Done）

- [ ] 3.1 Homebrew の Linux cask 対応範囲を確認し、KatanA が対象にできる条件（Linux binary cask、architecture、prefix、非対応条件）を `design.md` または本タスク内に記録する
- [ ] 3.2 `homebrew-katana` の既存 Cask / Formula 構成を調査し、macOS cask、Linux formula、Linux cask の責務分離を決める
- [ ] 3.3 Linux cask の token、artifact URL、sha256、実行ファイル配置、desktop entry、icon 配置、uninstall / zap 相当の方針を固定する
- [ ] 3.4 `brew install --cask katana-desktop` が Linux で `KatanA-linux-x86_64.tar.gz` を取得し、GUI アプリとして起動できるように tap 側の cask を更新する
- [ ] 3.5 release automation が GitHub Release の Linux asset から Linux cask の sha256 と URL を更新できるようにする
- [ ] 3.6 `make linux-up` 環境で `brew tap`、`brew install --cask`、起動確認、`brew uninstall --cask` を実行できる手順を整備する
- [ ] 3.7 既存 Formula を残す場合は、ユーザー向け install docs と release notes で「GUI アプリは cask、CLI/互換用途は formula」のように役割を明確化する
- [ ] 3.8 Linux cask が不成立だった場合は Formula へ戻すのではなく、不成立理由と Homebrew 側制約を記録し、後続の配布方式（AppImage / deb / tar.gz / Flatpak など）へ分離する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. 採用した単一 Mermaid renderer に出力 policy を実装する

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

- [ ] 4.1 採用した単一 Mermaid renderer の render width、capture width、padding、scale を明示的な policy として扱う
- [ ] 4.2 SVG `getBBox()` / `viewBox` / screenshot 対象の扱いが、過度な横長化や余白過多を生まないことを確認する
- [ ] 4.3 background / transparent background / theme variables が PNG 出力に反映されることを確認する
- [ ] 4.4 HTML export に埋め込まれる Mermaid / Draw.io 出力も同じ policy で生成されることを確認する
- [ ] 4.5 gantt の今日線など、出力サイズを壊す特殊マーカーの扱いを限定的な後処理として維持または改善する
- [ ] 4.6 出力 policy を変えた場合は Mermaid cache key の version を更新する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Fixture と回帰テストを追加する

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

- [ ] 5.1 各図形ごとの fixture を用意する（labels, edges, theme-sensitive elements を含む）
- [ ] 5.2 採用した単一 KatanA renderer で PNG または renderer-neutral output の生成に成功することを確認する
- [ ] 5.3 画像サイズが極端に横長・縦長・余白過多にならない最小回帰テストを追加する
- [ ] 5.4 `scripts/screenshot` で確認できるものは、手動操作不要なシナリオにする
- [ ] 5.5 比較結果を `COMPATIBILITY_MATRIX.md` に記録する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. 差分分類と後続計画

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

- [ ] 6.1 差分を layout / size / theme / typography / marker / interaction / error handling / cache behavior に分類する
- [ ] 6.2 各差分について「今回実装」「後続 versioned change 化」「許容差分（文書化）」のいずれかを判定する
- [ ] 6.3 SVG 後処理が必要な候補は、対象図形と対象 SVG 要素を明確に限定する
- [ ] 6.4 後続 versioned change を作成する場合は、本 change の `FINDINGS.md` を参照元として記録する
- [ ] 6.5 既存の `proposal.md` / `design.md` / `spec.md` と矛盾がないことを確認する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 7. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 7.1 ユーザーへ実装完了の報告および比較証跡を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする
- [ ] 7.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）
- [ ] 7.3 フィードバック: 現状は「OS 依存、遅い」。通常 preview が OS の Chrome / Chromium アプリへ依存する状態は NG。Mermaid / Draw.io は KatanA 管理下の単一経路へ移し、高速かつ正確な表示を判断基準にする
- [/] 7.4 フィードバック: HTML 生成時の SVG 読み込み表示文言で `{{lucide:hourglass}}` のようなテンプレート文字列が展開されずに見えている。v0.22.10 の残課題として修正する
- [/] 7.5 フィードバック: Rust 管理 JS runtime の不足 API を追加して対応できないか。text content、append、layout size、`getComputedStyle()` 初期値を追加し、Mermaid 全パターン fixture を 26/26 に更新する
- [/] 7.6 フィードバック: `sample_mermaid_all.md` は維持したまま、26個の個別 Markdown、スクリーンショットシナリオ、チェックシートを追加し、26個すべての正確な描画を目指す
- [/] 7.7 フィードバック: 26種別を人間が目視で正解判定し続けるのは難しいため、公式 Mermaid.js を実ブラウザーで描画した参照画像を基準に比較できるようにする
- [/] 7.8 P1 フィードバック: Mermaid preview 表示機構と検証機構は、HTML / PDF / PNG / JPEG export の境界も含めて `katana-renderer` として早期に別 repository 化する。KatanA は描画専門性を負わない。interface 汎用化、Mermaid.js version 固定、分離設計は v0.22.11 `v0-22-11-renderer-runtime-interface-and-versioning` へ移管し、v0.22.10 は表示最適化へ集中する
- [/] 7.9 最優先フィードバック: 公式 Mermaid.js の実ブラウザー描画との比較評価を行い、今回補正できる差分は補正する
- [/] 7.10 フィードバック: Mermaid.js に渡せる値との互換性を保つ。`theme` / `themeVariables` / `securityLevel` / diagram-specific config を Mermaid.js config として扱い、KatanA 独自 policy を外側へ分離する interface 整理は v0.22.11 へ移管する。v0.22.10 では表示補正のための最小限の既存設定変更に留める
- [/] 7.11 フィードバック: `file:///private/var/folders/ql/4640yx8s22zg367pjjld7yc00000gn/T/Uhwpk_assets_fixtures_sample_mermaid_all.html` で確認した HTML 生成不正を v0.22.10 内で修正する
- [/] 7.12 フィードバック: PDF / PNG / JPEG export が Chromium 依存のまま残っているため、v0.22.10 内で修正する
- [/] 7.13 フィードバック: 26種別比較で、01/04 のラベル背景、05/06 の図形内文字、08 の凡例色、09 の `<<satisfies>>`、10 の branch label、24 の Venn 配色・位置を公式参照画像へ寄せる
- [ ] 7.14 フィードバック: 26種別比較で、05 ER の公式乖離、06 Journey の section label、07 Gantt の背景（background）と境界線、08 Pie の凡例見切れ、13 Timeline の文字改行を公式参照画像へ寄せる
- [ ] 7.15 フィードバック: 26種別比較で、14 Quadrant の点色、15 XY chart の単位位置、23 Ishikawa の文字切れ・head位置・矢印、24 Venn の円内配色、25 Treemap の `Cache` 見切れを公式参照画像へ寄せる
- [/] 7.16 フィードバック: Draw.io は全体的に描画がNG。Mermaid 向けの SVG 生成・補正と Draw.io 向けの SVG 生成・補正を分離し、共通後処理へ混ぜない
- [/] 7.17 フィードバック: Draw.io は自前描画をやめ、Mermaid.js方式と同じく公式 Draw.io JavaScript を Rust 管理の JavaScript 実行環境で動かす。自前描画への退避（fallback）は持たない。JavaScript 実行失敗時は preview 上で `not supported` とコードブロック表示に落とし、詳細はログへ出す
- [/] 7.18 フィードバック: 05 ER は table layout の背景（background）表現が不足し、`DIAGRAM` が object の上下左右中央になっている。07 Gantt は dark theme 時だけ背景（background）配色が不正。24 Venn は本家に比べて円内配色に乖離がある
- [/] 7.18 フィードバック: Draw.io 標準の圧縮済み `<mxfile><diagram>...</diagram></mxfile>` は、KatanA独自展開ではなく公式 `Editor.extractGraphModel` / `Editor.parseDiagramNode` / `Graph.decompress` 経路で `mxGraphModel` に変換する
- [/] 7.19 フィードバック: Draw.io の `shape=mxgraph.*` と `image=img/lib/...` は、公式リソースをV8内の仮想リソースローダーへ渡して解決する。公式JSや公式リソースの中身を書き換える退避実装は持たない
- [ ] 7.20 フィードバック: 今回同梱した公式リソースは `stencils/basic.xml` / `stencils/aws4.xml` / `shapes/mxAWS4.js` / `img/lib/ibm/miscellaneous/cognitive_services.svg` に限定している。追加の `mxgraph.*` ライブラリは、同じ仮想リソースローダーへ公式ファイルをそのまま登録する形で拡張する
- [/] 7.21 フィードバック: 05 ER の `DIAGRAM` が object 中央に見えず、table header も左右中央になっていない。ER の header / singleton node text を画像化（rasterize）後の見た目で上下左右中央へ補正する
- [/] 7.22 フィードバック: 24 Venn の円内配色改善が preview 上で確認できない。透明背景のまま preview 背景へ混ざる状態を避け、本家比較と同じ背景（background）込みで円内配色を確認できるようにする
- [/] 7.23 フィードバック: 03 Class の `Error` 下の余白が大きい。公式表示と同じ程度に、空の methods 区画を詰める
- [ ] 7.24 フィードバック: 04 State の線が汚い。状態遷移図（state diagram）の線・枠・矢印を公式表示と同じ滑らかさへ寄せる
- [/] 7.25 フィードバック: 空の Mermaid ブロック（中身が空の ` ```mermaid ` / `~~~mermaid`）で `UnknownDiagramError` が出る。renderer へ渡さず、Markdown として扱う
- [/] 7.26 フィードバック: ER diagram 全体で属性行の文字が罫線に重なり、ヘッダー・属性・単一オブジェクトの左右中央配置が崩れる。05 固有ではなく ER 全体の行高と列配置として補正する
- [/] 7.27 フィードバック: ZenUML は現在の `mermaid.min.js` 単体では図種別検出できず `UnknownDiagramError` になる。`zenuml` で始まる Mermaid ブロックは renderer へ渡さず、通常のコードブロックとして表示する
- [/] 7.28 劣後フィードバック: `@mermaid-js/mermaid-zenuml` は追加図種別として取り込める可能性があるが、KatanA の `mermaid.min.js` 単体ロードではなく、外部図種別の登録（`registerExternalDiagrams`）とJS version固定を含む設計が必要。repository 分離後の `katana-renderer` 側で扱う
- [/] 7.29 フィードバック: 今回追加した KatanA 所有の JS / TS は Biome で format / lint を検査し、JSON / JSONC は format のみ対象にする。pre-commit の先頭で Rust / JS / TS / JSON / JSONC の format を必ず実行する。Draw.io 公式リソース配下の JS は、公式ファイルをそのまま保持するため対象外にする。JS / TS は分割前提で、1ファイル200行、1関数30行、認知的複雑度（noExcessiveCognitiveComplexity）1を上限にする
- [/] 7.30 フィードバック: Mermaid が `tag.match is not a function` で大量に `not supported` へ落ち、Draw.io が `Not a diagram file` / `invalid distance too far back` / `Cannot read properties of undefined (reading 'length')` を出す。Mermaid は計測用 JS の関数名衝突を解消し、Draw.io は公式 viewer 入力の XML 宣言除去、OS依存の日時整形回避、DOM text node の `nodeValue` 互換を追加する
- [/] 7.31 フィードバック: `assets/feature/katana-architecture.md` のように `<mxfile><diagram><mxGraphModel>...</mxGraphModel></diagram></mxfile>` 形式で保存された非圧縮 Draw.io が `invalid distance too far back` になり、以前表示できていた図も `not supported` へ落ちる。公式 viewer へ渡す前に非圧縮 `<mxGraphModel>` を抽出し、圧縮済み `<diagram>` はそのまま扱う
- [/] 7.32 フィードバック: Draw.io でも Mermaid と同様に配色と object 内文字の欠落が発生している。公式 Draw.io.js の出力を維持しつつ、KatanA のSVG表示で扱える文字（text）と背景（background）を生成する
- [/] 7.33 フィードバック: `assets/fixtures/mermaid.md` の stateDiagram / stateDiagram-v2 で `Still` などの状態名が左右中央に見えない。状態ノード内の文字を公式表示へ寄せる
- [/] 7.34 フィードバック: `assets/fixtures/mermaid.md` の mindmap が極端に小さく、表示範囲と配置が公式表示から大きく崩れている
- [/] 7.35 フィードバック: `assets/fixtures/mermaid.md` の block-beta が極端に小さく、`DB` や矢印、block のサイズと配置が公式表示から崩れている
- [/] 7.36 フィードバック: `assets/fixtures/mermaid.md` の ishikawa-beta で `Blurry Photo` が右端オブジェクトからはみ出している
- [/] 7.37 フィードバック: `assets/fixtures/mermaid.md` の kanban でカード内テキストが枠からはみ出し、列高さとカード余白が公式表示から崩れている
- [/] 7.38 フィードバック: `assets/fixtures/mermaid.md` の venn-beta で集合と重なり領域の配色が公式表示と異なる
- [/] 7.39 フィードバック: `assets/fixtures/mermaid.md` の wardley-beta は背景（BG）がないため、黒い線や文字が preview 背景に沈んで見える
- [/] 7.40 フィードバック: `assets/fixtures/mermaid.md` の xychart-beta で `Revenue (in $)` の縦軸ラベルが目盛りへ重なっている
- [/] 7.41 フィードバック: Draw.io も Mermaid と同様に、公式 Draw.io JavaScript の実ブラウザー描画、KatanA 描画、左右比較画像を更新できる検証機構を用意する。まず `assets/fixtures/drawio/basic` を対象にする
- [/] 7.42 フィードバック: Draw.io の `Graph.getSvg()` 中に `katanaDetachChild` が `childNodes` を持たない親ノードで落ちる。KatanA DOM の子ノード移動を実ブラウザーの動作へ寄せる
- [/] 7.48 フィードバック: `assets/fixtures/drawio/basic` で、ページ全体の余白・巨大な白ラベル背景・暗色テーマ配色のズレが残っている。公式 Draw.io.js の SVG を維持しつつ、KatanA 側で基本図形が比較可能なサイズと暗色配色になるよう補正する
- [/] 7.54 フィードバック: `assets/fixtures/drawio/official/templates/aws` を Draw.io の次の評価対象にし、Mermaid と同じ形式で公式画像 / KatanA画像のスコア評価（scores.json と採点表）を出す。v0.22.10 の KatanA では手動比較ツールとして扱い、保存時チェック（pre-commit）や CI/CD への採点ゲート組み込みは `katana-renderer` 分離後に移す
- [/] 7.43 再フィードバック: `assets/fixtures/mermaid.md` の 05 Entity Relationship Diagram で、まだ左右中央に見えない文字がある
- [/] 7.44 再フィードバック: `assets/fixtures/mermaid.md` の 14 ishikawa-beta で `Blurry Photo` が右端オブジェクトからはみ出している。文字列長に応じてオブジェクト横幅を動的に広げ、文字を常に枠内へ収める
- [/] 7.49 再フィードバック: `assets/fixtures/sample_mermaid_all.md` の 19 Kanban で、短い1行カードまで縦に伸び、公式 Mermaid.js 描画よりカードと列が高くなるデグレードがある
- [/] 7.50 再フィードバック: `assets/fixtures/sample_mermaid_all.md` の 23 Ishikawa Diagram で、右端 head が公式 Mermaid.js 描画より横長になり、文字配置も一致していない
- [/] 7.51 再フィードバック: Mermaid 公式比較を目視だけに頼らず、公式画像と KatanA 画像を同一サイズへ正規化したうえで、100点満点の一致度スコアを出し、目安 99 点で機械判定できるようにする
- [/] 7.52 再フィードバック: `assets/fixtures/sample_mermaid_all.md` の 19 Kanban と 12 Mindmap で、文字位置や余白（margin）の一部が公式 Mermaid.js 描画と一致していない。スコア評価でこのズレを検知できるようにする
- [/] 7.53 再フィードバック: Kanban は単純な `assets/fixtures/mermaid_all/19-kanban.md` ではなく、`assets/fixtures/mermaid.md` の 15 kanban にある長文カード、チケット番号、担当者、複数列を含む複雑ケースを主対象として検証する
- [ ] 7.54 再フィードバック: スコア評価で検知できるだけでは不十分である。評価方式は正しい前提で扱い、対象図について描画差分を修正する。v0.22.10 ではまず公式ドキュメント由来の Draw.io fixture 全体を図として認識できる 85 点以上へ寄せる。日本語版（ja）評価、99 点到達、採点ゲート化は `katana-renderer` 分離後の責務として劣後にする
- [/] 7.55 再フィードバック: `assets/fixtures/mermaid.md` の preview で Mermaid 図が全体的に `not supported` と表示されるデグレードが発生している。個別 SVG 生成だけでなく Markdown preview 経由で再現・修正する
- [/] 7.45 再フィードバック: `assets/fixtures/mermaid.md` の 15 kanban で、カード内文字列が上下中央ではなく左上寄せになっていない
- [/] 7.46 再フィードバック: `assets/fixtures/mermaid.md` の 26 venn-beta で、円の枠線と枠内の色が公式表示と異なる
- [/] 7.47 再フィードバック: `assets/fixtures/mermaid.md` の 28 xychart-beta で、単位 `Revenue (in $)` が目盛りへめり込んでいる
- [/] 7.56 再フィードバック: 一時HTML `Uhwpk_tmp_mermaid-test.html` と同じ `assets/fixtures/mermaid.md` を評価対象にできていなかったため、一覧Markdownを一時fixtureへ分割し、公式描画・KatanA描画・スコア評価を同一入力で実行する `make mermaid-sample-compare` を追加する
- [ ] 7.57 再フィードバック: `make mermaid-sample-compare` は描画可能な28図で最低 72.85 点のため、99点ゲート未達の差分を描画側で修正する
- [/] 7.58 再フィードバック: `assets/fixtures/sample_diagrams.md` のHTML表示で、Mermaid sequence diagram のSVG内CSSが数式処理に壊され、`path{fill:#AAAAAA...}` などのCSS断片が本文へ漏れる。HTML export では描画済みSVGをMarkdown/KaTeX処理から保護し、同一HTML内のMermaid SVG ID重複も防ぐ
- [/] 7.59 再フィードバック: HTMLでは正しく表示できているが、PDF / PNG / JPEG export はHTMLをそのまま形式変換せず独自の簡易SVGへ再解釈しているため、HTML側の背景色・文字色が失われるうえ、`width="100%"` のSVGが親SVG内で巨大化する。また本文の日本語が欠け文字になる。Native export でもHTML exportの本文背景色・文字色を反映し、埋め込みSVGのrootサイズを数値へ固定し、本文用フォントを日本語対応font優先にする。絵文字はresvgの混在font処理で後続文字まで欠け文字化するため、PDF/PNG/JPEG本文では装飾絵文字だけ除外する。同じMarkdownからHTML/PDF/PNG/JPEGを検証できる入口を用意する
- [/] 7.60 再フィードバック: `assets/fixtures/sample_mermaid_ja.md` の日本語を含む Mermaid 図がほとんど描画されない。原因はfontではなく、Mermaid.js の一部図種が日本語などASCII外文字をIDや構文値として受け付けず、SVG生成前に構文解析で落ちること。Mermaid入力の多言語正規化を専用JSモジュールへ分離し、Mermaid.jsへ渡す内部ID/構文トークンだけASCIIプレースホルダーへ置換し、SVG生成後に表示文字へ戻す。対象は requirementDiagram / quadrantChart / xychart-beta / sankey-beta / architecture-beta / wardley-beta とし、日本語だけでなく他言語のASCII外文字にも同じ経路で対応する
- [/] 7.61 再フィードバック: `assets/fixtures/sample_mermaid_ja.md` は日本語fixtureとして、Gitなど英語前提の構文や識別子を除き、画面に出るサンプル文言を全体的に日本語化する。Sankey は処理フローの小さい例とエネルギーフローの大きい例を2つ残し、どちらも描画できることを回帰テストへ含める。requirementDiagram の要求名・要素名、ishikawa-beta の要因名など、図種別に構文上ASCII前提になりやすい箇所は専用の入力正規化として追加する
- [/] 7.62 再フィードバック: `assets/fixtures/sample_mermaid_ja.md` の 16.2 エネルギーフローで、日本語 Sankey の同一ラベルが別ノードとして扱われ、0 高さノードが大量発生して文字が縦に潰れる。Sankey の入力正規化では表示ラベルではなく同一文字列を再利用する内部IDを使い、流れの合算とノード高さを維持する
- [ ] 7.63 劣後フィードバック: `assets/fixtures/sample_mermaid_ja.md` も、英語版と同じように公式描画 / KatanA描画 / スコア評価（scores.json と採点表）を出せるようにする。公式 Mermaid.js が日本語などASCII外文字を構文値として受け付けない図種は、比較用入力の正規化方法を明示し、KatanA固有補正を採点から隠さない。v0.22.10 の直近ゴールは公式ドキュメント由来の Draw.io fixture 85 点以上とし、日本語版（ja）の採点整備は後続へ回す
- [ ] 7.64 再フィードバック: Mermaid の日本語正規化と同じ種類の問題が Draw.io で起きるかを検証する。Draw.io は XML の `mxCell id` と表示ラベルが分離されているため、同じ処理をそのまま使う前提にせず、ラベル文字・セルID・公式 viewer 入力・SVG後処理のどこで多言語問題が出るかを切り分ける

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 8. Final Verification & Release Work

### 準備完了条件（Definition of Ready）

- [ ] Task 7（User Review）が完了している
- [ ] `FINDINGS.md` / `COMPATIBILITY_MATRIX.md` に調査結果と後続判断が記録済みである
  - [x] `RUST_MANAGED_JS_SPIKE.md` に V8 直接実行の結果を記録済みである

### 完了条件（Definition of Done）

- [ ] 8.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 8.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 8.3 `./scripts/openspec validate v0-22-10-mermaid-rendering-compatibility-investigation --strict` を実行し、OpenSpec の整合性を確認する
- [ ] 8.4 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `make check` / `make check-light` を二重実行しない
- [ ] 8.5 Create PR from `release/v0.22.10` targeting `master`
- [ ] 8.6 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL / Release Readiness) — blocking merge if any fail
- [ ] 8.7 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 8.8 Verify GitHub Release completion and archive this change using `/opsx-archive`
