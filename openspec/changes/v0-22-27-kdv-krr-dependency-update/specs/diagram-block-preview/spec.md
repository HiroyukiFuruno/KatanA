## ADDED Requirements

### Requirement: 図形プレビューは KDV と crates.io KRR 経由で行う

システムは、Mermaid / Draw.io / PlantUML の図形プレビューを KatanA 内部実装、`katana-canvas-forge`（KCF）、または `katana-diagram-renderer`（KDR）への直接依存ではなく、`katana-document-viewer`（KDV）と crates.io 経由の `katana-render-runtime`（KRR）dependency を通して処理しなければならない（MUST）。

#### Scenario: KRR を crates.io dependency として参照する

- **WHEN** KatanA v0.22.27 の workspace dependencies を解決する
- **THEN** `katana-render-runtime = "0.3.3"` が crates.io dependency として解決される
- **THEN** `katana-diagram-renderer` は workspace dependency graph に含まれない
- **THEN** `katana-render-runtime` は git dependency または path dependency として解決されない

#### Scenario: 図形プレビューは KDV adapter を通る

- **WHEN** active Markdown document に Mermaid、Draw.io、または PlantUML ブロックが含まれる
- **THEN** KatanA は document、theme snapshot、diagram cache context を KDV adapter へ渡す
- **THEN** KatanA は KCF adapter、KCF DTO、または KDR wrapper を呼び出さない

### Requirement: KRR の version と profile は cache key に含める

システムは、KRR backed renderer で使った runtime version、renderer profile、runtime checksum、KRR crate version を cache invalidation の識別情報に含めなければならない（MUST）。

#### Scenario: KRR runtime と profile の差分で cache key が変化する

- **WHEN** KRR の runtime version、renderer profile、または runtime checksum が変わった時
- **THEN** KatanA の diagram cache key は変化する
- **THEN** KatanA は古い KRR 出力を再利用しない

#### Scenario: KDR の crate version を手書きで固定しない

- **WHEN** KatanA が KRR backed renderer の cache key または backend version を組み立てる
- **THEN** system は実際の `katana-render-runtime` dependency version、runtime version、renderer profile、runtime checksum から識別情報を得る
- **THEN** `crate=katana-diagram-renderer:0.1.0` のような古い手書き文字列を cache invalidation の根拠にしない
