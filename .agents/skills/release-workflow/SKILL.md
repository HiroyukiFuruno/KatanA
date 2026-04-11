---
name: release-workflow
description: KatanAのリリースプロセスを管理し、GitHub Releaseの作成から配布までを遂行するスキル。衝突時の強制上書き（FORCE）もサポート。
---

# KatanA Release Workflow Skill

このスキルは、プロジェクトの新しいバージョンを公開するための手順を定義します。原則として GitHub Actions 上で動作しますが、AIエージェントはそのトリガーと監視を担当します。

## 核心原則

1. **タグの直接操作禁止**: `git tag` をローカルで実行しないでください。
2. **検証ファースト**: リリース前に `/opsx-verify` や `make check` が完了していることを確認します。
3. **Verified Commit (署名済み) 徹底**: 全てのリリース関連コミット（ボット含む）を GitHub 上で「Verified」にするため、CI 上での変更は `gh api` を経由させます。
4. **Info.plist 同期ルール**: macOS 向けビルドの整合性を保つため、`Cargo.toml` の更新時は必ず `crates/katana-ui/Info.plist` を同一バージョンに更新する必要があります。

## 手順

### 1. リリース準備

- `changelog-writing` スキルを使用して、`CHANGELOG.md` (UTC) と `CHANGELOG.ja.md` (JST) を更新、プッシュします。
- **(⚠️ 必須) 手動バージョン更新**: `make release VERSION=X.Y.Z` を実行して、`Cargo.toml` と `crates/katana-ui/Info.plist` の `version` を新しいリリースバージョンに一括書き換えします。コミット・プッシュの際は GPG 署名を行い、マージ後のコミットが署名付き (Verified) になるようにしてください。
- **(⚠️ 必須) PR作成前の機械적チェック**: PRを作成する前に、必ず `./scripts/release/check-pr-ready.sh` (または `lefthook run pre-pr`) を実行してください。
  - **マージ条件**: `Cargo.toml`, `Cargo.lock`, `Info.plist` がすべて指定したバージョンで揃っており、かつブランチ名が `release/vX.Y.Z` 形式である必要があります。
- `create_pull_request` スキルを使用して対象のリリース用機能ブランチから `master` に向けたPRを作成します。

### 2. CI と署名の確認

- **リリースチェック CI**: PR 作成時、自動的に `Release Readiness` ジョブが走り、バージョン不整合がないか再検証されます。これがパスしない限りマージはできません。
- **Verified ステータスの確認**: PR 内のすべてのコミットに「Verified」バッジがついていることを確認してください。

### 3. マージとデプロイ

通常は PR (release/vX.Y.Z -> master) がマージされることでリリースのアクションが自動実行されます。

- **アテステーション (Provenance)**: `build-and-release.yml` は実行時に **Build Attestation** を生成します。これにより、配布されるバイナリの出自が GitHub Actions であることが「Verified」として証明されます。
- **自動署名リカバリ**: 万が一 CI 上でボットがバージョンを微調整する場合、`scripts/release/bump-version.sh` は `gh api` を使用して GitHub 署名付きのコミットを作成します。

### 4. リカバリー処理（衝突発生時）

もし `gh release create` が「Already exists」で失敗した場合：

1. YAML ファイルに `--clobber` が含まれているか確認。
2. `FORCE=1` 引数（`force: true` パラメータ）を付けて再トリガーする。

### 5. 事後処理

1. **OpenSpec アーカイブ**: リリース作業が終了したら、`/opsx-archive` を使用して対象のディレクトリをアーカイブします。
2. **ローカルブランチの掃除 (⚠️ 必須)**: 役割を終えたローカルブランチ（例: `release/vX.Y.Z`）を必ず削除（`git branch -D`）し、作業ディレクトリをクリーンに保ってください。

## 監視項目

- **Preflight**: バージョン文字列が `vX.Y.Z` 形式か、CHANGELOG に日付が入っているか。
- **Build**: macOS, Linux, Windows の各ビルドがバイナリ署名をパスしているか。
- **Publish**: GitHub Release にすべてのアセット（.dmg, .msi, .tar.gz, .zip）と `checksums.txt` が揃っているか。
