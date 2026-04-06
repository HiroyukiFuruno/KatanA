## Tasks

- [ ] **1. Assets & Setup**
    - [ ] 1.1 `assets/icons/files/explorer.svg` の新規作成 (multi-files SVG)
    - [ ] 1.2 `crates/katana-ui/src/icon/types.rs` への `Icon::Explorer` 登録
- [ ] **2. Refactoring - State & Actions**
    - [ ] 2.1 `LayoutState` の `show_workspace` -> `show_explorer`
    - [ ] 2.2 `AppAction` の `ToggleWorkspace` -> `ToggleExplorer`
- [ ] **3. Refactoring - Directory & Components**
    - [ ] 3.1 `sidebar/workspace` -> `sidebar/explorer` へのディレクトリ移動
    - [ ] 3.2 サイドバーアイコンとツールチップの更新
- [ ] **4. i18n & Verification**
    - [ ] 4.1 全ローケルファイルの `workspace_title` -> `explorer_title` への修正（エクスプローラー用）
    - [ ] 4.2 `workspace_history_title` の新設（ワークスペース用）
    - [ ] 4.3 ビルドと UI の確認

## Branch Rule
(なし)

## Release Process
(なし)
