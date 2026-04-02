## MODIFIED Requirements

### Requirement: 設定のJSON永続化

システムは、全アプリケーション設定を各 OS の標準 config directory 配下の JSON file として永続化しなければならない（MUST）。

#### Scenario: 設定の保存

- **WHEN** ユーザーがテーマ、フォントサイズ等の設定を変更する
- **THEN** 変更は current OS の標準 config directory 配下の `KatanA/settings.json` に保存される
- **THEN** macOS では `~/Library/Application Support/KatanA/settings.json`、Windows では `%APPDATA%/KatanA/settings.json`、Linux では `$XDG_CONFIG_HOME/KatanA/settings.json` または `~/.config/KatanA/settings.json` が解決される

#### Scenario: 設定の復元

- **WHEN** アプリを起動する
- **THEN** 前回の設定が自動的に復元される

#### Scenario: 設定ファイルが存在しない場合

- **WHEN** 設定ファイルが存在しない状態でアプリを起動する
- **THEN** デフォルト設定が適用され、設定変更時に新規ファイルが作成される

#### Scenario: 設定ファイルが破損している場合

- **WHEN** 設定ファイルの JSON が不正な形式である
- **THEN** デフォルト設定が適用される
- **THEN** 破損ファイルは backup され、recoverable failure として扱われる
