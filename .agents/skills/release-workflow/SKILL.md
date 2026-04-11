---
name: release-workflow
description: KatanAのリリースプロセスを管理し、GitHub Releaseの作成から配布までを遂行するスキル。衝突時の強制上書き（FORCE）もサポート。
---

# KatanA Release Workflow Skill

このスキルは、プロジェクトの新しいバージョンを公開するための手順を定義します。原則として GitHub Actions 上で動作しますが、AIエージェントはそのトリガーと監視を担当します。

## 核心原則

1. **タグの直接操作禁止**: `git tag` をローカルで実行しないでください。
2. **検証ファースト**: リリース前に `/opsx-verify` や `make check` が完了していることを確認します。
3. **副作用の明示**: `[skip ci]` を用いたプッシュにより、再帰的なパイプライン実行を防ぎます。

## 手順

### 1. リリース準備

- `changelog-writing` スキルを使用して、`CHANGELOG.md` (UTC) と `CHANGELOG.ja.md` (JST) を更新、プッシュします。
- **(⚠️ 必須)** `Cargo.toml` の `version` を新しいリリースバージョンに手動で書き換え、コミット・プッシュします（これを忘れると事前検証CIで弾かれます）。
- `create_pull_request` スキルを使用して対象のリリース用機能ブランチから `master` に向けたPRを作成します。
- `self-review` スキルを実行し、PRの変更内容をレビューして品質を担保します。
- **リモートCIの成功確認 (⚠️ 必須)**: `gh pr checks --watch` 等を用いて GitHub Actions 上のテストが **すべて Success** になるまで待機します。ローカルでのパス確認を理由にした強制マージは禁止です。

### 2. リリースの自動・手動トリガー

通常は PR (release/vX.Y.Z -> master) がマージされることでリリースのアクションが自動実行されます。マージ時、**必ず全ての CI チェックがグリーンであることを確認**してください。

- **Rule Bypass 禁止 (⚠️ 厳守)**: `gh pr merge` 時に `--admin` フラグ等を使用して CI 保護を回避することは**絶対禁止**です。

```bash
# 基本
make release VERSION=0.18.6

# 既存のリリースを上書きする場合 (衝突時)
make release VERSION=0.18.6 FORCE=1
```

### 3. リカバリー処理（衝突発生時）

もし `gh release create` が「Already exists」で失敗した場合：

1. YAML ファイルに `--clobber` が含まれているか確認。
2. `FORCE=1` 引数（`force: true` パラメータ）を付けて再トリガーする。

### 4. 事後処理

1. **OpenSpec アーカイブ**: リリース作業が終了したら、`/opsx-archive` を使用して対象のディレクトリをアーカイブします。
2. **ローカルブランチの掃除 (⚠️ 必須)**: 役割を終えたローカルブランチ（例: `release/vX.Y.Z`）を必ず削除（`git branch -D`）し、作業ディレクトリをクリーンに保ってください。

## 監視項目

- **Preflight**: バージョン文字列が `vX.Y.Z` 形式か、CHANGELOG に日付が入っているか。
- **Build**: macOS, Linux, Windows の各ビルドがバイナリ署名をパスしているか。
- **Publish**: GitHub Release にすべてのアセット（.dmg, .msi, .tar.gz, .zip）と `checksums.txt` が揃っているか。
