---
description: 指定バージョンの実装・修正からリリース準備、PR作成・自己レビューまでを一気通貫で自律的に遂行する Implementation & Release Autopilot ワークフロー。
---

# /impl-release vX.Y.Z (Implementation & Release Autopilot)

指定バージョンの OpenSpec に基づく実装・修正から、リリース準備（バージョン同期、CHANGELOG更新）、PR 作成、自己レビュー＆改善までを自律的に実行します。
PR マージ後は `build-and-release.yml` が自動発火し、マルチプラットフォームのビルド・配布が行われます。

## 前提条件

- リリース対象の OpenSpec ディレクトリ（`openspec/changes/vX-Y-Z-*`）が存在し、設計とタスクが確定していること。
- 作業開始前に `master` の最新状態が反映されていること。

## 参照するスキル・ワークフロー

| 参照先 | 用途 |
|--------|------|
| `.agents/workflows/openspec-branching.md` | ブランチ命名規則と運用ルール（Release Case を適用） |
| `.agents/workflows/task-micro-cycle.md` | 実装フェーズにおける単一タスク単位の厳格なサイクル |
| `.agents/workflows/openspec-delivery.md` | 各タスクの検証・コミット・マージ・同期の自動化 |
| `.agents/skills/changelog-writing/SKILL.md` | CHANGELOG の日英同期記載 |
| `.agents/skills/commit_and_push/SKILL.md` | コミット・プッシュの規約（日本語メッセージ必須） |
| `.agents/skills/create_pull_request/SKILL.md` | PR 作成の手順とテンプレート |
| `.agents/skills/self-review/SKILL.md` | 自己レビューチェックリスト |
| `scripts/screenshot/run.sh` | ユーザーレビュー前のスクリーンショット・動画生成 |
| Global `branch-hygiene` skill | マージ後のローカルブランチ・リモートブランチ・`git worktree` 掃除 |

---

## 遂行プロセス

## `task-micro-cycle` との優先関係

`impl-release` 実行中は、このワークフローを最上位の進行ルールとして扱う。

- `task-micro-cycle.md` の「単一タスクごとの停止」と「コミット前の承認待ち」は、通常の実装依頼向けの安全策であり、`impl-release` では適用しない。
- `impl-release` では、各 Task Group の実装、検証、コミット、PR 作成、統合ブランチへのマージまでを AI が連続して進める。
- ユーザーへの確認で停止する場所は、Phase 3 のユーザーレビュー、Phase 8 のマージ承認、またはテスト失敗・競合・仕様不明などのブロッカー発生時だけとする。
- Task 1、Task 2 などの個別タスク完了時に「ここで止めてよいか」を聞いてはならない。
- ユーザー手動の OpenSpec 整理差分、別バージョンの OpenSpec 移動、または実装対象外の文書差分が同じ作業ツリーに存在しても、それだけを理由に停止してはならない。`git status --short --branch` で存在を把握し、Task Group のコミット対象を明示的に分離して継続する。

## Codex 作業計画と補助エージェント（subagent）並列化

`impl-release` 起動時の Codex 可視タスク計画は、現在の個別 Task Group で終わらせず、OpenSpec の User Review Phase `x.1` までを含める。

- 計画は単なる順番待ちリストにせず、親エージェントが進める直近の作業と、補助エージェント（subagent）に任せる並列作業を分けて書く。
- タスクグループ（Task Group）間の依存関係を確認し、依存しない調査・実装・検証・ハーネス更新は補助エージェント（subagent）に移譲して並列化する。
- 親エージェントは直近のクリティカルパス（現在 Task Group の実装判断、統合ブランチ同期、PR 作成・マージ、依存関係の解消）を手放さない。
- 補助エージェント（subagent）へ渡す時は、書き込み範囲をファイルまたはディレクトリ単位で明示し、補助エージェント同士や親の作業範囲を重ねない。
- 依存関係があるタスクグループ（Task Group）、同じファイルを編集する変更、統合ブランチの状態判断は順序を守り、親が最終的に差分を確認して取り込む。
- 1つのタスクグループ（Task Group）が大きすぎる場合は、そのまま1ブランチへ詰め込まず、計画段階で `2A` / `2B` / `2C` のような分割タスクへ再編する。
- 分割単位は AsIs/ToBe の乖離、保存モデル、共通 UI 部品（widget: 再利用できる画面部品）、Lint 連携、移行テストなど、責務と書き込み範囲が分かれる境界を使う。
- 分割した各タスクには、依存関係、親または補助エージェントの担当、書き込み範囲、検証範囲、ブランチ名（例: `feature/vX.Y.Z-task2a`）を明記する。
- 分割後もユーザー確認の停止位置は変えず、全分割タスクを統合ブランチへ順次マージしてから User Review Phase `x.1` へ進む。

