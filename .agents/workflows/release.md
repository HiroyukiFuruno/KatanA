---
description: katanaプロジェクトの公式リリース手順。release/vX.Y.Zブランチでのリリース準備からPR→masterマージ後のmake releaseまでのフローを定義。
---

# KatanA リリースワークフロー

本プロジェクト (KatanA) におけるバージョンリリースの公式手順です。

## ⚠️ 最重要ルール

- **リリースは `master` ブランチからのみ実行可能**: `release.sh` は現在のブランチが `master` でない場合は即座に中断します。
- **master への直接 push は禁止**: 必ず `release/vX.Y.Z` ブランチを作成し、PR経由で master にマージしてください。
- **PR の CI チェック（Lint / Coverage / CodeQL）が全通過していること**: ブランチ保護により、これらがパスしないと master へのマージ自体がブロックされます。
- **手動でのタグ打ちは禁止**: `git tag vX.Y.Z` を手動で打ってプッシュすることは固く禁止します。
- **手動での Cargo.toml バージョン変更は非推奨**: 必ず `make release` を通じて更新してください。

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


### Step 4: リリースの実行（`master` 上で）

PR マージ後、**必ず master に切り替えてから** 実行してください。

```bash
git checkout master
git pull origin master
make release VERSION=X.Y.Z FORCE=1
```

> [!IMPORTANT]
> コマンド実行時に内部で以下のステップが自動実行されます：
>
> 1. **現在のブランチが `master` であることを検証**（master 以外は即中断）
> 2. GPG署名キーの事前検証（GitHub API）
> 3. `Cargo.toml`, `Cargo.lock`, `crates/*/Cargo.toml`, `crates/katana-ui/Info.plist` のバージョン自動更新
> 4. 品質ゲートの実行（`make check`）
> 5. `CHANGELOG.md` など更新対象のステージングとコミット（`chore: vX.Y.Z リリース準備`）
> 6. Git 注釈付き署名タグの作成とリモートへのプッシュ
> 7. macOS DMG をローカルビルドして GitHub Release に自動アップロード
> 8. GitHub Actions 発火 → Linux / Windows ビルド → Linuxbrew / Winget 自動公開

### Step 5: リリース後確認

```bash
# リリースブランチの削除（ローカル・リモート）
git branch -d release/vX.Y.Z
git push origin --delete release/vX.Y.Z

# タグとリリースページの確認
gh release view vX.Y.Z

# GitHub Actions の完了を監視（Linux/Windows ビルド）
gh run watch --repo HiroyukiFuruno/KatanA
```

---

## 💡 トラブルシューティング

**DMG・Info.plist のバージョンズレ**: `package-mac` にてビルド後に `Cargo.toml` からバージョンを抽出して `Info.plist` に強制注入する仕組みを入れているため、再発しません。

**「master ブランチ以外でのリリースをブロック」**: `release.sh` が `git rev-parse --abbrev-ref HEAD` でブランチを確認し、`master` 以外であれば即座にエラー終了します。
