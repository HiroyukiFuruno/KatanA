## ADDED Requirements

### Requirement: user は file 単位で Ollama local LLM autofix を要求できなければならない

システムは、user が markdownlint diagnostics のある file を起点に、Ollama 経由の local LLM に file 単位の autofix を要求できるようにしなければならない（SHALL）。

#### Scenario: file diagnostics から autofix を開始する

- **WHEN** user が markdownlint diagnostics を含む file に対して autofix を要求する
- **THEN** system は official rule code、message、file path、location を含む file-level fix request を構築する
- **THEN** system は active Ollama provider に修正候補を要求する

### Requirement: autofix request は KML の一括 fix 後 content を context に含めなければならない

システムは、KML (`katana-markdown-linter`) が一括 fix できる範囲を反映した後の content を local LLM の context に含めなければならない（SHALL）。

#### Scenario: KML fix 後 content を渡す

- **WHEN** system が file-level autofix request を構築する
- **THEN** system は元 content と KML 一括 fix 後 content を request context に含める
- **THEN** system は残存 diagnostics を request context に含める
- **THEN** system は全 diagnostics の解消を目的とした提案を active Ollama provider に要求する

### Requirement: autofix は file-level diff preview を提示しなければならない

システムは、local LLM が返した修正候補を適用する前に、元 content と提案 content の差分 preview を user に提示しなければならない（SHALL）。

#### Scenario: file-level diff を確認する

- **WHEN** system が local LLM から file-level autofix 候補を受け取る
- **THEN** system は元 content と提案 content の差分を preview 表示する
- **THEN** system は user が差分を確認できるまで apply を実行しない

### Requirement: autofix は user confirmation の後にのみ適用されなければならない

システムは、local LLM が返した修正候補を user の確認なしに適用してはならない（MUST NOT）。

#### Scenario: 修正候補を確認して適用する

- **WHEN** system が local LLM から file-level autofix 候補を受け取る
- **THEN** system は apply 前に user が差分を確認できる表示を行う
- **THEN** user が承認した場合にのみ fix を適用する

### Requirement: provider 未設定または利用不可時は autofix を実行してはならない

システムは、active local provider が未設定または利用不可の場合、autofix を実行してはならず、その理由を user に示さなければならない（MUST NOT / SHALL）。

#### Scenario: provider が利用できない

- **WHEN** user が provider 未設定または unavailable 状態で autofix を要求する
- **THEN** system は fix request を送信しない
- **THEN** system は設定または接続確認へ戻る導線を表示する
