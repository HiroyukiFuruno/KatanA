## ADDED Requirements

### Requirement: user は active local AI provider を設定して切り替えできなければならない

システムは、user が local AI provider の種類、endpoint、model を設定し、active provider を切り替えできるようにしなければならない（SHALL）。

#### Scenario: local provider を選択する

- **WHEN** user が `Ollama`、`LM Studio`、または OpenAI 互換 local endpoint を設定する
- **THEN** system はその provider 設定を永続化する
- **THEN** system は active provider として選択できる

### Requirement: local provider は利用可否を検証できなければならない

システムは、configured local provider に対して利用可否確認を行い、利用可能状態を user に示さなければならない（SHALL）。

#### Scenario: provider 接続確認を行う

- **WHEN** user が configured local provider の接続確認を実行する
- **THEN** system は availability を判定する
- **THEN** system は success または failure の状態を user に表示する
