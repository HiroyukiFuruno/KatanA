---
description: katanaプロジェクトの公式リリース手順。release/vX.Y.Zブランチでのリリース準備からPRマージ後の`gh workflow run`による全自動リリースまでのフローを定義。
---

# KatanA リリースワークフロー

本プロジェクト (KatanA) におけるバージョンリリースの公式手順です。

## ⚠️ 最重要ルール

- **リリースは GitHub Actions 上で完結**: ローカルでのビルド・タグ作成・GitHub Release 作成は一切行いません。
- **master への直接 push は禁止**: 必ず `release/vX.Y.Z` ブランチを作成し、PR経由で master にマージしてください。
- **PR の CI チェック（Lint / Coverage / CodeQL）が全通過していること**: ブランチ保護により、これらがパスしないと master へのマージ自体がブロックされます。
- **手動でのタグ打ちは禁止**: `git tag vX.Y.Z` を手動で打ってプッシュすることは固く禁止します。タグは GitHub Actions の `gh release create` により API 経由で自動作成されます。
- **手動での Cargo.toml バージョン変更は不要**: GitHub Actions がビルド時にワークスペース内で in-place にバージョンを書き換えます。

---

## リリースフロー

### Step 1: リリースブランチの作成

```bash
git checkout master
git pull origin master
git checkout -b release/vX.Y.Z
```

### Step 2: リリース前準備

リリースブランチ上で以下を確認・実施します。

1. **品質ゲートの事前確認**: `make check` がローカルで通ることを確認してください。
2. **OpenSpec の統合確認 (⚠️ 必須)**: 対象バージョンの OpenSpec Base Feature Branch がすべて `master` へマージ済みであることを確認してください。

   ```bash
   # 未マージのfeatureブランチがないことを確認
   git branch -a | grep -v master | grep -v HEAD | grep -v release/
   ```

### Step 3: バージョン書き換え & CHANGELOG記載 & push

`Cargo.toml` の `version` を新しいリリースバージョン（例: `0.18.7`）に書き換えます。その後、`changelog-writing` スキル (`@[.agents/skills/changelog-writing]`) を使用して、今回のバージョンの変更履歴を `CHANGELOG.md` と `CHANGELOG.ja.md` に記載し、コミット・プッシュします。

```bash
git add Cargo.toml CHANGELOG.md CHANGELOG.ja.md
git commit -m "release: vX.Y.Z"
git push origin release/vX.Y.Z
```

### Step 4: master PR作成

PRを作成する前に、必ず機械的なチェックを実行します。

```bash
lefthook run pre-pr
```

不備がないことを確認したら、`create_pull_request` スキル (`@[.agents/skills/create_pull_request]`) を使用し、対象ブランチから `master` に向けたPRを作成します。
※PRのタイトルは「release: vX.Y.Z」、本文は自動生成されたものを適宜調整します。

> [!TIP]
> `gh release-pr --title "release: vX.Y.Z"` エイリアスを使用すると、このチェックと PR 作成を一度に行うことができ、不備がある場合は自動的に中断されます。

### Step 5: PRをレビュー＆改善

`self-review` スキル (`@[.agents/skills/self-review]`) とコーディングルール (`@[docs/coding-rules.ja.md]`) を使用し、PRの内容を自動レビューおよび必要に応じて改善コミットを行います。

### Step 6: CIの all pass を確認

PR がオープンされると以下の CI チェックが自動実行されます。
**すべてグリーンになるまでマージしてはいけません。**
**⚠️ 厳格化ルール**: AIエージェントは「ローカルの `make check` が通った」という理由でリモートの CI 確認をスキップ・ショートカットしてマージ (`gh pr merge`) を強行しては**絶対にいけません**。必ず `gh pr checks --watch` 等を用いて GitHub Actions の全てのテスト（macOS, Linux, Windows等）が完了し Success になったことを確認してください。もし `paths-ignore` の影響で CI がスキップ・実行されない場合は、空コミット (`git commit --allow-empty -m "trigger ci" && git push`) 等を作成して強制的に CI を発火させ、リモートテストのパスを確認すること。

