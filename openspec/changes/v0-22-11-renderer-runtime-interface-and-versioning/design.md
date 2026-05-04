## Context

v0.22.10 では、OS の Chrome / Chromium app に依存しない高速な Mermaid 描画として Rust 管理 JS を評価した。表示互換は公式 Mermaid.js 利用で見込めるが、KatanA 本体が Mermaid.js の DOM/SVG 互換、版管理、Draw.io 描画、HTML/PDF/PNG/JPEG export、公式比較画像、採点評価まで抱える状態は、責務として広すぎる。

KML を別 repository へ完全分離した判断と同じ枠で、描画と export を `katana-canvas-forge`（kcf）へ完全分離する。v0.22.11 は「文書化と interface 整理」ではなく「実 repository 構築 + library 利用への移行」を目的とする（B 案）。

## Goals

- `katana-canvas-forge` を実 repository として確立し、library として KatanA から consume する。
- KatanA から Mermaid 描画実装、Draw.io 描画実装、HTML/PDF/PNG/JPEG export 実装を kcf 側へ移す。
- KatanA 側に残る責務は、Markdown block 抽出、テーマ snapshot、cache 保存先、preview/export UI、kcf へ渡す入力の組み立て、結果の表示と保存に限定する。
- 描画 cache key は kcf の `RuntimeVersion` と `RendererProfile` を必ず含め、kcf 側の更新が KatanA 側 cache を無効化できるようにする。
- 公式比較画像更新、採点評価、保存時 pre-commit、CI/CD 検証は kcf 側で運用する。
- v0.31.0 で議論されていた Mermaid backend 候補選定と PlantUML 候補選定も kcf 側 roadmap に吸収する。
- v0.22.12 / v0.22.13 / v0.26.0 / v0.27.0 が kcf 境界の上に乗る形で再記述できる土台を、本 change 完了時点で提供する。

## Non-Goals

- KatanA UI の preview / editor crate 分離は本 change で完了させない（v0.26.0 / v0.27.0 の責務）。
- LLM chat UI / ACP integration は本 change で扱わない（v0.22.14 の責務）。
- KatanA 側に renderer fallback path（OS Chrome / Chromium app への退避）を実装しない。

## 新 repository

- URL: `https://github.com/HiroyukiFuruno/katana-canvas-forge`（public、init scaffold push 済み）
- 構成:
  - `crates/katana-canvas-forge` — library
  - `crates/katana-canvas-forge-cli` — `kcf` CLI binary
  - `vendor/mermaid/<version>/mermaid.min.js` + `.sha256`
  - `docs/` — 設計、移管手順、採点ポリシー
  - `.github/workflows/ci.yml` — fmt / clippy / test
- Cargo workspace は `edition = "2024"`、`rust-version = "1.95.0"` で KatanA と整合。

## KatanA からの利用方法

- `Cargo.toml` の workspace dependencies に `katana-canvas-forge = { git = "https://github.com/HiroyukiFuruno/katana-canvas-forge", tag = "v0.0.x" }` を追加する。
- 開発時のみ `path = "../katana-canvas-forge/crates/katana-canvas-forge"` を `[patch.crates-io]` で当てるかは別途判断（KML と同じ運用方針に揃える）。
- KatanA 側は kcf の `Renderer` trait / DTO を使う薄い adapter のみ持つ。

## 接続境界

KatanA から kcf へ渡す入力は、次の 4 層に分ける（DTO は kcf 側で定義済み）。

1. `source`: 図形の生テキスト（Mermaid / Draw.io）。
2. `config`: vendor 互換 config（Mermaid.js なら `theme` / `themeVariables` / `securityLevel` / `htmlLabels` / diagram-specific config）。
3. `policy`: 最大幅、最大高さ、余白、背景、cache profile などの KatanA 独自制約。
4. `context`: テーマ fingerprint、文書情報、診断 metadata。

戻り値は次を含む。

- 描画済み SVG、幅、高さ、viewBox
- `RuntimeVersion`（`name` / `version` / `checksum`）
- `RendererProfile`
- `RenderDiagnostics`（warnings / errors）
- `cache_fingerprint`

