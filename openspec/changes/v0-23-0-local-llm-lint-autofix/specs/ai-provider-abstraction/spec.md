## ADDED Requirements

### Requirement: user は Ollama local AI provider を設定できなければならない

システムは、user が Ollama endpoint と model を設定し、active local provider として利用できるようにしなければならない（SHALL）。

#### Scenario: Ollama provider を設定する

- **WHEN** user が Ollama endpoint と model を設定する
- **THEN** system はその provider 設定を永続化する
- **THEN** system は Ollama を active local provider として利用できる

### Requirement: user は軽量 model の推奨導線を確認できなければならない

システムは、user が 1桁GB級の local model を選びやすいように、Ollama model 選択時に推奨導線を提示しなければならない（SHALL）。

#### Scenario: 軽量 model の推奨を見る

- **WHEN** user が local LLM model を選択する
- **THEN** system は軽量 model を優先して選ぶための説明または推奨表示を提示する
- **THEN** system は user が最終的な model を選べる状態を維持する

### Requirement: user は request 送信前に Ollama model を選択しなければならない

システムは、chat または autofix request を送信する前に、user が Ollama model を明示的に選択できるようにしなければならない（SHALL）。

#### Scenario: model 未選択で request する

- **WHEN** user が model 未選択の状態で chat または autofix request を送信しようとする
- **THEN** system は request を送信しない
- **THEN** system はモデル選択へ戻る導線を表示する

#### Scenario: model を選択する

- **WHEN** user が Ollama model を選択する
- **THEN** system は selected model を active local provider 設定として保持する
- **THEN** system は selected model を chat と autofix request に使用する

### Requirement: local provider は利用可否を検証できなければならない

システムは、configured Ollama provider に対して利用可否確認を行い、利用可能状態を user に示さなければならない（SHALL）。

#### Scenario: provider 接続確認を行う

- **WHEN** user が configured Ollama provider の接続確認を実行する
- **THEN** system は availability を判定する
- **THEN** system は success または failure の状態を user に表示する
