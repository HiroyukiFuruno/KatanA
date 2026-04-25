## Why

Mermaid は `mmdc`、PlantUML は Java / `plantuml.jar` に依存しており、ネイティブデスクトップアプリとしての初回体験と配布説明に負荷がある。まず backend adapter を作り、既存外部 backend と将来の Rust-native backend を同じ契約で扱えるようにする。

## What Changes

- Mermaid / PlantUML rendering を KatanA-owned backend adapter 経由にする。
- 現行の `mmdc` / Java jar 実装を behavior-preserving backend として包む。
- Rust-native backend 候補を parity gate の対象として評価できる fixture と contract を作る。
- Rust-native backend が十分でない場合も、外部 backend fallback を維持する。
- preview / export / cache が backend 実装詳細へ直接依存しないようにする。

## Capabilities

### New Capabilities

- `diagram-backend-adapter`: Mermaid / PlantUML backend selection、fallback、parity gate を adapter contract として提供する。

### Modified Capabilities

- `diagram-block-preview`: diagram preview が直接 `mmdc` や `java` を呼ぶのではなく adapter output を消費する。

## Impact

- `crates/katana-core/src/markdown/mermaid_renderer/*`
- `crates/katana-core/src/markdown/plantuml_renderer/*`
- `crates/katana-core/src/markdown/diagram.rs`
- `crates/katana-ui/src/preview_pane/*`
- README / setup docs
- diagram fixture tests, export tests, cache key tests
