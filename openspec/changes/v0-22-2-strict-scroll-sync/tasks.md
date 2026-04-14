## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `release/v0-22-2`
- **作業ブランチ**: `feature/v0-22-2-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. Core Logic (ScrollMapper)

- [x] 1.1 `ScrollMapper` に文末（EOF）アンカーの明示的なサポートを追加
- [x] 1.2 段落・ブロックレベルのアンカー登録インターフェースの実装
- [x] 1.3 高密度アンカーを効率的に処理するための補完ロジックの改善

### Definition of Done (DoD)

- [x] ユニットテストによる境界条件（EOF等）の検証
- [x] `/openspec-delivery` ワークフローを実行して全デリバリ手順を完了する

## 2. UI Integration (Editor & Preview)

### Definition of Ready (DoR)

- [x] タスク1のデリバリサイクルが完了し、`release/v0-22-2` に統合されていること。

- [x] 2.1 Markdown レンダラから全ブロック要素の同期アンカーを送出するよう拡張
- [x] 2.2 エディタ側に適応型下部パディング（Ghost Space）を実装
- [x] 2.3 プレビュー側に適応型下部パディング（Ghost Space）を実装
- [x] 2.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 2.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] 実機でのスクロール連動の滑らかさと精度の確認
- [x] `/openspec-delivery` ワークフローを実行して全デリバリ手順を完了する

## 3. Final Verification & Release Work

- [ ] 3.1 `docs/coding-rules.ja.md` および `.agents/skills/self-review/SKILL.md` を用いた自己レビューの実施
- [ ] 3.2 `make check` が終了コード 0 でパスすることを確認
- [ ] 3.3 `release/v0-22-2` から `master` への PR を作成
- [ ] 3.4 PR 上の CI チェック（Lint / Coverage / CodeQL）の成功を確認
- [ ] 3.5 master へのマージ (`gh pr merge --merge --delete-branch`)
- [ ] 3.6 GitHub Release の完了確認および `/opsx-archive` による本チェンジのアーカイブ
