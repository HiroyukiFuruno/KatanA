## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. Icon Settings Configuration Schema & Core Setup

### Definition of Ready (DoR)

- [x] Requirements are understood and base structures established.

- [x] 1.1 `crates/katana-core/src/config/` のスキーマ定義を拡張し、`IconOverride`、`IconConfig`、および `icon_presets` を追加する。
- [x] 1.2 `IconRegistry` やテーマ描画ロジックにおいて対象アイコンのパスプレフィックス(vendor)から色を決定するヘルパー実装を追加する（KatanaはcurrentColor、他はベンダー指定色）。
- [x] 1.3 `Settings` ロード・セーブ時に新しいアイコン設定群が適切にSerialize/Deserializeされることをアサートする単体テストを追加する。

### Definition of Done (DoD)

- [x] `katana-core` 側のコンパイルと単体テストが通ること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. Vendor-based Default Icon Coloring & Preview UI

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `crates/katana-ui/src/views/settings.rs` のアイコン設定画面にて、`IconRegistry::iter()` から取得したアイコンをベンダー(プレフィックス)ごとにグループ化して `egui::CollapsingHeader` に格納する。
- [ ] 2.2 グループ化された各ベンダーのアイコンプレビューに対して、`Icon::draw` に渡す色設定をベンダー別のデフォルト色で反映させる。
- [ ] 2.3 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.4 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] `katana-ui` の設定画面でベンダー単位でのプレビューと個別カラーが正しく描画されていること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. Advanced Icon Settings & Preset Management UI

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 各アイコンに「Advanced/Edit」ボタンを配置し、それを押すことで個別のベンダー切り替え・色指定ができるポップアップまたはダイアログUIを実装する。
- [ ] 3.2 カスタマイズ状態をプリセットとして名前をつけて保存(Save Preset As)、および読み込み(Load Preset)・初期化(Revert to Default)を行えるUIを組み込む。
- [ ] 3.3 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.4 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] プリセットと高度な個別設定がUI上で利用できること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.17.2`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
