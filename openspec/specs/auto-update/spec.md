# auto-update Specification

## Purpose
TBD - created by archiving change v0-22-11-update-auto-relauncher-asset-fix. Update Purpose after archive.
## Requirements
### Requirement: Update discovery must survive public API rate limits

システムは、GitHub REST API の未認証 rate limit に到達した場合でも、認証情報や外部サービスをユーザーへ要求せず、GitHub の公開リリースページから最新版を特定して更新確認を継続しなければならない（MUST）。

#### Scenario: Latest release API returns a rate-limit status

- **WHEN** 最新リリース API がステータス 403 または 429 を返す
- **THEN** システムは公開された `/releases/latest` の redirect をたどる
- **THEN** 最終 URL の `/releases/tag/{version}` から SemVer tag と release URL を取得する
- **THEN** fallback が成功した場合は更新サーバーのステータスエラーを表示しない
- **THEN** proxy 環境変数が不正でも、到達済み API の rate-limit 応答を proxy エラーで上書きしない

### Requirement: Update download URL must use the asset name actually published by the release pipeline

システムは、自動アップデート時のアセットダウンロード URL を、`.github/workflows/build-and-release.yml` が GitHub Release に publish しているアセット名と完全に一致させなければならない（MUST）。

#### Scenario: Linux client downloads tar.gz, not zip

- **WHEN** Linux 上で自動アップデートを実行する
- **THEN** システムは `KatanA-linux-x86_64.tar.gz` を `https://github.com/HiroyukiFuruno/KatanA/releases/download/{tag}/KatanA-linux-x86_64.tar.gz` から取得する
- **THEN** システムは `KatanA-linux-x86_64.zip` を要求しない
- **THEN** システムは `installer.rs` の archive 拡張子分岐で `.tar.gz` 経路を選択し、tar 展開を行う

#### Scenario: macOS and Windows asset names remain stable

- **WHEN** macOS または Windows 上で自動アップデートを実行する
- **THEN** macOS は `KatanA-macOS.zip` を取得する
- **THEN** Windows は `KatanA-windows-x86_64.zip` を取得する
- **THEN** これらのアセット名は本 change では変更しない

#### Scenario: Asset name is defined once per platform

- **WHEN** updater が download URL を組み立てる
- **THEN** プラットフォーム別アセット名は単一の関数または定数で定義される
- **THEN** `check_for_updates` と `check_for_updates_simple` は同じソースを参照する

### Requirement: Windows relauncher must confirm both evacuation and replacement before launching the new binary

システムは、Windows での自動アップデート完了判定を、退避（旧 exe を `.bak` へ移動）と差し替え（新 exe を target へ移動）の両方が観測可能なシグナルで成功したときに限り、新 exe を起動しなければならない（MUST）。

#### Scenario: Successful update launches the new binary only

- **WHEN** 退避 Move-Item が例外を投げず完了し、かつ差し替え後の `target` の `LastWriteTime` が relauncher 起動時刻より新しく、`extracted` ファイルが消滅している
- **THEN** システムは新 exe を `Start-Process -WindowStyle Hidden` で起動する
- **THEN** システムは `update.log` に `evacuate ok` と `replace ok` と `launch ok` を記録する

#### Scenario: Silent evacuation failure must surface as failure

- **WHEN** 退避 Move-Item が ファイルロック等で失敗する
- **THEN** システムは `-ErrorAction SilentlyContinue` で失敗を握り潰さず、try/catch で捕捉する
- **THEN** システムは `$evacuated = $false` を確定させる
- **THEN** システムは差し替えフェーズへ進まない

#### Scenario: Replacement detection does not rely solely on file presence

- **WHEN** 差し替え結果を判定する
- **THEN** システムは `Test-Path $target` 単独で成功と判定しない
- **THEN** システムは `(Get-Item $target).LastWriteTime` が relauncher 起動時刻より新しいことを確認する
- **THEN** システムは `extracted` ファイルがもう存在しないことを確認する

#### Scenario: Failure path performs rollback only

- **WHEN** 退避と差し替えのいずれかが失敗する
- **THEN** システムは `.bak` から target を復元する
- **THEN** システムは新規プロセスを起動しない（`Start-Process $target` を呼ばない）
- **THEN** システムはユーザーに Update Failed のメッセージボックスを表示する
- **THEN** システムは `update.log` に rollback と理由を記録する

### Requirement: Auto-update relauncher must persist diagnostic logs locally

システムは、自動アップデートの relauncher 実行中に、フェーズと結果を `%LOCALAPPDATA%\KatanA\update.log` に追記しなければならない（MUST）。

#### Scenario: Log records phase, result, target path, and reason without PII

