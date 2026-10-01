# Dependency audit evidence

## Re-audit on 2026-10-01 (current graph)

- `cargo update --dry-run --verbose`: zero compatible lock updates.
- `cargo upgrade --dry-run --incompatible allow --pinned allow --verbose`: the only direct updates are the explicitly pinned egui family 0.36.1 to 0.36.2; no additional major migration was identified.
- `bun outdated` and `bun outdated --json`: zero JavaScript updates. Bun 1.4.2, MathJax 4.1.3, TypeScript 7.0.2 and `@types/bun` 1.4.2 remain current.
- Published siblings adopted in the current graph: KDV 0.5.7, KRR 0.4.21, KUC 0.3.17, KMM 0.2.3, katana-ast-lint 0.5.2 and KML 0.19.5. The graph contains one V8 version, 152.2.0. The historical KUC 0.3.3 and duplicate-V8 holds below no longer describe this graph.
- `cargo outdated` still fails because its temporary workspace omits the local `vendor/egui-winit` override. Its failed run is not an update result; the two Cargo dry runs use the real graph instead.
- egui 0.36.2 migration is implemented after clean checkpoint `85017694`. The root, UI dev dependency, screenshot and paint-metrics pins/locks are synchronized. The input adapter matches official0.36.2 except the two retained JIS mappings; window settings and dropped-file sources match exactly. No sibling path/git dependency is introduced.
- Full locked metadata succeeds for the root and screenshot graphs; both resolve egui/egui_kittest0.36.2. Native release all-workspace/all-target strict Clippy and the two paint-metrics tests pass. Post-migration whole coverage, input/capture acceptance and combined gates remain pending.
- After migration, `cargo update --dry-run` locks zero packages; `cargo upgrade --dry-run --incompatible allow --pinned allow` reports63 latest packages and no proposed direct update. `bun outdated --json` reports no JavaScript update. `generic-array`0.14.9 remains constrained by transitive upstream requirements.
- The resolver initially unified screenshot swash0.2.10 with an older allowed skrifa. Because capture must use the same font parser as the application, its existing public0.44.0 edge is retained in both locks; swash's upstream range is `>=0.31.1, <=0.44`. Full `cargo metadata --locked` accepts the aligned graph without a source override.
- `just check-full` now passes formatting and workspace strict Clippy and is running fixture integration tests. Completion of the full gate, vendor migration and packaged acceptance is not yet claimed.

Earlier sections are historical audit evidence, not current blocker status.

## Re-audit on 2026-09-13

- Adopted compatible registry updates `ureq 3.4.2`,
  `katana-markdown-model 0.2.2`, `comrak 0.55.0`, `open 5.4.4`,
  `uuid 1.26.1`, and the remaining compatible transitive lock updates.
- Updated `@types/bun` to 1.4.2. `bun outdated` reports no remaining
  JavaScript updates; TypeScript checking, bundle generation, and a real
  `x^2 + y^2` to SVG call pass. Two consecutive bundle generations produce
  SHA-256 `8352ceae524e4b592c8dd69f30caac0b64c15e72cec6e4718154c144a0faff99`.
- The locked KatanA core/UI check passes. The filtered HTML/Markdown regression
  run passes 84 tests across 17 suites.
- comrak 0.55 contains upstream autolink/GFM denial-of-service fixes and does
  not require a KatanA API migration.
- `cargo update --dry-run --verbose` reports only the deliberately held KUC
  0.3.10 update after the refresh. `dirs 7`, `generic-array 0.14.9`, and V8
  150.4 remain outside the current dependency requirements or are
  upstream-owned.
- Public KDV 0.5.5 declares a range that admits KUC 0.3.10, but that graph does
  not compile: KDV constructs `GridCellAppearance` without KUC's new required
  `borders` field. The temporary public-graph lock therefore retains KUC 0.3.3;
  KDV 0.5.6 must adopt and compile against the exact fixed KUC registry release
  before KatanA removes this compatibility hold.
- After restoring the known-compatible KUC graph, locked all-target/all-feature
  `cargo check` and Clippy with warnings denied pass. The refreshed supply-chain
  gate also passes all advisory, ban, license, and source checks.
- egui 0.36.2 is compatible with the workspace MSRV but cannot be applied as a
  lockfile-only change. The vendored `egui-winit` must first be synchronized
  to upstream 0.36.2 while preserving only the two KatanA JIS physical-key
  mappings, and eframe/egui companion crates must resolve together. Until that
  checkpoint, the 0.36.1 family is explicitly pinned and `cargo tree` confirms
  that `eframe` resolves the vendored `egui-winit` rather than registry 0.36.2.
- The egui vendor synchronization is deferred until a verified checkpoint
  commit, as required for a multi-file vendor replacement.

## Re-audit on 2026-09-07

- Updated the MathJax toolchain requirements and lockfile to Bun 1.4.2 and
  `@types/bun` 1.4.1; retained MathJax 4.1.3 and TypeScript 7.0.2.
