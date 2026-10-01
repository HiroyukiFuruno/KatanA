# Dependency audit evidence

## Published KDV batch adoption on 2026-10-01

- Adopted exact registry KDV `=0.5.8` after successful release run36834399570, non-draft GitHub Release and actual downloaded crate checksum verification: `d77723a4ae0fb4dfe267810a23c77ba558c2d7dd4293539228d1f62c5511b5c0`. Its published dependency manifest uses registry KUC `=0.4.0` and KRR `^0.4.20`, without dependency path/git overrides. KatanA resolves KRR `=0.4.21` and singleton V8 `152.2.0`; the render dependency contract passes.
- Root and screenshot locks additionally update compatible `lazy_static`1.5.1 and `glam`0.33.12. Both compatible Cargo update dry runs lock zero further packages. Complete incompatible/pinned manifest upgrade dry runs exit0, reporting64 current root packages and10 current screenshot packages, with no direct update proposal.
- `generic-array`0.14.9 was evaluated but rejected by the actual resolver: `crypto-common`0.1.7 requires exactly0.14.7 through cipher/aes/umya/office2pdf. The existing upstream constraint remains; no override or quality relaxation is introduced.
- The actual repository JavaScript package is `vendor/mathjax_svg/js`, not the ancestor Bun workspace. Its `bun outdated` exits0 with no update and `bun run typecheck` passes. The existing Biome tooling pin is already2.5.15. The maintained direct font parser is `skrifa`0.47.0; historical0.44.0 references below do not describe that direct edge.
- New graph targeted verification passes: real Office-worker document surface68 tests, test-inclusive UI/core strict Clippy and all four supply-chain categories. The batch border projection uses a single borrowed entry traversal, preserves typed errors and existing paint count/coordinate rejection, and adds4096-entry/all-side/order and late-invalid-entry regressions. Full post-adoption coverage, platform, real-input and packaged acceptance remain required, separately from these targeted results.

## Re-audit on 2026-10-01 (current graph)

- Later heartbeat recheck: `cargo upgrade --dry-run --incompatible allow --pinned allow` exits0 and reports63 latest packages with no direct upgrade proposal. Root `bun outdated` discovered Biome2.5.15, but this worktree has no package.json: Bun's ancestor-package result is not a KatanA package/lock audit. The actual KatanA tooling pin is `Justfile`'s Biome2.4.13. Updated that pin and `biome.jsonc` to2.5.15 using the official migration preview; the promoted file-length rule moves from nursery to style with error/200-line limit unchanged, and function30/cognitive1 limits remain unchanged. Existing JS lint/JS format/50-file JSON format checks exit0. Logs: `tmp/dependency-direct-heartbeat-20261001.log`, `tmp/dependency-js-heartbeat-20261001.log`, `tmp/biome-2515-migrated-heartbeat-20261001.log`. Earlier root Bun zero-update observations below must not be treated as complete repository JavaScript dependency evidence.

- `cargo update --dry-run --verbose`: zero compatible lock updates.
- `cargo upgrade --dry-run --incompatible allow --pinned allow --verbose`: the only direct updates are the explicitly pinned egui family 0.36.1 to 0.36.2; no additional major migration was identified.
- `bun outdated` and `bun outdated --json`: zero JavaScript updates. Bun 1.4.2, MathJax 4.1.3, TypeScript 7.0.2 and `@types/bun` 1.4.2 remain current.
- Published siblings adopted in the current graph: KDV 0.5.7, KRR 0.4.21, KUC 0.3.17, KMM 0.2.3, katana-ast-lint 0.5.2 and KML 0.19.5. The graph contains one V8 version, 152.2.0. The historical KUC 0.3.3 and duplicate-V8 holds below no longer describe this graph.
- Latest live check on 2026-10-01: KUC v0.4.0 GitHub Release was published at 02:59:22 UTC. Public KDV0.5.7 pins KUC exactly to0.3.17, so KatanA cannot independently replace that transitive dependency with0.4.0. KDV's existing owner was asked to evaluate the new registry version and unchanged gates during Issue56 delivery. Source inspection suggests additive consumer-artifact v2 APIs, not verified consumer compatibility; public KDV adoption and full downstream acceptance remain required. KRR's latest public release remains0.4.21; PR99 is Draft/Open/Blocked, not a public0.4.22.
  - The public sparse registry index independently returns KUC0.4.0, `yanked=false`, Rust minimum1.95.0, checksum `9f2e0ae7eb5d0706dd289c3121058132c042be778f659c071c25cf4fa0a621d0`. The crates.io HTTP API returned403; registry publication is evidenced by the sparse index, not inferred from GitHub Release or that failed API request.
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
# MathJax JavaScript dependency scope confirmation

- The actual repository package is `vendor/mathjax_svg/js/package.json`, not the ancestor workspace package selected by running Bun from the repository root.
- Running `bun outdated` from that package exits0 with no outdated entries. Its current declarations are `@mathjax/src ^4.1.3`, `@types/bun ^1.4.2`, `bun ^1.4.2` and `typescript ^7.0.2`.
- The Biome2.5.15 tooling update is committed separately in714bbe15. No dependency-update scope or release quality gate is reduced.
