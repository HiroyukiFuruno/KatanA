## ADDED Requirements

### Requirement: Release は対応 OS ごとの artifact を生成する

システムは、対応 desktop OS ごとに配布可能な release artifact を生成しなければならない（MUST）。

#### Scenario: macOS artifact を維持する

- **WHEN** release workflow が macOS artifact を build する
- **THEN** `KatanA-Desktop-<version>.dmg` と `KatanA-macOS.zip` が生成される

#### Scenario: Windows artifact を生成する

- **WHEN** release workflow が Windows x86_64 artifact を build する
- **THEN** `KatanA-windows-x86_64.zip` が生成される

#### Scenario: Linux artifact を生成する

- **WHEN** release workflow が Linux x86_64 artifact を build する
- **THEN** `KatanA-linux-x86_64.tar.gz` が生成される

### Requirement: CI は対応 OS の build を継続的に検証する

システムは、macOS、Windows、Ubuntu の CI runner 上で build または smoke test を実行しなければならない（MUST）。

#### Scenario: Windows CI build

- **WHEN** CI が Windows runner で実行される
- **THEN** KatanA workspace は build または smoke test を通過する

#### Scenario: Linux CI build

- **WHEN** CI が Ubuntu runner で実行される
- **THEN** KatanA workspace は build または smoke test を通過する

### Requirement: インストール導線は対応 OS ごとに文書化される

システムは、README と開発者向け文書に、対応 desktop OS ごとの install / run 手順を記載しなければならない（MUST）。

#### Scenario: README で support matrix を案内する

- **WHEN** ユーザーが repository root の README を読む
- **THEN** macOS、Windows、Linux の support status と配布形式が分かる

#### Scenario: 開発者向け文書で build prerequisites を案内する

- **WHEN** 開発者が development guide を読む
- **THEN** Windows / Linux を含む build prerequisites と実行手順が分かる
