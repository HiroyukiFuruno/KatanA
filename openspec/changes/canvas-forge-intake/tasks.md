# Tasks: katana-canvas-forge intake — KatanA

> Mermaid / Draw.io / export 実装はすべて `katana-canvas-forge`（kcf）repo 側で行う。
> kcf 側の実装タスクは [katana-canvas-forge openspec](https://github.com/HiroyukiFuruno/katana-canvas-forge) を参照。
> 本 tasks.md は KatanA 側の refactoring intake（git dependency 追加 + 描画実装除去 + kcf API 呼び出しへの差し替え）のみを扱う。

## Branch Rule

利用者向けの機能差分を持たない内部リファクタリングとして、`master` で直接作業する。

---

## 準備完了条件（Definition of Ready）

- [x] `katana-canvas-forge` v0.1.0 が crates.io に公開されていること
- [x] `Renderer` trait と DTO（`RenderInput` / `RenderOutput` / `RuntimeVersion` / `RendererProfile`）が確定していること
- [x] `katana-canvas-forge`（neutral interface）が egui を含まないことを確認していること

---

## 1. katana-canvas-forge を crates.io dependency として追加する

- [x] 1.1 root `Cargo.toml` に以下を追加する
  ```toml
  katana-canvas-forge = "0.1.0"
  ```
- [x] 1.2 `cargo build` が通ること
- [x] 1.3 `cargo tree` で `katana-canvas-forge` に `egui` が含まれないことを確認する

---

## 2. KatanA 側描画を kcf 経由に切り替える

### 準備完了条件

- [x] Task 1 完了

- [x] 2.1 Mermaid block 描画を kcf の `Renderer` trait 経由に切り替える（薄い adapter のみ残す）
- [x] 2.2 Draw.io 描画を kcf の `Renderer` 経由に切り替える
- [x] 2.3 HTML / PDF / PNG / JPEG export を kcf の `Exporter` 経由に切り替える
- [x] 2.4 cache key に kcf の `RuntimeVersion` と `RendererProfile` を含める

---

## 3. KatanA 側の描画実装を除去する

### 準備完了条件

- [x] Task 2 完了

- [x] 3.1 `crates/katana-core/src/markdown/mermaid_renderer/` を除去する
- [x] 3.2 `crates/katana-core/src/markdown/drawio_renderer/` を除去する
- [x] 3.3 `crates/katana-core/src/markdown/export/` 実装本体を除去する
- [x] 3.4 `vendor/mermaid/`、`scripts/mermaid/`、`assets/fixtures/mermaid_all/` を除去する
- [x] 3.5 `rg -n "mermaid_renderer|drawio_renderer" crates scripts just Justfile biome.jsonc .github` で KatanA のコード・運用定義に直接参照が残っていないことを確認する

---

## 4. 検証

- [ ] 4.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 4.2 `./scripts/openspec validate canvas-forge-intake --strict` がエラーなし（exit code 0）で通過すること
