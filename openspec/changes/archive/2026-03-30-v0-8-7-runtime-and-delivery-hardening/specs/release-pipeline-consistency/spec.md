## ADDED Requirements

### Requirement: ローカルとCIのreleaseは同一のpreflight契約を使う

ローカル release と GitHub Actions release は、version、changelog、OpenSpec 完了状態、artifact 契約を同一の preflight entrypoint で検証しなければならない（MUST）。

#### Scenario: GitHub Actionsからreleaseを実行する

- **WHEN** maintainer が release workflow を手動実行する
- **THEN** workflow は `make release` と同じ preflight entrypoint を実行する
- **THEN** changelog 欠落、version 不一致、未完了の OpenSpec tasks がある場合は publish 前に失敗する

#### Scenario: ローカルからreleaseを実行する

- **WHEN** maintainer が `make release VERSION=x.y.z` を実行する
- **THEN** GitHub Actions と同一の preflight entrypoint が実行される
- **THEN** 事前検証を通らない限り push / publish へ進まない

### Requirement: Release artifact 契約は明示的に検証される

release で生成される成果物は、命名規則と内容の整合性を publish 前に検証しなければならない（SHALL）。

#### Scenario: release 成果物を生成する

- **WHEN** release build が成功する
- **THEN** DMG、ZIP、checksums は canonical なファイル名で生成される
- **THEN** checksums は publish 対象となるすべての binary artifact を対象に含む

#### Scenario: release version と成果物名が食い違う

- **WHEN** artifact filename や release note section の version が要求 version と一致しない
- **THEN** release は publish 前に失敗する

### Requirement: Release helper は smoke test 可能である

release helper script 群は、外部 publish を伴わない smoke validation を CI から実行できなければならない（MUST）。

#### Scenario: CIでrelease helperを検証する

- **WHEN** CI が release pipeline の回帰を検証する
- **THEN** preflight と artifact verification は push / tag / GitHub Release / Homebrew 更新なしで実行できる
- **THEN** script の引数契約や必須ファイル検証の回帰を publish 前に検出できる
