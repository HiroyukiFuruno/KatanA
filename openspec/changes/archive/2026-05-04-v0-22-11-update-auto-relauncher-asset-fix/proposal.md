## Why

v0.22.10 リリース直後、自動アップデート経路で 2 件の致命的回帰が確認された。両方とも「最新版を入手できない／更新後も古い版が起動する」というユーザー体験になり、配布サイクル全体の信頼性を損なう。v0.22.11 は OpenSpec として hotfix 1 本に絞って解決する。

### 不具合 1: Linux 版で更新確認が `http status: 404` で必ず失敗する

実機で以下を確認した。

- `https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.10/KatanA-linux-x86_64.zip` → **404**
- `https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.10/KatanA-linux-x86_64.tar.gz` → **302（実体あり）**

原因は `crates/katana-core/src/update/version.rs:40,125` の Linux アセット名ハードコード `KatanA-linux-x86_64.zip` と、`.github/workflows/build-and-release.yml:249` で publish される実アセット名 `KatanA-linux-x86_64.tar.gz` のドリフト。`afc114b2 (feat: Linux 向けに .tar.gz 形式の配布)` で publish 側だけ tar.gz へ移行し、updater 側のアセット名が取り残された。`crates/katana-core/src/update/installer.rs:16-20` で URL 末尾が `.tar.gz` の場合の extraction 経路はすでに実装済みのため、updater のアセット名 1 行を直せば疎通する。

ダイアログ表示が「更新の確認に失敗しました」になるのは、`crates/katana-ui/src/app/update.rs:114-122,159-166` でダウンロード失敗エラーも check エラーと同じ `state.update.check_error` に投入されるため。本質はダウンロード時の 404 であり、UI 文言は別 change で扱う。

### 不具合 2: Windows 版で in-app update 後に再起動しない／旧版のまま起動する

`crates/katana-core/src/update/scripts.rs:50-93` の PowerShell relauncher の成功判定が壊れている。具体的には:

1. 旧 exe を `.bak` に退避する `Move-Item -Force $target $bak -ErrorAction SilentlyContinue` がファイルロック等で黙って失敗しても処理が継続する。
2. その後の `Move-Item -Force $extracted $target -ErrorAction SilentlyContinue` も target がすでに存在するため失敗する。
3. 成功判定の `Test-Path $target` は **古い exe** を見つけて true を返す。
4. 結果として `$success = $true` のまま `Start-Process -WindowStyle Hidden $target` で **古い exe** が起動する。

ユーザー視点では「再起動しなかった／立ち上げ直しても旧版」になる。v0.22.9 の `Wait-Process` とリトライは正しい方向だが、成功検知のシグナルが「ファイル存在」一点のため、退避失敗を検出できない。

## What Changes

- Linux アセット名を `KatanA-linux-x86_64.zip` から `KatanA-linux-x86_64.tar.gz` へ修正し、`crates/katana-core/src/update/version.rs` の `check_for_updates` / `check_for_updates_simple` 双方を整合させる。可能なら 2 重定義の `#[cfg(target_os = "linux")]` ブロックを 1 箇所に集約する。
- Windows relauncher の成功判定を「ファイル存在」から「`extracted` が消えた かつ `target` の `LastWriteTime` が relauncher 起動時刻より新しい」へ変更する。
- 退避 Move-Item の `-ErrorAction SilentlyContinue` を外し、退避失敗を成功判定へ反映できるようにする。
- 失敗時の旧版再起動（二重起動）を停止し、ロールバックのみを行う。
- 失敗時の reason を `$env:LOCALAPPDATA\KatanA\update.log` に追記し、今後同種障害の再現可能性を残す。
- `crates/katana-core/src/update/scripts.rs` の Windows test に新条件をアサートする項目を追加する。
- 自動アップデートの E2E 検証手順を `docs/release/update-verification.md`（仮）として整備し、Linux と Windows のいずれも次回リリース前に通過させる。
- `docs/CHANGELOG.en.md` / `docs/CHANGELOG.ja.md` に Linux 404 / Windows old-binary 双方の修正を同期記載する。
- リリースノートに「v0.22.10 → v0.22.11 への乗り換えは Linux で 404、Windows で旧版残留が起きるため、ユーザーは手動で zip / tar.gz をダウンロードして上書きする必要がある」と注記する。

## Capabilities

### New Capabilities

- `auto-update`: KatanA Desktop の in-app 自動アップデート機構（更新検知・ダウンロード・展開・差し替え・再起動）の振る舞いを扱う。本 change で初めて OpenSpec 上の capability として登録する。

### Modified Capabilities

なし（`auto-update` は新設）。

## Impact

- `crates/katana-core/src/update/version.rs`
- `crates/katana-core/src/update/scripts.rs`
- `crates/katana-core/src/update/installer.rs`（必要に応じて log 追加経路の整合）
- `crates/katana-core/src/update/tests.rs`（URL 構築の Linux ケース追加）
- `.github/workflows/build-and-release.yml`（アセット名はこちらが正。updater 側を合わせる前提を文書化）
- `docs/release/update-verification.md`（新規、E2E 検証手順）
- `docs/CHANGELOG.en.md` / `docs/CHANGELOG.ja.md`
- `openspec/specs/auto-update/spec.md`（archive 後に追加される新規 capability spec）

## Out of Scope

- アップデート機構の汎用 interface 化（`katana-renderer` 分離と同じ理由で別 change）。
- macOS 経路の改修（症状なし）。
- Windows MSI installer 経由の updater 対応（本 change は portable zip 経路のみ）。
- 「更新の確認」UI 文言とダウンロード失敗の分離表示（別 change で扱う）。
