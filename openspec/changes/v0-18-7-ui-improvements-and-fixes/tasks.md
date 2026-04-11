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

---

## 8. Windows Packaging & Winget Readiness

- [ ] 8.1 `scripts/release/sync-external.sh` を見直し、`HiroyukiFuruno.katana-desktop` が upstream `microsoft/winget-pkgs` に未存在の間は `komac update` を実行せず、bootstrap path が必要だと明示する
- [ ] 8.2 初回再申請用の winget submit 導線を明確化する。`komac new` は non-TTY CI で使えない前提とし、manifest file を生成して `komac submit` する path、または maintainer local TTY での bootstrap path のどちらかに固定する
- [ ] 8.3 Windows binary の VC++ runtime 依存を build policy 側で解消する。`x86_64-pc-windows-msvc` 向けに `crt-static` を有効化し、manifest dependency を消すだけの対症療法にしない
- [ ] 8.4 Windows runner 上で packaged `KatanA.exe` の import table を確認し、`VCRUNTIME140*.dll` / `api-ms-win-crt-*` 依存が除去されたことを検証する
- [ ] 8.5 `komac analyze` / generated manifest review により、`KatanA-windows-x86_64.msi` が `Scope: user` / `InstallerType: wix` を維持しつつ、problematic な `Dependencies: Microsoft.VCRedist.2015+.x64` を含まないことを確認する
- [ ] 8.6 `README.md` / `CHANGELOG.md` / release note に、Windows 配布形式と install prerequisites の実態が一致していることを確認

### Definition of Done (DoD)

- [ ] 初回再申請と将来の version update の flow が混同されていない
- [ ] `KatanA-windows-x86_64.msi` / `KatanA.exe` が VC++ runtime 外部依存なしで配布できる
- [ ] CI と local helper が同じ Windows artifact 名と publish URL 契約を使っている

## 9. Windows Installer UX Refresh

- [ ] 9.1 `crates/katana-ui/wix/main.wxs` の current flow (`WixUI_FeatureTree`) は維持したまま、初回導入時の見た目と文言を KatanA 向けに整理する
- [ ] 9.2 installer metadata（Product 名、説明、ARP 表示、Feature 名）を更新し、古い印象を与える既定表現を除去する
- [ ] 9.3 `WixUIBannerBmp` / `WixUIDialogBmp` を追加し、`Product.ico` と整合する branding asset を適用する
- [ ] 9.4 Windows installer 画面の確認証跡を取得し、`v0.18.7` の申請時に参照できる状態にする

### Definition of Done (DoD)

- [ ] installer 画面が KatanA branding と整合し、既定の古い WiX 画面に見えにくい状態になっている
- [ ] Windows 向け install 導線のスクリーンショットまたは同等の証跡が残っている

## 10. Final Verification & Release Work

- [ ] 10.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 10.2 Ensure `make check` passes with exit code 0
- [ ] 10.3 Confirm Windows release artifacts and GitHub Release asset URLs for `v0.18.7`
- [ ] 10.4 Create PR from Base Feature Branch targeting `master`
- [ ] 10.5 Merge into master and execute `make release VERSION=0.18.7`
- [ ] 10.6 Verify `scripts/release/sync-external.sh` does not silently fail for `HiroyukiFuruno.katana-desktop` and that the chosen bootstrap / update path for `v0.18.7` is documented and reproducible
