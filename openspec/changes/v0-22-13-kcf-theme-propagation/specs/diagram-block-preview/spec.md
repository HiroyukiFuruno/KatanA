## MODIFIED Requirements

### Requirement: ダイアグラムプレビューは現在のテーマスナップショットを使用する

システムは、アプリ起動時点のスナップショットや dark/light 切り替えだけに依存するのではなく、現在のテーマスナップショットに基づいてダイアグラムプレビューを描画しなければならない（SHALL）。kcf backed renderer を利用する Mermaid / Draw.io でも、KatanA が渡したテーマスナップショットを kcf の実描画が使用しなければならない（MUST）。

#### Scenario: Mermaid プレビューが同一モード内の色変更へ追従する

- **WHEN** ユーザーが dark/light モードを変えずに preview text color や関連テーマ色を変更した時
- **THEN** Mermaid 描画は更新後のテーマスナップショットを使用する
- **THEN** プレビューは旧色セットで描かれた古いダイアグラム画像を再利用しない

#### Scenario: PlantUML プレビューが同一モード内の色変更へ追従する

- **WHEN** ユーザーが dark/light モードを変えずに preview text color や関連テーマ色を変更した時
- **THEN** PlantUML 描画は更新後のテーマスナップショットを使用する
- **THEN** プレビューは旧色セットで描かれた古いダイアグラム画像を再利用しない

#### Scenario: kcf backed Mermaid プレビューが light テーマを使用する

- **WHEN** KatanA の active theme が light mode の状態で Mermaid block を描画する
- **THEN** KatanA は kcf の `RenderInput` に light テーマの名前、背景、文字色、塗り、線、矢印、Mermaid theme を渡す
- **THEN** kcf が返す SVG は kcf 内部の dark 既定値ではなく、KatanA が渡した light テーマに基づく
- **THEN** 画面上の Mermaid 図形は dark 背景・白文字寄りの配色へ戻らない

#### Scenario: kcf backed Draw.io プレビューが light テーマを使用する

- **WHEN** KatanA の active theme が light mode の状態で Draw.io block を描画する
- **THEN** KatanA は kcf の `RenderInput` に light テーマの名前、背景、文字色、塗り、線、矢印、Draw.io label color を渡す
- **THEN** kcf が返す SVG は kcf 内部の dark 既定値ではなく、KatanA が渡した light テーマに基づく
- **THEN** 画面上の Draw.io 図形は dark 背景・白文字寄りの配色へ戻らない

### Requirement: ダイアグラムキャッシュキーはテーマ差分を識別する

システムは、永続化されるダイアグラムキャッシュキーに active なダイアグラムテーマの fingerprint を含めなければならない（SHALL）。kcf backed renderer では、KatanA 側の cache key と kcf 側の `cache_fingerprint` が、実描画に使われたテーマ差分で変化しなければならない（MUST）。

#### Scenario: テーマ fingerprint が変化する

- **WHEN** 同じ markdown file、diagram kind、source に対して active なダイアグラムテーマ fingerprint が変わった時
- **THEN** キャッシュキーは変化する
- **THEN** システムは古いキャッシュ結果を再利用せず、ダイアグラムを再描画する

#### Scenario: kcf runtime と profile の差分で cache key が変化する

- **WHEN** kcf の runtime version または renderer profile が変わった時
- **THEN** KatanA の diagram cache key は変化する
- **THEN** KatanA は古い kcf 出力を再利用しない

#### Scenario: kcf の crate version を手書きで固定しない

- **WHEN** KatanA が kcf backed renderer の cache key または backend version を組み立てる
- **THEN** system は実際の `katana-canvas-forge` dependency version、`RenderOutput.runtime`、`RenderOutput.profile` から識別情報を得る
- **THEN** `crate=katana-canvas-forge:0.1.0` のような古い手書き文字列を cache invalidation の根拠にしない
