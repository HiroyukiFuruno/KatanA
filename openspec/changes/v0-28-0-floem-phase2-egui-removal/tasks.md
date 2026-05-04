# Tasks: v0.28.0 Floem Phase 2（egui 完全除去）— KatanA

> chrome（toolbar / sidebar / split pane / tab bar）の Floem 実装と eframe 完全除去。
> 詳細設計は着手時に確定する（計画未定）。

## Branch Rule

`release/v0.28.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] v0.27.0（Floem Phase 1 intake）が完了していること
- [ ] Floem の taffy レイアウトで KatanA chrome 構造が再現できる見通しが立っていること
- [ ] 詳細設計（design.md）が本 change 着手時に確定していること

---

## タスク詳細

着手時に design.md を確定させてから tasks を追記する。

現時点での確定スコープ：

- [ ] eframe アプリループを Floem のウィンドウ・イベントループに完全置き換える
- [ ] toolbar / sidebar / split pane / tab bar を taffy + vello で実装する
- [ ] `Cargo.toml` から `egui`、`eframe`、`egui_*` 系依存を全て除去する
- [ ] `cargo tree` で egui / epaint / eframe が含まれないことを確認する
- [ ] `just check` がエラーなし（exit code 0）で通過すること
- [ ] `release/v0.28.0` ブランチから PR を作成し master へ merge する
