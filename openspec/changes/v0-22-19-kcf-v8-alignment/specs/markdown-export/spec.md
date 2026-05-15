## ADDED Requirements

### Requirement: V8 を使う Markdown 出力依存関係はバージョン整合している

システムは、HTML / PDF / PNG / JPEG 出力（export）で kcf が利用する V8 を使う依存関係（V8-backed dependencies）を、作業領域（workspace）内で単一の互換 `v8` バージョンに揃えなければならない（MUST）。出力は、`katana-canvas-forge` と `katana-diagram-renderer` の `v8` 固定指定（pin）不整合により停止してはならない（MUST NOT）。

#### Scenario: HTML 出力は整合した依存関係で図形ブロックを描画する

- **WHEN** Mermaid または Draw.io ブロックを含む Markdown 文書を HTML へ出力する
- **THEN** kcf は `katana-canvas-forge = "0.1.7"` として解決される
- **THEN** kcf と kdr が異なる `v8` バージョンを要求することにより、出力経路が失敗しない

#### Scenario: ネイティブ出力でも図形描画を利用できる

- **WHEN** Mermaid または Draw.io ブロックを含む Markdown 文書を PDF / PNG / JPEG へ出力する
- **THEN** 出力経路は、作業領域で整合した `v8 = "=147.4.0"` の依存関係グラフ（dependency graph）を使う
- **THEN** V8 バージョン分裂に起因するワーカー切断の失敗（failure）により、図形描画が省略または置換されない

#### Scenario: 依存関係のずれはリリース前に検出される

- **WHEN** 将来のリリースで kcf、kdr、または作業領域の `v8` を更新する
- **THEN** リリース検証は、依存関係グラフに互換性のない V8 を使う描画器（V8-backed renderer）バージョンが含まれないことを確認する
- **THEN** Mermaid / Draw.io 出力の回帰テストは、リリース完了扱いの前に実行される
