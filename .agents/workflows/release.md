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


### Step 4: リリースの実行（ワークフロー発火）

PR マージ後、ローカルから以下のコマンドでリリースワークフローを発火します。

```bash
gh workflow run Release -f version=X.Y.Z
```

> [!IMPORTANT]
> このコマンドにより、GitHub Actions 上で以下のステップが完全自動で実行されます：
>
> 1. **Preflight**: `Cargo.toml`, `Info.plist` のバージョンを in-place で書き換え → CHANGELOG・OpenSpec の検証
> 2. **Build（並列）**: macOS (.dmg/.zip) / Linux (.tar.gz) / Windows (.msi/.zip) をそれぞれのネイティブ環境でビルド
> 3. **Publish**: `gh release create` により **タグと GitHub Release を API 経由で同時作成** し、全アーティファクトをアップロード
> 4. **配布**: Homebrew Cask / Linuxbrew Formula / Winget レジストリへの自動公開

> [!NOTE]
> ローカルでの `make release` は不要です。`Cargo.toml` のバージョンは master 上では開発版のまま維持されます。
> リリース用のバージョン書き換えは CI 上の各ビルドジョブ内で in-place に行われ、コミットされません。

### Step 5: リリース後確認

```bash
# ワークフロー実行状況の監視
gh run watch --repo HiroyukiFuruno/KatanA

# リリースページの確認
gh release view vX.Y.Z

# リリースブランチの削除（ローカル・リモート）
git branch -d release/vX.Y.Z
git push origin --delete release/vX.Y.Z
```

---

## 💡 トラブルシューティング

**DMG・Info.plist のバージョンズレ**: `package-mac` にてビルド後に `Cargo.toml` からバージョンを抽出して `Info.plist` に強制注入する仕組みを入れているため、再発しません。

**ワークフローが発火しない**: `gh workflow run` には `repo` スコープのトークンが必要です。`gh auth status` でログイン状態を確認してください。

**特定プラットフォームだけ再ビルドしたい場合**: `gh workflow run Release -f version=X.Y.Z -f target=macOS` のように `target` パラメータを指定できます。