## 検証粒度の原則

`impl-release` では品質担保を落とさず、検証の実行タイミングを分ける。

- **対象確認**: FB対応や小修正では、変更箇所に直結する単体テスト、統合テスト、lint、スクリーンショット確認だけを実行する。
- **節目確認**: Task Group の配送直前、図形描画・設定保存・i18n など影響範囲が広い変更後は `make check-light` を実行する。
- **正式ゲート**: Task PR push と release PR push は、通常の `git push` による `pre-push` hook を正式な品質ゲートとする。
- **重複禁止**: 同じ差分に対して `make check-light` / `make check` と `pre-push` hook を連続で二重実行しない。すでに同等以上のゲートを通した場合は、以降の小修正では対象確認に戻し、次の節目まで全体ゲートを繰り返さない。
- **証跡**: 実行した対象確認、節目確認、正式ゲートは `tasks.md` または PR 本文に残し、未実行の全体ゲートを「通った」と扱わない。

### Phase 1: 環境準備

1. `master` を最新化し、**`release/vX.Y.Z`** ブランチを作成する。
   これが今回の全実装およびリリースのための**統合ブランチ**となる。

```bash
git switch master && git pull origin master
git switch -c release/vX.Y.Z
```

1. 対応する OpenSpec ディレクトリ（`openspec/changes/vX-Y-Z-*`）を特定する。

### Phase 2: 実装フェーズ (Implementation)

1. `tasks.md` の未完了タスクに対し実装を進める。
   - **完全自律進行の原則**: 各タスク（Task 1, Task 2...）やサブタスク（x.x）の完了ごとにユーザーへ都度進行の確認や承認を求めることは**禁止**します。実装、対象検証、通常の `git push` による `pre-push` hook、PR作成、PRマージまでの全サイクルをAIが自律的かつ連続的に遂行し、完了次第すぐに次のタスクへ進んでください。
   - ユーザーへの確認は、最終確認フェーズ前の「ユーザーレビューフェーズ（Phase 3）」で行うものとし、タスクごとの個別確認は行いません。

2. 各 Major Task Group ごとに **`feature/vX.Y.Z-taskN`** ブランチを作成し、実装完了後に **`openspec-delivery.md`** を使用して `release/vX.Y.Z` へのマージと同期を行う。

> [!IMPORTANT]
> すべての実装は `release/vX.Y.Z` に対して行われ、この段階では `master` には一切触れない。
> 全タスクが `[x]` になり、`release/vX.Y.Z` に統合されるまで繰り返す。

### Phase 3: ユーザーレビュー (User Review)

1. 全ての実装タスクが完了し `release/vX.Y.Z` に統合された後、最終確認フェーズの前にユーザーへのレビュー依頼を行う。
   - UI の動作確認は、ユーザーにアプリ起動や手動操作を依頼する前に、`scripts/screenshot` 配下のシナリオでスクリーンショットまたは動画を生成し、成果物を提示して確認できる状態にすること。
   - シナリオ定義は git 管理対象にしてよいが、生成されたスクリーンショット・動画の出力先は `.gitignore` に追加してから使うこと。
   - 既存の実行入口は `scripts/screenshot/run.sh`。最小確認は `./scripts/screenshot/run.sh --request scripts/screenshot/examples/workspace-main.json --output scripts/screenshot/output/vX.Y.Z-review` で実行する。
   - request は `schema_version: "1"`、`fixture`、`steps` を持つ JSON とし、スクリーンショットは `{"type":"screenshot","output_name":"..."}`、動画は `record_start` / `record_stop` を使う。動画生成には `ffmpeg` が必要。
   - 生成物は `scripts/screenshot/output/`、`outputs/`、`captures/`、`recordings/` など `.gitignore` 済みの出力先へ保存し、ユーザーには生成された画像または動画のパスを提示する。
   - ユーザーから受けた指摘事項（技術的負債に関する指摘を含む）は、すべて `tasks.md` に書き溜めて対応履歴を管理すること。
   - ユーザーから「個別劣後（後回しでよい）」と明言されない限り、技術的負債を含め、指摘されたすべての事項をこのセッション内で解決してから次のフェーズへ進むこと。

