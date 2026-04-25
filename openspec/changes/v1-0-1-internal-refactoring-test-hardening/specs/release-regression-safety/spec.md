## ADDED Requirements

### Requirement: Release regression plan is based on refreshed source analysis

システムは、2026-04-25 の策定時点から実装着手時までの差分を反映したうえで、release regression gate の対象 workflow を確定しなければならない（MUST）。

#### Scenario: Refresh regression plan

- **WHEN** implementer が release regression gate の実装へ入る
- **THEN** implementer は現在の test runner、Makefile、既存 integration tests、既存 regression tests を確認する
- **THEN** implementer は不足している workflow と既に十分な workflow を分けて tasks に反映する

### Requirement: Release regression gate covers core product workflows

システムは、正式リリース後に壊してはいけない core workflow を release regression gate として定義しなければならない（SHALL）。

#### Scenario: Core workflow gate runs

- **WHEN** release regression gate が実行される
- **THEN** document open / edit / save、workspace navigation、preview render、diagnostics、settings persistence、export の代表 workflow を検証する
- **THEN** gate の対象 workflow は tasks または test manifest で確認できる

### Requirement: Tests are organized by contract

システムは、UT / IT を file size だけでなく、守る contract 単位で整理しなければならない（SHALL）。

#### Scenario: Pure logic test

- **WHEN** pure logic や state transition を検証する
- **THEN** test は unit contract として production module 近くに配置する
- **THEN** UI harness を不要にする

#### Scenario: User workflow test

- **WHEN** user action から UI state までを検証する
- **THEN** test は integration contract として harness を使う
- **THEN** assertion は semantic state または rendered semantic output を確認する

### Requirement: Regression tests preserve known bug contracts

システムは、過去に修正した重要 bug について、再発検知できる regression test を維持しなければならない（MUST）。

#### Scenario: Add regression test

- **WHEN** bug fix が行われる
- **THEN** fix 前に失敗する regression test または同等の再現 test を追加する
- **THEN** test 名または fixture が守る behavior を説明する

### Requirement: Test harness avoids fixed waiting

システムは、integration test harness で固定待機に依存せず、状態条件または event 条件で待機しなければならない（MUST）。

#### Scenario: Wait for preview render

- **WHEN** test が preview render 完了を待つ
- **THEN** harness は render state または semantic condition を待つ
- **THEN** arbitrary sleep による成功を合格条件にしない

### Requirement: Visual snapshot is not the primary regression oracle

システムは、新規 regression gate の主判定を visual snapshot の一致に依存してはならない（MUST NOT）。

#### Scenario: Verify UI behavior

- **WHEN** UI behavior を検証する
- **THEN** test は state、layout rect、semantic text、action dispatch、render metadata を確認する
- **THEN** pixel snapshot の一致だけを合格条件にしない
