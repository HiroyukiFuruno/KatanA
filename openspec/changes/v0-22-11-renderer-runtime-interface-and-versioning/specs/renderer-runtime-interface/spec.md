## ADDED Requirements

### Requirement: Diagram rendering is owned by the `katana-canvas-forge` library

システムは、Mermaid / Draw.io 描画と HTML / PDF / PNG / JPEG export を、KatanA 内部実装ではなく外部 library `katana-canvas-forge`（kcf）が所有しなければならない（MUST）。KatanA は kcf を git tag pinned dependency として consume する。

#### Scenario: KatanA depends on `katana-canvas-forge` as a pinned external library

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に `katana-canvas-forge` が git tag pinned で含まれる
- **THEN** KatanA repository 内に Mermaid 描画 / Draw.io 描画 / HTML / PDF / PNG / JPEG export の実装本体が含まれない
- **THEN** KatanA は kcf の `Renderer` trait と DTO のみを呼ぶ薄い adapter を持つ

#### Scenario: KatanA does not implement renderer fallbacks

- **WHEN** kcf 経由で描画 / export が失敗する
- **THEN** KatanA は OS Chrome / Chromium app への暗黙 fallback を持たない
- **THEN** KatanA は kcf の diagnostic（`RenderError::NotImplemented` 等）を UI 上で明示する

### Requirement: Diagram rendering uses a versioned runtime interface from kcf

システムは、Mermaid 図形を描画するとき、kcf が公開する版付き描画 runtime interface を通して入力、設定、出力、診断を扱わなければならない（MUST）。

#### Scenario: Pass vendor-compatible config separately from KatanA policy

- **WHEN** KatanA が kcf へ描画を依頼する
- **THEN** システムは vendor 互換 config（Mermaid.js なら `theme` / `themeVariables` / `securityLevel` / `htmlLabels` / diagram-specific config）を `RenderConfig` として渡す
- **THEN** システムは最大幅、余白、背景、cache profile など KatanA 独自の制約を `RenderPolicy` として `RenderConfig` の外側に渡す
- **THEN** vendor 互換 config に KatanA 独自キーを混ぜない

#### Scenario: Return render output with diagnostics

- **WHEN** kcf が描画を完了する
- **THEN** システムは SVG、幅、高さ、viewBox、`RuntimeVersion`、`RendererProfile`、`RenderDiagnostics`、`cache_fingerprint` を返す
- **THEN** システムは warning または error がある場合、preview fallback と検証ログで扱える形で診断情報を返す

#### Scenario: Build cache keys from rendering-relevant inputs

- **WHEN** KatanA が描画 cache key を作る
- **THEN** システムは source、kcf `RuntimeVersion`、kcf `RendererProfile`、`RenderConfig`、`RenderPolicy`、テーマ fingerprint を含める
- **THEN** kcf release で `RuntimeVersion` または `RendererProfile` が変わった場合、古い描画 cache を再利用しない

### Requirement: Mermaid.js bundle is pinned and owned by kcf

システムは、実行時に利用する公式 Mermaid.js を kcf repository 内で版（version）と checksum 付きで管理しなければならない（MUST）。

#### Scenario: Load the embedded pinned Mermaid.js from kcf

- **WHEN** kcf が Mermaid runtime を初期化する
- **THEN** システムは kcf repository 内 `vendor/mermaid/<version>/mermaid.min.js` を読み込む
- **THEN** システムは実行時に CDN、npm install、OS の Chrome / Chromium アプリを必須にしない

#### Scenario: Update the Mermaid.js bundle through kcf

- **WHEN** Mermaid.js の版を更新する
- **THEN** kcf 側 just recipe または script で版指定を受け取る
- **THEN** kcf は埋め込み JS、checksum、公式比較画像、cache profile を同じ更新単位で扱う
- **THEN** KatanA 側は kcf の新 release を tag bump で取り込むだけで済み、Mermaid.js bundle を直接編集しない

