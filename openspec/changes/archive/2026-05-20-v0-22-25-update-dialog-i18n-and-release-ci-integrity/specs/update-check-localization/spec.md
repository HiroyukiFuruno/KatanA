## ADDED Requirements

### Requirement: OS locale follow mode for UI language

システムは、ユーザーが言語設定として "auto" を選択した場合、起動毎に OS のロケールを再評価して UI 表示言語を決定しなければならない (SHALL)。

#### Scenario: Auto mode follows OS locale on every launch

- **GIVEN** the user's saved language preference is `"auto"`
- **AND** `OsLocaleOps::get_default_language()` returns `"ja"`
- **WHEN** KatanA initialises i18n at launch
- **THEN** the system SHALL use the `ja` locale for all UI text
- **AND** it SHALL NOT persist `"ja"` into the language setting

#### Scenario: Auto mode falls back to en when OS locale is unresolved

- **GIVEN** the user's saved language preference is `"auto"`
- **AND** `OsLocaleOps::get_default_language()` returns `None`
- **WHEN** KatanA initialises i18n at launch
- **THEN** the system SHALL fall back to the `en` locale
- **AND** it SHALL still keep the saved preference as `"auto"`

#### Scenario: Explicit language preference overrides OS locale

- **GIVEN** the user's saved language preference is an explicit code such as `"ja"`
- **AND** the current OS locale resolves to `"en"`
- **WHEN** KatanA initialises i18n at launch
- **THEN** the system SHALL use the `ja` locale (the explicit preference wins)
- **AND** it SHALL NOT silently rewrite the saved preference to `"auto"`

#### Scenario: First-launch initialises preference to auto

- **GIVEN** KatanA is launched for the first time on a clean settings repository
- **WHEN** `SettingsService::apply_os_default_language` runs
- **THEN** the system SHALL write `"auto"` as the saved language preference
- **AND** it SHALL NOT write the concrete OS locale (e.g. `"ja"`) into the saved preference

### Requirement: Update check dialog is fully localised

システムは、更新確認ダイアログのすべての可視文字列 (title / body / error 詳細 / アクションラベル) を i18n bundle から解決された localized 文字列で表示しなければならない (SHALL)。raw な英語 error 文字列 (例: ureq の `Display`) を UI に直接流し込んではならない (MUST NOT)。

#### Scenario: Network refused error is localised per active locale

- **GIVEN** the active locale is `ja`
- **AND** the update check failed with a refused TCP connection
- **WHEN** the update dialog renders the error
- **THEN** the dialog SHALL display the Japanese phrasing bound to `update_check_error_network_unreachable`
- **AND** the dialog SHALL NOT display the raw `"io: Connection refused"` string

#### Scenario: HTTP status error includes the numeric code through placeholder

- **GIVEN** the active locale is any of the 10 supported locales
- **AND** the update check failed with a non-2xx HTTP status code (e.g. 429)
- **WHEN** the update dialog renders the error
- **THEN** the dialog SHALL use the i18n template bound to `update_check_error_server_status`
- **AND** the template SHALL include the numeric code through the `{status}` placeholder

#### Scenario: Unknown error variant is localised, never raw

- **GIVEN** the update fetch failed with a variant the mapping does not specifically cover (`CheckUpdateError::Other`)
- **WHEN** the update dialog renders the error
- **THEN** the dialog SHALL display the i18n phrase bound to `update_check_error_unknown`
- **AND** the dialog MAY append technical raw text as a secondary detail line, but it SHALL NOT use the raw text as the primary error phrase

### Requirement: All 10 supported locales carry the new update-check error keys

システムは、`crates/katana-ui/locales/` 配下の 10 言語 JSON (`de / en / es / fr / it / ja / ko / pt / zh-CN / zh-TW`) すべてに、本 capability で導入される i18n キー全件を同期して持たなければならない (SHALL)。

#### Scenario: All locales must carry update_check_error_* keys

- **GIVEN** the locale JSON files in `crates/katana-ui/locales/`
- **WHEN** the i18n coverage AST linter runs
- **THEN** every locale SHALL contain non-empty values for:
    - `update_check_error_network_unreachable`
    - `update_check_error_network_timed_out`
    - `update_check_error_server_status`
    - `update_check_error_proxy_failed`
    - `update_check_error_invalid_payload`
    - `update_check_error_unknown`
- **AND** the `ja.json` values SHALL be Japanese translations, not English fallbacks

#### Scenario: Direct raw display of check_error is rejected

- **GIVEN** a file under `crates/katana-ui/src/views/modals/update/**.rs`
- **WHEN** that file passes `state.update.check_error` (or its `Display` form) into `ui.label` / `format!` without going through the i18n bundle
- **THEN** the AST linter rule `no-raw-update-check-error-display` SHALL flag a violation
- **AND** the violation message SHALL point the implementer at `CheckUpdateError::i18n_key`
