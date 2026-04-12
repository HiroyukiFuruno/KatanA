## ADDED Requirements

### Requirement: user-facing commands は shared inventory で定義されなければならない

システムは、menu、command palette、future shortcut editor が参照する user-facing commands を shared inventory で定義しなければならない（MUST）。

#### Scenario: menu と palette が同じ command metadata を使う

- **WHEN** system が user-facing command を menu または palette に表示する
- **THEN** 両者は同じ label、group、availability metadata を参照する
- **THEN** surface ごとの command drift が発生しない

### Requirement: command availability は surface 間で一貫しなければならない

システムは、workspace や active document の有無に応じた command availability を surface 間で一貫して扱わなければならない（SHALL）。

#### Scenario: active document がない

- **WHEN** active document が存在しない
- **THEN** save、export、document diagnostics など document-dependent command は disabled になる
- **THEN** menu と palette は同じ availability judgment を反映する
