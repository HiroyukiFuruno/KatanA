#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "🚀 Katana Linux 検証環境 (Ubuntu WebTop) を起動します..."

cd "$DIR"
docker compose up -d

echo "✅ コンテナが起動しました！"

echo "📦 開発に必要なツール (Homebrew / OpenJDK 25) をセットアップしています..."
echo "   初回は数分かかることがあります。"
# コンテナが立ち上がるまで少し待つ
sleep 5

CONTAINER_ID="$(docker compose ps -q ubuntu-webtop)"
docker cp "$DIR/provision.sh" "${CONTAINER_ID}:/tmp/katana-linux-provision.sh"
docker compose exec -T ubuntu-webtop bash -lc '
    chmod +x /tmp/katana-linux-provision.sh
    chown abc:abc /tmp/katana-linux-provision.sh
'
docker compose exec -T -u abc ubuntu-webtop bash -lc '
    /tmp/katana-linux-provision.sh
'

echo "✅ ツールのセットアップが完了しました！"
echo "ブラウザで以下のURLを開いて検証を開始してください:"
echo ""
echo "    http://localhost:3000/"
echo ""
echo "【検証手順】"
echo "1. ブラウザ内のデスクトップ環境で、ターミナルを開く"
echo "2. cd /config/workspace/katana"
echo "3. make run-release"
echo "4. ローカルの未リリース版が起動するので、クラッシュしないか・フォントが崩れないか確認"
echo "※ ターミナルで brew / java / rustc / cargo が利用可能です。"
echo ""

# Macの場合は自動でブラウザを開く
if command -v open &> /dev/null; then
    echo "⏳ 3秒後にブラウザを自動で開きます..."
    sleep 3
    open http://localhost:3000/
fi
