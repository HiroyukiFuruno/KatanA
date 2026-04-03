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

### Requirement: macOS 起点でも対応 OS の検証結果をレビューできる

システムは、macOS を主開発環境とする maintainer でも Windows / Linux の support verification を再現できるよう、target OS CI の結果と manual verification の入口を提供しなければならない（MUST）。

#### Scenario: Windows CI 結果を macOS からレビューする

- **WHEN** maintainer が macOS 上から Windows support verification を確認する
- **THEN** Windows runner の build / smoke result、log、生成 artifact を参照できる

#### Scenario: Linux CI 結果を macOS からレビューする

- **WHEN** maintainer が macOS 上から Linux support verification を確認する
- **THEN** Ubuntu runner の build / smoke result、log、生成 artifact を参照できる

#### Scenario: CI だけで足りない runtime verification を補う

- **WHEN** target OS の GUI runtime を CI だけで証明できない
- **THEN** 文書には approved manual verification method が記載される
- **THEN** approved method は VM、remote machine、physical machine のいずれかで実行できる
- **THEN** required evidence と release blocking 条件が分かる

### Requirement: インストール導線は対応 OS ごとに文書化される

システムは、README と開発者向け文書に、対応 desktop OS ごとの install / run 手順を記載しなければならない（MUST）。

#### Scenario: README で support matrix を案内する

- **WHEN** ユーザーが repository root の README を読む
- **THEN** macOS、Windows、Linux の support status と配布形式が分かる

#### Scenario: 開発者向け文書で build prerequisites を案内する

- **WHEN** 開発者が development guide を読む
- **THEN** Windows / Linux を含む build prerequisites と実行手順が分かる
