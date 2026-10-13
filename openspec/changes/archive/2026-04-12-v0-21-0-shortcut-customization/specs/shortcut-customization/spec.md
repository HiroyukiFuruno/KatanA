## ADDED Requirements

### Requirement: user は command shortcut を設定画面から変更できなければならない

システムは、user が command ごとの shortcut を設定画面から確認および変更できるようにしなければならない（SHALL）。

#### Scenario: shortcut 一覧を表示する

- **WHEN** user が shortcut settings を開く
- **THEN** system は command 名と current shortcut を一覧表示する

#### Scenario: shortcut を変更する

- **WHEN** user が command の shortcut を新しい key combination に変更する
- **THEN** system はその binding を保存し、次回起動時にも復元する

### Requirement: duplicate shortcut は登録してはならない

システムは、既に別 command へ割り当てられている shortcut を重複登録してはならない（MUST NOT）。

#### Scenario: 既存割当と衝突する

- **WHEN** user が既存 binding と同じ shortcut を別 command に設定しようとする
- **THEN** system は保存を拒否する
- **THEN** popup で既存割当先 command を表示する

### Requirement: user は default shortcuts を復元できなければならない

システムは、user が default shortcuts を復元できるようにしなければならない（SHALL）。

#### Scenario: defaults に戻す

- **WHEN** user が restore defaults を実行する
- **THEN** custom shortcut bindings は default set に戻る
