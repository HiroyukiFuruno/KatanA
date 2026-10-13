## Why

KatanA は view-first の Markdown 開発支援ツールとして、preview を主要な利用面に置きます。一方で現在の preview 実装は `katana-ui` から Markdown parser、`egui_commonmark` 系の vendor fork、diagram/math/table/emoji の描画都合へ直接触れやすく、アプリ本体と描画実装の境界が弱くなっています。

`katana-markdown-linter` は KatanA 側を adapter 越しの contract に寄せることで、独立した linter として成立しました。preview も同じ思想に寄せ、まずはユーザー体験を変えない移行だけを行い、その後の preview 主導編集に必要な source span / hit-test metadata を安全に載せられる境界を作ります。

## What Changes

- Native egui preview を `katana-ui` から直接扱うのではなく、preview adapter が所有する API 経由で利用する。
- `katana-ui` は Markdown text、theme、workspace context、action sink などの KatanA 側 DTO を渡し、parser/vendor/rendering の内部型へ直接依存しない。
- 現行の Markdown preview、GFM、diagram、math、table、anchor、scroll-sync、TOC、emoji 関連挙動は維持する。
- preview 関連の vendor hack を棚卸しし、preview adapter の implementation detail として閉じ込める。KatanA root の `[patch.crates-io]` や `vendor/` を残す場合も、残す理由と所有境界を明文化する。
- WebView、React、DOM runtime は導入しない。
- preview 主導編集そのものは v0.29.0 に分離し、この変更では編集 UX を変更しない。

## Capabilities

### New Capabilities

- `preview-adapter-architecture`: Native preview を adapter contract 越しに利用し、KatanA UI と parser/vendor/rendering 実装を疎結合にする。

### Modified Capabilities

- なし。この変更は migration-only とし、既存 preview / editor の user-visible behavior は変えない。

## Impact

- `crates/katana-ui`: preview 呼び出し点を adapter API 利用へ移行する。
- preview rendering modules: 現行の parser/rendering/diagram/math/table/emoji 連携を adapter implementation に寄せる。
- `Cargo.toml` / `vendor/`: preview 関連 patch の所有境界を整理し、KatanA UI から直接参照しない構造へ寄せる。
- Tests: current preview behavior と render metadata contract の回帰テストを追加または更新する。
