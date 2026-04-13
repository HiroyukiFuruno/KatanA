## ADDED Requirements

### Requirement: explorer は Markdown から参照されている local image asset の thumbnail を表示できなければならない

システムは、workspace 内 Markdown から参照され、かつ decode 可能な local image asset について、explorer row 上に thumbnail を表示できなければならない（SHALL）。

#### Scenario: referenced local image に thumbnail を表示する

- **WHEN** explorer row が Markdown から参照されている local image asset を表している
- **THEN** system は row 上に thumbnail を表示する

#### Scenario: missing または decode 不能な image は安全にフォールバックする

- **WHEN** explorer row の image asset が missing または decode 不能である
- **THEN** system は explorer row を壊さず、thumbnail なしまたは placeholder 表示へフォールバックする

### Requirement: explorer thumbnail は lazy hydration で描画しなければならない

システムは、多数の image asset を含む workspace でも explorer 初期表示を劣化させないため、thumbnail を遅延表示しなければならない（SHALL）。

#### Scenario: 初回 explorer load では text row を優先する

- **WHEN** user が referenced image asset を多数含む workspace を開く
- **THEN** system は explorer row の text / icon を先に表示する
- **THEN** thumbnail は後続の lazy hydration cycle で順次表示される
