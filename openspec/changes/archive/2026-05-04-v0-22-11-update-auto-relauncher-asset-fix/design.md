# Design: v0.22.11 Auto-Update Hotfix

## 1. Linux アセット名の整合化

### 現状

`crates/katana-core/src/update/version.rs:33-40` および `:118-125` で、プラットフォーム別アセット名を `const ASSET_NAME` として 2 箇所に定義している。Linux は両方とも `KatanA-linux-x86_64.zip`。

`.github/workflows/build-and-release.yml:249` で publish しているのは `KatanA-linux-x86_64.tar.gz`。GitHub Release の実体も tar.gz。

### 設計方針

1. `version.rs` 内の Linux ASSET_NAME を `KatanA-linux-x86_64.tar.gz` に修正する（2 箇所）。
2. 重複を避けるため、ASSET_NAME 定数を 1 箇所に集約する別関数 `platform_asset_name()` を切り出す。`#[cfg(target_os = "...")]` 分岐を 1 関数に閉じ込め、`check_for_updates` / `check_for_updates_simple` の両方から参照する。
3. `installer.rs:16-20` の archive 拡張子分岐は **手を入れない**（すでに `.tar.gz` 対応済み）。
4. `download.rs::extract_update` の tar.gz 経路も既存（`afc114b2`）。

### 不変条件

- macOS: `KatanA-macOS.zip`（既存維持）
- Windows: `KatanA-windows-x86_64.zip`（既存維持）
- Linux: `KatanA-linux-x86_64.tar.gz`（修正）

### テスト

- `cargo test -p katana-core update::tests::*linux*` で URL 末尾が `tar.gz` で終わることをアサートする unit を追加する。
- 既存の Windows / macOS テストは変更しない。

---

## 2. Windows relauncher の成功判定アルゴリズム

### 現状の判定ロジック（壊れている）

```powershell
$retryCount = 0;
$success = $false;
while ($retryCount -lt 5) {
    Move-Item -Force $extracted $target -ErrorAction SilentlyContinue;
    if (Test-Path $target) {                # ← 古い exe でも true
        $success = $true;
        break;
    }
    $retryCount++;
    Start-Sleep -s 1;
}
```

`$target` には `.bak` 退避が成功していなければ古い exe が残るため、`Test-Path` は無条件で true になる。退避用 Move-Item にも `-ErrorAction SilentlyContinue` が付いており、退避失敗を呼び出し側で観測できない。

### 新設計

成功シグナルを 2 つの独立な観測点で確認する。

1. **退避フェーズ**: `Move-Item $target $bak` を `-ErrorAction Stop` 付きで try/catch する。`$target` が存在せず `$bak` も作られないケース（target が消えた）を除き、catch に入ったら `$evacuated = $false` を確定させる。
2. **差し替えフェーズ**: relauncher 起動時刻 `$startedAt = Get-Date` を保持しておき、Move-Item 後に `(Get-Item $target -ErrorAction SilentlyContinue).LastWriteTime -gt $startedAt` かつ `-not (Test-Path $extracted)` の両方が真であれば `$replaced = $true` とする。
3. **総合判定**: `$success = $evacuated -and $replaced`。
4. **失敗時の挙動変更**:
   - `$bak` を `$target` へ戻す（既存挙動）。
   - **`Start-Process $target` を呼ばない**。失敗時に走っているのは旧版で、ユーザーが既に旧版のセッションを開いている可能性が高い。relauncher が更に旧版を起動すると二重起動になり、設定ファイル衝突などを誘発する。
   - エラーダイアログは既存通り `MessageBox` で出す。
5. **観測ログ**:
   - `$logDir = Join-Path $env:LOCALAPPDATA 'KatanA'`
   - `$logPath = Join-Path $logDir 'update.log'`
   - `Add-Content -Path $logPath -Value "$(Get-Date -Format o) <phase> <result> <reason>"` を退避・差し替え・成功・失敗の 4 点で追記する。
   - PII を含めない。記録するのは時刻、フェーズ、`$target` パス（exe フルパス）、例外メッセージのみ。

### 擬似コード

