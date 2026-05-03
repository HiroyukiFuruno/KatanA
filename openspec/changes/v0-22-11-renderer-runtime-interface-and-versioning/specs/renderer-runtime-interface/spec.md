## ADDED Requirements

### Requirement: Mermaid rendering uses a versioned runtime interface

システムは、Mermaid 図形を描画するとき、KatanA 内部の個別実装詳細ではなく、版付き描画 runtime interface を通して入力、設定、出力、診断を扱わなければならない（MUST）。

#### Scenario: Pass Mermaid.js-compatible config separately from KatanA policy

- **WHEN** KatanA が Mermaid runtime へ描画を依頼する
- **THEN** システムは Mermaid.js に渡せる `theme` / `themeVariables` / `securityLevel` / `htmlLabels` / diagram-specific config を config として渡す
- **THEN** システムは最大幅、余白、背景、cache profile など KatanA 独自の制約を policy として config の外側に渡す
- **THEN** Mermaid.js config に KatanA 独自キーを混ぜない

#### Scenario: Return render output with diagnostics

- **WHEN** Mermaid runtime が描画を完了する
- **THEN** システムは SVG、幅、高さ、viewBox、Mermaid.js 版、renderer profile を返す
- **THEN** システムは warning または error がある場合、preview fallback と検証ログで扱える診断情報を返す

#### Scenario: Build cache keys from rendering-relevant inputs

- **WHEN** Mermaid 図形の cache key を作る
- **THEN** システムは source、Mermaid.js 版、renderer profile、Mermaid.js config、KatanA policy、テーマ fingerprint を含める
- **THEN** Mermaid.js 版または renderer profile が変わった場合、古い描画 cache を再利用しない

### Requirement: Mermaid.js bundle is pinned and embedded

システムは、実行時に利用する公式 Mermaid.js を無印ファイルではなく、明示された版（version）と checksum で管理しなければならない（MUST）。

#### Scenario: Load the embedded pinned Mermaid.js

- **WHEN** KatanA が Mermaid runtime を初期化する
- **THEN** システムは repository 内で管理された特定版の `mermaid.min.js` を読み込む
- **THEN** システムは実行時に CDN、npm install、OS の Chrome / Chromium アプリを必須にしない

#### Scenario: Update the Mermaid.js bundle reproducibly

- **WHEN** Mermaid.js の版を更新する
- **THEN** システムは更新用の just recipe または script で版指定を受け取る
- **THEN** システムは埋め込み JS、checksum、公式比較画像、cache profile を同じ更新単位で扱う

### Requirement: `katana-renderer` extraction boundary is documented

システムは、Mermaid / Draw.io / export 描画責務を将来 `katana-renderer` へ分離するため、KatanA 側に残す責務と外へ出す責務を文書化しなければならない（MUST）。

#### Scenario: Assign rendering-specific ownership outside KatanA

- **WHEN** `katana-renderer` 分離設計を確認する
- **THEN** Mermaid.js 版管理、Rust 管理 JS runtime、DOM / SVG / layout shim、SVG 正規化、公式比較画像生成は `katana-renderer` 側の責務として記録されている
- **THEN** Markdown block 抽出、テーマ snapshot、preview / export UI、cache 保存先は KatanA 側の責務として記録されている

#### Scenario: Keep KatanA integration stable during future extraction

- **WHEN** `katana-renderer` が別 repository として実装される
- **THEN** KatanA は v0.22.11 で定義した interface を経由して統合できる
- **THEN** KatanA は Mermaid.js の内部 DOM / SVG 互換処理を直接所有しない

#### Scenario: Keep preview separation one-directional

- **WHEN** preview が将来 KatanA 本体から分離される
- **THEN** preview は `katana-renderer` を利用できる
- **THEN** `katana-renderer` は preview、egui、KatanA UI state へ依存しない
- **THEN** preview と `katana-renderer` の間に循環依存を作らない

### Requirement: Draw.io and export ownership concerns are tracked

システムは、Draw.io 描画と HTML / PDF / PNG / JPEG export を Mermaid 分離後の所有境界と矛盾しない形で扱わなければならない（SHALL）。

#### Scenario: Document unresolved Draw.io runtime ownership

- **WHEN** Draw.io 描画 runtime が Mermaid runtime と同じ境界で扱えない
- **THEN** システムは Draw.io を Mermaid と混ぜて暫定実装せず、未解決の所有境界として記録する

#### Scenario: Document export runtime boundaries

- **WHEN** HTML / PDF / PNG / JPEG export が diagram runtime を必要とする
- **THEN** システムはどの部分が KatanA 所有で、どの部分が diagram runtime 所有かを記録する
- **THEN** 未接続の export 経路は、OS アプリ依存へ黙って戻さない

### Requirement: Rendering performance is recorded against the former `mmdc` baseline

システムは、`katana-renderer` の価値である高速描画を、旧 `mmdc` 経路との比較証跡として記録できなければならない（SHALL）。

#### Scenario: Compare first render and repeated render time

- **WHEN** 代表 Mermaid fixture の性能を測る
- **THEN** システムは初回描画と連続描画の時間を記録する
- **THEN** 比較対象として旧 `mmdc` 相当の実行時間を参照できる
- **THEN** 速度改善を感覚だけでなく証跡として確認できる

### Requirement: Renderer CLI uses the same core API as KatanA integration

システムは、将来の `katana-renderer` CLI を、KatanA 組み込みとは別経路の再実装ではなく、同じ core API の利用者として設計しなければならない（SHALL）。

#### Scenario: Render from CLI without KatanA UI dependencies

- **WHEN** `katana-renderer` CLI が Mermaid 図形を描画する
- **THEN** CLI は KatanA preview、egui、KatanA UI state に依存しない
- **THEN** CLI は KatanA 組み込みと同じ renderer runtime interface を使う
- **THEN** KatanA runtime は CLI 実行を必須にしない

#### Scenario: Use CLI for reference update and benchmark

- **WHEN** 公式比較画像更新または性能計測を行う
- **THEN** CLI は固定版 Mermaid.js と同じ renderer profile を使う
- **THEN** 結果は KatanA 側の検証証跡として参照できる
