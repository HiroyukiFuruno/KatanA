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

1. **CHANGELOG の更新**: `changelog-writing` スキルを実行し、今回のバージョンの変更履歴を `CHANGELOG.md` / `CHANGELOG.ja.md` に記載してください。
2. **品質ゲートの事前確認**: `make check` がローカルで通ることを確認してください。
3. **OpenSpec の統合確認 (⚠️ 必須)**: 対象バージョンの OpenSpec Base Feature Branch がすべて `master` へマージ済みであることを確認してください。

   ```bash
   # 未マージのfeatureブランチがないことを確認
   git branch -a | grep -v master | grep -v HEAD | grep -v release/
   ```

### Step 3: PR の作成と CI 確認・マージ

```bash
git push origin release/vX.Y.Z

# PRを作成（base: master）
gh pr create --base master --head release/vX.Y.Z \
  --title "release: v[Target Version]" \
  --body "CHANGELOG 更新を含むリリース準備ブランチ"
```

PR がオープンされると以下の CI チェックが自動実行されます。
**すべてグリーンになるまでマージしてはいけません。**

- ✅ Lint（`cargo clippy -D warnings`）
- ✅ Coverage（テスト＋カバレッジ）
- ✅ CodeQL Security Scan

CI の完了をターミナルでリアルタイム監視します：

```bash
gh run watch
```

全チェックが緑になったら、マージを実行します：

```bash
gh pr merge --merge --delete-branch
```


### Step 4: リリースの自動実行（PR マージ時）

PR (release/vX.Y.Z -> master) がマージされると、GitHub Actions の **Release** ワークフローが自動的に発火します。

> [!TIP]
> **推奨フロー**: PR マージによりタグ作成・ビルド・配布がすべて連鎖的に実行されます。
> 手動での `gh workflow run` は、ビルドに失敗して特定のプラットフォームだけ再送したい場合などの例外的なケースでのみ使用してください。

### Step 5: リリース状況の監視と事後確認

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
