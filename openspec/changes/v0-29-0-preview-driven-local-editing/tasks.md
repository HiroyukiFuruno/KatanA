## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-29-0-preview-driven-local-editing` またはリリース用統合ブランチ（例: `release/v0.29.0`）
- **作業ブランチ**: 標準は `v0-29-0-preview-driven-local-editing-task-x`、リリース用は `feature/v0.29.0-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

## 1. Editable Metadata Contract

### Definition of Ready (DoR)

- [ ] v0.28.0 preview adapter migration is merged or available as the base branch.

### Definition of Done (DoD)

- [ ] Extend the preview adapter contract with editable node descriptors, supported edit commands, source ranges, and hit-test metadata.
- [ ] Ensure descriptors are renderer-neutral and do not expose parser/vendor internals.
- [ ] Add contract tests for editable descriptor generation.
- [ ] Execute `/openspec-delivery` workflow for this task.

- [ ] 1.1 Define `EditableNodeDescriptor`, `EditableNodeKind`, `PreviewEditCommand`, `SourceRange`, and stale-check metadata or equivalent local types.
- [ ] 1.2 Map rendered preview hit targets to editable descriptors.
- [ ] 1.3 Cover paragraph, heading, fenced code, diagram, math, table, link, and image discovery in fixtures.

## 2. Source Range Patch Engine

### Definition of Ready (DoR)

- [ ] Task 1 completed its delivery cycle and the base branch is synced.

### Definition of Done (DoD)

- [ ] Implement validated source range patching against the in-memory Markdown buffer.
- [ ] Reject stale patches when the expected original text or buffer snapshot no longer matches.
- [ ] Preserve existing dirty/save semantics.
- [ ] Execute `/openspec-delivery` workflow for this task.

- [ ] 2.1 Add patch command application with expected-original validation.
- [ ] 2.2 Add parser/preview validation around patch commit and rollback on failure.
- [ ] 2.3 Add tests for successful patch, stale range rejection, cancel, and dirty state.

## 3. Preview Local Edit UI

### Definition of Ready (DoR)

- [ ] Task 2 completed its delivery cycle and the base branch is synced.

### Definition of Done (DoD)

- [ ] Add preview actions that open local edit sessions from selected editable nodes.
- [ ] Provide scoped edit surfaces for block-level and metadata-oriented edits.
- [ ] Keep the rest of the document in preview mode during local edits.
- [ ] Keep full egui `TextEdit` replacement and native input surface architecture out of scope; use `x-x-x-native-input-surface` if that capability is needed later.
- [ ] Execute `/openspec-delivery` workflow for this task.

- [ ] 3.1 Implement edit surfaces for paragraph and heading nodes.
- [ ] 3.2 Implement edit surfaces for fenced code, Mermaid, Draw.io, and math blocks.
- [ ] 3.3 Implement table edit surface or scoped raw table source fallback.
- [ ] 3.4 Implement link and image metadata edit surfaces.

## 4. Source Inspector and Fallback Mode

### Definition of Ready (DoR)

- [ ] Task 3 completed its delivery cycle and the base branch is synced.

### Definition of Done (DoD)

- [ ] Make the default source panel act as a read-only inspector for the active document or selected preview node.
- [ ] Gate direct source editing behind an explicit fallback source-edit mode if it remains available.
- [ ] Ensure fallback source-edit mode uses the same dirty/save buffer semantics as preview-driven edits.
- [ ] Execute `/openspec-delivery` workflow for this task.

- [ ] 4.1 Add read-only source inspector state and navigation from preview selection.
- [ ] 4.2 Add explicit fallback source-edit mode entry and exit behavior.
- [ ] 4.3 Verify inspector viewing never mutates the buffer.

## 5. Final Verification & Release Work

- [ ] 5.1 Run formatting and lint checks for modified Rust and OpenSpec files.
- [ ] 5.2 Run preview-driven edit contract/unit tests.
- [ ] 5.3 Run UI integration tests covering preview selection, local edit commit, cancel, stale range rejection, source inspector, and save behavior.
- [ ] 5.4 Run diagram/math/table local edit regression tests.
- [ ] 5.5 Run `openspec validate v0-29-0-preview-driven-local-editing`.
- [ ] 5.6 Confirm no WebView, React, DOM runtime, or bundled web app is introduced.
- [ ] 5.7 Confirm native input surface ownership remains with `x-x-x-native-input-surface`, not this preview-local editing change.
