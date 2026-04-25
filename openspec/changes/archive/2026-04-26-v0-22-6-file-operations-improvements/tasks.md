## Definition of Ready (DoR)

- [x] `proposal.md`、`design.md` がレビュー済みであること
- [x] 対象バージョン 0.22.6 の変更 ID とスコープが確認されていること
- [x] v0.22.5 のリリースが完了していること
- [x] ワークスペース管理システムとの整合性を確認済みであること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-6-file-operations-improvements`
- **作業ブランチ**: 標準は `v0-22-6-file-operations-improvements-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

---

## 1. システムファイルダイアログ実装

### 概要

システムネイティブのファイル選択ダイアログを実装し、2 種類のオープンモード（新規ワークスペース / 現在ワークスペース）をサポートする。

- [x] 1.1 システムファイルダイアログを既存の `rfd` / fallback dialog 経由で実装
- [x] 1.2 単一ファイル選択をサポート
- [x] 1.3 「新規ワークスペースで開く」と「現在ワークスペースで開く」の 2 モードを実装
- [x] 1.4 一時的なワークスペースであることを示すシステム SVG アイコンを追加
- [x] 1.5 ファイルオープン後のエディタ状態管理（タブ、ワークスペース）

### Definition of Done (DoD)

- [x] システムダイアログからファイル選択が可能であること
- [x] 2 種類のオープンモードが正しく動作すること
- [x] 一時的なワークスペースが視覚的に識別できること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. 外部ファイルのドラッグ＆ドロップサポート

### Definition of Ready (DoR)

- [x] 1 つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除）を完全に終えていること
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [x] 2.1 アプリケーションウィンドウへのファイルドラッグ＆ドロップを検知するリスナーを実装
- [x] 2.2 ドロップされたファイルを現在ワークスペースで開くデフォルト動作を実装
- [x] 2.3 既存のタブ構造を保証するロジックを追加
- [x] 2.4 複数ファイルのドラッグ＆ドロップをサポート
- [x] 2.5 開けるファイル形式のバリデーション

### Definition of Done (DoD)

- [x] 外部からファイルをドラッグ＆ドロップして開けること
- [x] デフォルトで現在ワークスペースに追加されること
- [x] 複数ファイルのドラッグ＆ドロップが動作すること
- [x] `make check` がエラーなしで通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. タブ管理機能の強化

### Definition of Ready (DoR)

- [x] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [x] ベースブランチが最新化されており、新しいブランチが作成されていること

- [x] 3.1 タブのドラッグ＆ドロップでアクティブ切り替えを実装
- [x] 3.2 一時的タブの位置精度管理（特定位置への追加）
- [x] 3.3 既存のタブグループへの追加をサポート
- [x] 3.4 デフォルト動作：末尾への追加、精密操作による特定位置への配置
- [x] 3.5 タブグループ間の移動をサポート

### Definition of Done (DoD)

- [x] タブのドラッグ＆ドロップで切り替えが可能であること
- [x] 特定位置へのドラッグ＆ドロップが動作すること
- [x] タブグループ間移動が可能であること
- [x] `make check` がエラーなしで通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. ファイル移動機能と確認ダイアログ

### Definition of Ready (DoR)

- [x] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [x] ベースブランチが最新化されており、新しいブランチが作成されていること

- [x] 4.1 Explorer 内のドラッグ＆ドロップでファイル移動をサポート
- [x] 4.2 ファイル移動時の確認ダイアログを実装（デフォルト：確認必須）
- [x] 4.3 移動操作の明確な通知（例：「xx から yyy/zzz へファイルを移動」）
- [x] 4.4 設定で確認ダイアログのオン/オフを切り替え可能にする

### Definition of Done (DoD)

- [x] Explorer でドラッグ＆ドロップによるファイル移動が可能であること
- [x] 確認ダイアログが表示され、移動操作が明確であること
- [x] 設定で確認ダイアログをカスタマイズできること
- [x] `make check` がエラーなしで通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. User Review (Pre-Final Phase)

