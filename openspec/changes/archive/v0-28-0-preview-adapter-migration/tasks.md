## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-28-0-preview-adapter-migration` またはリリース用統合ブランチ（例: `release/v0.28.0`）
- **作業ブランチ**: 標準は `v0-28-0-preview-adapter-migration-task-x`、リリース用は `feature/v0.28.0-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

## 1. Adapter Contract

実装状態: initial DTO / contract は `preview-adapter-contract` から移管済み。無印 change は `openspec/changes/archive/2026-04-25-superseded-preview-adapter-contract/` へ archive した。残作業は metadata requirements の fixture 固定と current renderer migration への接続である。

### Definition of Done (DoD)

- [x] Define the preview adapter API using KatanA-owned DTOs for input, theme, workspace context, actions, render metadata, and errors.
- [x] Document which types are allowed to cross from the adapter into `katana-ui`.
- [ ] Ensure parser/vendor/renderer internal types are not part of the public adapter contract.
- [ ] Execute `/openspec-delivery` workflow for this task.

- [x] 1.1 Define `PreviewInput`, `PreviewThemeSnapshot`, `PreviewWorkspaceContext`, `PreviewRenderMetadata`, `PreviewAction`, and preview error DTOs or equivalent local types.
- [ ] 1.2 Add compile-time boundaries that prevent `katana-ui` from depending on renderer-specific or vendor-specific types.
- [ ] 1.3 Capture current TOC, scroll sync, block highlight, search, and action hook metadata requirements in adapter-level tests.

## 2. Current Preview Migration

### Definition of Ready (DoR)

- [ ] Task 1 completed its delivery cycle and the base branch is synced.

### Definition of Done (DoD)

- [ ] Move current native preview calls behind the preview adapter without changing user-visible behavior.
- [ ] Preserve current Markdown, GFM, table, math, diagram, anchor, scroll-sync, TOC, and emoji behavior.
- [ ] Keep source editor and split-view behavior unchanged.
- [ ] Execute `/openspec-delivery` workflow for this task.

- [ ] 2.1 Wrap the current renderer implementation behind the adapter.
- [ ] 2.2 Replace `katana-ui` preview call sites with adapter API usage.
- [ ] 2.3 Verify existing preview fixtures and integration tests still pass.

## 3. Vendor Hack Containment

### Definition of Ready (DoR)

- [ ] Task 2 completed its delivery cycle and the base branch is synced.

### Definition of Done (DoD)

- [ ] Inventory preview-related `[patch.crates-io]` and `vendor/` dependencies.
- [ ] Move direct usage of preview-specific fork APIs behind the adapter implementation.
- [ ] Document any remaining root patch or vendor dependency with its owning concern and reason.
- [ ] Execute `/openspec-delivery` workflow for this task.

- [ ] 3.1 Classify each current vendor patch as preview-owned, platform-owned, or unrelated.
- [ ] 3.2 Remove `katana-ui` direct calls to preview-owned fork-specific APIs.
- [ ] 3.3 Add a short maintenance note describing the remaining vendor ownership map.

## 4. Final Verification & Release Work

- [ ] 4.1 Run formatting and lint checks for modified Rust and OpenSpec files.
- [ ] 4.2 Run preview adapter unit/contract tests.
- [ ] 4.3 Run relevant UI integration tests that cover preview rendering, TOC, scroll sync, diagram fallback, and emoji rendering.
- [ ] 4.4 Run `openspec validate v0-28-0-preview-adapter-migration`.
- [ ] 4.5 Confirm this change does not include preview-driven editing behavior.
