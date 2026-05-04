## ADDED Requirements

### Requirement: local LLM が有効な場合、eligible な dynamic English text は自動翻訳表示されなければならない

システムは、local LLM が有効で UI language が英語以外の場合、eligible な dynamic / external English text を自動翻訳表示しなければならない（SHALL）。

#### Scenario: diagnostics の英語説明を自動翻訳表示する

- **WHEN** local LLM が有効で、markdownlint diagnostic の英語説明が表示される
- **THEN** system は原文を保持したまま translated view を自動表示する

#### Scenario: AI generation result の英語 text を自動翻訳表示する

- **WHEN** local LLM が有効で、AI generation result に英語 text が含まれる
- **THEN** system は eligible text に translation overlay を適用する

### Requirement: original English text は参照可能でなければならない

システムは、translation overlay を適用しても original English text を user が参照できるようにしなければならない（SHALL）。

#### Scenario: 原文を確認する

- **WHEN** user が translated view を見ている
- **THEN** system は original English text を参照する導線を提供する

### Requirement: translation failure 時は英語表示へ安全に fallback しなければならない

システムは、translation request が失敗した場合、表示を壊さず original English text に fallback しなければならない（SHALL）。

#### Scenario: translation request が失敗する

- **WHEN** local provider unavailable、timeout、または invalid response により translation が失敗する
- **THEN** system は original English text を表示し続ける
- **THEN** system は翻訳失敗で対象 UI を壊さない

### Requirement: translation result は再利用可能な cache を持たなければならない

システムは、同一 source text の繰り返し翻訳を避けるため、translation result を cache しなければならない（SHALL）。

#### Scenario: 同じ text を再表示する

- **WHEN** user が同じ source text を再び表示する
- **THEN** system は利用可能な cache があればそれを再利用する

### Requirement: translation overlay は自分自身または非対象 text を再翻訳してはならない

システムは、overlay によって生成した translated text、既に非英語と判定できる text、または translation in progress の text を再び translation target に含めてはならない（MUST NOT）。

#### Scenario: translated overlay を再評価する

- **WHEN** translated overlay text が再描画される
- **THEN** system はその overlay text を新たな translation request の source にしない
- **THEN** original English source のみを translation source of truth として扱う

#### Scenario: 非英語 text が表示される

- **WHEN** source text が英語以外であると判定できる
- **THEN** system は translation request を送信しない
