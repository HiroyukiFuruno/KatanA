# v0.13.0 Markdown Diagnostics and Problems Panel 実装計画

本計画は、`openspec/changes/v0-13-0-markdown-diagnostics-and-problems-panel/tasks.md` における全タスクを達成するための技術的なマイルストーンを定義します。

## 対象スコープ

具体的なタスク内容・スコープについては `openspec/changes/v0-13-0-markdown-diagnostics-and-problems-panel/tasks.md` を参照してください。

## アプローチ: TDD (Red -> Green -> Refactor)

KatanAのコーディング原則に従い、UIフィードバックとロジック実装を以下のTDDサイクルで各ステップごとに進めます。

1. **Red**: 実装開始前に、要件を満たさないことを証明する失敗するテスト（Unit Test または Integration Test）を作成します（※画像比較は用いません）。
2. **Green**: テストを通過させる最小限の実装を追加します。
3. **Refactor**: グリーン状態を維持しつつ、責務の分離とコードのクリーンアップを実施します。

## フェーズ 1: Diagnostics 契約定義と Engine (katana-linter) の拡張

- `katana-linter` に `MarkdownDiagnostic` 構造体を追加し、（Severity、Range、Message、RuleID など）を定義。
- `katana-linter` 上に `Markdown` 評価基盤を用意。
- 基本的なルール（Missing Local Assets, Broken Relative Links, Heading Sync etc.）の deterministic なロジック部分の TDD 実装。

## フェーズ 2: App State と Background 更新の統合

- `katana-ui` の `AppState` に `problems: Vec<MarkdownDiagnostic>` などを追加。
- ファイル保存契機、手動 Refresh 契機で linter を実行し `AppState` を更新するフローを実装。
- 古いファイルや未解決Locationがあってもクラッシュしない回復可能なロジックの担保。

## フェーズ 3: Problems Panel UI 実装とナビゲーション

- Bottom panel として `Problems Panel` を新設。
- 診断結果の一覧表示とEmpty state の表現。
- クリックによる対象エディタ・プレビューへの `jump` 操作の実装。
- **※UIは実装後にスクリーンショット等でユーザーへ提示しフィードバックを受けて微調整します。**

## フェーズ 4: 最終検証 (Verification)

- `tests/integration/` などにおいて、Markdown ファイルの問題点を検知・ジャンプできること等を試験。
- 全体の検証を実施し、OKが出次第リリースフローへと移行する。
