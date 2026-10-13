## MODIFIED Requirements

### Requirement: テーマ設定（Dark / Light）

システムは、Dark / Light の 2 種類のテーマを提供し、ユーザーが切り替え可能にしなければならない（MUST）。初回起動時の既定テーマは、OS theme を取得できる対応 desktop OS ではその mode に追従し、取得できない場合は Dark を既定としなければならない（MUST）。

#### Scenario: テーマの切り替え

- **WHEN** ユーザーが設定メニューからテーマを Dark から Light に変更する
- **THEN** アプリ全体の配色が Light テーマに切り替わる

#### Scenario: 初回起動時に OS theme を反映する

- **WHEN** ユーザーが Windows、Linux、macOS のいずれかで初回起動する
- **AND** system theme が取得可能である
- **THEN** 既定テーマはその system theme と同じ dark / light mode になる

#### Scenario: OS theme を取得できない場合の既定値

- **WHEN** ユーザーが初回起動し、system theme が取得できない
- **THEN** Dark テーマが適用される

#### Scenario: テーマ設定の永続化

- **WHEN** ユーザーがテーマを切り替える
- **THEN** 選択したテーマが設定に保存され、次回起動時に復元される

#### Scenario: テーマ切替の即時反映

- **WHEN** テーマを切り替える
- **THEN** 再起動なしで即座に UI 全体に反映される
