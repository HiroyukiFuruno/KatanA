---
description: 指定バージョンのリリース準備からPR作成・自己レビュー・改善までを一気通貫で自律的に遂行する Release Autopilot ワークフロー。
---

# /impl-release vX.Y.Z (Release Autopilot Workflow)

指定バージョンのリリース準備（バージョン同期、CHANGELOG更新）から PR 作成、自己レビュー＆改善までを自律的に実行します。
PR マージ後は `build-and-release.yml` が自動発火し、マルチプラットフォームのビルド・配布が行われます。

## 前提条件

- リリース対象の機能・修正がすべて `master` にマージ済みであること。
- 対応する OpenSpec ディレクトリ（`openspec/changes/vX-Y-Z-*`）が存在する場合、全タスクが完了済み（`[x]`）であること。

## 参照するスキル・ワークフロー

| 参照先 | 用途 |
|--------|------|
| `.agents/workflows/release.md` | リリースフロー全体の公式手順（本ワークフローの上位仕様） |
| `.agents/workflows/openspec-branching.md` | ブランチ命名規則と運用ルール |
| `.agents/skills/changelog-writing/SKILL.md` | CHANGELOG の日英同期記載 |
| `.agents/skills/commit_and_push/SKILL.md` | コミット・プッシュの規約（日本語メッセージ必須） |
| `.agents/skills/create_pull_request/SKILL.md` | PR 作成の手順とテンプレート |
| `.agents/skills/self-review/SKILL.md` | 自己レビューチェックリスト |

---

## 遂行プロセス

### Phase 1: 環境準備

1. `master` を最新化し、**`release/vX.Y.Z`** ブランチを作成する。
   ブランチ命名は `openspec-branching.md` の「Release Case」に従う。

```bash
git switch master && git pull origin master
git switch -c release/vX.Y.Z
```

2. 対応する OpenSpec ディレクトリ（`openspec/changes/vX-Y-Z-*`）を特定し、`tasks.md` の状態を確認する。

### Phase 2: リリース準備

3. `make release VERSION=X.Y.Z` を実行し、`Cargo.toml`, `Cargo.lock`, `Info.plist` を一括更新する。

4. `changelog-writing` スキルを起動し、`CHANGELOG.md`（UTC）と `CHANGELOG.ja.md`（JST）に変更内容を記載する。

### Phase 3: 整合性チェック

5. `./scripts/release/check-pr-ready.sh X.Y.Z` を実行し、全項目が **[OK]** になるまで修正を繰り返す。

6. `make check` を実行し、Lint / Test / Coverage のローカルゲートを通過させる。

### Phase 4: コミット & PR 作成

7. `commit_and_push` スキルに従い、**日本語メッセージ**でコミットする。
   リリースコミットのため `--no-verify` は使用不可。

```bash
git add .
git commit -S -m "release: vX.Y.Z リリース準備"
git push origin release/vX.Y.Z
```

8. `lefthook run pre-pr` を実行し、PR 前の機械検証をパスさせる。

9. `create_pull_request` スキルを使用し、`release/vX.Y.Z` → `master` の PR を作成する。

> [!IMPORTANT]
> **`build-and-release.yml` のトリガー条件**: PR マージ時に `release/v*` ブランチからのマージであることを検知して自動発火します。
> ブランチ名が `release/vX.Y.Z` でなければリリースは発火しません。

### Phase 5: 自己レビュー & 継続的改善

10. `self-review` スキルを起動し、PR の差分に対してリリース固有の観点を含む監査を行う：
    - バージョン文字列に漏れや誤記はないか
    - CHANGELOG の日付・バージョン・日英内容が正しいか
    - 不要なデバッグコードや一時的な変更が混入していないか

11. 指摘事項がある場合、即座に修正し、再度 `check-pr-ready.sh` をパスさせた上で追記コミットを push する。

12. GitHub Actions の CI チェックがすべてパスするまで `gh pr checks --watch` で監視する。

> [!WARNING]
> **`paths-ignore` 注意**: `build-and-release.yml` は `.md`, `scripts/**`, `.agents/**` 等を `paths-ignore` に設定しています。
> CHANGELOG のみの修正コミットではリリースワークフローが発火しない可能性があります。
> 必ず `Cargo.toml` / `Cargo.lock` / `Info.plist` の変更を含むコミットが PR に含まれていることを確認してください。

### Phase 6: マージ承認依頼

13. すべてのチェックがパスしたことをユーザーに報告し、マージの承認を仰ぐ。

14. 承認後、`gh pr merge --merge --delete-branch` で PR をマージする。
    これにより `build-and-release.yml` が自動発火し、配布物が公開される。

15. マージ後のブランチクリーンアップ（`openspec-branching.md` Step 6 に準拠）：

```bash
git switch master && git pull
git branch -D release/vX.Y.Z
```

### Phase 7: 事後処理

16. `gh run list --workflow Release --limit 5` でリリースワークフローの実行を確認する。

17. 対応する OpenSpec が存在する場合、`/opsx-archive` でアーカイブを実施する。

---

## 完了の定義

- [ ] `release/vX.Y.Z` → `master` の PR が作成され、全 CI がパスしている
- [ ] 自己レビューによる改善が完了し、Verified なコミット履歴が維持されている
- [ ] ユーザーの承認を得てマージが完了し、リリースワークフローが正常に発火している
- [ ] ローカル・リモートの `release/vX.Y.Z` ブランチが削除済みである
- [ ] 該当する OpenSpec のアーカイブが完了している（該当がある場合）
