## Context

Katana already has most of the primitives needed for an in-product demo flow: native Help menu wiring (`native_menu/mod.rs`, `macos_menu.m`), document opening through `handle_select_document()`, grouped tabs in `DocumentState.tab_groups`, and per-tab view state. What it does not have is a curated bundle workflow that resolves localized demo assets from `assets/feature` and opens them coherently inside the existing tab strip.

The current document model also assumes every loaded text file is editable. `Document` stores only buffer/dirtiness/load state, and the editor panel always renders an interactive `TextEdit`. That is insufficient for the requested "reference mode" because disabling only the save action would still allow accidental buffer mutation and dirty-state churn.

This change is cross-cutting because it touches menu dispatch, asset resolution, document metadata, tab grouping, and editor behavior. A design document is warranted before implementation.

## Goals / Non-Goals

**Goals:**
- Add a Help-menu demo entry that opens bundled feature assets from `assets/feature`
- Resolve localized Markdown demo assets deterministically: Japanese uses `*.ja.md`, every other locale uses the base English file without a locale suffix
- Open the resolved bundle inside the existing tab surface and cluster it under a `demo` tab group
- Route non-Markdown textual assets in the bundle to a new reference mode
- Enforce reference mode as non-editable in both the UI layer and mutation/save action paths
- Keep failure cases recoverable when the demo bundle or localized files are missing

**Non-Goals:**
- Building a separate tutorial window, carousel, or onboarding-specific renderer
- Making all Help or changelog documents read-only
- Introducing locale variants beyond `ja` and the default English filenames in this change
- Persisting a separate demo manifest format or asset metadata file
- Redesigning generic tab grouping beyond what is necessary to create/reuse the `demo` group

## Decisions

### 1. Resolve the demo bundle from real files under the active workspace root

The demo flow will treat `assets/feature` as a real directory under the active workspace root and will open those files through the existing document-loading path, not through `Katana://` virtual documents.

- Rationale: this reuses existing filesystem loading, tab previews, relative-path handling, and normal tab/session behavior instead of creating a second content pipeline for the same files
- Alternative considered: expose the demo bundle as `Katana://Demo/...` virtual documents
  - Rejected: virtual docs would require bespoke content loading, group membership handling, and reference-mode storage while the requested source already exists on disk

If no workspace is open, or if `<workspace>/assets/feature` does not exist, the action will fail with a recoverable status message and leave the current tabs untouched.

### 2. Localize only Markdown demo documents, with English as the base filename contract

The bundle resolver will treat Markdown demo files as a paired localization surface:

- Base English/default document: `name.md`
- Japanese override: `name.ja.md`

Resolution rules:

- If UI language is `ja` and `name.ja.md` exists, open that file and suppress `name.md`
- If UI language is `ja` but `name.ja.md` does not exist, fall back to `name.md`
- For every non-`ja` UI language, open `name.md` and ignore `name.ja.md`
- Non-Markdown code/reference assets are language-neutral and are opened without locale substitution

- Rationale: this exactly matches the user's requested "ja only for Japanese, bare English for everything else" behavior while remaining resilient when a Japanese counterpart is absent
- Alternative considered: create `*.en.md` and `*.ja.md` pairs for every locale
  - Rejected: the user explicitly requested bare English filenames as the default contract

### 3. Open demo assets into a stable `demo` tab group inside the existing tab bar

The Help action will enumerate the resolved asset set, open missing files in the current tab collection, and create or reuse a stable tab group whose user-visible name is `demo`.

Implementation contract:

- Existing non-demo tabs remain open
- The demo group uses a stable internal ID so repeated invocations reuse the same group instead of spawning duplicates
- Group membership is rebuilt from the resolved asset list on each invocation so stale removed files are not left behind
- Markdown demo docs are opened before code/reference files, and each class is ordered deterministically by relative path

