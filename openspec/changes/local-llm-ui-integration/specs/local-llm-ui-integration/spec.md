## ADDED Requirements

### Requirement: Integration work starts after v0.23.0 MVP reaches master

システムは、`v0-23-0-local-llm-lint-autofix` の LLM MVP が `master` に merge された後に、local LLM UI integration の実装へ進まなければならない（MUST）。

#### Scenario: v0.23.0 implementation is only on release branch

- **WHEN** Ollama 設定、chat UI 土台、file-level autofix / diff preview が `release/v0.23.0` にだけ存在する
- **THEN** system は local LLM UI integration の task を完了済みとして扱わない
- **THEN** implementer は `master` merge 後の残差分だけをこの change の対象にする

### Requirement: Local LLM settings are the recovery entry point

システムは、local LLM の provider 未設定、model 未選択、接続不可状態を検出したとき、AI 設定画面へ戻る復旧導線を提供しなければならない（SHALL）。

#### Scenario: Model is missing from chat

- **WHEN** user が model 未選択の状態で chat message を送信しようとする
- **THEN** system は request を送信しない
- **THEN** system は model 選択へ進む導線を表示する

#### Scenario: Provider is missing from autofix

- **WHEN** user が provider 未設定の状態で diagnostics autofix を実行しようとする
- **THEN** system は autofix request を送信しない
- **THEN** system は AI 設定画面へ戻る導線を表示する

### Requirement: Diagnostics autofix entry point reflects availability

システムは、diagnostics UI 上の file-level autofix entry point について、実行可能状態と実行不能理由をユーザーに示さなければならない（SHALL）。

#### Scenario: Autofix is available

- **WHEN** 対象 file に diagnostics があり、provider と model が利用可能である
- **THEN** system は file-level autofix を開始できる操作を表示する
- **THEN** system は single diagnostic ではなく file 単位の autofix request を開始する

#### Scenario: Autofix is unavailable

- **WHEN** provider、model、diagnostics のいずれかが不足している
- **THEN** system は不足理由を表示する
- **THEN** system は不足状態のまま LLM request を送信しない

### Requirement: Chat and autofix use consistent provider status

システムは、chat と diagnostics autofix が同じ provider availability の判断を使うようにしなければならない（SHALL）。

#### Scenario: Provider check fails

- **WHEN** Ollama availability check が失敗する
- **THEN** chat UI は送信不可状態を表示する
- **THEN** diagnostics autofix UI も同じ provider unavailable 状態を表示する

### Requirement: UI verification avoids visual snapshot dependency

システムは、local LLM UI integration の回帰検知を、画像 snapshot だけに依存せず、状態と action の semantic assertions で検証しなければならない（MUST）。

#### Scenario: Verify disabled state

- **WHEN** UI test が provider 未設定状態を構築する
- **THEN** test は表示文言、disabled state、settings navigation action を確認する
- **THEN** test は pixel snapshot の一致だけを合格条件にしない
