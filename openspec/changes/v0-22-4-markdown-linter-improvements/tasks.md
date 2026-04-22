## Definition of Ready (DoR)

- [x] `proposal.md`、`design.md` がレビュー済みであること
- [x] 対象バージョン 0.22.4 の変更 ID とスコープが確認されていること
- [x] v0.22.3 のリリースが完了していること
- [x] markdownlint の全ルール仕様を確認済みであること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-4-markdown-linter-improvements`
- **作業ブランチ**: 標準は `v0-22-4-markdown-linter-improvements-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

---

## 1. Markdownlint 全ルールサポート実装

### 概要

現在の MD001 のみサポートから、markdownlint の全公式ルールをサポートするように拡張する。

- [x] 1.0 `refresh_preview` のショートカットを `refresh_document` に統合
  - `os_commands.json` から `refresh_preview` キーを削除
  - `guide_ja.md` / `guide_en.md` の表記を `refresh_document` へ変更
  - `locales/*.json` (全言語) の `shortcut_refresh` 内の参照を `refresh_document` へ変更
- [x] 1.1 `crates/katana-linter/src/rules/markdown/` に markdownlint の全ルール実装を追加
  - `rules/` サブディレクトリを新設し、ルール実装ファイルを整理
  - `helpers.rs` に共有ユーティリティを `RuleHelpers` struct として集約
- [x] 1.2 各ルールの検証ロジックを実装（MD001-MD052 の全ルール）
  - MD003, MD004, MD011→MD012, MD022-MD023, MD025-MD029, MD032-MD033, MD035-MD036, MD040-MD042, MD045, MD047 を実装
  - AST linter 全項目（file-length, nesting-depth, magic-numbers, no-pub-free-fn）に完全準拠
  - `make check` が exit code 0 で通過
- [x] 1.3 ルールカテゴリ別に自動修正可能なルールと手動修正が必要なルールを分類
  - `OfficialRuleMeta.is_fixable` フィールドで各ルールの自動修正可否を管理
- [x] 1.4 既存の MD001 ルールとの後方互換性を確認
  - `HeadingStructureRule` を `HeadingIncrementRule` のエイリアスとして再エクスポート

### Definition of Done (DoD)

- [x] markdownlint 公式の主要ルールが動作すること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] ルールの分類が正しく行われ、自動修正可能なルールが識別できること
- [x] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 2. Lint 設定 UI 実装

### 概要

現在ショートカット画面に混在しているルールトグルを、専用の `設定 → Lint` セクションに移行。重大度の3段階制御と高度なワークスペース設定を提供する。

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

### タスク

- [ ] 2.1 設定画面に `Lint` セクションを新設
  - マスタースイッチ「Markdown Linter を有効化」（デフォルト: 有効）
- [ ] 2.2 各ルールの重大度設定ドロップダウンを実装
  - `無視`（無効） / `警告`（Warning） / `エラー`（Error）の3段階
  - デフォルト: 全ルール `警告`
- [ ] 2.3 重大度設定を `MarkdownDiagnostic` の `severity` に反映
  - `disabled_rules: HashSet<String>` → `rule_severity: HashMap<String, Severity>` への移行
- [ ] 2.4 ショートカット画面からリンタールール切替コマンドを削除
  - `linter_commands.rs` のコマンド群を設定 UI に完全移行
- [ ] 2.5 高度なワークスペース設定 UI を実装
  - ワークスペースごとの `.markdownlint.json` を生成・編集できる画面
  - ルールパラメータの詳細設定（markdownlint 公式の設定形式に準拠）

### Definition of Done (DoD)

- [ ] 設定画面から各ルールの重大度が変更できること
- [ ] 設定変更が即座に lint 結果に反映されること
- [ ] ワークスペース設定 JSON の生成・編集が動作すること
- [ ] `make check` がエラーなしで通過すること
- [ ] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 3. エディタ内視覚インジケーター実装（VSCode スタイル）

### 概要

lint 問題をエディタ上で直感的に把握できるよう、波線（squiggly underline）と💡ガターアイコンを実装する。

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

### タスク

