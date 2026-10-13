## ADDED Requirements

### Requirement: Demo bundle locale resolution uses Japanese-only override and base English fallback

The system MUST resolve localized Markdown demo documents from `assets/feature` using `ja` overrides only for the Japanese UI locale and the base English filenames for every other locale.

#### Scenario: Japanese locale selects `.ja.md`

- **WHEN** the current UI language is `ja` and both `feature/foo.md` and `feature/foo.ja.md` exist
- **THEN** the demo bundle opens `feature/foo.ja.md`
- **THEN** it does not also open `feature/foo.md` as a duplicate localized tab

#### Scenario: Non-Japanese locale selects base English file

- **WHEN** the current UI language is anything other than `ja` and both `feature/foo.md` and `feature/foo.ja.md` exist
- **THEN** the demo bundle opens `feature/foo.md`
- **THEN** it ignores `feature/foo.ja.md`

#### Scenario: Japanese locale falls back to base English when no override exists

- **WHEN** the current UI language is `ja` and `feature/foo.ja.md` does not exist but `feature/foo.md` does
- **THEN** the demo bundle opens `feature/foo.md`
- **THEN** the localized demo launch still succeeds without requiring a Japanese-only file set
