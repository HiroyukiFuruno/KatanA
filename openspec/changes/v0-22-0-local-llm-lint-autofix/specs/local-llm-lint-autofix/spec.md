## ADDED Requirements

### Requirement: user は markdownlint diagnostics に対して local LLM autofix を要求できなければならない

システムは、user が markdownlint diagnostics を起点に local LLM autofix を要求できるようにしなければならない（SHALL）。

#### Scenario: diagnostic から autofix を開始する

- **WHEN** user が markdownlint diagnostic に対して autofix を要求する
- **THEN** system は official rule code、message、file path、location を含む fix request を構築する
- **THEN** system は active local provider に修正候補を要求する

### Requirement: autofix は user confirmation の後にのみ適用されなければならない

システムは、local LLM が返した修正候補を user の確認なしに適用してはならない（MUST NOT）。

#### Scenario: 修正候補を確認して適用する

- **WHEN** system が local LLM から autofix 候補を受け取る
- **THEN** system は apply 前に user が内容を確認できる表示を行う
- **THEN** user が承認した場合にのみ fix を適用する

### Requirement: provider 未設定または利用不可時は autofix を実行してはならない

システムは、active local provider が未設定または利用不可の場合、autofix を実行してはならず、その理由を user に示さなければならない（MUST NOT / SHALL）。

#### Scenario: provider が利用できない

- **WHEN** user が provider 未設定または unavailable 状態で autofix を要求する
- **THEN** system は fix request を送信しない
- **THEN** system は設定または接続確認へ戻る導線を表示する