### Phase 4: リリース準備 (Release Prep)

1. `make release VERSION=X.Y.Z` を実行し、`Cargo.toml`, `Cargo.lock`, `Info.plist` を一括更新する。

2. `changelog-writing` スキルを起動し、実装された内容に基づいて `CHANGELOG.md`（UTC）と `CHANGELOG.ja.md`（JST）に変更内容を記載する。

### Phase 5: 整合性チェック & QA

1. `./scripts/release/check-pr-ready.sh X.Y.Z` を実行し、全項目が **[OK]** になるまで修正を繰り返す。

2. push 前の正式な品質ゲートは `pre-push` hook とする。通常の Task PR では、重い `make check` / `make check-light` を push 直前に二重実行せず、必要な対象検証を実施した上で通常の `git push` を行い、hook に通す。

3. `git push --no-verify` は原則禁止する。例外は、hook 自体の不具合、または同一コミットで同等以上の hook ゲートを通した直後の再 push に限る。例外を使う場合は、理由、直前に通したゲート、対象コミット、実行コマンドを `tasks.md` または PR 本文に記録する。

4. markdownのフォーマット（format）および Lint修正（lintfix）を実行し、`tasks.md` を含む全ドキュメントの体裁を整える。

### Phase 6: リリース PR 作成 (to master)

1. **OpenSpec のアーカイブ**: `/opsx-archive` を実行し、対象の OpenSpec ディレクトリを `archive/` へ移動する。

- これにより、仕様の「完了」と「リリース」が同一の PR に含まれることになる。
- `opsx-archive` 時には、delta specs の main specs への同期（Sync）も同時に実施すること。

1. `commit_and_push` スキルに従い、バージョン更新・CHANGELOG・アーカイブ移動を一括して**日本語メッセージ**でリリースコミットを行う。

```bash
git add .
git commit -S -m "release: vX.Y.Z リリース準備完了 (OpenSpec アーカイブ含む)"
git push origin release/vX.Y.Z
```

1. `create_pull_request` スキルを使用し、`release/vX.Y.Z` → `master` の PR を作成する。

> [!CAUTION]
> **`build-and-release.yml` のトリガー条件**:
> PR が `release/v*` ブランチから `master` へマージされた時に自動発火します。命名規則を遵守すること。

### Phase 7: 自己レビュー & 継続的改善

1. `self-review` スキルを起動し、PR の差分全体に対して監査を行う：
    - 実装が OpenSpec の意図通りか
    - アーカイブ移動や specs の同期が正しく反映されているか
    - バージョン文字列に漏れや誤記はないか
    - CHANGELOG の日英内容が正しく、かつユーザーフレンドリーか

2. GitHub Actions の CI チェックがすべてパスするまで `gh pr checks --watch` で監視する。

### Phase 8: マージ & 事後処理

1. すべてのチェックがパスし、自己レビューによる修正も完了したことをユーザーに報告し、マージ承認を得る。

2. 承認後、`gh pr merge --merge --delete-branch` で PR をマージする。
    これにより CD ワークフローが発火し、配布物が公開される。

3. `branch-hygiene` スキルを使用し、ローカルブランチ・リモートブランチ・`git worktree` をクリーンアップする。
    タスクブランチが残っている場合は、未コミット差分と未統合状態を確認した上で明示的に削除する。

```bash
git switch master
git fetch --all --prune
git pull
git worktree list --porcelain
git branch -d release/vX.Y.Z
git branch --format='%(refname:short)' | grep "feature/vX.Y.Z-task" || true
git worktree prune
```

最後に `git worktree list --porcelain`、`git branch -a`、`git status --short --branch` を再確認し、削除したものと残したものを報告する。

---

## 完了の定義

- [ ] `tasks.md` の全タスクが完了し、`release/vX.Y.Z` に統合されている
- [ ] `./scripts/release/check-pr-ready.sh` と通常 `git push` による `pre-push` hook がすべてパスしている
- [ ] `release/vX.Y.Z` → `master` の PR がマージされ、CD (build-and-release) が開始されている
- [ ] OpenSpec 変更ディレクトリが `archive/` に移動されている
- [ ] ローカル環境の作業ブランチ、リモートブランチ、`git worktree` がクリーンアップまたは理由付きで残存報告されている
