## Branch Rule

本タスクでは、ユーザーの指定に基づき以下のブランチ運用を厳格に適用します：

- **統合（Base）ブランチ**: `release/v0.18.7`
- **各タスクの作業ブランチ**: `release/v0.18.7-task-x` (xはタスク番号)

各タスクの実装開始前に、`release/v0.18.7` から `release/v0.18.7-task-x` を作成して作業してください。
実装完了後は `/openspec-delivery` を使用して統合ブランチ（`release/v0.18.7`）へPRを作成・マージしてください。

## 1. Search Noise Reduction & Auto-link Fix

- [x] 1.1 `katana-core/src/search/mod.rs` を修正し、`#[allow(...)]` 行をフィルタリングするロジックを実装
- [x] 1.2 `egui_commonmark` / `egui_commonmark_backend` を修正し、平文URLの自動リンク検出を改善（フラッシュ処理の追加により描画崩れを修正）
- [x] 1.3 `katana-core` の関連テストを実行し、意図せぬデグレードがないか確認

### Definition of Done (DoD)

- [x] **R2: guide_en.md の同期**: `guide_ja.md` (正) の最新の内容を `guide_en.md` に反映する。
- [x] **R3 / Task 1.2: URL 自動リンク（Autolink）の修正**: リスト項目内などで平文URLが正しく検出されず、描画がオーバーラップする問題を修正する。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 2. Meta Information UI Renewal

- [x] 2.1 `katana-ui/src/views/modals/meta_info.rs` を刷新し、Finder風の整理されたレイアウトを実装
- [x] 2.2 メタ情報の各項目（パス、サイズ、作成日時等）をセクション分けして表示
- [x] 2.3 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 2.4 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] メタ情報ダイアログがFinder風の見た目になっていることを確認
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 3. Editor Toolbar Integration & Explorer UI Polish

- [x] 3.1 Editor Toolbar を 3カラム Finder スタイルに刷新し、Breadcrumbs を中央に統合
- [x] 3.2 `TabToolbar` コンポーネントの `ui.horizontal()` 等を `egui::Layout` に移行し Linter をパス
- [x] 3.3 Explorer Tree (Dir/File Entry) のインデントとアイコン間隔を Finder スタイルに精密調整
- [x] 3.4 全 UI コードからマジックナンバーを排除し、`shell/mod.rs` の定数に集約
- [x] 3.5 `make check-local` およびアライメント回帰テストのパスを確認

### Definition of Done (DoD)

- [x] Toolbar が中央揃えの Breadcrumbs を持ち、Finder ライクな質感になっている
- [x] Explorer のアライメントが Dir/File 間で完全に一致している
- [x] `ast-lint` がすべての項目をパスしている
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 4. Diagram Fullscreen & UI Polish

- [x] 4.1 ダイアグラム全画面表示時のオーバーレイ背景を不透明化（アルファ値 1.0）
- [x] 4.2 ダイアグラム全画面表示を閉じる際のアニメーション実装（および軽微な修正）

### Definition of Done (DoD)

- [x] 全画面表示で背景が透けず、図に集中できることを確認
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 5. Sidebar Continuity & Popup UI

- [x] 5.1 `katana-ui/src/app/action/dispatch.rs` を修正し、他パネル展開時もエクスプローラーを表示維持する
- [x] 5.2 サイドバーアイコンクリック時のアニメーション付きポップアップUIの実装（`Area` を使用）

## 6. Tab Group Operations & Explorer Integration

- [x] 6.1 タブ名入力時の `Return` キー/`Blur` での確定・クローズ処理を実装
- [x] 6.2 エクスプローラーのコンテキストメニューに「タブグループを作成」「既存グループに追加」アクションを追加

## 7. Help Enrichment (Welcome & Guide)

- [x] 7.1 「ようこそ」画面をタブ形式で開くように変更（初回起動時含む）
- [x] 7.2 「操作ガイド」メニューを追加し、Markdownタブとして表示

## 8. Slideshow UI Modernization & Fixes

- [x] 8.1 削除: ストーリービューの既存の固定ギアアイコンおよび余分な「設定」タイトルを削除
- [x] 8.2 新機能: ストーリービュー用スライドアウト式（ドロワー）サイドバータブの実装およびアニメーション連携
- [x] 8.3 修正: スライドショーホバー時ハイライトの消失バグを修正し、トグルスイッチでダイアグラムおよび全ノードへのハイライト設定を制御可能にする

## 9. Final Verification & Release Work

- [ ] 9.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 9.2 Ensure `make check` passes with exit code 0
- [ ] 9.3 Confirm Windows release artifacts and GitHub Release asset URLs for `v0.18.7`
- [ ] 9.4 Create PR from Base Feature Branch targeting `master`
- [ ] 9.5 Merge into master and execute `make release VERSION=0.18.7`
- [ ] 9.6 Verify `scripts/release/sync-external.sh` does not silently fail for `HiroyukiFuruno.katana-desktop`, that the chosen bootstrap / update path for `v0.18.7` is documented and reproducible, and that winget sync no longer relies on `github.token` fallback
