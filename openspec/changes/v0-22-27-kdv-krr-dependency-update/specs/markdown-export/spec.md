## ADDED Requirements

### Requirement: Export 時の図形ブロックは KDV と KRR で描画される

システムは、HTML / PDF / PNG / JPEG export に含まれる Mermaid / Draw.io / PlantUML 図形ブロックを、KDV と crates.io KRR dependency を通して描画しなければならない（MUST）。KatanA は export 経路で KCF または KDR wrapper に直接依存してはならない（MUST NOT）。

#### Scenario: HTML 出力は KDV と KRR で図形ブロックを描画する

- **WHEN** Mermaid または Draw.io ブロックを含む Markdown 文書を HTML へ出力する
- **THEN** KDV は crates.io dependency として解決される
- **THEN** KRR は `katana-render-runtime = "0.3.3"` として解決される
- **THEN** KCF と KDR wrapper は workspace dependency graph に含まれない

#### Scenario: 依存関係のずれはリリース前に検出される

- **WHEN** 将来のリリースで KDV、KRR、または作業領域の `v8` を更新する
- **THEN** リリース検証は、依存関係グラフに互換性のない V8 を使う描画器（V8-backed renderer）バージョンが含まれないことを確認する
- **THEN** Mermaid / Draw.io 出力の回帰テストは、リリース完了扱いの前に実行される
