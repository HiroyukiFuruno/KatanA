## ADDED Requirements

### Requirement: theme settings は markdown diagnostics decoration color を変更できなければならない

システムは、theme settings または custom theme editor から markdown diagnostics decoration color を変更できなければならない（SHALL）。

#### Scenario: diagnostics decoration color を変更する

- **WHEN** user が theme settings で markdown diagnostics decoration color を変更する
- **THEN** editor 上の warning decoration color は即時に更新される

#### Scenario: diagnostics decoration color 設定を復元する

- **WHEN** user が diagnostics decoration color を変更した後にアプリを再起動する
- **THEN** system は前回選択した diagnostics decoration color を復元する
