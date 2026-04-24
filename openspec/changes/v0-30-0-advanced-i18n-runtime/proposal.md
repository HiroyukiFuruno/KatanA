## Why

PR #236 raised several i18n issues. The concrete findings after checking current `master` are:

- `task_todo` is already translated naturally in the current locale JSON files and should not become a new change by itself.
- `I18nOps::tf` still performs simple `{key}` string replacement and cannot express plural, gender, or locale-aware formatting rules.
- `crates/katana-ui/src/i18n/logic.rs` still has hard panic paths for dictionary mismatch and unhandled language codes, which is too brittle for user-provided settings or environment-derived language values.
- An existing `x-x-x-advanced-i18n-framework` change captures the broad idea, but it is not a valid sequential release change and lacks OpenSpec spec coverage.

This change regularizes that direction as v0.30.0 and scopes it to an adapter-based i18n runtime migration: first make language lookup safe, then introduce a formatter abstraction that can support Fluent or ICU-style message formatting without coupling UI call sites directly to a specific engine.

## What Changes

- Add an i18n runtime adapter boundary for message lookup and formatted message rendering.
- Replace hard panic behavior for unknown or unsupported language codes with deterministic fallback to the default language.
- Replace ad-hoc `I18nOps::tf` replacement with a structured formatting API that can represent plural and gender choices.
- Keep existing JSON locale files usable during migration; do not require a one-shot conversion of every locale file in the first task.
- Evaluate Fluent/ICU-compatible Rust crates behind the adapter and migrate high-value messages first.
- Add locale quality checks that operate on current locale content rather than stale PR findings.

## Capabilities

### New Capabilities

- `advanced-i18n-runtime`: Safe language fallback and structured message formatting through an i18n engine adapter.

### Modified Capabilities

- `i18n`: Existing deterministic dictionary initialization is extended with no-panic fallback and formatter semantics.

## Impact

- `crates/katana-ui/src/i18n/*`: language selection, message lookup, formatter API, and tests.
- `crates/katana-ui/locales/*`: high-value pluralized or parameterized messages may move to a richer representation after the adapter is in place.
- `crates/katana-linter/src/rules/domains/locales/*`: locale quality checks may need to understand the new message representation.
- Existing `x-x-x-advanced-i18n-framework`: should be treated as superseded planning material after this change is accepted.
