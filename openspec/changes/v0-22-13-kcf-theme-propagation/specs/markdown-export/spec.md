## ADDED Requirements

### Requirement: Export 時の図形ブロックは現在テーマで描画される

システムは、HTML / PDF / PNG / JPEG export に含まれる Mermaid / Draw.io 図形ブロックを、export 開始時点の KatanA テーマスナップショットで描画しなければならない（SHALL）。export thread は kcf 内部の既定テーマや、export 実行時に変化しうるグローバル状態に依存してはならない（MUST NOT）。

#### Scenario: HTML export の Mermaid が light テーマで描画される

- **WHEN** KatanA の active theme が light mode の状態で Mermaid block を含む Markdown を HTML export する
- **THEN** export された HTML 内の Mermaid SVG は KatanA が渡した light テーマに基づく
- **THEN** HTML 全体の CSS と Mermaid 図形の配色が矛盾しない

#### Scenario: PDF / PNG / JPEG export の Mermaid が light テーマで描画される

- **WHEN** KatanA の active theme が light mode の状態で Mermaid block を含む Markdown を PDF / PNG / JPEG export する
- **THEN** native export の入力 HTML に含まれる Mermaid SVG は light テーマに基づく
- **THEN** 出力画像または PDF 内で Mermaid 図形だけが dark 的な配色へ戻らない

#### Scenario: Export thread はテーマスナップショットを受け取る

- **WHEN** export 処理が background thread で実行される
- **THEN** export 開始時点で取得した theme snapshot が thread へ渡される
- **THEN** thread 内で `DiagramColorPreset::current()` のようなグローバル状態だけを読み直してテーマを決めない
