## ADDED Requirements

### Requirement: Lint general settings remain separate from markdownlint rule config

システムは、一般設定で扱う Lint のオン/オフと「無視 / 警告 / エラー」の選択を、`.markdownlint.json` / `.markdownlint.jsonc` に保存されるルール有効化や詳細値と分離しなければならない（SHALL）。

#### Scenario: Change severity without editing rule details

- **WHEN** user changes a rule from warning to error in the general Lint settings
- **THEN** system stores the KatanA severity preference
- **THEN** system does not remove that rule's detailed `.markdownlint.json` properties

#### Scenario: Ignore a rule without deleting detailed config

- **WHEN** user changes a rule to ignore in the general Lint settings
- **THEN** system hides or suppresses diagnostics for that rule in KatanA
- **THEN** system keeps the previous detailed rule config restorable
- **THEN** system does not permanently delete the rule's `.markdownlint.json` properties

#### Scenario: Re-enable a previously ignored rule

- **WHEN** user changes an ignored rule back to warning or error
- **THEN** system restores the effective rule configuration for lint execution
- **THEN** system keeps the user's previous detailed rule settings where they existed

### Requirement: Workspace-local markdownlint config toggle does not open advanced settings

システムは、ワークスペース固有の `.markdownlint.json` を使用する設定をオン/オフしても、高度な設定画面へ自動的に切り替えてはならない（MUST NOT）。

#### Scenario: Enable workspace-local config

- **WHEN** user turns on the setting to use workspace-local `.markdownlint.json`
- **THEN** system keeps the current general settings screen visible
- **THEN** system refreshes diagnostics using the effective workspace config

#### Scenario: Disable workspace-local config

- **WHEN** user turns off the setting to use workspace-local `.markdownlint.json`
- **THEN** system keeps the current general settings screen visible
- **THEN** system refreshes diagnostics using the effective global config

### Requirement: Effective markdownlint settings use workspace override before global settings

システムは、Lint の effective config を `workspace > global > default` の優先順位で解決しなければならない（SHALL）。

#### Scenario: Workspace config exists

- **WHEN** workspace-local config is enabled and the active workspace contains `.markdownlint.json`
- **THEN** system uses that workspace file as the primary markdownlint config
- **THEN** system applies KatanA workspace-specific severity preferences after loading the file

#### Scenario: Workspace config is missing

- **WHEN** workspace-local config is enabled and the active workspace does not contain `.markdownlint.json` or `.markdownlint.jsonc`
- **THEN** system reports that no workspace config exists
- **THEN** system offers to create a workspace config file instead of silently falling back without notice

#### Scenario: Workspace override wins over global config

- **WHEN** both global config and workspace config exist
- **THEN** system uses workspace config for files inside that workspace
- **THEN** global config remains unchanged

### Requirement: Advanced settings edit the selected markdownlint config file

システムは、高度な設定を一般設定への上乗せとして扱い、選択中の effective `.markdownlint.json` / `.markdownlint.jsonc` を編集しなければならない（SHALL）。

#### Scenario: Open advanced settings with workspace config enabled

- **WHEN** user explicitly opens advanced settings while workspace config is enabled
- **THEN** system loads the workspace `.markdownlint.json` or `.markdownlint.jsonc`
- **THEN** changes are saved back to the workspace config file

#### Scenario: Open advanced settings with workspace config disabled

- **WHEN** user explicitly opens advanced settings while workspace config is disabled
- **THEN** system loads the global `.markdownlint.json`
- **THEN** changes are saved back to the global config file

### Requirement: KML receives the effective markdownlint config

システムは、KML（katana-markdown-linter）へ診断またはフォーマットを依頼する時、KatanA が解決した effective markdownlint config を渡さなければならない（SHALL）。

#### Scenario: KML API accepts config path

- **WHEN** KML exposes an API that accepts a markdownlint config file path
- **THEN** system passes the resolved full path for the active file or workspace

#### Scenario: KML API requires config data

- **WHEN** KML does not expose an API that accepts a markdownlint config file path
- **THEN** system loads `.markdownlint.json` or `.markdownlint.jsonc` in KatanA
- **THEN** system converts it into the config structure required by KML
- **THEN** system passes that structure to KML

#### Scenario: Formatting uses the same effective config as diagnostics

- **WHEN** user formats a Markdown file from KatanA
- **THEN** system uses the same effective config resolution as diagnostics
- **THEN** formatting output is not based on stale default rule settings