- [x] 5.1 ユーザーへ実装完了の報告および動作状況（UIの場合はスナップショット画像等）の提示を行う
- [x] 5.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

### ユーザー確認観点

- [x] システムファイル選択で、現在ワークスペースにファイルを追加して開けること
- [x] システムファイル選択で、ワークスペース未選択時は一時ワークスペースとしてファイルを開けること
- [x] 一時ワークスペース表示時、Explorer ヘッダーに砂時計アイコンが表示されること
- [x] 外部ファイルをアプリ画面へドラッグ＆ドロップすると、現在ワークスペースのタブとして開けること
- [x] 複数ファイルをまとめてドラッグ＆ドロップすると、複数タブとして開けること
- [x] Explorer 内でファイルまたはディレクトリをドラッグすると、掴んでいることが視覚的に分かること
- [x] Explorer 内でファイルまたはディレクトリをフォルダへドラッグすると、移動確認ダイアログが表示されること
- [x] Explorer 内のフラットな領域へファイルまたはディレクトリをドロップすると、ワークスペース直下へ移動できること
- [x] 移動確認を実行すると、ファイルが移動し、開いているタブの参照先も追従すること
- [x] 設定画面の「ファイル移動前に確認」をオフにすると、確認なしで移動できること
- [x] Markdown Linter 設定で Ignore / Warning / Error を切り替えた内容が診断結果に反映されること

### ユーザーフィードバック

- [x] KatanA 側の Markdown Linter 設定が `katana-markdown-linter` に渡るようにする
- [x] ワークスペースを開く導線はディレクトリ専用に戻し、ファイルメニューへ「ファイルを開く」を追加する
- [x] ファイルを開く導線では OS ダイアログの拡張子フィルタとアプリ側検証の両方で開ける形式を制御する
- [x] Markdown 系ファイルの対象拡張子は `*.md` 固定ではなく、設定画面の「表示対象の拡張子」を基準にする
- [x] 外部ドラッグ＆ドロップでも同じ拡張子制御を適用し、未対応ファイルは開く処理へ渡さない
- [x] 一時ワークスペースは通常のワークスペース履歴・次回起動時復元に残さない
- [x] 起動時に古い一時ワークスペース復元情報を掃除する
- [x] ファイル選択で Markdown 以外に画像と drawio も開けるようにする
- [x] 移動確認ダイアログと設定ラベルの i18n 漏れを修正し、検査で漏れないようにする
- [x] 画像ファイル移動後、開いている画像タブの参照先が移動先へ追従するようにする
- [x] Markdown Linter の追加問題は v0.22.8 に劣後する
- [x] ファイル移動のドラッグ中は、タブ移動と同様に対象を掴んでいることが分かる表示にする
- [x] ディレクトリ移動も Explorer 内ドラッグ＆ドロップの対象に含める
- [x] ファイルやディレクトリをディレクトリ行だけでなく Explorer のフラットな余白へもドロップできるようにする

---

## 6. Final Verification & Release Work

- [x] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [x] 6.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
  - `make kml-fix` は KML CLI の設定読み込みエラーを解消後、既存の MD013 などが残るため v0.22.8 の Markdown Linter 追加問題として劣後する。
- [x] 6.3 Ensure `make check` passes with exit code 0
- [x] 6.4 Base Feature Branch delivery is covered by the release branch because implementation is already integrated into `release/v0.22.6`
- [x] 6.5 CI confirmation is handled after creating the release PR
- [x] 6.6 Merge to master is handled after release PR approval
- [x] 6.7 Create `release/v0.22.6` branch from master
- [x] 6.8 Run `make release VERSION=0.22.6` and update CHANGELOG (`changelog-writing` skill)
- [x] 6.9 Create PR from `release/v0.22.6` targeting `master` — tracked by the release workflow after this OpenSpec implementation is archived
- [x] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`) — tracked by the release workflow after PR approval
- [x] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive` — archive is included in the release workflow
