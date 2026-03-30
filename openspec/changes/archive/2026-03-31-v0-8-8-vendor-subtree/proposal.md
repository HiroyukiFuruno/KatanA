## 背景

KatanA は現在、`vendor/egui_commonmark` と `vendor/egui_commonmark_backend` に patch 済みの crate snapshot を抱えている。これらはもはや単なる third-party copy ではない。

- `Cargo.toml` の `[patch.crates-io]` は、workspace から両方の vendored crate を直接参照している
- `crates/katana-ui/src/svg_loader/mod.rs` は `vendor/egui_commonmark_backend/src` 配下の SVG asset を直接読み込んでいる
- `crates/katana-ui/tests/underline_rendering.rs` や preview 関連コードは、vendor fork 側の Katana-specific parser 挙動を前提にしている
- `vendor/egui_commonmark` には `katana-core` 依存や inline emoji rendering など、Katana 専用統合が既に入っている

つまり vendor 変更は継続的に product work として発生しているが、現状の repository には upstream との関係が git 履歴として記録されていない。そのため upstream との差分確認や同期は毎回手作業になり、`egui_commonmark` 単体だけを見ても実態を説明しきれない。`egui_commonmark_backend` も同じ upstream repo と同じ runtime path に含まれるため、両者を一体で管理し直す必要がある。

ただし、これは現行 roadmap の blocking prerequisite ではない。`v0-8-7-preview-refresh-and-tasklist-fixes` は vendor parser 挙動を実際に変更中であり、upstream HEAD もすでに `egui_commonmark 0.23 / egui 0.34` へ進んでいる。KatanA はまだ `egui_commonmark 0.22 / egui 0.33` なので、本 change の目的は「今すぐ最新へ上げる」ことではなく、「互換 revision を subtree として表現し、その上に Katana patch を review 可能な形で積む」ことにある。

## 変更内容

- ad hoc な vendored crate snapshot を廃止し、upstream `lampsitter/egui_commonmark` repository を `git subtree` で 1 つ取り込む
- subtree は KatanA の `egui 0.33` と互換な `0.22.x` 系 revision に pin する
- Cargo の patch path と asset/path 直接参照を subtree layout へ張り替え、`egui_commonmark`、`egui_commonmark_backend`、および upstream sibling crate を 1 つの upstream root から解決する
- Katana-specific change は opaque な copied directory に混ぜず、subtree import 後の明示的な commit として再適用する
- 今後の upstream pull、local patch review、regression verification を再現可能にするため、sync/update 手順を文書化する

## 実施タイミング

- 推奨実施タイミングは `v0-8-7-preview-refresh-and-tasklist-fixes` の merge と安定化の後
- active な vendor behavior fix と同じ branch へ subtree migration を混ぜない
- 次に `vendor/*egui_commonmark*` を触る予定 change の直前で着手するのが望ましい

## ケイパビリティ

### 変更されるケイパビリティ

- `vendor-dependency-management`: `egui_commonmark` の upstream 追跡を再現可能かつ review しやすい形にし、Katana-specific patch layer と両立させる

## 影響範囲

- `Cargo.toml`
- `crates/katana-ui/src/svg_loader/mod.rs`
- vendor behavior に依存する test と preview/runtime path
- `vendor/egui_commonmark*` の layout と、将来の upstream sync を行う保守手順
