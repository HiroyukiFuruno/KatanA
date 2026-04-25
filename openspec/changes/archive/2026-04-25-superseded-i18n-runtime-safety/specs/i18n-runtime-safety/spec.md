## ADDED Requirements

### Requirement: Unsupported runtime language falls back safely

システムは、runtime で選択された language code が embedded dictionary に存在しない場合、panic せず fallback language を使わなければならない（SHALL）。

#### Scenario: Unknown language from settings

- **WHEN** settings から未知の language code が読み込まれる
- **THEN** system は fallback language を選ぶ
- **THEN** app startup は panic しない

#### Scenario: Embedded locale is invalid

- **WHEN** embedded locale JSON が schema と一致しない
- **THEN** system は開発時の検証または startup で fail fast する
- **THEN** user-provided language fallback と embedded corruption を混同しない

### Requirement: Parameterized message formatting uses an adapter

システムは、parameterized i18n message の formatting を UI call site の文字列置換ではなく、KatanA-owned formatter adapter 経由で実行しなければならない（SHALL）。

#### Scenario: Format message with named parameters

- **WHEN** UI が `{count}` や `{file}` を含む message を表示する
- **THEN** UI は formatter adapter に message id と typed arguments を渡す
- **THEN** UI は手作業で `{key}` を置換しない

#### Scenario: Missing parameter

- **WHEN** formatter が必要な named parameter を受け取れない
- **THEN** system は deterministic な fallback 表示または error state を返す
- **THEN** UI は panic しない

### Requirement: Selected plural-sensitive messages support locale-aware formatting

システムは、移行対象として選ばれた count-based message について、locale-aware な plural formatting を扱えなければならない（SHALL）。

#### Scenario: One result

- **WHEN** formatter が count `1` を受け取る
- **THEN** system は対象 locale の単数または相当表現を選ぶ

#### Scenario: Multiple results

- **WHEN** formatter が count `2` 以上を受け取る
- **THEN** system は対象 locale の plural rule に沿った表現を選ぶ

### Requirement: Locale quality checks cover current data

システムは、現在の locale files に対して missing key、pseudo-translation、formatter key 不足を検査しなければならない（MUST）。

#### Scenario: Locale key is missing

- **WHEN** supported locale が required message key を持たない
- **THEN** locale quality check は失敗する

#### Scenario: Pseudo-translation remains

- **WHEN** locale value が fallback marker または pseudo-translation marker を含む
- **THEN** locale quality check は失敗する