- [ ] 3.1 問題行に波線（squiggly underline）を描画
  - 重大度に応じた色分け: 黄色（警告） / 赤色（エラー）
  - テーマカラー適用可能
- [ ] 3.2 行番号ガターに💡アイコンを表示
  - 問題がある行の行番号横にライトバルブアイコンを描画
- [ ] 3.3 💡クリックまたはホバーで診断ポップアップを表示
  - ルール ID、ルール名、説明文、重大度を表示
- [ ] 3.4 lint 問題の位置情報（行番号、列番号）をエディタ状態に格納
- [ ] 3.5 大量問題時の描画最適化（パフォーマンス確保）

### Definition of Done (DoD)

- [ ] 警告は黄色、エラーは赤色の波線で視覚表示されること
- [ ] 💡アイコンが正しく表示され、クリックで詳細が確認できること
- [ ] 大量の lint 問題でも UI がカクつかないこと
- [ ] `make check` がエラーなしで通過すること
- [ ] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 4. 診断ポップアップと自動修正機能

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

### タスク

- [ ] 4.1 診断ポップアップに詳細情報を表示
  - ルールの説明、修正例、ドキュメントリンク
- [ ] 4.2 自動修正可能なルールに対して修正ボタンを表示
  - `is_fixable: true` のルールのみ修正アクションを提供
- [ ] 4.3 自動修正実行時のファイル変更管理（undo stack の管理）
- [ ] 4.4 一括修正機能（Fix All）を実装
  - 全文書・全ルールの自動修正を一括実行

### Definition of Done (DoD)

- [ ] 💡ポップアップから自動修正が実行できること
- [ ] 自動修正が正しく適用され、undo 可能であること
- [ ] 一括修正機能が動作すること
- [ ] `make check` がエラーなしで通過すること
- [ ] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 5. アプリ内ルールドキュメントビューアー

### 概要

ルールドキュメント（MDXXX.md）を外部ブラウザではなく KatanA 内でネイティブ表示し、セッションキャッシュで快適な閲覧体験を提供する。

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

### タスク

- [ ] 5.1 ルールドキュメントリンクのクリックをインターセプト
  - `docs_url` のクリック時に外部ブラウザを開かず、内部ハンドラに委譲
- [ ] 5.2 非同期 HTTP で markdownlint 公式 GitHub から Markdown を取得
  - `reqwest` 等を使用した非同期取得
- [ ] 5.3 セッションキャッシュの実装
  - 取得済みドキュメントをメモリにキャッシュし、同一セッション内での再取得を防止
- [ ] 5.4 取得した Markdown を KatanA の仮想プレビュー領域でレンダリング
  - 既存の Markdown プレビューエンジンを流用

### Definition of Done (DoD)

- [ ] ルールドキュメントがアプリ内でネイティブ表示されること
- [ ] 2回目以降のアクセスがキャッシュから即時表示されること
- [ ] ネットワークエラー時にフォールバック（外部ブラウザ起動等）が動作すること
- [ ] `make check` がエラーなしで通過すること
- [ ] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 6. Final Verification & Release Work

- [ ] 6.1 自己レビューを実行（`docs/coding-rules.ja.md` および `.agents/skills/self-review/SKILL.md`）
- [ ] 6.2 `make check` が exit code 0 で通過すること
- [ ] 6.3 Base Feature Branch から `master` を対象に PR を作成
- [ ] 6.4 PR 上の CI チェック（Lint / Coverage / CodeQL）が通過することを確認
- [ ] 6.5 master にマージ（`gh pr merge --merge --delete-branch`）
- [ ] 6.6 master から `release/v0.22.4` ブランチを作成
- [ ] 6.7 `make release VERSION=0.22.4` を実行し、CHANGELOG を更新（`changelog-writing` スキル）
- [ ] 6.8 `release/v0.22.4` から `master` を対象に PR を作成 — `Release Readiness` CI 通過を確認
- [ ] 6.9 リリース PR を master にマージ（`gh pr merge --merge --delete-branch`）
- [ ] 6.10 GitHub Release の完了を確認し、`/opsx-archive` でこの変更をアーカイブ
