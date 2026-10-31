## Context

Issue399 was confirmed by the user: freeing disk capacity allowed the macOS update to complete. The old preparation error was stringified and presented as an unknown update-check failure. A 320MiB isolated APFS volume reproduced real download→extract→ENOSPC28, while sufficient storage allowed production preparation of the same public ZIP.

## Goals / Non-Goals

**Goals:** Check storage before archive writes and before extraction/replacement; display caller-available and required MB; keep original app intact on preparation failure; keep all platform and localization contracts.

**Non-Goals:** Delete user data automatically, resize storage, change archive security protections, modify KDV/KRR, manipulate the running app or user preferences, or require manual QA to close the Issue.

## Decisions

- One updater-local `UpdateStorageOps` owns cross-platform capacity budgets. `fs4::available_space` checks the filesystem containing the actual staging/destination paths, rather than the user's home drive by assumption.
- Phase1 reads HTTP Content-Length and checks compressed bytes plus a64MiB metadata/script/safety reserve before creating the download file. Missing length does not invent an archive size; runtime IO remains checked.
- Phase2 reads ZIP central metadata or scans tar.gz entries without extracting files. Staging budget is two uncompressed copies plus64MiB; target budget is one copy plus64MiB. This conservatively covers retained staging and replacement copies, including sidecars. Both destinations are checked, without relying on a same-volume rename.
- Required MB rounds up in decimal megabytes; available MB rounds down. Known insufficient capacity retains both byte counts through worker events; racing ENOSPC and permission failures retain their own typed classification. Unknown errors keep their technical chain internally and are never displayed as raw text.
- Installation/preparation failure state is distinct from update-check failure state. Retries clear both errors. The modal uses preparation-specific headings and localized explanations.

## Risks / Trade-offs

- [Space can change after preflight] → Retain typed runtime write/extraction errors; the normal app is not replaced during preparation.
- [Archive payload unknown until download completes] → Check download size first and exact expansion budget before extraction; do not pretend to know the entire expansion before downloading.
- [Conservative copy budget may reject a same-volume rename that could fit] → Prefer sufficient space for a safe replacement; document the copy reserve rather than claim a theoretical minimum.
- [tar.gz size inspection adds a streaming pass] → Do not allocate full payloads or write expanded bytes during inspection.
- [Malformed metadata/overflow] → Checked arithmetic fails closed, preserving archive protections.

## Migration Plan

Normal release/v0.22.45 commits, Draft reviews, P0/P1 repair/reply/resolve, Ready required CI, merge and public artifact verification. Existing configuration remains compatible; no settings migration.

## Open Questions

None.
