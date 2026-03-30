# Katana Vendor Subtrees

This directory contains vendored subtrees that Katana depends on and patches
locally.
Specifically, `egui_commonmark` and its related crates have been incorporated as
a Git `subtree` to maintain a clear boundary between the upstream code and
Katana-specific customizations.

## Subtree Repository Information

- **Upstream Repository**: `https://github.com/lampsitter/egui_commonmark`
- **Subtree Prefix**: `vendor/egui_commonmark_upstream`
- **Current Pinned Upstream Revision**: `v0.22.0`
- **Related Cargo crates**: `egui_commonmark`, `egui_commonmark_backend`,
  `egui_commonmark_macros`

## Patch Layer Policy

To support Katana's unique features without losing track of upstream logic,
we strictly enforce a two-layered structure:

1. **Base Layer**: The raw `git subtree` pull from the upstream repository.
2. **Patch Layer**: The Katana-specific file overrides applied directly over
   the subtree output.

When upgrading upstream versions, **never** manually copy files over. Always
use the Git Subtree commands to perform clean merges, resolving conflicts
against the Katana patch layer.

## How to Perform a Subtree Sync (Pull)

If a future maintainer or AI agent needs to update the subtree to a newer
upstream version, follow these exact steps to preserve history and apply
conflict resolution cleanly:

### 1. Verification of Compatibility Assumptions

Before pulling, confirm that the new upstream version remains compatible with
Katana's core dependencies (e.g., `egui` versions).
If the upstream version requires `egui` `0.34`, but Katana is on `0.33`,
**DO NOT PROCEED**.
See the "Stop-and-Correct Rule" below.

### 2. Pulling the Subtree

From the repository root, run the subtree pull command targeting the new tag
or branch:

```bash
git subtree pull --prefix=vendor/egui_commonmark_upstream \
https://github.com/lampsitter/egui_commonmark \
<UPSTREAM_TAG_OR_BRANCH> --squash
```

*Note: The `--squash` flag is mandatory to prevent importing the entire upstream
Git history into the Katana repository footprint.*

### 3. Conflict Resolution (Patch Re-application)

Git subtree merges the upstream baseline into our directory.
If upstream changed code that Katana has patched, you will experience a
merge conflict. You must manually resolve these conflicts, carefully combining
upstream bug fixes with Katana's custom behavior overrides (such as
`extract_task_list_spans`, rendering integrations, and SVG icon paths).

### 4. Verification

After resolution, you must prove zero regressions:

```bash
make check
```

Because the test suite includes deep visual and structural markdown regressions,
standard tests adequately enforce that rendering integrations have survived
the subtree merge.

## 🛑 Stop-and-Correct Rule

If you discover that an upstream sync requires architectural changes across
Katana, or breaks fundamental dependency bounds (e.g., upgrading `egui` to
a breaking major version), you **MUST NOT** proceed with the sync
implementation immediately.

You must follow the **Stop-and-Correct Rule**:

1. Pause the sync operation.
2. Fall back to `/openspec-propose` or utilize an explicit workflow
   (`v0-x-y-some-migration`) to create a new, formal OpenSpec feature proposal
   (`proposal.md`, `design.md`, `tasks.md`).
3. Document *why* the compatibility premise changed and define how the KatanA
   system must migrate before doing the raw sync.
4. Only implement the sync as a documented step in the new OpenSpec `tasks.md`.

*Failure to observe the Stop-and-Correct Rule risks undocumented configuration
drift and broken integrations taking root in `master`.*
