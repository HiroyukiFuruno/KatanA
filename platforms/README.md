# Katana Platform Verification

このディレクトリ（`platforms/`）には、macOS (Apple Silicon) 上で KatanA の Windows および Linux 版を検証するための構築自動化ツールやガイドが格納されています。

クロスプラットフォーム検証は手作業が多くなりがちですが、このツールキットを利用することで可能な限り敷居を下げることができます。

## ディレクトリ構成

- `linux/` : Docker を用いた Ubuntu (XFCE デスクトップ) の一発起動ツール群
- `windows/` : UTM と CrystalFetch を用いた Windows 11 ARM 環境の構築支援スクリプト群

## 使い方

検証を行いたいプラットフォームに応じたスクリプトを実行してください。

### Linux 環境の構築と検証

```bash
# 依存: docker, docker-compose
./platforms/linux/init.sh
```

上記を実行すると、ブラウザ上で操作可能な Ubuntu デスクトップが起動します。
アクセス先は `<http://localhost:3000/`> です。
その後、ブラウザ内のターミナルからリリースアセット (`KatanA-linux-x86_64.tar.gz` または `deb`) をダウンロードし、動作確認を行います。

### Windows 環境の構築と検証

```bash
# 依存: Homebrew
./platforms/windows/init.sh
```

Windows環境は完全自動化ができないため、スクリプトが仮想マシン作成用アプリ（UTM）とISOダウンローダー（CrystalFetch）をインストールし、構築のガイドを表示します。
詳細な手動構築手順については、[windows/README.md](./windows/README.md) を参照してください。

## 検証チェックリスト

各OSで環境が立ち上がったら、以下のスモークテストを実施してください。

1. **インストール可否**: MSI / DEB / TAR.GZ からアプリが正常に導入・展開できるか。
2. **起動確認**: アイコン（またはコマンド）からクラッシュせずに起動するか。
3. **ワークスペース**: 既存のフォルダをワークスペースとして開けるか。
4. **Markdown描画**: 期待通りにテキストやプレビューが表示されるか。フォント等の崩れはないか。
5. **IME入力 (重要)**: 日本語入力が正常に行えるか。

これらをクリアすれば、リリース判定「合格」となります。