- **WHEN** relauncher が実行される
- **THEN** システムは `evacuate`, `replace`, `launch`, `rollback` の 4 フェーズいずれかで結果を追記する
- **THEN** ログ行は ISO 8601 形式の時刻、フェーズ名、`ok` / `fail` / `retry` のいずれか、target の絶対パス、例外メッセージを含む
- **THEN** システムはユーザー名や document の内容を記録しない

#### Scenario: Log directory is created on demand

- **WHEN** `%LOCALAPPDATA%\KatanA` が存在しない
- **THEN** システムは relauncher 実行時にディレクトリを作成する

### Requirement: Release notes must document the manual upgrade path for clients still on broken versions

システムは、v0.22.10 のクライアントが v0.22.11 へ自動アップデートできないことをリリースノートに明記しなければならない（MUST）。

#### Scenario: Linux v0.22.10 client receives 404 on auto-update

- **WHEN** v0.22.10 の Linux クライアントが自動アップデートを試行する
- **THEN** クライアントは `KatanA-linux-x86_64.zip` を要求し 404 を受け取る
- **THEN** リリースノートは `KatanA-linux-x86_64.tar.gz` を手動ダウンロードする手順を案内する

#### Scenario: Windows v0.22.10 client may keep the old binary after auto-update

- **WHEN** v0.22.10 の Windows クライアントが in-app update を実行する
- **THEN** クライアントは旧 relauncher を使うため、退避失敗時に旧版が起動する可能性がある
- **THEN** リリースノートは portable zip / MSI による手動再インストール手順を案内する

### Requirement: 自動更新はアセット内ファイル名と実行中ファイル名の差異を吸収する

システムは、アーカイブ内の既知ファイル名と、現在実行中のファイル名が異なる場合でも、自動更新を完了しなければならない（MUST）。展開元の解決はリリースアセット内の既知名を一次候補とし、展開先は現在実行中のファイルパスを維持しなければならない（MUST）。

#### Scenario: Linuxbrew の既存 `katana-desktop` 名で更新できる

- **GIVEN** Linuxbrew 旧 formula 経由のインストールで、現在実行中のファイル名が `katana-desktop` である
- **AND** `KatanA-linux-x86_64.tar.gz` の直下には `KatanA` が含まれる
- **WHEN** ユーザーがアプリ内の「アップデート＆再起動」を 1 回クリックする
- **THEN** インストーラーは展開元として `KatanA` を選択する
- **AND** 展開先として現在実行中の `katana-desktop` パスを維持する
- **AND** 更新後もユーザーは同じ起動導線で新バージョンを起動できる

#### Scenario: Linux 手動ダウンロードの `KatanA` 名で更新できる

- **GIVEN** Linux の手動ダウンロード環境で、現在実行中のファイル名が `KatanA` である
- **WHEN** ユーザーがアプリ内の「アップデート＆再起動」を 1 回クリックする
- **THEN** インストーラーは展開元として `KatanA` を選択する
- **AND** 更新後に `./KatanA` から新バージョンを起動できる

#### Scenario: Windows の実行ファイル名差異を吸収する

- **GIVEN** Windows の更新アセット直下に `KatanA.exe` が含まれる
- **AND** 現在実行中のファイル名が `KatanA.exe` と異なる
- **WHEN** ユーザーがアプリ内の「インストールして再起動（Install and Restart）」を 1 回クリックする
- **THEN** インストーラーは展開元として `KatanA.exe` を選択する
- **AND** 展開先として現在実行中のファイルパスを維持する

#### Scenario: 想定ファイルが無い場合は曖昧な候補を採用しない

- **GIVEN** 更新アーカイブの展開先直下に既知名の実行ファイルが存在しない
- **WHEN** 展開先直下に採用可能な通常ファイルが 0 件または複数件存在する
- **THEN** インストーラーは自動的に推測して更新を続行しない
- **AND** エラーには展開先直下の内容が含まれる

### Requirement: 自動更新はユーザーに手動復旧を要求しない

システムは、既存の Linuxbrew ユーザーが自動更新に失敗していた状態からでも、次回のアプリ内更新操作だけで自己修復しなければならない（MUST）。ユーザーに `brew reinstall`、ターミナル操作、ファイルリネーム、再ダウンロードを要求してはならない（MUST NOT）。

#### Scenario: 既存ユーザーは 1 回の更新操作だけで復旧する

- **GIVEN** Linuxbrew 旧 formula により `katana-desktop` 名で起動している既存ユーザーがいる
- **WHEN** ユーザーがアプリ内の「アップデート＆再起動」を 1 回クリックする
- **THEN** 更新、差し替え、再起動が自動で完了する
- **AND** ユーザーに追加の復旧手順は表示されない
