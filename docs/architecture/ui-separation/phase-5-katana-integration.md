# katana (KatanA 本体) — Phase 5 KatanA integration 抜粋

作成日: 2026-05-17  
canonical: [`detailed-design-and-tasks.md`](detailed-design-and-tasks.md) 同ディレクトリ

## このファイルの位置付け

本ファイルは KatanA ecosystem の **UI 分離構想 master** から KatanA 本体 (`katana` repo) 担当部分を抜粋したもの。task ID は master と同一。**master が単一情報源**。

## Repository の役割

KatanA 本体は **application boundary** に戻す。

- 主な責務: startup / config / workspace open-close / project state / persistence / command dispatch / update orchestration / plugin boundary。
- UI 詳細を知らない。`katana-ui::MainPanel` を組み込むだけ。
- preview / export / editor の logic を直接持たない。KDV / KLE / KUC の adapter を組み合わせる。
- KLE 用 markdown port implementation (`katana-language-editor-md`) を起動時に inject する。
- どの runtime adapter (Floem / GPUI / egui / 互換) で起動するかを決める。

詳細: master [`5.5 katana 詳細設計`](detailed-design-and-tasks.md#55-katana-詳細設計) と [`6.6 Phase 5 KatanA integration`](detailed-design-and-tasks.md#phase-5-katana-integration)

## 担当 Phase

- **Phase 5**: KatanA 本体側 integration (本 repo のメイン作業)
- **P4-0**: Primary adapter 選定 (本 repo の ADR 起票責任)
- **P4-E**: Widget adoption (本 repo の UI コードを `katana-ui` Component model で書き直す)
- **P0-D-008**: OpenSpec validation を KatanA 側の integration gate に入れる
- **横断**: P8 docs / P9 release

## P5 スコープ

目的: KatanA 本体を app boundary に戻す。

- direct egui preview dependency を消す
- direct export task を KDV command に移す
- KatanA は `katana-ui::MainPanel` を組み込むだけにする
- KLE は port 経由で利用、markdown 用 port は `katana-language-editor-md` を inject

## Task list (master 抜粋)

### P5-A. Dependency update

- [ ] P5-A-001: `KatanA` root Cargo.toml に `katana-ui-core` dependency を追加する必要がないことを確認する。
- [ ] P5-A-002: `KatanA/crates/katana-ui` に `katana-ui-core` dependency を追加する。
- [ ] P5-A-003: `KatanA/crates/katana-ui` に `katana-document-viewer` dependency を追加する。
- [ ] P5-A-004: `KatanA/crates/katana-ui` に `katana-language-editor` dependency を追加する。
- [ ] P5-A-005: `KatanA` から direct `egui_commonmark` usage を減らす計画を作る。
- [ ] P5-A-006: `KatanA` から direct KCF export usage を KDV forge に置換する計画を作る。
- [ ] P5-A-007: `[patch.crates-io]` の egui_commonmark vendor patch 削除条件を定義する。
- [ ] P5-A-008: `katana-canvas-forge` direct dependency を transitional として記録する。

### P5-B. Preview migration

出力先: P5-B-001〜005 の「特定する」系タスクは `docs/inventory/katana-preview-current.md` に code path (file:line) + 役割を一覧記録する。

- [ ] P5-B-001: current `PreviewPane` construction point を特定する。
- [ ] P5-B-002: current preview cache key を特定する。
- [ ] P5-B-003: current scroll sync input を特定する。
- [ ] P5-B-004: current diagram rendering call を特定する。
- [ ] P5-B-005: current task list action flow を特定する。
- [ ] P5-B-006: `DocumentViewerPane` adapter を追加する。
- [ ] P5-B-007: `PreviewPane` を KDV viewer state に置換する。
- [ ] P5-B-008: task list action を KDV event に変換する。
- [ ] P5-B-009: scroll sync を KDV sync map に変換する。
- [ ] P5-B-010: preview fixture regression を追加する。

### P5-C. Export migration

出力先: P5-C-001〜004 の「特定する」系タスクは `docs/inventory/katana-export-current.md` に code path (file:line) + format / output policy を記録する。

- [ ] P5-C-001: current `ExportTask` creation point を特定する。
- [ ] P5-C-002: current export format list を特定する。
- [ ] P5-C-003: current export output path policy を特定する。
- [ ] P5-C-004: current open-on-complete policy を特定する。
- [ ] P5-C-005: `ExportTask` を KDV `ExportRequest` に変換する。
- [ ] P5-C-006: KDV `ExportOutput` を app notification に変換する。
- [ ] P5-C-007: export progress event を定義する。
- [ ] P5-C-008: export error handling を定義する。
- [ ] P5-C-009: export regression fixture を追加する。
- [ ] P5-C-010: KCF direct export call を削除する。

### P5-D. Editor migration

出力先: P5-D-001〜005 の「特定する」系タスクは `docs/inventory/katana-editor-current.md` に code path (file:line) + 周辺型を記録する。

- [ ] P5-D-001: current editor content model を特定する。
- [ ] P5-D-002: current cursor restore logic を特定する。
- [ ] P5-D-003: current syntax highlight layouter を特定する。
- [ ] P5-D-004: current markdown authoring actions を特定する。
- [ ] P5-D-005: current diagnostics flow を特定する。
- [ ] P5-D-006: KLE `BufferSnapshot` に接続する。
- [ ] P5-D-007: KLE `EditorEvent` を app command に接続する。
- [ ] P5-D-008: KLE diagnostics を diagnostics pane に接続する。
- [ ] P5-D-009: KMM source mapping を editor-viewer sync に接続する。
- [ ] P5-D-010: editor fixture regression を追加する。

### P5-E. Application shell thinning

- [ ] P5-E-001: `KatanaApp` から preview cache を削除する条件を定義する。
- [ ] P5-E-002: `KatanaApp` から export tasks を削除する条件を定義する。
- [ ] P5-E-003: `KatanaApp` から editor cursor raw egui type を削除する条件を定義する。
- [ ] P5-E-004: `KatanaApp` から settings preview pane を削除する条件を定義する。
- [ ] P5-E-005: `KatanaApp` に `MainPanelState` だけを持たせる中間状態を作る。
- [ ] P5-E-006: app action dispatch を `MainPanelEvent` 経由に変える。
- [ ] P5-E-007: persistence schema を app core に移す。
- [ ] P5-E-008: settings schema を app core に移す。
- [ ] P5-E-009: platform service を app boundary に移す。
- [ ] P5-E-010: UI framework-specific handles を app state から削除する。

### P5-F. Vendor / patch cleanup

出力先: P5-F-001〜003 の「特定する」系タスクは `docs/inventory/katana-vendor-current.md` に file:line で記録する。P5-F-004〜006 の「寄せる / 確定する」系タスクは判断結果を同 ADR section に記録する。判断基準: (a) syntax highlight は editor 表示 path のみで使われているか / (b) preview と editor の双方で使われているか — 後者の場合は KMM neutral DTO 経由を選択する。

- [ ] P5-F-001: `vendor/egui_commonmark_upstream` の使用箇所を特定する。
- [ ] P5-F-002: `egui_commonmark_backend` 使用箇所を特定する。
- [ ] P5-F-003: `CommonMarkCache` 使用箇所を特定する。
- [ ] P5-F-004: syntax highlight dependency を KLE / KDV どちらに寄せるか確定する。
- [ ] P5-F-005: preview rendering dependency を KDV に寄せる。
- [ ] P5-F-006: editor syntax highlight dependency を KLE に寄せる。
- [ ] P5-F-007: `[patch.crates-io] egui_commonmark` 削除 PR を準備する (P5-B 完了が前提)。
- [ ] P5-F-008: `[patch.crates-io] egui-winit` 削除条件を確認する。
- [ ] P5-F-009: mathjax_svg patch の所有者を KDV forge に寄せる。
- [ ] P5-F-010: vendor cleanup regression test を追加する。

## P4-0 / P4-E (本 repo 起票責任)

P4-0 は KatanA repo の ADR `docs/adr/katana-ui-primary-adapter.md` で確定する。詳細は master [`P4-0. Primary adapter 選定`](detailed-design-and-tasks.md#p4-0-primary-adapter-選定) を参照。

P4-E は KatanA の現状 UI コードを `katana-ui` Component model で書き直すタスク。詳細は master [`P4-E. Widget adoption`](detailed-design-and-tasks.md#p4-e-widget-adoption) を参照。

## 前提 (depends on) / 出力 (provides)

- **前提**:
  - P2 (KDV facade / viewer / forge / cli_api) 完成
  - P3 (KLE domain + ports + md adapter) 完成
  - P4 (katana-ui composition) 完成
  - P4-0 primary adapter 確定
  - P6 (KMM canonical 切替) との接続準備

- **出力**:
  - `KatanaApp` thin shell (preview / export / editor cache を持たない)
  - egui 直接依存の解消
  - vendor patch 削除
  - `katana-language-editor-md` を inject する markdown editor 起動 flow

## Done criteria

本 repo に関する master 9 章 Done criteria のうち、該当項目:

- [ ] KatanA で Markdown edit が動く
- [ ] KatanA で document preview が動く
- [ ] KatanA で diagram rendering が動く
- [ ] KatanA で export が KDV forge 経由で動く
- [ ] KatanA で scroll sync が KMM source mapping 経由で動く
- [ ] KatanA で UI composition が `katana-ui::MainPanel` から起動できる
- [ ] KatanA コード上に floem / egui / gpui の型が直接現れていない (起動 entrypoint のみ)
- [ ] vendor patch (egui_commonmark / egui-winit / mathjax_svg) の整理が完了

## drift 検出

- 本ファイルの task ID は master と完全一致する。
- P8-A-001 の CI script が master と本ファイルの task ID 一致を検査する。

## 参照リンク

- [master detailed-design-and-tasks.md](detailed-design-and-tasks.md)
- [master principles.md](principles.md)
- [overview README](README.md)
- [KUC 抜粋](../../../katana-ui-widget/docs/ui-separation-plan.md) (`katana-ui-widget` ディレクトリは P0-B-012 で rename 予定)
- [KDV 抜粋](../../../katana-document-viewer/docs/ui-separation-plan.md)
- [KLE 抜粋](../../../katana-language-editor/docs/ui-separation-plan.md)
- [KCF 抜粋](../../../katana-canvas-forge/docs/ui-separation-plan.md)
- [KMM 抜粋](../../../katana-markdown-model/docs/ui-separation-plan.md)
- [KCU 抜粋](../../../katana-chat-ui/docs/ui-separation-plan.md)
- [既存 docs/coding-rules.md](../../coding-rules.md)
- [既存 docs/development-guide.md](../../development-guide.md)
