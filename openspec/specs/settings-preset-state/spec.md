# settings-preset-state Specification

## Purpose
TBD - created by archiving change v0-22-7-markdownlint-workspace-formatting. Update Purpose after archive.
## Requirements
### Requirement: Settings presets share one storage model

システムは、テーマ、アイコン、Lint のプリセット状態を、同じ保存仕様で保持しなければならない（SHALL）。保存仕様は、現在値、元プリセット、変更状態、ユーザープリセット一覧を持たなければならない（SHALL）。

#### Scenario: Apply a built-in preset

- **GIVEN** a built-in preset exists for theme, icon, or Lint settings
- **WHEN** user applies the preset
- **THEN** system copies the preset value into the current setting value
- **THEN** system records the applied preset as the source preset
- **THEN** system marks the current setting as not modified from that source
- **THEN** system does not modify the built-in preset

#### Scenario: Modify after applying a preset

- **GIVEN** user has applied a preset
- **WHEN** user changes an individual theme, icon, or Lint detail
- **THEN** system keeps the source preset reference
- **THEN** system marks the current setting as custom
- **THEN** system can display that the current setting is a custom setting based on the source preset

#### Scenario: Save current setting as a user preset

- **GIVEN** user has a current theme, icon, or Lint setting
- **WHEN** user saves the current setting as a user preset
- **THEN** system stores the current value as a named preset for the same setting kind
- **THEN** system makes the user preset available for later selection
- **THEN** applying that user preset copies its value into the current setting instead of linking the current setting to the preset

#### Scenario: Migrate existing setting values

- **GIVEN** existing saved settings do not have source preset metadata
- **WHEN** system migrates those settings into the unified preset model
- **THEN** system assigns a source preset only when the current value exactly matches a known preset
- **THEN** system marks unmatched values as custom with unknown source
- **THEN** system does not guess a source preset without exact evidence

#### Scenario: Migrate existing theme settings without changing appearance

- **GIVEN** a fixture settings file contains existing `theme.preset`, `theme.custom_color_overrides`, `theme.active_custom_theme`, and `theme.custom_themes`
- **WHEN** system migrates the settings into the unified preset model
- **THEN** system preserves the effective theme colors seen before migration
- **THEN** system converts existing custom themes into user presets for theme settings
- **THEN** system preserves the active custom theme as the current theme value when it was active before migration
- **THEN** saving settings after migration does not clear custom color overrides unexpectedly

#### Scenario: Migrate existing icon settings without changing appearance

- **GIVEN** a fixture settings file contains existing `theme.icon_pack`, `icon.active_preset`, `icon.active_overrides`, and `icon.custom_presets`
- **WHEN** system migrates the settings into the unified preset model
- **THEN** system preserves the effective icon pack seen before migration
- **THEN** system preserves the effective icon overrides seen before migration
- **THEN** system converts existing icon custom presets into user presets for icon settings
- **THEN** saving settings after migration does not reset the icon pack or active overrides unexpectedly

#### Scenario: Fixture-backed migration avoids false source attribution

- **GIVEN** fixture settings values are close to a known preset but not exactly equal
- **WHEN** system migrates the settings into the unified preset model
- **THEN** system marks the migrated value as custom with unknown source
- **THEN** system does not display a known preset name as the source

### Requirement: Settings preset UI uses a shared widget

システムは、テーマ、アイコン、Lint のプリセット操作を、同じ再利用ウィジェット（widget: 再利用できる画面部品）で表示しなければならない（SHALL）。

#### Scenario: Render shared preset controls

- **WHEN** user opens theme, icon, or Lint settings
- **THEN** system renders preset selection through the shared preset widget
- **THEN** system renders the current preset or custom state through the shared preset widget
- **THEN** system renders save-current-as-preset through the shared preset widget
- **THEN** system renders reset-to-source through the shared preset widget when a source preset exists

#### Scenario: Keep setting-specific details inside the widget slot

- **WHEN** user opens a setting-specific detail area from the shared preset widget
- **THEN** system shows theme-specific color controls for theme settings
- **THEN** system shows icon-pack and icon-override controls for icon settings
- **THEN** system shows markdownlint rule details for Lint settings
- **THEN** changes from those detail controls update the same unified current setting state

#### Scenario: Show custom source state consistently

- **GIVEN** user has modified a setting after applying a preset
- **WHEN** user views theme, icon, or Lint settings
- **THEN** system labels the setting as custom
- **THEN** system shows the source preset name when it is known
- **THEN** system avoids showing a guessed source preset when source is unknown

