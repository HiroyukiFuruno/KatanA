#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "🚀 Katana Linux 検証環境 (Ubuntu WebTop) を起動します..."

cd "$DIR"
docker compose up -d

echo "✅ コンテナが起動しました！"

echo "📦 開発に必要なパッケージ (Homebrew) をバックグラウンドでインストールしています..."
# コンテナが立ち上がるまで少し待つ
sleep 5

# コンテナ内で必要なパッケージをバックグラウンド（-d）でインストールする
# 先に build-essential を入れ、その後 Homebrew を導入します
docker compose exec -d -u abc ubuntu-webtop bash -c '
    sudo apt-get update
    sudo apt-get install -y build-essential curl git
    NONINTERACTIVE=1 bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" || true
    grep -qxF "eval \"\$(/home/linuxbrew/.linuxbrew/bin/brew shellenv bash)\"" ~/.bashrc || echo "eval \"\$(/home/linuxbrew/.linuxbrew/bin/brew shellenv bash)\"" >> ~/.bashrc
    echo "setup complete" > /tmp/homebrew_setup_done
'
echo "ブラウザで以下のURLを開いて検証を開始してください:"
echo ""
echo "    https://localhost:3001/"
echo ""
echo "※ 初回は自己署名証明書の警告が出るため、ブラウザで許可してください。"
echo ""
echo "【検証手順】"
echo "1. ブラウザ内のデスクトップ環境で、ターミナルを開く"
echo "2. wget 等で Katana の .tar.gz または .deb ファイルをダウンロードする"
echo "   例: wget https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.18.0/KatanA-linux-x86_64.tar.gz"
echo "3. 展開して実行、クラッシュしないか・フォントが崩れないか確認"
echo "※ ターミナルで brew コマンドも利用可能です。"
echo ""

# Macの場合は自動でブラウザを開く
if command -v open &> /dev/null; then
    echo "⏳ 3秒後にブラウザを自動で開きます..."
    sleep 3
    open https://localhost:3001/
fi
