## MODIFIED Requirements

### Requirement: Deterministic Dictionary Initialization

The internationalization dictionary memory store SHALL iterate over its language map deterministically by using a List (`Vec<I18nDictionaryEntry>`) structure under `OnceLock`, and MUST resolve unsupported runtime language values through the configured fallback language instead of panicking.

#### Scenario: Dictionary access mechanism

- **WHEN** the system queries for UI localization strings
- **THEN** it iterates over a continuous array memory space, retrieving the translation without structural ambiguity

#### Scenario: Unknown runtime language falls back safely

- **WHEN** the selected language code is not present in the embedded dictionary
- **THEN** the system uses the configured fallback language
- **THEN** the application remains usable without a startup panic

## ADDED Requirements

### Requirement: I18n formatting is routed through an engine adapter

The system SHALL route parameterized i18n message formatting through a KatanA-owned engine adapter rather than ad-hoc template replacement in UI call sites.

#### Scenario: Format a parameterized message

- **WHEN** UI code needs to render a message with named parameters
- **THEN** it calls the i18n formatting adapter with a message identifier and typed arguments
- **THEN** the UI call site does not manually replace `{key}` placeholders

#### Scenario: Swap formatting engine implementation

- **WHEN** the underlying formatter is migrated from JSON-compatible formatting to Fluent or ICU-style formatting
- **THEN** UI call sites continue to use the KatanA-owned adapter contract
- **THEN** engine-specific data structures remain inside the i18n runtime layer

### Requirement: Locale-aware plural formatting is supported for selected messages

The system SHALL support locale-aware plural categories for messages selected by the migration inventory.

#### Scenario: Render one result

- **WHEN** a pluralized message is formatted with count `1`
- **THEN** the formatter chooses the locale-specific singular or equivalent form
- **THEN** the message is not built by concatenating the number with a static suffix

#### Scenario: Render multiple results

- **WHEN** a pluralized message is formatted with count greater than `1`
- **THEN** the formatter chooses the locale-specific plural or equivalent form
- **THEN** the message remains valid for languages whose plural rules differ from English

### Requirement: Locale quality checks use current locale content

The system SHALL validate locale completeness and pseudo-translation risks against the current locale files before accepting i18n runtime changes.

#### Scenario: Validate localized task labels

- **WHEN** locale quality checks evaluate task-related labels such as `task_todo`
- **THEN** they compare the current locale files rather than stale external reports
- **THEN** already-localized values are not re-opened as defects without fresh evidence

#### Scenario: Reject pseudo-translations

- **WHEN** a locale value contains a pseudo-translation marker or an untranslated fallback pattern
- **THEN** the linter reports the value as invalid
- **THEN** the i18n runtime migration cannot be completed until the locale value is corrected or explicitly exempted
