#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "🚀 Katana Linux 検証環境 (Ubuntu WebTop) を起動します..."

cd "$DIR"
docker compose -f compose-webtop.yml up -d

echo "✅ コンテナが起動しました！"
echo "ブラウザで以下のURLを開いて検証を開始してください:"
echo ""
echo "    http://localhost:3000/"
echo ""
echo "【検証手順】"
echo "1. ブラウザ内のデスクトップ環境で、ターミナルを開く"
echo "2. wget 等で Katana の .tar.gz または .deb ファイルをダウンロードする"
echo "   例: wget https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.18.0/KatanA-linux-x86_64.tar.gz"
echo "3. 展開して実行、クラッシュしないか・フォントが崩れないか確認"
echo ""

# Macの場合は自動でブラウザを開く
if command -v open &> /dev/null; then
    echo "⏳ 3秒後にブラウザを自動で開きます..."
    sleep 3
    open http://localhost:3000/
fi
