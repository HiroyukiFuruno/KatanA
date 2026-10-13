## Definition of Ready (DoR)

- [x] proposal.md が作成されていること
- [x] kcf 0.1.7（v8 = "=147.4.0"）が crates.io に公開されていること（[kcf #15](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/15) 対応済み）
- [x] kdr 0.1.0 が crates.io に公開されていること
- [x] kdr 描画 / kcf export の責務分離方針が確定していること（export 機能は kdr に無いので kcf を継続利用）

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `release/v0.22.19`
- **作業ブランチ**: `feature/v0.22.19-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

## Status

- [x] 本 change の実装と検証は `v0-22-19-kcf-v8-alignment` に統合済みであり、同 change の Verification Log を実行記録として扱う
- [x] 本 docs-only 追従コミットでは、未追跡だった本 change を PR に含め、kdr 描画 / kcf export の責務分離を OpenSpec 上で読める状態にする

## 1. workspace dependency 整合

- [x] 1.1 `Cargo.toml` の `[workspace.dependencies]` に `katana-diagram-renderer = "0.1.0"` を追加する
- [x] 1.2 `Cargo.toml` の `[workspace.dependencies]` の `katana-canvas-forge` を `0.1.7` に更新する
- [x] 1.3 `Cargo.toml` の `[workspace.dependencies]` の `v8` を `"=147.4.0"` に更新する
- [x] 1.4 `crates/katana-core/Cargo.toml` の `[dependencies]` に `katana-diagram-renderer.workspace = true` を追加する（`katana-canvas-forge.workspace = true` は export 用に残置）
- [x] 1.5 `cargo update -p katana-canvas-forge -p katana-diagram-renderer -p v8` で `Cargo.lock` を整合させる

## 2. 描画系の kdr 移行

- [x] 2.1 `crates/katana-core/build.rs` を `KATANA_CANVAS_FORGE_VERSION` → `KATANA_DIAGRAM_RENDERER_VERSION` へ切替し、`lock_package_version` の対象を `katana-diagram-renderer` に変更する
- [x] 2.2 `crates/katana-core/src/markdown/diagram_runtime_assets.rs` の `katana_canvas_forge::{RuntimePathResolver, DiagramKind}` を `katana_diagram_renderer::` へ切替する（`kcf_kind` → `kdr_kind` 含む）
- [x] 2.3 `crates/katana-core/src/markdown/diagram_backend/kcf_theme_adapter.rs` を `kdr_theme_adapter.rs` にリネームし、`KcfThemeAdapter` → `KdrThemeAdapter` / `katana_canvas_forge::` → `katana_diagram_renderer::` へ切替する
- [x] 2.4 `crates/katana-core/src/markdown/diagram_backend/mod.rs` の `mod kcf_theme_adapter` を `mod kdr_theme_adapter` に変更する
- [x] 2.5 `crates/katana-core/src/markdown/diagram_backend/katana_backend.rs` の renderer import / バックエンド ID（`kcf-mermaid` / `kcf-drawio` → `kdr-mermaid` / `kdr-drawio`）/ 関数名（`kcf_input` → `kdr_input`、`kcf_error_to_backend` → `kdr_error_to_backend`）/ 定数（`KCF_*` → `KDR_*`）を更新する
- [x] 2.6 `crates/katana-core/src/markdown/diagram_backend/impls.rs` の `DiagramBackendVersion::from_kcf` を `from_kdr` にリネームし、version 文字列フォーマットを `crate=katana-diagram-renderer:{crate_version};...` に変更する
- [x] 2.7 `crates/katana-core/src/markdown/diagram_backend/tests.rs` の `kcf-mermaid` / `from_kcf` / `katana-canvas-forge` の参照を `kdr-*` / `from_kdr` / `katana-diagram-renderer` に揃える
- [x] 2.8 export 系（`crates/katana-core/src/markdown/export/mod.rs`）は kcf 残置のまま変更しないことを確認する（HtmlExporter / ImageExporter / PdfExporter / `markdown::color_preset::DiagramColorPreset` は kcf 0.1.7 経由）

### Definition of Done (DoD)

- [x] `just type-check` が成功していること
- [x] `cargo test -p katana-core diagram_backend -- --nocapture` が成功していること（kdr-mermaid / kdr-drawio backend / from_kdr 系を含む）
- [x] `cargo test -p katana-ui --test ui_integration_serial diagram_rendering -- --test-threads=1 --nocapture` が成功していること
- [x] `./scripts/screenshot/run.sh --request scripts/screenshot/examples/v0-22-14-light-diagrams.json --output tmp/v0-22-19-kcf-v8-alignment-screenshot` が成功していること
- [x] `Diagram render worker disconnected before producing a result.` が再現しないことをスクリーンショット生成で確認していること

## 3. docs / openspec 追従

- [x] 3.1 `katana/openspec/project.md` の component map / roadmap を kdr 描画 + kcf export の二本立てに更新する
- [x] 3.2 `katana/openspec/specs/diagram-block-preview/spec.md` の `kcf backed` / `katana-canvas-forge` 記述を kdr に置き換える
- [x] 3.3 `katana/openspec/changes/canvas-forge-intake/specs/diagram-block-preview/spec.md` を kdr 経由要件に更新し、past intake から kdr 分離への経緯を Note で残す
- [x] 3.4 `katana/openspec/changes/establish-kme-markdown-platform/`（design / proposal / handoff / tasks / kme-markdown-platform spec）を kdr 描画 + kcf export の二本立てに更新する
- [x] 3.5 `katana/openspec/changes/v0-26-0-floem-phase1-intake/design.md` の境界記述を kdr / kcf 並記に更新する
- [x] 3.6 `katana/openspec/changes/adopt-kme-in-katana/proposal.md` および `katana/openspec/changes/extract-katana-ast-lint/{proposal,tasks}.md` の consumer 列に kdr を追加する
- [x] 3.7 周辺リポジトリ docs を追従更新する:
  - `katana-chat-ui/README.md`（Diagram rendering → kdr / Document export → kcf に分割）
  - `katana-markdown-model/docs/responsibility-boundary.md`
  - `katana-markdown-model/openspec/project.md`
  - `katana-ast-lint/openspec/project.md`
  - `katana-document-viewer/openspec/changes/v0-1-0-document-preview-extraction/tasks.md`

### Definition of Done (DoD)

- [x] `./scripts/openspec validate "v0-22-19-kdr-intake-v8-alignment" --strict` が pass すること
- [x] アクティブな OpenSpec から kdr 描画 / kcf export の二本立てを読み取れること
- [x] 周辺リポジトリ docs は本 change の要求として列挙し、KatanA 側の PR では KatanA repository 内の OpenSpec 差分だけを対象にすること

---

## 4. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [x] 4.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする。Mermaid 描画失敗（Venn / Wardley / XY Chart / ZenUML 等）が解消されたことを示すスクリーンショットを残す。
  - 生成物: `tmp/v0-22-19-kcf-v8-alignment-screenshot/01-light-diagram-preview.png`
- [x] 4.2 ユーザーから受けたフィードバックを本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）。
  - 追加フィードバック: 未追跡だった本 OpenSpec change も PR に含める

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 5. Final Verification & Release Work

- [x] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` の観点で自己レビューを実行する
- [x] 5.2 更新した Markdown 文書を整形し、lint 修正（lint-fix）を行う
- [x] 5.3 docs-only 差分だけを staging し、別件のコード差分を含めないことを確認する
- [x] 5.4 ユーザー指示に従い、docs-only 追従コミットは `--no-verify` で commit / push する
- [x] 5.5 `release/v0.22.19` から `master` 向けの PR に本 change を含める

## Post-PR Release Follow-up

PR 作成後の CI 確認、ユーザー承認後の merge、GitHub Release 完了確認、`/opsx-archive`、katana [#293](https://github.com/HiroyukiFuruno/KatanA/issues/293) の close は、この docs-only 追従コミットの完了条件ではなく PR 作成後のリリース運用で扱う。
