## ADDED Requirements

### Requirement: Markdown editor は常設の authoring toolbar を提供しなければならない

システムは、editable な active Markdown document に対して、見出し、装飾、リスト、表、画像挿入を GUI から実行できる authoring toolbar を表示しなければならない（SHALL）。

#### Scenario: editable document で authoring toolbar を表示する

- **WHEN** user が editable な active Markdown document を開く
- **THEN** system は editor 近傍に authoring toolbar を表示する
- **THEN** toolbar の icon は system SVG icon で描画される

#### Scenario: 編集不能 document では unsupported control を無効化する

- **WHEN** active document が reference document または非編集状態である
- **THEN** system は編集系 toolbar action を disabled として表示する

### Requirement: GUI authoring control は command / shortcut と同じ transform contract を使わなければならない

システムは、toolbar や context menu から実行される Markdown authoring action について、既存 command / shortcut と同じ text transform contract を共有しなければならない（SHALL）。

#### Scenario: toolbar から見出しや装飾を適用する

- **WHEN** user が toolbar の heading、bold、list、table などの control を実行する
- **THEN** system は command / shortcut 実行時と同じ Markdown transform を active selection または cursor へ適用する
- **THEN** preview は更新後の source buffer に同期する
