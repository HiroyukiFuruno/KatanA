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

# abcユーザーでHomebrewを非対話形式でインストール
docker compose exec -u abc ubuntu-webtop bash -c 'NONINTERACTIVE=1 bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"' || true
docker compose exec -u abc ubuntu-webtop bash -c 'grep -qxF "eval \"\$(/home/linuxbrew/.linuxbrew/bin/brew shellenv bash)\"" ~/.bashrc || echo "eval \"\$(/home/linuxbrew/.linuxbrew/bin/brew shellenv bash)\"" >> ~/.bashrc' || true
docker compose exec -u abc ubuntu-webtop bash -c 'sudo apt-get update && sudo apt-get install -y build-essential' || true

echo "ブラウザで以下のURLを開いて検証を開始してください:"
echo ""
echo "    http://localhost:3000/"
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
    open http://localhost:3000/
fi
