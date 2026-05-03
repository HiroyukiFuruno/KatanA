# Tasks: v0.22.11 Renderer Runtime Interface and Versioning

## 0. 準備完了条件（Definition of Ready）

- [x] v0.22.10 は公式 Mermaid.js 比較と表示最適化に集中する
- [x] v0.22.11 は interface 整理、Mermaid.js 版固定、`katana-renderer` 分離設計を扱う
- [x] 分離先の仮称は、Mermaid / Draw.io / HTML / PDF / PNG / JPEG の描画境界を含められる `katana-renderer` とする
- [x] preview 分離を前提に、preview が `katana-renderer` を利用し、`katana-renderer` は preview / egui / KatanA UI に依存しない方針で進める
- [x] 既存の KML v0.16.1 取り込みは戻さない

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `release/v0.22.11`
- **作業ブランチ**: `feature/v0.22.11-task-x`（xはタスク番号）

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. v0.22.10 からの責務移管を固定する

### 実施内容

v0.22.10 の表示最適化と、v0.22.11 の接続境界整理が混ざらないように、対象責務、未対象責務、後続移管を文書で固定する。

### 完了条件（Definition of Done）

- [ ] 1.1 v0.22.10 の残作業が公式比較、表示差分補正、サイズ / 配色 / SVG 互換の最適化に閉じていることを確認する
- [ ] 1.2 interface 汎用化、Mermaid.js 版固定、`katana-renderer` 分離設計が v0.22.11 に移管済みであることを確認する
- [ ] 1.3 Draw.io と HTML / PDF / PNG / JPEG export は v0.22.11 の所有境界整理対象として明記する
- [ ] 1.4 preview 分離と `katana-renderer` の依存方向が文書化されていることを確認する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Renderer runtime interface を整理する

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

KatanA と描画 runtime の接続境界を、preview / export / 将来の repository 分離で使える中立 DTO（データだけの型）へ整理する。

### 完了条件（Definition of Done）

- [ ] 2.1 入力を `source`、Mermaid.js 互換 `config`、KatanA 独自 `policy`、テーマ / 文書 `context` に分ける
- [ ] 2.2 `theme` / `themeVariables` / `securityLevel` / `htmlLabels` / diagram-specific config を Mermaid.js config と互換の形で扱う
- [ ] 2.3 最大幅、最大高さ、余白、背景、cache profile は KatanA policy として config の外側へ分離する
- [ ] 2.4 戻り値に SVG、幅、高さ、viewBox、runtime version、renderer profile、diagnostics を含める
- [ ] 2.5 `katana-renderer` 側が preview、egui、KatanA UI state に依存しないことを型と module 境界で確認する
- [ ] 2.6 cache key が source、Mermaid.js 版、renderer profile、config、policy、theme fingerprint を識別することをテストする
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Mermaid.js の版固定と埋め込み管理を追加する

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

無印の `mermaid.min.js` を使う状態をやめ、公式 Mermaid.js の特定版を repository 内で一時的に埋め込み管理する。最終所有者は `katana-renderer` とする。

### 完了条件（Definition of Done）

- [ ] 3.1 埋め込み配置を `vendor/mermaid/<version>/mermaid.min.js` と checksum で管理する
- [ ] 3.2 更新入口を `just VERSION=<version> mermaid-js-update` のような明示コマンドへ集約する
- [ ] 3.3 runtime が固定版 Mermaid.js を読み込み、実行時に CDN / npm install / OS Chrome アプリへ依存しないことを確認する
- [ ] 3.4 公式比較画像の更新が、固定版 Mermaid.js と同じ version を使うことを確認する
- [ ] 3.5 Mermaid.js 版を cache key、比較証跡、診断情報へ含める
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. `katana-renderer` 分離設計を文書化する

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

KatanA から描画専門性を外すため、`katana-renderer` の責務、KatanA に残す責務、移行順序、検証機構を文書化する。

### 完了条件（Definition of Done）

- [ ] 4.1 `katana-renderer` が所有する Mermaid.js 版管理、Rust 管理 JS runtime、DOM / SVG / layout shim、公式比較画像生成、採点評価を明記する
- [ ] 4.2 KatanA に残す Markdown block 抽出、テーマ snapshot、preview / export UI、cache 保存先を明記する
- [ ] 4.3 preview 分離後も `preview -> katana-renderer` の一方向依存になることを図または文章で明記する
- [ ] 4.4 `katana-renderer` repository 側の更新、release、検証画像更新、保存時チェック（pre-commit）と CI/CD での採点検証、KatanA への取り込み手順を記録する
- [ ] 4.5 KML と同じく、専門性を外部 component に出す設計判断として記録する
- [ ] 4.6 `mmdc` より軽く速く、描画待ちを体感しない描画体験を価値として扱い、初回描画と連続描画の性能証跡を残す方針を記録する
- [ ] 4.7 `katana-renderer` CLI を想定し、単体 render、公式比較画像更新、性能計測を core API の利用者として設計する
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Draw.io と export runtime の所有境界を整理する

### 準備完了条件（Definition of Ready）

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### 実施内容

Draw.io 描画と HTML / PDF / PNG / JPEG export が、`katana-renderer` の責務に入るのか、KatanA 側の document export に残るのかを切り分ける。

### 完了条件（Definition of Done）

- [ ] 5.1 Draw.io preview / HTML export / PDF export / PNG export / JPEG export の現行 runtime 依存を棚卸しする
- [ ] 5.2 Draw.io が Mermaid と同じ renderer interface に乗るか、別 backend として扱うかを判断する
- [ ] 5.3 HTML / PDF / PNG / JPEG export で `katana-renderer` が担う範囲と KatanA が担う範囲を文書化する
- [ ] 5.4 未接続の export 経路は OS Chrome / Chromium アプリへ黙って戻さず、明示的な未対応または後続移管として扱う
- [ ] 5.5 v0.22.11 で実装する範囲と、後続 versioned change に送る範囲を分ける
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 6.1 ユーザーへ実装完了の報告および設計証跡を提示する。UI の動作確認が必要な場合は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする
- [ ] 6.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 7. Final Verification & Release Work

### 準備完了条件（Definition of Ready）

- [ ] Task 6（User Review）が完了している
- [ ] `katana-renderer` 分離設計、Mermaid.js 版固定、preview 分離との依存方向、Draw.io / export runtime 境界が文書化済みである

### 完了条件（Definition of Done）

- [ ] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 7.2 Format and lint-fix all updated markdown documents
- [ ] 7.3 `./scripts/openspec validate v0-22-11-renderer-runtime-interface-and-versioning --strict` を実行し、OpenSpec の整合性を確認する
- [ ] 7.4 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
- [ ] 7.5 Create PR from `release/v0.22.11` targeting `master`
- [ ] 7.6 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL / Release Readiness) — blocking merge if any fail
- [ ] 7.7 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.8 Verify GitHub Release completion and archive this change using `/opsx-archive`