- Rationale: the user asked for the demo to appear in the existing tabs as one grouped burst, and a stable group ID is the simplest way to make repeated Help actions idempotent
- Alternative considered: always create a fresh group with a timestamp ID
  - Rejected: repeated Demo clicks would clutter the tab strip with duplicate groups

### 4. Model reference mode as per-document access policy, not as a global shell view mode

Reference mode will be represented by document metadata such as `DocumentAccess::Editable | DocumentAccess::Reference`, stored alongside the document buffer/state. The existing shell `ViewMode` (`PreviewOnly`, `CodeOnly`, `Split`) remains unchanged.

Behavior:

- Markdown demo docs open as normal editable documents
- Reference assets open with `ViewMode::CodeOnly`
- The active document carries its access policy so editor and save flows can branch deterministically

- Rationale: "reference vs editable" is orthogonal to "preview vs code vs split"; making it a per-document policy avoids contorting shell layout state for a mutability concern
- Alternative considered: add `ViewMode::Reference`
  - Rejected: this would conflate layout selection with access control and complicate existing view-mode persistence

### 5. Enforce reference immutability in both editor rendering and action handling

Reference-mode safety will be enforced twice:

1. UI layer: render the code pane with a non-interactive text widget while preserving scrolling, line numbers, and text selection/copy affordances as much as egui allows
2. Action layer: block `handle_update_buffer()`, `handle_replace_text()`, and `handle_save_document()` for reference documents so keyboard shortcuts, paste paths, or future call sites cannot mutate them indirectly

Reference documents must never become dirty, and save attempts should no-op with a recoverable status message instead of writing to disk.

- Rationale: a UI-only lock is too weak because non-UI action paths can still mark documents dirty or attempt writes
- Alternative considered: disable only the Save command
  - Rejected: that still allows accidental edits and inconsistent dirty-state behavior

### 6. Open only textual files from `assets/feature`; treat binary assets as support files

The demo enumerator will open:

- Resolved Markdown documents (`.md` and localized `.ja.md` according to the rules above)
- Other UTF-8 text/code assets under `assets/feature`

It will skip binary/support assets such as images that are meant to be referenced by the demo Markdown instead of opened as tabs.

- Rationale: the user asked to expand files under `feature`, but opening every binary dependency as a tab would degrade the demo experience and fight the existing text-oriented editor
- Alternative considered: require a dedicated manifest file to list openable demo assets
  - Rejected: the request did not ask for a new manifest authoring workflow, and deterministic directory walking is sufficient for this change

## Risks / Trade-offs

- [Risk] Japanese-localized bundles can drift from the English file set
  → Mitigation: pair files by basename, fall back to base English when a Japanese variant is missing, and cover the resolver with tests
- [Risk] Reference-mode immutability may be bypassed by future mutation call sites
  → Mitigation: centralize the access-policy checks in document mutation/save helpers, not only in the editor widget
- [Risk] Reopening the demo repeatedly may create duplicated tabs or stale group membership
  → Mitigation: use a stable internal group ID and reconcile membership against the resolved bundle on every invocation
- [Risk] Binary files under `assets/feature` may be misclassified as openable tabs
  → Mitigation: restrict the enumerator to UTF-8 text inputs and skip unreadable/binary assets
- [Risk] Reference mode could accidentally leak into normal workspace files
  → Mitigation: set the access policy only for files opened through the demo-bundle resolver

## Migration Plan

1. Add `assets/feature` bundle files following the base-English plus optional `.ja.md` naming contract
2. Introduce a new Help-menu action and route it through native menu polling
3. Implement the demo-bundle resolver and grouped-tab opening flow
4. Extend the document model with access policy metadata and wire reference-mode defaults for demo-opened code assets
5. Update the editor and save/mutation flows to enforce non-editable reference mode
6. Add integration tests for menu dispatch, locale resolution, group reuse, and read-only behavior

Rollback can remove the Help Demo entry and ignore the new access policy field, returning all demo-opened documents to the normal editable path.

## Open Questions

- None at proposal time. The main behavioral contracts are now explicit enough to implement directly.
