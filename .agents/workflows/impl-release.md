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

---

## 遂行プロセス

### Phase 1: 環境準備

1. `master` を最新化し、**`release/vX.Y.Z`** ブランチを作成する。
   これが今回の全実装およびリリースのための**統合ブランチ**となる。

```bash
git switch master && git pull origin master
git switch -c release/vX.Y.Z
```

2. 対応する OpenSpec ディレクトリ（`openspec/changes/vX-Y-Z-*`）を特定する。

### Phase 2: 実装フェーズ (Implementation)

3. `tasks.md` の未完了タスクに対し、**`task-micro-cycle.md`** に従い実装を進める。

4. 各 Major Task Group ごとに **`feature/vX.Y.Z-taskN`** ブランチを作成し、実装完了後に **`openspec-delivery.md`** を使用して `release/vX.Y.Z` へのマージと同期を行う。

> [!IMPORTANT]
> すべての実装は `release/vX.Y.Z` に対して行われ、この段階では `master` には一切触れない。
> 全タスクが `[x]` になり、`release/vX.Y.Z` に統合されるまで繰り返す。

### Phase 3: リリース準備 (Release Prep)

5. `make release VERSION=X.Y.Z` を実行し、`Cargo.toml`, `Cargo.lock`, `Info.plist` を一括更新する。

6. `changelog-writing` スキルを起動し、実装された内容に基づいて `CHANGELOG.md`（UTC）と `CHANGELOG.ja.md`（JST）に変更内容を記載する。

### Phase 4: 整合性チェック & QA

7. `./scripts/release/check-pr-ready.sh X.Y.Z` を実行し、全項目が **[OK]** になるまで修正を繰り返す。

8. `make check` を実行し、Lint / Test / Coverage の最終ローカルゲートを通過させる。

### Phase 5: リリース PR 作成 (to master)

9. **OpenSpec のアーカイブ**: `/opsx-archive` を実行し、対象の OpenSpec ディレクトリを `archive/` へ移動する。
   - これにより、仕様の「完了」と「リリース」が同一の PR に含まれることになる。
   - `opsx-archive` 時には、delta specs の main specs への同期（Sync）も同時に実施すること。

10. `commit_and_push` スキルに従い、バージョン更新・CHANGELOG・アーカイブ移動を一括して**日本語メッセージ**でリリースコミットを行う。

```bash
git add .
git commit -S -m "release: vX.Y.Z リリース準備完了 (OpenSpec アーカイブ含む)"
git push origin release/vX.Y.Z
```

11. `create_pull_request` スキルを使用し、`release/vX.Y.Z` → `master` の PR を作成する。

> [!CAUTION]
> **`build-and-release.yml` のトリガー条件**:
> PR が `release/v*` ブランチから `master` へマージされた時に自動発火します。命名規則を遵守すること。

### Phase 6: 自己レビュー & 継続的改善

12. `self-review` スキルを起動し、PR の差分全体に対して監査を行う：
    - 実装が OpenSpec の意図通りか
    - アーカイブ移動や specs の同期が正しく反映されているか
    - バージョン文字列に漏れや誤記はないか
    - CHANGELOG の日英内容が正しく、かつユーザーフレンドリーか

13. GitHub Actions の CI チェックがすべてパスするまで `gh pr checks --watch` で監視する。

### Phase 7: マージ & 事後処理

14. すべてのチェックがパスし、自己レビューによる修正も完了したことをユーザーに報告し、マージ承認を得る。

15. 承認後、`gh pr merge --merge --delete-branch` で PR をマージする。
    これにより CD ワークフローが発火し、配布物が公開される。

16. ローカルブランチをクリーンアップする。

```bash
git switch master && git pull
git branch -D release/vX.Y.Z
```

---

## 完了の定義

- [ ] `tasks.md` の全タスクが完了し、`release/vX.Y.Z` に統合されている
- [ ] `./scripts/release/check-pr-ready.sh` および `make check` がすべてパスしている
- [ ] `release/vX.Y.Z` → `master` の PR がマージされ、CD (build-and-release) が開始されている
- [ ] OpenSpec 変更ディレクトリが `archive/` に移動されている
- [ ] ローカル環境の作業ブランチがクリーンアップされている
