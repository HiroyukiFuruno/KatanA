## ADDED Requirements

### Requirement: 図形プレビューは kdv と crates.io kdr 経由で行う

システムは、Mermaid / Draw.io / PlantUML の図形プレビューを KatanA 内部実装または `katana-canvas-forge`（kcf）ではなく、`katana-document-viewer`（kdv）v0.1.0 と crates.io 経由の `katana-diagram-renderer`（kdr）dependency を通して処理しなければならない（MUST）。

#### Scenario: kdv v0.1.0 を crates.io dependency として参照する

- **WHEN** KatanA v0.22.26 の workspace dependencies を解決する
- **THEN** `katana-document-viewer = "0.1.0"` が crates.io dependency として解決される
- **THEN** `katana-canvas-forge` は workspace dependency graph に含まれない

#### Scenario: kdr を crates.io dependency として参照する

- **WHEN** KatanA v0.22.26 の workspace dependencies を解決する
- **THEN** `katana-diagram-renderer` は crates.io の semver dependency として解決される
- **THEN** `katana-diagram-renderer` は git dependency または path dependency として解決されない

#### Scenario: 図形プレビューは kdv adapter を通る

- **WHEN** active Markdown document に Mermaid、Draw.io、または PlantUML ブロックが含まれる
- **THEN** KatanA は document、theme snapshot、diagram cache context を kdv adapter へ渡す
- **THEN** KatanA は kcf adapter または kcf DTO を呼び出さない

## MODIFIED Requirements

### Requirement: ダイアグラムプレビューは現在のテーマスナップショットを使用する

システムは、アプリ起動時点のスナップショットや dark/light 切り替えだけに依存するのではなく、現在のテーマスナップショットに基づいてダイアグラムプレビューを描画しなければならない（SHALL）。kdv / kdr backed renderer を利用する Mermaid / Draw.io / PlantUML でも、KatanA が渡したテーマスナップショットを実描画が使用しなければならない（MUST）。

#### Scenario: Mermaid プレビューが同一モード内の色変更へ追従する

- **WHEN** ユーザーが dark/light モードを変えずに preview text color や関連テーマ色を変更した時
- **THEN** Mermaid 描画は更新後のテーマスナップショットを使用する
- **THEN** プレビューは旧色セットで描かれた古いダイアグラム画像を再利用しない

#### Scenario: PlantUML プレビューが同一モード内の色変更へ追従する

- **WHEN** ユーザーが dark/light モードを変えずに preview text color や関連テーマ色を変更した時
- **THEN** PlantUML 描画は更新後のテーマスナップショットを使用する
- **THEN** プレビューは旧色セットで描かれた古いダイアグラム画像を再利用しない

#### Scenario: kdv backed Mermaid プレビューが light テーマを使用する

- **WHEN** KatanA の active theme が light mode の状態で Mermaid block を描画する
- **THEN** KatanA は kdv adapter を通じて kdr の `RenderInput` に light テーマの名前、背景、文字色、塗り、線、矢印、Mermaid theme を渡す
- **THEN** kdr が返す SVG は kdr 内部の dark 既定値ではなく、KatanA が渡した light テーマに基づく
- **THEN** 画面上の Mermaid 図形は dark 背景・白文字寄りの配色へ戻らない

#### Scenario: kdv backed Draw.io プレビューが light テーマを使用する

- **WHEN** KatanA の active theme が light mode の状態で Draw.io block を描画する
- **THEN** KatanA は kdv adapter を通じて kdr の `RenderInput` に light テーマの名前、背景、文字色、塗り、線、矢印、Draw.io label color を渡す
- **THEN** kdr が返す SVG は kdr 内部の dark 既定値ではなく、KatanA が渡した light テーマに基づく
- **THEN** 画面上の Draw.io 図形は dark 背景・白文字寄りの配色へ戻らない

### Requirement: V8 を使う図形プレビュー依存関係はバージョン整合している

システムは、Mermaid / Draw.io プレビュー（preview）で利用する V8 を使う描画依存関係（V8-backed renderer dependencies）を、作業領域（workspace）内とユーザーレビュー用の `scripts/screenshot` manifest 内で単一の互換 `v8` バージョンに揃えなければならない（MUST）。同じプロセス内の数式描画（MathJax）経路は V8 を初期化してはならない（MUST NOT）。対応済み図形ブロック（diagram block）は、`katana-document-viewer`、`katana-diagram-renderer`、または数式描画依存の不整合によりワーカー（worker）起動前に失敗してはならない（MUST NOT）。

#### Scenario: 作業領域の依存関係が kdv と kdr に揃う

- **WHEN** KatanA v0.22.26 向けに作業領域の依存関係（workspace dependencies）を解決する
- **THEN** `katana-document-viewer` は `0.1.0` として解決される
- **THEN** `katana-diagram-renderer` は crates.io dependency として解決される
- **THEN** `katana-canvas-forge` は依存関係グラフに含まれない
- **THEN** V8-backed renderer dependency は互換性のない複数の `v8` バージョンを要求しない
- **THEN** 数式描画依存は `v8` を要求しない

#### Scenario: Mermaid プレビューのワーカーは描画前に切断されない

- **WHEN** 開いている Markdown 文書に対応済み Mermaid ブロックが含まれる
- **THEN** プレビューは V8 を使う描画ワーカーをバージョン競合による panic なしで起動する
- **THEN** 描画を試みる前に、ブロックが `[Mermaid] Diagram render worker disconnected before producing a result.` へ置換されない

#### Scenario: Draw.io プレビューは整合した実行環境を使う

- **WHEN** 開いている Markdown 文書に対応済み Draw.io ブロックが含まれる
- **THEN** プレビューは Mermaid 描画と同じ、作業領域で整合した V8 実行環境（runtime）を使う
- **THEN** kdv と kdr の境界で V8 バージョン分裂によりブロックが失敗しない
