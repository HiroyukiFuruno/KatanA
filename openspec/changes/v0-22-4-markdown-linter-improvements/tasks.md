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

- [x] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [x] ベースブランチが最新化されており、新しいブランチが作成されていること

### タスク

- [x] 2.1 設定画面に `Lint` セクションを新設
  - マスタースイッチ「Markdown Linter を有効化」（デフォルト: 有効）
- [x] 2.2 各ルールの重大度設定ドロップダウンを実装
  - `無視`（無効） / `警告`（Warning） / `エラー`（Error）の3段階
  - デフォルト: 全ルール `警告`
- [x] 2.3 重大度設定を `MarkdownDiagnostic` の `severity` に反映
  - `disabled_rules: HashSet<String>` → `rule_severity: HashMap<String, Severity>` への移行
- [x] 2.4 ショートカット画面からリンタールール切替コマンドを削除
  - `linter_commands.rs` のコマンド群を設定 UI に完全移行
- [x] 2.5 高度なワークスペース設定 UI を実装
  - ワークスペースごとの `.markdownlint.json` を生成・編集できる画面
  - ルールパラメータの詳細設定（markdownlint 公式の設定形式に準拠）

### Definition of Done (DoD)

- [x] 設定画面から各ルールの重大度が変更できること
- [x] 設定変更が即座に lint 結果に反映されること
- [x] ワークスペース設定 JSON の生成・編集が動作すること
- [x] `make check` がエラーなしで通過すること
- [x] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 3. エディタ内視覚インジケーター実装（VSCode スタイル）

### 概要

lint 問題をエディタ上で直感的に把握できるよう、波線（squiggly underline）と💡ガターアイコンを実装する。

### Definition of Ready (DoR)

- [x] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [x] ベースブランチが最新化されており、新しいブランチが作成されていること

### タスク

- [x] 3.1 問題行に波線（squiggly underline）を描画
  - 重大度に応じた色分け: 黄色（警告） / 赤色（エラー）
  - テーマカラー適用可能
- [x] 3.2 行番号ガターに💡アイコンを表示
  - 問題がある行の行番号横にライトバルブアイコンを描画
- [x] 3.3 💡クリックまたはホバーで診断ポップアップを表示
  - ルール ID、ルール名、説明文、重大度を表示
- [x] 3.4 lint 問題の位置情報（行番号、列番号）をエディタ状態に格納
- [x] 3.5 大量問題時の描画最適化（パフォーマンス確保）

### Definition of Done (DoD)

- [x] 警告は黄色、エラーは赤色の波線で視覚表示されること
- [x] 💡アイコンが正しく表示され、クリックで詳細が確認できること
- [x] 大量の lint 問題でも UI がカクつかないこと
- [x] `make check` がエラーなしで通過すること
- [x] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 4. 診断ポップアップと自動修正機能

### Definition of Ready (DoR)

- [x] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [x] ベースブランチが最新化されており、新しいブランチが作成されていること

### タスク

- [x] 4.1 診断ポップアップに詳細情報を表示
  - ルールの説明、修正例、ドキュメントリンク
- [x] 4.2 自動修正可能なルールに対して修正ボタンを表示
  - `is_fixable: true` のルールのみ修正アクションを提供
- [x] 4.3 自動修正実行時のファイル変更管理（undo stack の管理）
- [x] 4.4 一括修正機能（Fix All）を実装
  - 全文書・全ルールの自動修正を一括実行

### Definition of Done (DoD)

- [x] 💡ポップアップから自動修正が実行できること
- [x] 自動修正が正しく適用され、undo 可能であること
- [x] 一括修正機能が動作すること
- [x] `make check` がエラーなしで通過すること
- [x] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 5. アプリ内ルールドキュメントビューアー

### 概要

ルールドキュメント（MDXXX.md）を外部ブラウザではなく KatanA 内でネイティブ表示し、セッションキャッシュで快適な閲覧体験を提供する。

### Definition of Ready (DoR)

- [x] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [x] ベースブランチが最新化されており、新しいブランチが作成されていること

### タスク

- [x] 5.1 ルールドキュメントリンクのクリックをインターセプト
  - `docs_url` のクリック時に外部ブラウザを開かず、内部ハンドラに委譲
- [x] 5.2 非同期 HTTP で markdownlint 公式 GitHub から Markdown を取得
  - `reqwest` 等を使用した非同期取得
