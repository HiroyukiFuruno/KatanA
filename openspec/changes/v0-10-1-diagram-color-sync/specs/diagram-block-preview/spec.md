## MODIFIED Requirements

### Requirement: ダイアグラムプレビューは現在のテーマスナップショットを使用する

システムは、アプリ起動時点のスナップショットや dark/light 切り替えだけに依存するのではなく、現在のテーマスナップショットに基づいてダイアグラムプレビューを描画しなければならない（SHALL）。

#### Scenario: Mermaid プレビューが同一モード内の色変更へ追従する

- **WHEN** ユーザーが dark/light モードを変えずに preview text color や関連テーマ色を変更した時
- **THEN** Mermaid 描画は更新後のテーマスナップショットを使用する
- **THEN** プレビューは旧色セットで描かれた古いダイアグラム画像を再利用しない

#### Scenario: PlantUML プレビューが同一モード内の色変更へ追従する

- **WHEN** ユーザーが dark/light モードを変えずに preview text color や関連テーマ色を変更した時
- **THEN** PlantUML 描画は更新後のテーマスナップショットを使用する
- **THEN** プレビューは旧色セットで描かれた古いダイアグラム画像を再利用しない

### Requirement: ダイアグラムキャッシュキーはテーマ差分を識別する

システムは、永続化されるダイアグラムキャッシュキーに active なダイアグラムテーマの fingerprint を含めなければならない（SHALL）。

#### Scenario: テーマ fingerprint が変化する

- **WHEN** 同じ markdown file、diagram kind、source に対して active なダイアグラムテーマ fingerprint が変わった時
- **THEN** キャッシュキーは変化する
- **THEN** システムは古いキャッシュ結果を再利用せず、ダイアグラムを再描画する
