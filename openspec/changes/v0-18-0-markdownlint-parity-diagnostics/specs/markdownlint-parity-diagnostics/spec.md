## ADDED Requirements

### Requirement: user-facing markdownlint diagnostics は official behavior に一致しなければならない

システムは、user-facing に出荷する markdownlint diagnostics について、rule code だけでなく detection behavior も official markdownlint behavior に一致させなければならない（MUST）。

#### Scenario: 出荷対象 rule の violation を検出する

- **WHEN** active Markdown document が official markdownlint rule に違反している
- **THEN** system は official rule code に対応する diagnostic を生成する
- **THEN** diagnostic location と severity はその rule の official behavior と整合する

#### Scenario: valid case を false positive なく通過させる

- **WHEN** active Markdown document が official markdownlint rule を満たしている
- **THEN** system はその rule に関する false positive を生成しない

### Requirement: Problems Panel は official markdownlint metadata を表示しなければならない

システムは、user-facing markdownlint diagnostics を Problems Panel に表示する際、official metadata を user-facing contract として扱わなければならない（SHALL）。

#### Scenario: official rule code と English description を表示する

- **WHEN** Problems Panel に markdownlint diagnostic が表示される
- **THEN** item は official rule code を表示する
- **THEN** item は official English description または title を表示する

#### Scenario: docs link を辿れる

- **WHEN** ユーザーが diagnostic の詳細を確認する
- **THEN** system はその official rule に対応する docs link を提供する

### Requirement: parity 未達の internal rules は official result として見せてはならない

システムは、official parity が取れていない internal rules を markdownlint official result として見せてはならず、user-facing default から分離しなければならない（MUST NOT / MUST）。

#### Scenario: parity 未達 rule が残っている

- **WHEN** internal rule が official parity を満たしていない
- **THEN** system はその rule を official markdownlint diagnostic として Problems Panel に表示しない
- **THEN** user は hidden または experimental の扱いを区別できる

### Requirement: diagnostics payload は future autofix で再利用できなければならない

システムは、markdownlint diagnostic payload に official rule code、message、location、file path を含め、future local LLM autofix がそのまま利用できる shape を維持しなければならない（SHALL）。

#### Scenario: diagnostics payload を autofix input に渡す

- **WHEN** future autofix flow が diagnostic item を参照する
- **THEN** payload には official rule code、message、file path、location が含まれる
- **THEN** additional lookup なしで autofix request を構築できる
