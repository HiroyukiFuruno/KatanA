## Why

The v0.22.43 macOS updater downloaded v0.22.44, then failed on storage exhaustion and displayed an unrelated unknown update-check error. The user confirmed that freeing storage allowed the update. Preventing insufficient-space preparation and explaining the required capacity avoids wasting a full extraction and hides fewer actionable failures.

## What Changes

- Check download capacity before writing the response, then inspect the real archive payload and check staging/copy and destination capacity before extraction.
- Show required and available MB, with instructions to free storage and retry. Preserve runtime capacity errors if available storage changes after the check.
- Separate update preparation failures from release-check errors; retain typed causes without exposing arbitrary technical strings.
- Cover ZIP, tar.gz, real filesystem space, HTTP downloads, failure-state routing, and all supported locale placeholders.

## Capabilities

### New Capabilities
- `update-storage-preflight`: Storage-budget checks and actionable update-preparation errors before application replacement.

### Modified Capabilities

None.

## Impact

Core updater download/installer/error boundaries; UI update state, modal, and locales; fs4 1.1.0 for cross-platform caller-available disk capacity. Both lockfiles retain all existing versions except the added dependency. No edits to KDV/KRR, user settings, or the running application.
