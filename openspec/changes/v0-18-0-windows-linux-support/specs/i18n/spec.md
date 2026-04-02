## ADDED Requirements

### Requirement: 初回起動時の既定言語は system locale に追従する

システムは、初回起動時に system locale を取得できる場合、その locale に対応する UI language を既定値として適用しなければならない（MUST）。

#### Scenario: 対応 locale が取得できる

- **WHEN** ユーザーが初回起動し、system locale が `ja`、`fr`、`de` などの対応言語として取得できる
- **THEN** UI language はその locale に対応する言語で開始する

#### Scenario: 非対応 locale または取得失敗

- **WHEN** ユーザーが初回起動し、system locale が未対応である、または取得に失敗する
- **THEN** UI language は `en` で開始する
- **THEN** locale 検出 failure は startup crash を引き起こさない

#### Scenario: 既存ユーザーの言語設定を尊重する

- **WHEN** 既に保存済みの language setting を持つユーザーが起動する
- **THEN** system locale の再検出で保存済み language setting を上書きしない