- ✅ Lint（`cargo clippy -D warnings`）
- ✅ Coverage（テスト＋カバレッジ）
- ✅ CodeQL Security Scan

CI の完了をターミナルでリアルタイム監視します：

```bash
gh pr checks --watch
```

### Step 7: merge

全チェックが緑で成功していることを確認したら、マージを実行します。

> [!WARNING]
> **Rule Bypass の絶対禁止**: `gh pr merge` に `--admin` 等のフラグを付けて CI チェックを回避してマージすることは**厳格に禁止**されています。必ず全 CI の成功を待ってください。

```bash
gh pr merge --merge --delete-branch
```

### Step 8: リリースの自動実行（PR マージ時）

PR (release/vX.Y.Z -> master) がマージされると、GitHub Actions の **Release** ワークフローが自動的に発火します。

> [!TIP]
> **推奨フロー**: PR マージによりタグ作成・ビルド・配布がすべて連鎖的に実行されます。
> 手動での `gh workflow run` は、ビルドに失敗して特定のプラットフォームだけ再送したい場合などの例外的なケースでのみ使用してください。

### Step 9: リリース状況の監視と事後確認

```bash
# ワークロー実行状況の監視
gh run list --workflow Release --limit 5
gh run watch [RUN_ID] --repo HiroyukiFuruno/KatanA

# リリースページの確認
gh release view vX.Y.Z
```

> [!IMPORTANT]
> GitHub Actions 上で以下のステップが完全自動で実行されます：
>
> 1. **Preflight**: バージョン整合性チェック（Cargo.toml, Info.plist, CHANGELOG）
> 2. **Build（並列）**: macOS (.dmg/.zip) / Linux (.tar.gz) / Windows (.msi/.zip) をそれぞれのネイティブ環境でビルド
> 3. **Publish**: `gh release create` により **タグと GitHub Release を API 経由で同時作成** し、全アーティファクトをアップロード
> 4. **配布**: Homebrew Cask / Linuxbrew Formula / Winget レジストリへの自動公開

### Step 10: OpenSpec のアーカイブ

リリース作業が完了し、`master` に反映された OpenSpec の変更（機能や修正等）がある場合は、`/opsx-archive` ワークフローを用いて対象の `openspec/changes/` ディレクトリをアーカイブへと移動します。

```text
/opsx-archive <change-name>
```

### Step 11: ローカルブランチの掃除

マージとアーカイブが完了したら、役割を終えたローカルブランチを削除して作業を完了します。

```bash
git switch master
git branch -D release/vX.Y.Z
```

---

## 💡 手動リリースの実行（例外用）

自動発火に失敗した場合や、特定のパラメータを指定して再実行したい場合のみ、ローカルから以下のコマンドを実行します。

```bash
# 基本実行
make release VERSION=X.Y.Z

# 既存のリリースがある場合の強制上書き（再実行時）
make release VERSION=X.Y.Z FORCE=1
```

> [!NOTE]
> 手動実行の場合、ワークフローの `preflight` ジョブが `Cargo.toml` のバージョン書き換えと `git push` を自動で行います。`FORCE=1` を指定すると、`gh release create --clobber` が実行され、既存のタグやリリースを上書きしてアセットを再アップロードします。

---

## 💡 トラブルシューティング

**DMG・Info.plist のバージョンズレ**: `package-mac` にてビルド後に `Cargo.toml` からバージョンを抽出して `Info.plist` に強制注入する仕組みを入れているため、再発しません。

**ワークフローが発火しない**: `gh workflow run` には `repo` スコープのトークンが必要です。`gh auth status` でログイン状態を確認してください。

**特定プラットフォームだけ再ビルドしたい場合**: `gh workflow run Release -f version=X.Y.Z -f target=macOS` のように `target` パラメータを指定できます。
