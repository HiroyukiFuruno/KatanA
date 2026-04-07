## Definition of Ready (DoR: 着手可能の定義)

- 対象範囲はバージョン `0.16.11` 向けのリッチブロックハイライトおよび画面分割スクロール同期（v0.16.8の微調整）に限定する
- この変更用ディレクトリ内に、提案（Proposal）、設計（design）、仕様（specs）が存在していること
- 再現しきれていない境界ドリフトやホバー時のマッピング漏れについての原因分析が完了していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. v0.16.11 リッチブロック微調整（ホバー＆スクロール同期）

- [ ] 1.1 `egui_commonmark` 内のブロックアンカーおよび `egui::Rect` の境界計算ロジックを見直し、スクロール配置や行高に起因するズレ（ドリフト）を修正する
- [ ] 1.2 ホバー判定時のオフセットやマージン等による座標ズレを補正し、エディタ上の対象行全体のみが正確にハイライトされるようにする
- [ ] 1.3 `ScrollMapper` の単調非減少制約（Monotonic points）を厳密に守るため、高さ0のブロックや位置が逆転する不正なマップポイントの計算エラーをフィルタ・補正する
- [ ] 1.4 新しく発生しているエッジケースが修正されたことを確認するため、統合テストまたはユニットテストを補強する
- [ ] 1.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] リッチブロック領域のホバーハイライトが、マージンやネストに影響されずエディタと正確に紐づいて機能すること
- [ ] 画面分割時のスクロールにおいて、要素前後での急激な位置飛びやドリフトが完全に解消されていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Final Verification & Release Work

- [ ] 2.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 2.2 Ensure `make check` passes with exit code 0
- [ ] 2.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 2.4 Create a PR targeting `master`
- [ ] 2.5 Merge into master (※ `--admin` is permitted)
- [ ] 2.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.16.11`
- [ ] 2.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
