## ADDED Requirements

### Requirement: 対応 OS ごとに command surface を提供する

システムは、対応 desktop OS のいずれでも、主要 command に到達できる command surface を提供しなければならない（MUST）。

#### Scenario: macOS では native menu を使う

- **WHEN** ユーザーが macOS 上で KatanA を使う
- **THEN** 主要 command は native menu から到達できる

#### Scenario: Windows / Linux では in-app command surface を使う

- **WHEN** ユーザーが Windows または Linux 上で KatanA を使う
- **THEN** 主要 command は in-app command surface から到達できる
- **THEN** その command surface は `AppAction` と同等の操作を dispatch する

### Requirement: shortcut は platform の primary modifier に従う

システムは、検索や document 操作の shortcut を current platform の primary modifier に従って解決しなければならない（MUST）。

#### Scenario: macOS の primary modifier

- **WHEN** ユーザーが macOS 上で search shortcut を実行する
- **THEN** primary modifier は `Command` として解決される

#### Scenario: Windows / Linux の primary modifier

- **WHEN** ユーザーが Windows または Linux 上で search shortcut を実行する
- **THEN** primary modifier は `Ctrl` として解決される
