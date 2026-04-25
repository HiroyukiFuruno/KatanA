## 1. Adapter Contract

- [ ] 1.1 Mermaid / PlantUML backend input、render options、theme snapshot、document context を定義する
- [ ] 1.2 backend output と error を `DiagramResult` 相当の renderer-neutral contract に揃える
- [ ] 1.3 cache key に backend id、backend version、render options を含める方針を決める

## 2. Behavior-Preserving Migration

- [ ] 2.1 現行 Mermaid CLI renderer を adapter implementation へ移す
- [ ] 2.2 現行 PlantUML jar renderer を adapter implementation へ移す
- [ ] 2.3 preview / export call site が adapter output だけを消費するように変更する
- [ ] 2.4 migration 前後で fallback behavior が維持されるテストを追加する

## 3. Rust-Native Candidate Gate

- [ ] 3.1 Mermaid Rust candidate の fixture parity test を設計する
- [ ] 3.2 PlantUML Rust candidate の fixture parity、license、packaging check を設計する
- [ ] 3.3 gate 合格前に default backend へ昇格しない guard を追加する

## 4. Documentation and Verification

- [ ] 4.1 README / setup docs の external runtime 説明を adapter / fallback 前提に更新する
- [ ] 4.2 diagram preview、export、cache key の回帰テストを通す
- [ ] 4.3 `make check` と `openspec validate diagram-backend-adapter` を通す
