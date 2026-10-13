## ADDED Requirements

### Requirement: Implementation plan is refreshed before work starts

システムは、2026-04-25 の策定時点から実装着手時までの差分を確認し、内部リファクタリング計画を最新のコード状態へ更新しなければならない（MUST）。

#### Scenario: Start implementation after planning date

- **WHEN** implementer がこの change の task0 に着手する
- **THEN** implementer は 2026-04-25 から着手日までの `master`、active OpenSpec、`katana-ui` 構造、test runner の差分を確認する
- **THEN** implementer は新しい事実を `design.md` または `tasks.md` に反映してから task1 以降へ進む

### Requirement: Refactoring separates mechanical moves from behavioral redesign

システムの内部リファクタリング計画は、単純なファイル移動と、実装責務の再設計を分けて扱わなければならない（MUST）。

#### Scenario: Mechanical move

- **WHEN** module の責務が変わらず file path だけを移す
- **THEN** task は機械的な移動として記録する
- **THEN** 同じ commit で挙動変更を混ぜない

#### Scenario: Boundary redesign

- **WHEN** action、state、service、view の責務境界を変える
- **THEN** task は redesign として扱う
- **THEN** 先に contract test または behavior-preserving verification を定義する

### Requirement: Application actions are grouped by domain

システムは、document、workspace、layout、settings、preview、diagnostics などの action を領域単位に分割し、root action は top-level routing に限定できる構造へ移行しなければならない（SHALL）。

#### Scenario: Add document action

- **WHEN** document 操作の action が追加される
- **THEN** document 領域 action に追加される
- **THEN** unrelated workspace や settings dispatch へ影響しない

#### Scenario: Dispatch action

- **WHEN** root dispatcher が action を受け取る
- **THEN** root dispatcher は domain handler へ routing する
- **THEN** 領域 handler が該当 feature の state invariants を守る

### Requirement: Feature state protects invariants

システムは、root state の public mutable field に直接依存する範囲を減らし、feature state が自身の invariants を守る API を提供しなければならない（SHALL）。

#### Scenario: Mutate active document

- **WHEN** active document buffer が更新される
- **THEN** document feature state は dirty flag、active index、virtual path の扱いを一貫して処理する
- **THEN** caller は複数 field を個別に更新しない

#### Scenario: Mutate workspace state

- **WHEN** workspace tree が refresh される
- **THEN** workspace feature state は current root、history、selection の整合性を内部で保持する

### Requirement: View modules remain presentation-focused

システムは、egui view module が domain mutation や filesystem mutation を直接所有しない構造へ移行しなければならない（SHALL）。

#### Scenario: Button triggers mutation

- **WHEN** view 上の button が document save を要求する
- **THEN** view は action を発行する
- **THEN** save の実処理は app service または domain handler が行う
