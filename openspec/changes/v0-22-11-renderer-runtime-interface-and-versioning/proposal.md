## Why

v0.22.10 で Rust 管理 JS（Rust-managed JavaScript runtime）による Mermaid 描画が高速な採用候補になった。一方、KatanA 本体が Mermaid.js の DOM/SVG 互換、版（version）管理、Draw.io 描画、HTML/PDF/PNG/JPEG export、公式比較画像と採点まで抱える状態は、責務境界として重い。これらを v0.22.11 で「文書化と interface 整理だけ」に留めると、後続の v0.22.12（document viewer）・v0.22.13（PDF export pagination）・v0.31.0（diagram backend adapters）が同じ責務曖昧さの上に積み上がり、再分離コストが膨らむ。

KML（katana-markdown-linter）を別 repository へ完全分離したのと同じ判断を、描画と export にも適用する。v0.22.11 では `katana-canvas-forge`（kcf）を **新 repository として実構築し、KatanA は library として利用する**。Mermaid 描画の実コードと vendor 管理を kcf 側へ移し、KatanA 側は描画 runtime を所有しない構造へ移行する。

## What Changes

### 新 repository `katana-canvas-forge` (kcf) を確立する

- `https://github.com/HiroyukiFuruno/katana-canvas-forge` を public で初期化済み（init scaffold 完了）。
- workspace 構成: `crates/katana-canvas-forge`（library）+ `crates/katana-canvas-forge-cli`（`kcf` CLI）。
- 描画 runtime interface と Mermaid backend の実コード、Mermaid.js 版固定、checksum、公式比較画像、採点評価、保存時チェック、CI/CD 検証は kcf が所有する。
- v0.31.0 で議論されていた Mermaid backend adapter 選定（`merman` / `mermaid-rs-renderer` / `selkie-rs`、Rust 管理 JS）と PlantUML 系（`plantuml-little` 他）も kcf 側の責務へ吸収する（PlantUML は kcf 内で別 backend として扱う）。

### KatanA 本体は kcf を library として consume する

- 現在の `crates/katana-core/src/markdown/mermaid_renderer/` 実装は kcf へ移管し、KatanA 側は kcf の trait（`Renderer`）と DTO（`RenderInput` / `RenderOutput` 等）を呼ぶだけにする。
- `vendor/mermaid/` および `mermaid.min.js` 関連 asset、関連 just recipe、scripts、公式比較画像 fixture（`assets/fixtures/mermaid_all/`）、検証 test の所有を KatanA から kcf へ移す。
- KatanA は描画 cache の保存先、Markdown block 抽出、テーマ snapshot、preview / export UI、cache fingerprint 組み立てだけを残す。
- Cargo workspace から kcf を git 依存または path 依存（開発時）で参照する。リリース版は git tag pinned 依存に揃える。

### Draw.io と HTML / PDF / PNG / JPEG export も kcf 側へ移管する

- Draw.io 描画 runtime、HTML export、HTML→PDF/PNG/JPEG 変換は kcf の Renderer 系統または `Exporter` 系統として kcf 側で実装する。
- KatanA 側に残るのは「export 操作の UI（メニュー、保存ダイアログ、preview tab）」「export 入力の組み立て」「結果ファイルの保存先決定」のみ。
- 未接続部分は黙って OS Chrome / Chromium app 依存に戻さず、kcf 側で `NotImplemented` 系 diagnostic として明示する。

### 関連文書とプロセスを kcf へ移す

- 公式比較画像更新手順、採点評価方針、保存時 pre-commit hook、CI/CD での採点検証は kcf 側 `docs/` と `.github/workflows/` で運用する。
- KatanA 側 design ドキュメントには「kcf を参照」のリンクのみ残す。

### 後続 versioned change との関係

- v0.22.12（document-viewer-expansion）・v0.22.13（PDF export pagination preview）は **kcf 境界が確立した前提**で再記述する。document viewer は markdown renderer とは別責務として KatanA 側に残置するが、PDF / HTML 関連は kcf 経由に揃える。
- v0.26.0（decouple-preview）・v0.27.0（decouple-editor）は **kcf を library 利用する**前提で構造を整理する。preview crate も kcf に依存する側に固定する。
- v0.31.0（native-diagram-renderer-backends）は Mermaid 範囲を kcf に吸収。残された PlantUML 範囲だけが change として残るか、kcf 側の roadmap として archive されるかを v0.22.11 内で判断する。

## Capabilities

### New Capabilities

- `renderer-runtime-interface`: KatanA と kcf の接続境界、Mermaid.js 版固定、Draw.io / export 所有境界を扱う。実装本体は kcf 側にある前提で、KatanA 側 spec は consume 側の契約を定義する。

### Modified Capabilities

- `diagram-block-preview`: Mermaid / Draw.io preview は KatanA 内部実装ではなく、kcf library 経由の結果として扱う。
- `markdown-export`: HTML / PDF / PNG / JPEG export は kcf の export runtime 経由で行い、KatanA 側は UI と入力組み立てのみ所有する。

## Impact

### KatanA 側

- 削除/移管: `crates/katana-core/src/markdown/mermaid_renderer/`、`crates/katana-core/src/markdown/drawio_renderer/`、`crates/katana-core/src/markdown/export/` の実装本体
- 削除/移管: `crates/katana-core/tests/markdown_mermaid.rs`、`markdown_drawio.rs`、`markdown_svg_rasterize.rs` の renderer 内部 test
- 削除/移管: `scripts/mermaid/`、`assets/fixtures/mermaid_all/`、関連 just recipe
- 追加: `Cargo.toml` に `katana-canvas-forge` 依存（git tag pinned）
- 追加: KatanA 側の薄い adapter（kcf DTO ↔ KatanA preview/export 状態の変換）
- KatanA 側の cache key には kcf の `RuntimeVersion` と `RendererProfile` を必ず含める

### `katana-canvas-forge` 側（KatanA repository 外、別 PR）

- 上記移管対象一式の受け入れ
- Mermaid runtime interface 実装、Mermaid.js 版固定、checksum、CLI、公式比較画像更新と採点
- Draw.io / export backend
- CI（fmt / clippy / test / 採点）

### 後続 change

- v0.22.12 / v0.22.13 / v0.26.0 / v0.27.0 / v0.31.0 の design.md に kcf 境界に従う旨の追記