### Requirement: Draw.io and export ownership is in kcf

システムは、Draw.io 描画と HTML / PDF / PNG / JPEG export 実装も kcf 側に置かなければならない（MUST）。

#### Scenario: Draw.io rendering goes through kcf

- **WHEN** KatanA preview が Draw.io block を描画する
- **THEN** KatanA は kcf に Draw.io 用 `RenderInput` を渡し、SVG と diagnostic を受け取る
- **THEN** KatanA repository 内に Draw.io 描画の実装本体は残らない

#### Scenario: HTML / PDF / PNG / JPEG export goes through kcf

- **WHEN** KatanA が HTML / PDF / PNG / JPEG export を実行する
- **THEN** KatanA は kcf の export API（`Exporter` 系統または `Renderer` 出力の post-process）を呼び、結果ファイルを保存先に書き出す
- **THEN** KatanA repository 内に HTML / PDF / PNG / JPEG export の実装本体は残らない
- **THEN** 未接続経路は kcf 側で `NotImplemented` 相当の診断を返し、KatanA UI 上で明示する

### Requirement: Reference image scoring is operated by kcf

システムは、公式 Mermaid.js との比較画像生成、採点評価、保存時 pre-commit、CI/CD での採点検証を kcf 側で運用しなければならない（MUST）。

#### Scenario: Reference update and scoring run on kcf side

- **WHEN** 公式比較画像更新または採点評価を実行する
- **THEN** kcf 側 `kcf mermaid reference-update` / `kcf mermaid compare --min-score <score>` で実行する
- **THEN** kcf 側 `.github/workflows/` で採点評価 job が実行される
- **THEN** KatanA 側 docs は kcf docs へリンクし、KatanA repository 内には採点運用の実体を持たない

### Requirement: kcf does not depend on KatanA UI

システムは、kcf を KatanA preview / editor / `egui` / KatanA UI state に依存させてはならない（MUST NOT）。

#### Scenario: kcf has no UI dependency

- **WHEN** kcf workspace を build する
- **THEN** kcf crate dependency graph に `egui`、KatanA preview、KatanA editor、KatanA UI state は含まれない
- **THEN** 描画結果は SVG 文字列とメタデータ（DTO）として返される

#### Scenario: Preview separation stays one-directional

- **WHEN** preview crate（v0.26.0 で分離）が描画を行う
- **THEN** preview crate は kcf を library として呼ぶ
- **THEN** kcf は preview crate へ依存しない
- **THEN** 循環依存にならない

### Requirement: Rendering performance is recorded against the former `mmdc` baseline

システムは、kcf の高速描画を旧 `mmdc` 経路との比較証跡として記録できなければならない（SHALL）。

#### Scenario: Compare first render and repeated render time

- **WHEN** 代表 Mermaid fixture の性能を測る
- **THEN** kcf 側 `kcf mermaid bench` が初回描画と連続描画の時間を記録する
- **THEN** 比較対象として旧 `mmdc` 相当の実行時間を参照できる
- **THEN** 速度改善を感覚だけでなく証跡として確認できる

### Requirement: kcf CLI uses the same core API as KatanA integration

システムは、`kcf` CLI を KatanA 組み込みとは別経路の再実装ではなく、kcf core library API の利用者として設計しなければならない（SHALL）。

#### Scenario: Render from CLI without KatanA UI dependencies

- **WHEN** `kcf` CLI が Mermaid 図形を描画する
- **THEN** CLI は KatanA preview、`egui`、KatanA UI state に依存しない
- **THEN** CLI は KatanA 組み込みと同じ kcf `Renderer` trait を使う
- **THEN** KatanA runtime は CLI 実行を必須にしない

#### Scenario: Use CLI for reference update and benchmark

- **WHEN** 公式比較画像更新または性能計測を行う
- **THEN** CLI は固定版 Mermaid.js と同じ `RendererProfile` を使う
- **THEN** 結果は KatanA 側の検証証跡として参照できる