- `bun install`, `bun run typecheck`, and `bun run build` passed.
- `bun outdated` reports no remaining JavaScript updates.
- Both bundle generations produced SHA-256
  `8352ceae524e4b592c8dd69f30caac0b64c15e72cec6e4718154c144a0faff99`.
- Rust `cargo update --dry-run --verbose` now reports 30 compatible updates,
  including KUC 0.3.7 and katana-markdown-model 0.2.2. These are candidates,
  not adopted or verified updates; the earlier zero-update audit is historical.
- `cargo upgrade --dry-run --incompatible allow --pinned allow` additionally
  identifies comrak 0.55 and dirs 7.0.0 for migration evaluation.
- KDV 0.5.6 and the required KRR fixes remain unpublished on GitHub. Exact
  registry adoption, Rust dependency updates, full host acceptance, and the
  unchanged full quality gates remain release requirements.

The JavaScript checks establish toolchain and bundle generation only, not
packaged KatanA rendering acceptance. No release completion is claimed.

## KatanA-owned updates

The 2026-08-29 release audit updated the remaining independently adoptable
requirements:

- `flate2` `1.1.9` to `1.1.10`.
- `uuid` `1.25.0` to `1.26.0`.
- `open` `5.4.2` to `5.4.3` after the newer compatible patch release became
  visible during the final dependency re-audit.
- The direct test-only `accesskit 0.24.1` dependency was removed. Tests now use
  `egui::accesskit`, which guarantees that query roles use the same AccessKit
  type as the current `egui 0.36.1` stack. Adding standalone AccessKit 0.25
  would create a second incompatible type graph while egui still publishes
  against 0.24.
- The MathJax build remains on Bun `1.4.0`, TypeScript `7.0.2`,
  `@mathjax/src 4.1.3`, and `@types/bun 1.4.0`.

## Audit results

- The 2026-09-02 re-audit refreshed newly published compatible lock entries,
  including `katana-ui-core 0.3.3`, `hermit-abi 0.5.3`, `libredox 0.1.23`,
  `ppmd-rust 1.4.1`, `rust_decimal 1.43.0`, and `smallvec 1.16.0`.
- The post-update `cargo update --dry-run --verbose` reports zero compatible
  lockfile updates. `generic-array 0.14.9` and `v8 150.4.0` remain outside the
  resolved requirements; the latter is owned by the pending KDV replacement.
- `cargo upgrade --dry-run --incompatible allow --pinned allow`: zero remaining
  direct manifest updates after the changes above; 63 dependencies reported at
  their latest published requirement and local vendored packages identified as
  local.
- `bun outdated`: zero outdated JavaScript dependencies.
- `cargo check --workspace --all-targets --locked`: passed after the re-audit.
- `just lint`: passed with Clippy warnings denied.
- `just supply-chain`: advisories, bans, licenses, and sources passed; existing
  duplicate-package notices remain warnings and duplicate V8 remains a separate
  hard release failure.
- `cargo check -p katana-ui --all-targets`: passed after removing the duplicate
  direct AccessKit dependency.
- `cargo test -p katana-ui --all-targets --no-fail-fast`: exit code 0 with
  1,051 passed and 6 explicitly ignored external/manual tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo deny check --hide-inclusion-graph`: advisories, bans, licenses, and
  sources all passed. Duplicate-package notices remain warnings; the release
  contract independently makes duplicate V8 a hard failure.

`cargo outdated` itself cannot analyze this workspace because its temporary
workspace copy omits the repository's `vendor/egui-winit` path override. The
failure is a tool/workspace-copy limitation, not an outdated-package result;
the two dry-run Cargo audits above preserve the real workspace graph.

## egui 0.36.2移行の再検証（2026-10-01）

root、screenshot、paint-metrics、vendorのlockと全直接pinを0.36.2へ同期した。公式vendorからのsource差分は既存JISマッピング2箇所のみで、独立最終レビューのP0/P1は0件。root/screenshotのswash0.2.10は公開skrifa0.44.0に揃え、full locked metadataが受理する。git sourceはない。

移行後のnative release全target strict Clippy、paint-metrics2件、供給網4カテゴリ、変更していない全coverage gateが成功。strict document surface100%、painter_grid184/184。証跡は`tmp/coverage-egui-0362-20261001.{log,json}`。compatible lock更新0件、直接依存更新0件、JS outdated0件を再確認した。新しい実mainのreleaseビルドと空workspace起動も成功したが、実入力再受入、platform/combined gate、公開上流の追加採用は完了扱いにしない。

## Remaining release boundary

Task 4.7 remains open until actual input/capture, platform and combined gates
pass after the verified egui migration.
The older duplicate-V8 evidence in `v8-dependency-link-gate.md` is historical;
the adopted KDV 0.5.7 / KRR 0.4.21 graph contains only V8 152.2.0.
