# KatanA UI 分離 — 設計ドキュメント配布マップ

作成日: 2026-05-17

本ディレクトリは **KatanA UI 分離構想**の canonical 配置。設計原則と Phase 別 task の単一情報源 (single source of truth) とする。

## このディレクトリの中身

| ファイル | 役割 |
| --- | --- |
| [`principles.md`](principles.md) | 設計原則 (責務境界、依存方向、独自 UI Core 方針、port 設計、primary / 互換 adapter) |
| [`detailed-design-and-tasks.md`](detailed-design-and-tasks.md) | Phase 別詳細設計と超細分化 task |
| [`phase-5-katana-integration.md`](phase-5-katana-integration.md) | KatanA 本体側の Phase 5 抜粋 |

## 各 repo 用抜粋ファイル

repo owner は自分の repo の抜粋を読めば作業が完結する設計。

| repo | 抜粋ファイル | 担当 Phase |
| --- | --- | --- |
| `katana-ui-core` (旧 `katana-ui-widget`) | [`docs/ui-separation-plan.md`](../../../../katana-ui-widget/docs/ui-separation-plan.md) | Phase 1 (KUC neutral core + runtime/window/surface) + P4-0 (primary 選定) + P1-K (互換 adapter) + P1-L (runtime/window/surface) |
| `katana-document-viewer` | [`docs/ui-separation-plan.md`](../../../../katana-document-viewer/docs/ui-separation-plan.md) | Phase 2 (KDV + forge facade) + Phase 7 (KDV 側) |
| `katana-language-editor` | [`docs/ui-separation-plan.md`](../../../../katana-language-editor/docs/ui-separation-plan.md) | Phase 3 (KLE domain + ports + md adapter) |
| `katana-canvas-forge` | [`docs/ui-separation-plan.md`](../../../../katana-canvas-forge/docs/ui-separation-plan.md) | Phase 7 (KCF 側 public API 縮小 / CLI delegate) |
| `katana-diagram-renderer` | [`docs/ui-separation-plan.md`](../../../../katana-diagram-renderer/docs/ui-separation-plan.md) | Phase 7 (KDR pure renderer 方針) |
| `katana-markdown-model` | [`docs/ui-separation-plan.md`](../../../../katana-markdown-model/docs/ui-separation-plan.md) | Phase 6 (KMM canonical) + KME compatibility |
| `katana-chat-ui` | [`docs/ui-separation-plan.md`](../../../../katana-chat-ui/docs/ui-separation-plan.md) | Phase 4 ChatPane 接続 |
| `katana` (KatanA 本体) | [`phase-5-katana-integration.md`](phase-5-katana-integration.md) | Phase 5 (KatanA integration) |

## 略語

| 略語 | 展開 |
| --- | --- |
| KUC | `katana-ui-core` (旧 `katana-ui-widget`。ADR-0002 で rename) |
| KDV | `katana-document-viewer` |
| KLE | `katana-language-editor` |
| KCF | `katana-canvas-forge` |
| KDR | `katana-diagram-renderer` |
| KMM | `katana-markdown-model` |
| KME | `katana-markdown-engine` |
| KCU | `katana-chat-ui` |

## Phase 一覧と依存

```text
P0 governance / naming / ADR
  ↓
P1 KUW neutral core ─────────┐
P2 KDV viewer + forge facade │  (P0 前提)
P3 KLE domain expansion ─────┤
                             ↓
                       P4 katana-ui composition
                             ↓
                       P5 KatanA integration
                       P6 KMM canonical (P0-B 前提)
                       P7 KCF / KDR / KDV forge 再編 (P2-E 前提)
                       P8 documentation / OpenSpec
                       P9 release strategy
```

詳細は [`detailed-design-and-tasks.md` 6.5 Phase 依存グラフ](detailed-design-and-tasks.md#65-phase-依存グラフ) を参照。

## 同期方針

- master ([`detailed-design-and-tasks.md`](detailed-design-and-tasks.md)) が単一情報源。
- 各 repo の `docs/ui-separation-plan.md` は master からの抜粋。drift を防ぐため、両方に同じ task ID を持たせる。
- 抜粋ファイル単独で task を追加・修正してはならない。**master を先に更新**し、影響する各 repo の抜粋 PR を追従させる。
- P8-A-001 で master と抜粋の task ID 一致を CI で検査する。

## 関連 ADR / OpenSpec change の配置

| 種類 | 配置 repo | パス例 |
| --- | --- | --- |
| ADR (横断的設計判断) | `katana` | `docs/adr/<topic>.md` |
| Repo 個別 ADR | 各 repo | `docs/adr/<topic>.md` |
| OpenSpec change | 各 repo | 既存 OpenSpec のルートに従う |
| Inventory (実装着手前の現状調査) | 各 repo | `docs/inventory/<topic>.md` |
| Release policy | `katana` | `docs/release/compatibility-windows.md` |

## 編集前に読むべきもの

1. `principles.md` — 責務境界と禁止依存
2. `detailed-design-and-tasks.md` の section 6.5 — Phase 依存グラフ
3. 自分の repo の `docs/ui-separation-plan.md`
4. 既存の repo 個別 docs (`coding-rules.md`、`roadmap.md` 等)

順番を守ると、Phase 間の不整合や、責務外への変更を防げる。
