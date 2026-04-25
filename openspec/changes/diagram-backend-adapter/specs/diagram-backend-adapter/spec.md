## ADDED Requirements

### Requirement: Diagram rendering uses backend adapters

システムは、Mermaid と PlantUML の rendering を concrete CLI / jar / crate 直接呼び出しではなく、KatanA-owned backend adapter 経由で実行しなければならない（SHALL）。

#### Scenario: Render Mermaid through adapter

- **WHEN** Mermaid block が render される
- **THEN** system は Mermaid backend adapter を通じて backend を選ぶ
- **THEN** preview code は `mmdc` を直接呼び出さない

#### Scenario: Render PlantUML through adapter

- **WHEN** PlantUML block が render される
- **THEN** system は PlantUML backend adapter を通じて backend を選ぶ
- **THEN** preview code は `java` や `plantuml.jar` を直接呼び出さない

### Requirement: Existing external backend behavior is preserved

システムは、adapter migration の初期段階で現行 `mmdc` と Java jar backend の user-visible behavior を維持しなければならない（MUST）。

#### Scenario: Mermaid CLI is available

- **WHEN** `mmdc` が利用可能で Mermaid block が valid である
- **THEN** preview は adapter 経由で diagram を表示する
- **THEN** 表示結果と fallback behavior は migration 前と同等である

#### Scenario: PlantUML jar is missing

- **WHEN** `plantuml.jar` が存在しない
- **THEN** system は adapter 経由で not-installed state を返す
- **THEN** Markdown preview 全体は維持される

### Requirement: Rust-native backends require parity gates

システムは、Rust-native diagram backend を default にする前に parity gate を通さなければならない（MUST）。

#### Scenario: Evaluate Rust Mermaid backend

- **WHEN** Rust Mermaid backend candidate が追加される
- **THEN** system は fixture parity、theme propagation、error behavior、export compatibility を検証する
- **THEN** gate 合格前は default backend にしない

#### Scenario: Evaluate Rust PlantUML backend

- **WHEN** Rust PlantUML backend candidate が追加される
- **THEN** system は fixture parity、license、packaging、Graphviz 関連制約を検証する
- **THEN** gate 合格前は default backend にしない

### Requirement: Diagram backend failure preserves preview

システムは、diagram backend が失敗しても Markdown preview 全体を壊してはならない（MUST）。

#### Scenario: Backend returns error

- **WHEN** selected backend が diagram render error を返す
- **THEN** system は対象 block に recoverable error state を表示する
- **THEN** 他の Markdown content は表示される
