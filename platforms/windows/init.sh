#!/usr/bin/env bash
set -e

echo "🪟 Katana Windows 検証環境 構築アシスタント"
echo "================================================="
echo "Apple Silicon (M1/M2/M3等) 上の macOS で Windows 11 を検証するためには、"
echo "仮想化ソフト『UTM』およびISOダウンローダー『CrystalFetch』のアシストが必要です。"
echo ""

# Homebrewの確認
if ! command -v brew &> /dev/null; then
  echo "❌ Homebrew がインストールされていません。インストールしてから再度実行してください。"
  exit 1
fi

echo "📦 必要な無料ツール (UTM, CrystalFetch) のインストール状態を確認します..."

if brew list --cask utm &>/dev/null || [ -d "/Applications/UTM.app" ]; then
    echo "✅ UTM は既にインストールされています。"
else
    echo "⏳ UTM をインストールしています..."
    brew install --cask utm
fi

if brew list --cask crystalfetch &>/dev/null || [ -d "/Applications/CrystalFetch.app" ]; then
    echo "✅ CrystalFetch は既にインストールされています。"
else
    echo "⏳ CrystalFetch をインストールしています..."
    brew install --cask crystalfetch
fi

echo ""
echo "🎉 ツールの準備が完了しました！"
echo "================================================="
echo "【環境構築ステップ】"
echo "1. 『CrystalFetch』アプリを起動してください。"
echo "2. 「Windows 11 / Apple Silicon」を選択し、ISOイメージをダウンロードしてください。"
echo "3. ダウンロードが終わったら、『UTM』アプリを起動してください。"
echo "4. 『UTM』で新規仮想マシンを作成し、「仮想化(Virtualize) > Windows」を選び、ISOを指定してインストールを進めます。"
echo "5. Windows が起動したら、GitHub Releases から MSI インストーラーをダウンロードし検証を実施してください。"
echo ""
echo "詳細な検証項目については platforms/README.md もしくは platforms/windows/README.md をご参照ください。"

read -p "いますぐ CrystalFetch を起動しますか？ (y/N): " -r
if [[ $REPLY =~ ^[Yy]$ ]]; then
    open -a CrystalFetch
fi
