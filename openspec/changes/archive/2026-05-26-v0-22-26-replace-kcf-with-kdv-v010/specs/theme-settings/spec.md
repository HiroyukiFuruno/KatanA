## MODIFIED Requirements

### Requirement: テーマ変更は再起動なしでダイアグラムプレビューへ反映される

システムは、アプリケーションの再起動を要求せずに、runtime のテーマ変更をダイアグラムプレビュー描画へ反映しなければならない（SHALL）。kdv / kdr backed renderer を利用する Mermaid / Draw.io / PlantUML でも、KatanA の current theme が kdv adapter と `RenderInput` 経由で実描画へ伝播しなければならない（MUST）。

#### Scenario: テーマモードが切り替わる

- **WHEN** ユーザーが dark theme と light theme の間で切り替えた時
- **THEN** ダイアグラムプレビューは新しく active になったテーマスナップショットで更新される

#### Scenario: プレビュー文字色が変更される

- **WHEN** ユーザーが設定 UI から preview 専用の文字色を変更した時
- **THEN** ダイアグラムプレビューは新しい色設定で更新される
- **THEN** 結果は以前のテーマではなく、現在の preview theme と一致する

#### Scenario: kdv / kdr の内部既定テーマが active theme を上書きしない

- **WHEN** KatanA が light theme の `RenderInput` を kdv adapter 経由で kdr へ渡す
- **THEN** kdv / kdr の内部既定値や `DiagramColorPreset::current()` の状態は、KatanA が渡した light theme を上書きしない
- **THEN** KatanA は外部 crate 内部のグローバル状態を同期するための隠れた呼び出しに依存しない

#### Scenario: light テーマの画面証跡を生成する

- **WHEN** release validation で light theme の screenshot scenario を実行する
- **THEN** Mermaid / Draw.io preview は light theme として確認できる配色で表示される
- **THEN** dark 的な濃い図形背景へ戻っていないことを screenshot で確認できる
