## ADDED Requirements

### Requirement: テーマ変更は KDV と KRR の描画へ伝播する

システムは、アプリケーションの再起動を要求せずに、runtime のテーマ変更を KDV adapter と KRR backed renderer へ伝播しなければならない（MUST）。KDV / KRR の内部既定値は、KatanA が渡した active theme を上書きしてはならない（MUST NOT）。

#### Scenario: KDV / KRR の内部既定テーマが active theme を上書きしない

- **WHEN** KatanA が light theme の render request を KDV adapter 経由で KRR backed renderer へ渡す
- **THEN** KDV / KRR の内部既定値や `DiagramColorPreset::current()` の状態は、KatanA が渡した light theme を上書きしない
- **THEN** KatanA は外部 crate 内部のグローバル状態を同期するための隠れた呼び出しに依存しない
