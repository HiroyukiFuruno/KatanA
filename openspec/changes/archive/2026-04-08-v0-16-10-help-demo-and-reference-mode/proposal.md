## Why

Katana already has strong preview and tab workflows, but there is no first-run or self-guided demo path that shows what the app can do from inside the product. A Help-driven demo bundle is needed now so users can open curated feature documents immediately, in their language, without manually hunting through the repository or accidentally editing sample code.

## What Changes

- Add a `Help -> Demo` entry that opens bundled feature demo documents from `assets/feature` inside the existing tab workspace instead of a separate modal or window
- Resolve localized demo documents by using Japanese (`*.ja.md`) only when the current UI language is `ja`, and the default English files without a locale suffix for every other language
- Open the resolved demo document set at once under a tab group named `demo`, so the feature bundle appears as a curated cluster rather than unrelated tabs
- Distinguish Markdown demo documents from code assets inside the bundle, and open code assets in a new reference mode instead of a normal editable editor path
- Add a reference mode contract that keeps code visible in the existing code pane while explicitly blocking editing and save-driven mutation for those documents
- Add regression coverage for Help menu dispatch, localized demo asset selection, grouped tab opening, and non-editable reference-mode behavior

## Capabilities

### New Capabilities

- `help-demo-bundle`: Help-driven opening of localized demo assets from `assets/feature`, including grouped tab expansion and code-vs-document routing
- `reference-mode`: A view-only code presentation mode for bundled reference assets that uses the existing tab/editor surface but disallows editing

### Modified Capabilities

- `menu-enhancement`: The Help menu gains a Demo entry that launches the bundled feature walkthrough
- `i18n`: Demo asset resolution follows a deterministic language fallback contract of `ja` for Japanese and default English filenames for all other locales

## Impact

- Affected code:
  - `crates/katana-ui/src/native_menu/mod.rs`
  - `crates/katana-ui/src/macos_menu.m`
  - `crates/katana-ui/src/i18n/types.rs`
  - `crates/katana-ui/src/i18n/logic.rs`
  - `crates/katana-ui/src/app_state.rs`
  - `crates/katana-ui/src/app/action.rs`
  - `crates/katana-ui/src/app/document.rs`
  - `crates/katana-ui/src/app/workspace/mod.rs`
  - `crates/katana-ui/src/views/panels/editor/ui.rs`
  - `crates/katana-core/src/document.rs`
- Affected assets:
  - `assets/feature/**`
- Affected tests:
  - `crates/katana-ui/src/shell/shell_tests.rs`
  - `crates/katana-ui/tests/integration/*.rs`
- No external API change is expected, but document-open behavior, Help menu behavior, and editor mutability rules will change for bundled demo assets.
