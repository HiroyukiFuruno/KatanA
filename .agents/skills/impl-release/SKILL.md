---
name: impl-release
description: KatanAで指定バージョンのOpenSpec実装からリリースPR作成までを一気通貫で進めるときに使う。/impl-release vX.Y.Z と同等のリリース実装ワークフロー。
---

# impl-release

このスキルは、`/impl-release vX.Y.Z` として扱いたいリリース実装ワークフローの入口です。

## 実行ルール

1. ユーザー指定のバージョン（例: `v0.22.6`）を対象にする。
2. 詳細手順は `.agents/workflows/impl-release.md` を正として読み込む。
3. 参照先ワークフローに従い、OpenSpec実装、検証、リリース準備、PR作成、自己レビュー、事後整理を進める。
4. 既存の作業差分がある場合は、作業開始前に `git status --short --branch` で確認し、関心事が混ざらないように扱う。
   - OpenSpec 変更はユーザーが手動で整理することがあるため、実装対象外の OpenSpec 差分だけを理由に停止しない。コミット対象だけを明示的に分け、ユーザーが受け入れている差分として作業を継続する。
5. Codex の作業計画は、現在の個別 Task ではなく、OpenSpec の User Review Phase の `x.1` までを含めて組み立てる。
6. 個別 Task 完了時に停止せず、各 Task Group を統合ブランチへ順次マージし、User Review Phase の `x.1` で初めてユーザー確認を求める。
7. タスクグループ（Task Group）間の依存関係を確認し、依存しない調査・実装・検証・ハーネス更新は補助エージェント（subagent）に移譲して並列化する。ただし親エージェントは直近のクリティカルパスを保持し、補助エージェントの書き込み範囲を明示して重複させない。
8. UI 変更を含む場合、User Review Phase `x.1` の前に `scripts/screenshot/run.sh` と git 管理された request JSON でスクリーンショットまたは動画を生成し、ユーザーへ成果物パスを提示する。生成物は `.gitignore` 対象の出力先に保存する。
9. Task ごとの通常 PR push では、push 前に重い `make check` / `make check-light` を二重実行せず、通常の `git push` で `pre-push` hook を通す。`git push --no-verify` は hook 自体の不具合、または同一コミットで同等以上のゲートを通した直後の再 push に限り、理由と証跡を `tasks.md` または PR 本文へ記録する。
10. 1つの Task Group が大きすぎる場合は、計画段階で `2A` / `2B` / `2C` のように分割する。分割単位は AsIs/ToBe の乖離、保存モデル、共通 UI 部品（widget: 再利用できる画面部品）、Lint 連携、移行テストなどを基準にし、各分割タスクの依存関係、書き込み範囲、担当、検証範囲、ブランチ名を明記する。分割後も User Review Phase `x.1` までは停止しない。
11. ユーザーFB対応中の検証は、変更箇所に直結する対象テスト、lint、またはスクリーンショット確認を優先する。`make check-light` / `make check` は、Task Group の配送直前、リリースPR直前、または影響範囲が広い変更後に限定し、同じ差分に対して繰り返し実行しない。品質担保は維持するが、FBごとの小修正で全体ゲートを毎回再実行して時間を浪費しない。

## 注意

- `workflows/impl-release.md` は手順書であり、Codexのスキル呼び出し対象ではない。
- `/` から呼び出したい場合は、この `impl-release` スキルを使う。
