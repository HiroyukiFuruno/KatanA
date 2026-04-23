## ADDED Requirements

### Requirement: KatanA SHALL pass rule configuration into Markdown rule evaluation

KatanA は、Markdown rule evaluation 時に対象 rule の configuration を渡さなければならない（SHALL）。

#### Scenario: rule config を評価へ渡す

- **WHEN** KatanA が Markdown document を lint する
- **THEN** KatanA は `.markdownlint.json` 由来の対象 rule config を `MarkdownRule::evaluate` に渡す
- **THEN** rule は config が存在しない場合に upstream default と同等の挙動を保つ

### Requirement: KatanA SHALL validate markdownlint configuration values before vendor switch

KatanA は、vendor linter へ切り替える前に markdownlint configuration values を validation できなければならない（SHALL）。

#### Scenario: invalid config を検出する

- **WHEN** user が不正な rule property を含む `.markdownlint.json` を保存する
- **THEN** KatanA は rule ID、property name、error kind を含む validation error を生成する
- **THEN** KatanA は unknown property、type mismatch、enum value mismatch を区別する

### Requirement: KatanA SHALL define a vendor linter adapter boundary

KatanA は、`katana-markdown-linter` の lint result を KatanA 内部 diagnostic へ変換する adapter boundary を定義しなければならない（SHALL）。

#### Scenario: vendor result を diagnostic に変換する

- **WHEN** `katana-markdown-linter` が lint result を返す
- **THEN** KatanA は rule ID、message、severity、range、fix availability を `MarkdownDiagnostic` に変換できる
- **THEN** KatanA は UI diagnostic rendering の contract を変更しない

### Requirement: KatanA SHALL gate dependency switch on vendor parity completion

KatanA は、`katana-markdown-linter` への dependency switch を vendor parity 完了条件で gate しなければならない（SHALL）。

#### Scenario: dependency switch を判断する

- **WHEN** KatanA が `vendor-linter` feature を有効化して dependency switch を検証する
- **THEN** `katana-markdown-linter` は全 active rule の check 実装を持つ
- **THEN** KatanA は internal implementation と vendor implementation の diagnostic count、rule ID、range、message、fix availability を比較する
- **THEN** regression がある場合は switch を完了しない

### Requirement: KatanA SHALL align with confirmed vendor package metadata

KatanA は、`katana-markdown-linter` の確定済み package metadata と矛盾してはならない（SHALL）。

#### Scenario: vendor package metadata を参照する

- **WHEN** KatanA が vendor linter を dependency として取り込む
- **THEN** package 名は `katana-markdown-linter` として扱う
- **THEN** Rust module 名は `katana_markdown_linter` として扱う
- **THEN** license は MIT として扱う
- **THEN** KatanA は CLI executable `kml` には runtime dependency を持たない

### Requirement: KatanA SHALL treat JSONC support as part of the future markdownlint config surface

KatanA は、`.markdownlint.jsonc` support を将来の markdownlint config surface として扱わなければならない（SHALL）。

#### Scenario: JSONC support を設計に含める

- **WHEN** KatanA が vendor linter との config boundary を設計する
- **THEN** `.markdownlint.json` と `.markdownlint.jsonc` の両方を想定する
- **THEN** JSONC support を internal implementation で即時実装しない場合も、adapter / validation の設計上の除外理由を記録する

### Requirement: KatanA SHALL use upstream drift tracking as successor to stub parity checks

KatanA は、Phase 5 の upstream drift tracking を `ast_linter_stubs_parity` の後継 gate として扱わなければならない（SHALL）。

#### Scenario: upstream drift tracking へ移行する

- **WHEN** `katana-markdown-linter` Phase 5 が利用可能になる
- **THEN** KatanA は `DavidAnson/markdownlint` default branch 追従の drift report を参照する
- **THEN** KatanA は local stub parity test の廃止可否を drift report で判断する
