## MODIFIED Requirements

### Requirement: テーマ変更は再起動なしでダイアグラムプレビューへ反映される

システムは、アプリケーションの再起動を要求せずに、runtime のテーマ変更をダイアグラムプレビュー描画へ反映しなければならない（SHALL）。

#### Scenario: テーマモードが切り替わる

- **WHEN** ユーザーが dark theme と light theme の間で切り替えた時
- **THEN** ダイアグラムプレビューは新しく active になったテーマスナップショットで更新される

#### Scenario: プレビュー文字色が変更される

- **WHEN** ユーザーが設定 UI から preview 専用の文字色を変更した時
- **THEN** ダイアグラムプレビューは新しい色設定で更新される
- **THEN** 結果は以前のテーマではなく、現在の preview theme と一致する
