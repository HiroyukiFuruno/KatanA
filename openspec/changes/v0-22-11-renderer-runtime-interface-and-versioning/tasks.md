# Tasks: v0.22.11 Renderer Runtime Interface — KatanA intake

> 描画実装・版管理・採点・CLI はすべて `katana-canvas-forge`（kcf）側で行う。
> kcf 側の実装タスクは [katana-canvas-forge openspec](https://github.com/HiroyukiFuruno/katana-canvas-forge) を参照。
> 本 tasks.md は KatanA 側の intake（git dependency 追加 + adapter 整理）のみを扱う。

## Branch Rule

interface 整理リファクタリングとして `master` で直接作業する（バージョンブランチ不要）。

---

## 準備完了条件（Definition of Ready）

- [ ] kcf `v0.1.0` release tag が切られていること（[katana-canvas-forge releases](https://github.com/HiroyukiFuruno/katana-canvas-forge/releases)）
- [ ] kcf の `Renderer` trait と DTO（`RenderInput` / `RenderOutput` / `RenderConfig` / `RenderPolicy` / `RenderContext` / `RenderDiagnostics` / `RuntimeVersion` / `RendererProfile`）が確定していること

---

## 1. kcf を KatanA の git dependency として追加する

- [ ] 1.1 root `Cargo.toml` に `katana-canvas-forge = { git = "https://github.com/HiroyukiFuruno/katana-canvas-forge", tag = "v0.1.0" }` を追加する
- [ ] 1.2 `cargo build` が通ることを確認する
- [ ] 1.3 `cargo tree` で kcf 側に `egui` が含まれないことを確認する

---

## 2. KatanA 側 Mermaid preview を kcf library 経由に切り替える

### 準備完了条件

- [ ] Task 1 完了

- [ ] 2.1 KatanA preview の Mermaid block 描画を kcf の `Renderer` trait 経由に切り替える（薄い adapter のみ KatanA 側に残す）
- [ ] 2.2 KatanA cache key に kcf の `RuntimeVersion` と `RendererProfile` を含める
- [ ] 2.3 `RuntimeVersion` が変わったとき KatanA 側 cache が無効化されることを確認する
- [ ] 2.4 `crates/katana-core/src/markdown/mermaid_renderer/` の実装本体が残っていないことを `git grep` で確認する

---

## 3. 移管完了の確認

- [ ] 3.1 `vendor/mermaid/`、`scripts/mermaid/`、`assets/fixtures/mermaid_all/` が KatanA 側から除去されていることを確認する（kcf 側へ移管済み）
- [ ] 3.2 Draw.io / export 実装本体が KatanA 側に残っていないことを `git grep` で確認する（kcf 側へ移管済み）
- [ ] 3.3 KatanA 側 docs に kcf docs へのリンクを追加する

---

## 4. 検証と commit

- [ ] 4.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 4.2 `./scripts/openspec validate v0-22-11-renderer-runtime-interface-and-versioning --strict` を実行し OpenSpec の整合性を確認する
- [ ] 4.3 commit & push（`master` 直接）
