## ADDED Requirements

### Requirement: 互換な upstream repository は単一の subtree として表現される

KatanA は、個別の copied crate snapshot を維持するのではなく、upstream `lampsitter/egui_commonmark` repository を `vendor/egui_commonmark_upstream/` 配下の単一 `git subtree` として表現しなければならない（SHALL）。

#### Scenario: 初回 subtree import

- **WHEN** vendor migration を実施した時
- **THEN** upstream repository root は `vendor/egui_commonmark_upstream/` 配下へ import される
- **AND** `egui_commonmark` と `egui_commonmark_backend` は、その subtree root 内の crate subdirectory から解決される
- **AND** 選ばれる revision は、既定では upstream HEAD ではなく KatanA の現行 `egui 0.33` 系と互換なものに pin される

### Requirement: Katana-specific vendor patch は raw subtree base から分離される

KatanA は、local vendor behavior を raw subtree import の上に載る明示的な patch layer として保持しなければならない（SHALL）。

#### Scenario: local divergence をレビューする

- **WHEN** 保守担当者が migration history を確認した時
- **THEN** raw subtree import は Katana-specific な follow-up commit と区別できる
- **AND** `katana-core` 統合、parser/rendering override、backend UI 調整、vendored asset 追加といった local behavior は Katana 側の差分として review 可能なまま残る

### Requirement: Migration は vendor fork を直接使う product consumer を維持する

KatanA は、runtime / build / test の consumer を同じ change 内で subtree layout へ移行しなければならない（SHALL）。

#### Scenario: migration 後の runtime と test の解決

- **WHEN** subtree migration が反映された時
- **THEN** Cargo の patch 解決は subtree path を使って成功する
- **AND** asset を直接読む consumer と vendor-dependent test は継続して正しく解決される
- **AND** active な code path は、削除済みの legacy `vendor/egui_commonmark*` directory に依存しない

### Requirement: Subtree migration は専用メンテナンス期間で実施される

KatanA は、active な vendor behavior work が安定化した後、かつ次の vendor-touching branch が始まる前に、この migration を計画しなければならない（SHALL）。

#### Scenario: change を実施計画へ載せる

- **WHEN** チームが `vendor-subtree` の実装時期を決めた時
- **THEN** `v0-8-7-preview-refresh-and-tasklist-fixes` は既に merge 済みで安定している
- **AND** migration は `vendor/*egui_commonmark*` を編集する別 change と同じ branch に束ねられない
