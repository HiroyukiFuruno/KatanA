# v0.22.14 自動アップデート検証手順

## Linux 自動アップデート（Linuxbrew 既存ユーザー自己修復）

画面上では、更新ダイアログに「アップデート＆再起動」ボタンが表示される。このボタンを 1 回クリックするだけで、ダウンロード、差し替え、再起動まで完了することを確認する。

前提:

- 事前バージョン: `v0.22.13`
- 目標バージョン: `v0.22.14`
- 事前インストール状態: Linuxbrew の旧 formula と同じく、実行中のファイル名が `katana-desktop`
- リリースアセット: `KatanA-linux-x86_64.tar.gz` の直下に `KatanA` が含まれる

実行手順:

1. Linuxbrew 旧 formula 相当の状態として、`KatanA` バイナリを `katana-desktop` 名で配置する。
2. `katana-desktop` から v0.22.13 を起動する。
3. 設定画面から更新確認を行う。
4. 更新ダイアログの「アップデート＆再起動」を 1 回だけクリックする。
5. 再起動後も、同じ `katana-desktop` 実行ファイルから v0.22.14 が起動していることを確認する。

期待値:

- ユーザーに `brew reinstall`、ターミナル操作、ファイルリネーム、再ダウンロードを求めない。
- 展開元が `KatanA`、展開先が `katana-desktop` でも更新が成功する。
- `katana-desktop` 名の実行ファイルは維持され、既存ユーザーの起動導線を壊さない。

## Linux 自動アップデート（手動ダウンロード）

前提:

- 実行中のファイル名: `KatanA`

実行手順:

1. `KatanA-linux-x86_64.tar.gz` を展開する。
2. `./KatanA` で v0.22.13 を起動する。
3. 更新ダイアログの「アップデート＆再起動」を 1 回だけクリックする。
4. 再起動後に v0.22.14 が起動していることを確認する。

期待値:

- Linuxbrew 既存ユーザー向けの自己修復により、通常の `KatanA` 名起動が退行しない。

## Windows 自動アップデート

前提:

- 事前バージョン: `v0.22.13`
- 目標バージョン: `v0.22.14`
- Portable ZIP の実行ファイル名: `KatanA.exe`

実行手順:

1. Portable ZIP または MSI の v0.22.13 起動環境を準備する。
2. 設定画面から更新確認を行う。
3. 更新ダイアログの「インストールして再起動（Install and Restart）」を 1 回クリックする。
4. 再起動後のバージョンが `v0.22.14` であることを確認する。

期待値:

- 実行中のファイル名が `KatanA.exe` と異なる環境でも、アーカイブ内の `KatanA.exe` を展開元として扱える。
- 失敗時は既存のロールバックと `%LOCALAPPDATA%\KatanA\update.log` が維持される。

## macOS 自動アップデート

前提:

- アプリバンドル名: `KatanA Desktop.app`

実行手順:

1. v0.22.13 の `KatanA Desktop.app` を起動する。
2. 更新ダイアログの「インストールして再起動（Install and Restart）」を 1 回クリックする。
3. 再起動後のバージョンが `v0.22.14` であることを確認する。

期待値:

- `.app` バンドル構造の検証が維持される。
- 防御用 fallback のバンドル名も `KatanA Desktop.app` と一致している。

## リリースアセット契約

リリース前に、公開済み直近アセットまたは生成済み成果物で次を確認する。

```bash
bash scripts/dev/inspect-release-asset.sh v0.22.13 linux
```

期待値:

- `KatanA-linux-x86_64.tar.gz` の直下に `KatanA` が存在する。
- `checksums.txt` の SHA-256 と実アセットの SHA-256 が一致する。
- Linuxbrew formula は新規インストール時に `KatanA` を別名へリネームしない。

## 手元自動テスト

```bash
cargo test -p katana-core update::
cargo test -p katana-core markdown::diagram_backend
./scripts/openspec validate v0-22-14-kcf-theme-propagation --strict
just check-local
```

確認内容:

- `resolve_extracted_file_uses_single_executable_fallback` で、展開元 `KatanA` が無い場合でも唯一の実行可能ファイルを採用できることを確認する。
- `resolve_extracted_file_prefers_linux_asset_name` で、通常の Linux アセット名 `KatanA` を一次候補として扱うことを確認する。
- `linux_script_replaces_target_even_when_asset_name_differs` で、展開元 `KatanA` と展開先 `katana-desktop` の差し替えコマンドが維持されることを確認する。
- `scripts/screenshot/examples/v0-22-14-light-diagrams.json` で、明るいテーマ（light theme）の Mermaid / Draw.io プレビューが暗い配色へ戻らないことを確認する。