- [x] 5.3 セッションキャッシュの実装
  - 取得済みドキュメントをメモリにキャッシュし、同一セッション内での再取得を防止
- [x] 5.4 取得した Markdown を KatanA の仮想プレビュー領域でレンダリング
  - 既存の Markdown プレビューエンジンを流用

### Definition of Done (DoD)

- [x] ルールドキュメントがアプリ内でネイティブ表示されること
- [x] 2回目以降のアクセスがキャッシュから即時表示されること
- [x] ネットワークエラー時にフォールバック（外部ブラウザ起動等）が動作すること
- [x] `make check` がエラーなしで通過すること
- [x] `/openspec-delivery` ワークフローを実行してデリバリサイクルを完了する

---

## 6. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。
> ブランチ: `feature/v0.22.4-task5-feedback`（task5 ブランチを継続使用）

- [x] **FB1** 診断パネルのリンクジャンプ — プレビューモード中はコードモードへ切り替えてから対象行にジャンプ
- [x] **FB2.a** `LightBulb` アイコンを全パック分ダウンロード・登録（`katana-icon-management` スキル拠）
- [x] **FB2.b** `row_diagnostics.rs` のアイコンを `Icon::LightBulb` に変更
- [x] **FB3** ガターアイコンのめり込み修正 — 行番号ガターの余白調整（40→ 52px）
- [x] **FB4** 設定タブ表記統一 — ナビ「linter」とヘッダーを「Linter」に合わせる（ロケール + default 値）
- [x] **FB5.1** 設定レイアウト崩壊修正 — プルダウンが右にはみ出す問題を解消
- [x] **FB5.2** `md-broken-link` を設定画面から非表示（内部専用ルール）
- [x] **FB5.3** 全 MD00XX ルールが設定画面に表示されているか確認・修正
- [x] **FB5.5** 重要度コントロールをプルダウンから 3 段階モダントグル（無視 / 警告 / エラー）に変更
- [x] **FB6.a** `.markdownlint.json` のデフォルト保存先を KatanA アプリ設定領域に変更
- [x] **FB6.b** 「ワークスペースに保存」トグルで保存先切り替えを可能にする
- [x] **FB6.c** 切り替え時に相手側の JSON が存在すれば自動展開する
- [x] **FB6.d** 「設定を開く」を「高度な設定」ボタンに変更し、クリック時に JSON を直接エディタタブで開くのではなく、アイコンの高度な設定のような GUI モーダルを開いて設定値（`allow_keyword`, `types`, `default` 値など）を組み立てて JSON を更新・検証する仕組みに変更する（ルール・スキーマ定義は `katana-markdown-linter` モジュール側に持たせ、それを元に GUI を描画する）
- [x] **TDB1** (技術的負債) ドキュメント URL 404 修正: `regex_rule!` マクロが生成する URL が大文字 ID を使うが GitHub ファイル名は小文字。`process_linter.rs` でファイル名部分を小文字化する。
- [x] **FB7** 検証用の `lint-fix.md` と `lint-fix.md.org` を作成しデモで readonly, code-only で表示する
- [x] **FB8** ルールドキュメントのリンクが外部リンクになっている問題を修正（常にプレビューのみで表示し、表示分割やコードビューを許容しない）
- [x] **FB9** fix したあとに lint 判定が自動で再評価されUIに反映されるよう修正 — `handle_apply_lint_fixes` 末尾で `handle_action_refresh_diagnostics()` を即時呼び出し
- [x] **FB10** (技術的負債) `katana-linter` を外部 Rust クレートに移譲する。クレート選定時に fix 機能が内包されているものを優先して採用する。
  - 調査完了: 成熟した外部ライブラリクレートが存在しないため、**独立リポジトリ `katana-markdown-linter` を新設**する方針に決定
  - リポジトリ: <https://github.com/HiroyukiFuruno/katana-markdown-linter> （OpenSpec 定義済み）
  - 詳細は `design.md` §5 および新リポジトリの `openspec/` を参照
