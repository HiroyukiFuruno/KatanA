## ADDED Requirements

### Requirement: KatanA chrome は Floem に完全移行し egui / eframe 依存を持たないようにしなければならない

システムは、toolbar / sidebar / split pane / tab bar 等の UI chrome を Floem（taffy + vello + cosmic-text + winit）で実装し、`egui` / `eframe` / `egui_extras` / `egui_*` 系依存を `Cargo.toml` から完全に除去しなければならない（MUST）。

#### Scenario: eframe アプリループを Floem に置き換える

- **WHEN** KatanA を起動する
- **THEN** ウィンドウ・イベントループは Floem（winit 直接利用）が管理する
- **THEN** `eframe::run_native` 系の呼び出しは KatanA に存在しない

#### Scenario: chrome を taffy + vello で実装する

- **WHEN** KatanA UI を表示する
- **THEN** toolbar、sidebar、split pane、tab bar は Floem の view と taffy layout で構築される
- **THEN** chrome 描画は vello scene を経由する

#### Scenario: egui / eframe 依存をゼロにする

- **WHEN** `cargo tree --workspace` を実行する
- **THEN** `egui`、`eframe`、`egui_extras`、`egui_*` 系の crate が一つも含まれない
- **THEN** `cargo build --workspace` は egui ゼロの状態で通過する
