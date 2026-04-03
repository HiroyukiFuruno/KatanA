## ADDED Requirements

### Requirement: post-v0.18 roadmap は minor version ごとに primary concern を割り当てる

システムは、post-`v0.18.0` の roadmap を versioned artifact として保持し、各 planned minor version に対して 1 つの primary concern と prerequisite を割り当てなければならない（SHALL）。

#### Scenario: minor version ごとの concern を定義する

- **WHEN** roadmap が `v0.19.0` 以降の release plan として記録される
- **THEN** 各 minor version entry は 1 つの primary concern を持つ
- **THEN** 各 entry は、その concern より先に必要な prerequisite version または prerequisite decision を持つ

#### Scenario: dependency order を保つ

- **WHEN** ある concern が先行する command, settings, diagnostics, or provider foundation に依存する
- **THEN** roadmap は dependent concern を prerequisite concern より後の minor version に配置する
- **THEN** roadmap entry は依存関係を明示する

### Requirement: roadmap entry は implementation readiness contract を持つ

システムは、各 roadmap entry に対して scope、affected modules/specs、Definition of Ready、Definition of Done、open questions または fixed assumptions を記録しなければならない（MUST）。

#### Scenario: entry に DoR / DoD がある

- **WHEN** roadmap entry が作成される
- **THEN** entry には implementation に入る前提条件を示す Definition of Ready が含まれる
- **THEN** entry には完了判定を示す Definition of Done が含まれる

#### Scenario: affected area を明示する

- **WHEN** roadmap entry が作成される
- **THEN** entry は関連する capability、module、または major file area を示す
- **THEN** future dedicated change を切る実装者が調査開始点を失わない

### Requirement: roadmap は高リスク concern に対して明示的な scoping constraint を保持する

システムは、editor model の変更、local LLM integration、official rule compatibility のような high-risk concern に対して、non-goals または initial scope constraints を記録しなければならない（MUST）。

#### Scenario: full parity を約束しない

- **WHEN** roadmap entry が external standard との compatibility を扱う
- **THEN** entry は initial release で同期する対象を明示する
- **THEN** entry は初期スコープで扱わない parity or behavior を non-goal として保持する

#### Scenario: runtime choice が未確定である

- **WHEN** roadmap entry が local LLM or translation runtime を扱う
- **THEN** entry は未確定事項を open question として残す
- **THEN** entry は runtime choice 未解決の状態を implementation-ready とみなさない