```powershell
param($parentPid)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$target    = '{target}'
$extracted = '{extracted}'
$bak       = "$target.bak"
$startedAt = Get-Date
$logDir    = Join-Path $env:LOCALAPPDATA 'KatanA'
$logPath   = Join-Path $logDir 'update.log'

function Write-UpdateLog($phase, $result, $reason) {
    if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Force -Path $logDir | Out-Null }
    Add-Content -Path $logPath -Value ("{0} {1} {2} {3} {4}" -f (Get-Date -Format o), $phase, $result, $target, $reason)
}

if ($parentPid) { Wait-Process -Id $parentPid -Timeout 30 -ErrorAction SilentlyContinue }

if (Test-Path $bak) { Remove-Item -Force $bak -ErrorAction SilentlyContinue }

$evacuated = $false
try {
    if (Test-Path $target) {
        Move-Item -Force $target $bak
    }
    $evacuated = $true
    Write-UpdateLog 'evacuate' 'ok' ''
} catch {
    Write-UpdateLog 'evacuate' 'fail' $_.Exception.Message
}

$replaced = $false
if ($evacuated) {
    for ($i = 0; $i -lt 5; $i++) {
        try {
            Move-Item -Force $extracted $target
            $info = Get-Item $target -ErrorAction SilentlyContinue
            if ($info -and $info.LastWriteTime -gt $startedAt -and -not (Test-Path $extracted)) {
                $replaced = $true
                Write-UpdateLog 'replace' 'ok' ''
                break
            }
        } catch {
            Write-UpdateLog 'replace' 'retry' $_.Exception.Message
        }
        Start-Sleep -s 1
    }
}

if ($evacuated -and $replaced) {
    Start-Process -WindowStyle Hidden $target
    Write-UpdateLog 'launch' 'ok' ''
} else {
    if (Test-Path $bak) { Move-Item -Force $bak $target -ErrorAction SilentlyContinue }
    Write-UpdateLog 'rollback' 'done' ''
    Add-Type -AssemblyName PresentationFramework
    [System.Windows.MessageBox]::Show('Could not complete the application update. The original version has been restored.', 'Update Failed', 'OK', 'Error') | Out-Null
}

Remove-Item -Recurse -Force '{temp_dir}' -ErrorAction SilentlyContinue
```

### Test 追加

`crates/katana-core/src/update/scripts.rs` の `tests` モジュール（`:131-163`）に以下のアサートを追加する。

- `assert!(content.contains("Get-Date"))`（起動時刻を保持していること）
- `assert!(content.contains(".LastWriteTime -gt $startedAt"))`（更新時刻判定）
- `assert!(content.contains("-not (Test-Path $extracted)"))`（extracted 消滅判定）
- `assert!(content.contains("update.log"))`（ログ追記）
- `assert!(content.contains("Move-Item -Force $target $bak"))` かつ `assert!(!content.contains("Move-Item -Force $target $bak -ErrorAction SilentlyContinue"))`（退避から SilentlyContinue を外したことの確認）

### Negative path の手動再現

target.exe を読み取り専用化または別プロセスで開いてロックした状態で `Install and Relaunch` を呼ぶ。期待挙動:

- 旧版 v0.22.10 が現プロセスとしては停止し、`.bak` から target に戻り、新プロセスは起動しない。
- `update.log` に `evacuate fail ...` または `replace retry ...` が記録されている。
- ユーザーには Update Failed のメッセージボックスが出る。

---

## 3. 互換性とロールアウト

### v0.22.10 → v0.22.11 への乗り換え

本 change の修正は **新しい relauncher / 新しいアセット名を持つ v0.22.11 以降から** 効く。v0.22.10 のクライアントは:

- Linux: `.zip` を取りに行って 404 で必ず失敗する。
- Windows: 旧 relauncher のままなので、in-app update が壊れている可能性が残る。

このため v0.22.11 のリリースノートには次の手動手順を記載する。

- Linux: `https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.11/KatanA-linux-x86_64.tar.gz` をダウンロードし、既存 `KatanA` バイナリへ手動上書き。
- Windows: portable zip `KatanA-windows-x86_64.zip` を取得して既存ディレクトリに展開上書き、または MSI を再インストール。

v0.22.11 以降のクライアントは、自動アップデートで v0.22.12 以降を取得できる。

### 観測

`update.log` は OS のロケールやファイルシステム、UAC 制約に依存しない `LOCALAPPDATA` 配下に置く。サイズ制御は本 change では行わず、ファイルがアプリ稼働で巨大化しないよう、最大 1MB ローテーションは後続 change（v0.22.12 以降）で扱う。

### Out of Scope

- macOS の relauncher（症状なし）。
- Linux の relauncher（症状はアセット名のみ。スクリプトは触らない）。
- 「更新の確認に失敗しました」というラベルを download 失敗で出さない UI 修正。
