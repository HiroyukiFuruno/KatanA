## ADDED Requirements

### Requirement: user は current document に local LLM 生成結果を挿入できなければならない

システムは、user が active Markdown document に対して local LLM の生成結果を挿入できるようにしなければならない（SHALL）。

#### Scenario: current document に生成結果を挿入する

- **WHEN** user が active document を対象に generation を実行する
- **THEN** system は current document context を用いて content を生成する
- **THEN** user の確認後にのみ指定位置へ生成結果を挿入する

### Requirement: user は新規 Markdown file を生成できなければならない

システムは、user が local LLM を用いて新規 Markdown file を生成し、workspace に保存できるようにしなければならない（SHALL）。

#### Scenario: new file を生成して保存する

- **WHEN** user が new file generation を実行し、保存先を確定する
- **THEN** system は generated Markdown を preview 可能にする
- **THEN** user の確認後に新規 file として保存する

### Requirement: user は template scaffold を生成できなければならない

システムは、user が preset または template context を基にした Markdown scaffold を生成できるようにしなければならない（SHALL）。

#### Scenario: template scaffold を生成する

- **WHEN** user が template-based generation を実行する
- **THEN** system は selected preset と user prompt を基に scaffold を生成する
- **THEN** user の確認後に target destination へ反映する

### Requirement: generation 結果は write 前に確認できなければならない

システムは、current document 挿入、新規 file 作成、template scaffold のいずれにおいても、write 前に生成結果を user が確認できるようにしなければならない（SHALL）。

#### Scenario: 生成結果を確認してから反映する

- **WHEN** system が generation result を受け取る
- **THEN** system は反映前の preview または差分確認を提示する
- **THEN** user が承認した場合にのみ write を行う