KatanA は cache key に `source` + `RuntimeVersion` + `RendererProfile` + `config` + `policy` + テーマ fingerprint を含める。

## Mermaid.js 版固定

無印 `~/.local/katana/mermaid.min.js` 利用は廃止する。kcf repository 内で次のように管理する。

```
vendor/mermaid/<version>/mermaid.min.js
vendor/mermaid/<version>/mermaid.min.js.sha256
```

更新入口は kcf 側 `just VERSION=<version> mermaid-js-update` 相当の recipe に集約し、KatanA 側からは更新コマンドを呼ばない。KatanA は kcf を git tag pinned で取り込むため、Mermaid.js 版変更は kcf release 単位でしか起きない。

## Draw.io と export

Draw.io 描画と HTML/PDF/PNG/JPEG export も kcf 側で次の方針で扱う。

- Draw.io が Mermaid と同じ `Renderer` trait に乗るか、別 `Renderer` 実装として並ぶかは kcf 側の実装判断（KatanA 側 spec はどちらでも consume できるよう DTO ベースで書く）。
- HTML / PDF / PNG / JPEG export は kcf 側に `Exporter` 系統を別途設けるか、`Renderer` の出力を post-process するかを kcf 側で決める。
- 未接続の経路は kcf 側で `RenderError::NotImplemented` 相当の diagnostic を返し、KatanA 側は OS Chrome / Chromium app への暗黙 fallback を持たない。

## v0.31.0 との統合

v0.31.0（native-diagram-renderer-backends）の Mermaid 範囲（`merman` / `mermaid-rs-renderer` / `selkie-rs` 候補評価、Rust 管理 JS）は本 change 完了時点で kcf 側の責務として吸収する。PlantUML 範囲（`plantuml-little` 等）も kcf 側 roadmap に組み込む方向。v0.31.0 自体は本 change 内で次のいずれかに整理する。

- archive 相当: 全責務が kcf 側へ吸収済みなら archive 候補として記録する。
- 残存: KatanA 側 UI 影響だけが残る場合は scope を絞って残す。

判断は本 change の Task 6（v0.31.0 整理）で行う。

## Preview / Editor 分離（v0.26.0 / v0.27.0）との関係

依存方向は次に固定する。

```
KatanA shell -> katana-document-preview -> katana-canvas-forge
KatanA shell -> katana-editor
katana-canvas-forge -> preview / editor / KatanA UI へは依存しない
```

preview crate（v0.26.0）は kcf を library として呼ぶだけにする。kcf は `egui` への直接依存を持たない（描画結果は SVG 文字列 + メタデータとして返す）。

## Risks

- 単一 release ブランチで repo 移管を行うため、長期作業になる。Branch Rule は `release/v0.22.11` だが、Task 単位での `feature/v0.22.11-task-x` 派生で逐次 merge し、master を緑に保つ。
- kcf 側に未だ Rust 管理 JS runtime の最終形が無い時点で移管を始めるため、kcf 側 PR と KatanA 側 PR が並走する。順序は「kcf 側で機能を追加 → tag → KatanA 側 update」を厳守する。
- v0.22.11 の長期化により v0.22.12 / v0.22.13 着手が遅れる可能性。並行作業は v0.22.14（chat UI / ACP）を優先する。
- KatanA → kcf 経由になることで、開発中の path 依存と release 時の git 依存切り替えで cargo lock 差分が増える。lock 戦略を design 段階で固める。

## Verification

- OpenSpec validation を strict で通す。
- KatanA 側 Cargo.toml が `katana-canvas-forge` を git tag pinned で参照していることを確認する。
- KatanA 側に Mermaid 描画 / Draw.io 描画 / HTML / PDF / PNG / JPEG export の実装本体が残っていないことを `git grep` 系で確認する。
- KatanA 側 cache key に `RuntimeVersion` / `RendererProfile` が含まれることを test で確認する。
- kcf 側 release が KatanA 側 cache を確実に無効化することを test または手順で示す。
- preview crate（v0.26.0 着手前段階）から kcf への依存方向が一方向であることを `cargo tree` で確認できる。
- v0.31.0 の責務整理結果を design.md または該当 change 側 design.md に記録する。
