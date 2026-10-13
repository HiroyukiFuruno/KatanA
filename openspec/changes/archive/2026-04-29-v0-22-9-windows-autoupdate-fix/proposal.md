# Proposal: Windows Auto-Update Mechanism Fix

## Problem Statement

Windows環境でKatanAをインストール後、アプリ内の更新機能が動作しない。

**症状:**

- ダウンロードと「更新」ボタン操作は表面上は完了する
- しかし再起動がかからない、または再起動後もバージョンが更新されない
- 同じバージョンで起動し続ける

## Root Cause Analysis

### Bug 1: 設定タブのアクションハンドラ欠落（アクション吸収버그）

**位置:** `crates/katana-ui/src/settings/tabs/updates.rs` (L61, 67)

設定タブ→アップデートセクションの「Download Update」「Install Update」ボタンが以下のアクションを発火する:

- `AppAction::StartUpdateDownload`
- `AppAction::InstallUpdateAndRestart`

しかし、`dispatch_secondary.rs` / `dispatch_tertiary.rs` のmatch文のいずれにもこれらのアクションのハンドラがない。結果として、これらのアクションは `_ => {}` で完全にサイレント無視される。

**対比: 正常なフロー（モーダルダイアログ経由）**

- モーダルダイアログ → `AppAction::InstallUpdate` → `dispatch_secondary.rs:91` でハンドラ存在
- モーダルダイアログ → `AppAction::ConfirmRelaunch` → `dispatch_tertiary.rs:33` でハンドラ存在

### Bug 2: PowerShellスクリプトのファイルロック競合（Windows再起動失敗）

**位置:** `crates/katana-core/src/update/scripts.rs` (L50-77, Windows向け)

PowerShellスクリプト生成部分で:

```powershell
Start-Sleep -s 2
target = ...         # KatanA.exe パス
extracted = ...      # 新バイナリパス
Move-Item -Force $target $bak -ErrorAction SilentlyContinue
Move-Item -Force $extracted $target -ErrorAction SilentlyContinue
if (Test-Path $target) {
    Start-Process ... $target  # 起動するバイナリ
}
```

**問題:**

- Windows EXEは実行中のプロセスが存在するファイルを移動できない
- `std::process::exit(0)` で親プロセス(KatanA)が終了した直後でも、OSがファイルロックを解放するまでに数秒かかる
- `Start-Sleep -s 2` では不十分な場合がある
- `Move-Item -ErrorAction SilentlyContinue` でエラーが握り潰されるため、失敗しても処理は続行
- `Test-Path $target` は真（旧EXEが依然としてそこにある）→ 旧バイナリで再起動 → **バージョン変わらず**

## Solution Design

### Fix 1: 設定タブのアクションハンドラを追加

`dispatch_secondary.rs` / `dispatch_tertiary.rs` に以下のハンドラを追加:

```rust
// dispatch_secondary.rs または dispatch_tertiary.rs
AppAction::StartUpdateDownload => self.handle_action_install_update(),
AppAction::InstallUpdateAndRestart => {
    if let Some(*prep) = self.pending*relaunch.take() {
        #[cfg(all(not(test), not(coverage)))]
        {
            let * = UpdateInstallerOps::execute*relauncher(_prep);
            std::process::exit(0);
        }
    }
}
```

### Fix 2: PowerShellスクリプトのリトライロジック

改善内容:

1. 親プロセスID（PPIDまたは`$PID`）を`execute_relauncher`からスクリプトへ引数として渡す
2. スクリプト内で `Wait-Process -Id $parentPid -Timeout 30` で親プロセス完全終了を待機
3. `Move-Item` をリトライループで実行（最大5回、1秒間隔）
4. リトライ全失敗時はロールバック＋ユーザー通知（ダイアログ表示）

## Implementation Scope

- **Task 1:** 設定タブアクションハンドラ追加（dispatch_secondary.rs / dispatch_tertiary.rs）
- **Task 2:** PowerShellスクリプトリトライロジック実装（scripts.rs / installer.rs / types.rs）
- **Task 3:** ユーザーレビュー
- **Task 4:** ファイナル検証＆リリース準備

## Files to Modify

- `crates/katana-ui/src/app/action/dispatch_secondary.rs`
- `crates/katana-ui/src/app/action/dispatch_tertiary.rs`
- `crates/katana-core/src/update/scripts.rs`
- `crates/katana-core/src/update/installer.rs`
- `crates/katana-core/src/update/types.rs`（必要に応じて）

## Testing Strategy

1. **ユニットテスト:** `scripts.rs` の PowerShell スクリプト生成部分にリトライロジック検証テストを追加
2. **手動テスト:** Windows環境（実機またはCI）で設定タブ→アップデートセクション→ダウンロード→インストールの一連フローを確認
3. **CI:** `make check-light` / `make check` がパスすることを確認

## Definition of Success

- 設定タブからの「Download Update」→「Install Update」フローが正常に動作する
- 再起動後、新しいバージョンのアプリが立ち上がる
- バージョン番号が更新される
- PowerShellスクリプトはファイルロック状況下でもリトライして成功する
