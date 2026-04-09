#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "🛑 Katana Linux 検証環境 (Ubuntu WebTop) を終了・削除します..."

cd "$DIR"
docker compose -f compose-webtop.yml down

echo "✅ コンテナとそのネットワークを正常に削除しました！"