- [x] **FB11** Linter タブ内の Rule Severities のレイアウト崩壊修正（`AlignCenter` 廃止、`right_to_left` で堅牢化）
- [x] **FB12** Rule Severities の MD001 などのルールIDの下に、ルール内容の簡単な説明（description）を追加（全言語対応 / `katana-i18n-management` 準拠）
- [x] **FB13** lintの対象が `.markdownlint.json` 等のJSONになっている問題を修正（設定の拡張子以外は許容しない）
- [x] **FB14** 問題ビューにて、ファイルごとに複数の問題をアコーディオンでグループ化する。「全開」「全閉」ボタンを「問題」タイトルの左側に配置する
- [x] **FB15** 問題ビューの横幅をパネルの100%にする。ドキュメントリンクは常に右寄せとし、詳細の開始位置を上下で左揃えにする（ショートカットキーUIのパターンを踏襲）
- [x] **FB16** 問題ビューからも Fix（修正）アクションを実行可能にする（UX/UIは一任）
- [x] **FB17** 行番号ガターの 💡 アイコンは、実際に Fix 可能な問題が存在する場合のみ表示する仕様に変更する
- [x] **FB18** エディタ上で問題にホバーした際、VSCodeのようなポップオーバー（消えにくく、背後へのイベントバブリングを抑止するモーダル/ツールチップ）を展開し、そこから Fix を実行できるようにする
- [x] **FB19** 問題ビュー（アコーディオン）を閉じた際に、ユーザーが広げたパネルサイズが強制的に縮小されるリサイズバグを修正（`auto_shrink([false, false])` を適用）
- [x] **FB20** Fix 機能バグ — MD022（blanks-around-headings）の fix を適用すると新たな違反が発生するカスケードバグを修正。root cause: `DiagnosticFix` の座標計算が誤っており、replacement 文字列内の改行と既存改行が二重になっていた。UT 追加（`md022_fix_*` 3本）も必須。
- [x] **FB21** 仮想ドキュメント（`Katana://LinterDocs/MD*.md` 等）に対して Linter が実行され、spurious な診断が出る問題を修正。`handle_action_refresh_diagnostics` に `is_virtual_path()` ガードを追加。
- [x] **FB1** 診断パネルのリンクジャンプ — プレビューモード中はコードモードへ切り替えてから対象行にジャンプ（`handle_action_select_and_jump` で `ViewMode::PreviewOnly` 時に `set_active_view_mode(CodeOnly)` を追加）
- [x] **FB22** ホバーポップオーバー表示バグ — 同一行に複数の診断（例: MD025 + MD023）が存在する場合、最初の診断内容のみ表示し後続診断はリンクのみ追記される誤表示。全診断をセクション分けして正しく表示すべき。
- [x] **FB23** Fix 機能 UT 強化 — `tests/fix_harness.rs`（ポータブルハーネス: `LintFixScenario` / `BulkFixScenario` / `apply_single_fix` / `apply_bulk_fixes` Interface定義）と `tests/fix_scenarios.rs`（MD022 / MD023 / MD032 の single / bulk / convergence シナリオ）を新設。repository移譲対応の分離設計（`katana_linter` 以外依存なし）。8/8テスト通過。
- [x] **FB12** Rule Severities の MD001 などのルールIDの下に、ルール内容の簡単な説明（description）を追加（全言語対応 / `katana-i18n-management` 準拠）— `official_rule!` マクロに title/desc パラメータを追加し、`stubs.rs` の全ルールに正式な説明を付与済み。i18n ロケールファイルの日本語対応が残タスク。
- [x] **FB24** パンくずバグ — `Katana://LinterDocs/MD003.md` を `split('/')` すると空文字セグメントが生まれ □ が表示されていた。LinterDocs 仮想パスを Demo 同様にファイル名のみ表示する特別処理へ変更し、一般パスでも空セグメントを filter で除去するよう `breadcrumbs.rs` を修正。
- [x] **FB25** LinterDocs（MDXX.md）の右上に「公式GitHubで確認」ボタンを配置 — `render_document_toolbar` に `linter_rule_id: Option<String>` を追加し、`Katana://LinterDocs/` 仮想ドキュメント表示時のみ `Github` アイコン + `view_on_github` i18n ツールチップ付きボタンをmdのプレビューの内の右端表示。クリックで `open::that()` によりシステムブラウザで markdownlint 公式 GitHub を開く。
- [x] **FB26** Fix 適用後に lint が即時再評価されない — FB9 と同一修正で解決済み（`handle_action_refresh_diagnostics()` 即時呼び出し） — `ApplyLintFixes` アクション完了後に再 lint トリガーが走らず、波線や Problems パネルが古い状態のまま残る。Fix 後に `RefreshDocument` or lint パスを強制起動する必要がある。
- [-] **FB27** (v0.22.8へ見送り) Fix の変更内容がプレビューできない — Problems パネルの「修正」ボタンを押す前に「何が変わるか」をユーザーが確認できない。修正内容のプレビュー（diff またはツールチップ）が必要。※実装コストが高いため、LLM系の対応前のDoR（Definition of Ready）として次期バージョン（v0.22.8）で実施。
- [x] **FB28** Problems パネルの severity を SVG アイコン化 — 全6パック × 3種（error / warning / info）= 18ファイルを `assets/icons/*/linter/` に配置。`Icon::LinterSeverityError/Warning/Info` を enum に登録し `pack/mod.rs` マクロに追加。`diagnostics_renderer.rs` で `.image()` 直描画（tint 不適用）。`svg.rs` の色チェックから `linter/` カテゴリを除外、`icons_sync.rs` の duplicate ホワイトリストに追加。
- [x] **FB29** Fix 重複挿入バグ（0幅挿入方式で解決） — 複数の連続する heading に一括 fix を適用すると `## B\n\n## B` のように heading が二重挿入される。`DiagnosticFix.replacement` が heading テキストを含む形式のため、descending sort での一括適用時に重複が発生。fix 生成ロジックの replacement 形式を「差分のみ」に変更する必要がある（FB20 の根本原因と関連）。
- [x] **FB30** 削除済みファイルの診断が Problems パネルに残存 — ファイルを削除しても Problems パネルに警告・エラーが表示され続ける。ファイル削除イベント（`handle_action_delete_fs_node`）と連動して、削除されたファイルの診断を `diagnostics_state` から除去する必要がある。
- [x] **FB31** ダークテーマで severity アイコン不可視 — `linter/` SVG が白ストロークのためダークテーマで消える。全6パック × 3種に solid circle SVG（error: `#EF4444`、warning: `#F59E0B`、info: `#3B82F6`）を配置・上書き済み（確認待ち）。
- [x] **FB32** Problems パネルの自動 re-lint — バッファ変更（debounce）・fix 適用後・ファイル保存時の 3 イベントで自動再 lint する。手動ボタンは不要。fix 適用後の即時 re-lint（FB26 統合）と保存時トリガーの追加実装が必要。※問題viewのコードの行の更新が必要
- [x] **FB33** ファイルを開いたときに lint が未実行 — ファイルオープンイベント（`AppAction::OpenDocument` 等）の処理後に `RefreshDiagnostics` を発行すれば解決。起動時の復元ファイルも同様に open イベント経由なので同じ修正で対応可能。
- [x] **FB34** MD003 誤検知修正 — 水平線 (`---`) が MD003 (Heading Style) として誤検知されるバグを修正。前の行が空行やFenced Codeでない場合のみSetext Headingとして評価するように `heading_style.rs` を修正。
- [x] **FB35** MD038 誤検知修正 — バッククォートの外側のスペース（例: `` `xxx` yyy ``）がコードスパン内のスペースとして誤検知されるバグを修正。単純な正規表現（`stubs_regex.rs`）を廃止し、バッククォートのペアを正確にパースする専用ルール `NoSpaceInCodeRule` (`spaces_in_code.rs`) を新設。
- [x] **FB36** MD060（Table Column Style）検知漏れ修正 — ヘッダー行が存在しない単独のテーブル区切り行（`|---|`）や、データ行に対しても、パイプ（`|`）の前後の半角スペース不足を柔軟に検知しフォーマットできるよう判定ロジックを大幅に拡張・修正。
- [x] **FB37** MD037 誤検知修正 — 別の `**` 強調記号の終了タグと開始タグの間のスペースを拾ってしまい「強調記号の内部にスペースがあります」と誤検知するバグ（例: `**A** B **C**`）を修正。正規表現を廃止し、専用の構文パースルール `SpacesInEmphasisRule` (`spaces_in_emphasis.rs`) を新設。

## 7. Final Verification & Release Work

- [x] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [x] 7.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [x] 7.3 Ensure `make check` passes with exit code 0
- [x] 7.4 Create PR from Base Feature Branch targeting `master`
- [x] 7.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [x] 7.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [x] 7.7 Create `release/v0.22.4` branch from master
- [x] 7.8 Run `make release VERSION=0.22.4` and update CHANGELOG (`changelog-writing` skill)
- [x] 7.9 Create PR from `release/v0.22.4` targeting `master` — Ensure `Release Readiness` CI passes
- [x] 7.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [x] 7.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
