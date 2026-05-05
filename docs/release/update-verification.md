# v0.22.11 自動アップデート検証手順

## Linux 自動アップデート（成功パス）

前提:
- 事前バージョン: `v0.22.10`
- 目標バージョン: `v0.22.11`

実行手順:

1. `just linux-up` で v0.22.10 環境を起動する。
2. 自動アップデート実行を起動し、更新後のバイナリバージョンを確認する。
3. 下記コマンドで v0.22.11 tar.gz URL を事前確認する。

```bash
curl -sI -L "https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.10/KatanA-linux-x86_64.tar.gz"
```

期待値:
- HTTP ステータスが `200` または `302`。

実行結果（2026-05-04）:

```text
HTTP/2 302
```

## Windows 成功パス

1. v0.22.11 portable zip で新規インストール済みの起動環境を準備する。
2. 設定画面から Update → Install and Relaunch を実行する。
3. 起動後のバージョンを確認し、`v0.22.11` を確認する。

補足:
- 画面録画は Windows VM 環境の手元手順で実行し、`scripts/screenshot/output/v0-22-11-review/` に保存する。

## Windows 失敗パス

1. `target.exe` をロックする構成（読み取り専用化や別プロセス保持）を作る。
2. Install and Relaunch を実行する。
3. ロールバックと `update.log` の `evacuate` / `replace` / `rollback` 記録を確認する。

補足:
- 画面録画は Windows VM 環境で実行し、`scripts/screenshot/output/v0-22-11-review/` に保存する。

## 追加チェック（CI）

- リリースブランチPRでは、`KatanA-linux-x86_64.tar.gz` が存在することを `curl -sI` で確認する。
