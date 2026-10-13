## ADDED Requirements

### Requirement: 更新確認は current platform に合う release asset を解決する

システムは、更新確認時に current platform / architecture に一致する release asset を解決しなければならない（MUST）。

#### Scenario: macOS で更新を確認する

- **WHEN** macOS 上で更新確認を実行する
- **THEN** 更新候補の asset は `KatanA-macOS.zip` として解決される

#### Scenario: Windows で更新を確認する

- **WHEN** Windows x86_64 上で更新確認を実行する
- **THEN** 更新候補の asset は `KatanA-windows-x86_64.zip` として解決される

#### Scenario: Linux で更新を確認する

- **WHEN** Linux x86_64 上で更新確認を実行する
- **THEN** 更新候補の asset は `KatanA-linux-x86_64.tar.gz` として解決される

### Requirement: 更新の install path は platform policy に従って切り替わる

システムは、platform policy に従って update action を切り替えなければならない（MUST）。

#### Scenario: macOS は auto-install を継続する

- **WHEN** macOS 上でユーザーが更新を適用する
- **THEN** 既存の download / install / restart path が利用される

#### Scenario: Windows は manual download にフォールバックする

- **WHEN** Windows x86_64 上でユーザーが更新を適用する
- **THEN** アプリケーションは壊れた install path を実行しない
- **THEN** ユーザーは release page または matching asset の download 導線へ誘導される
- **THEN** UI は `Install and Restart` のような auto-install を示す文言を表示しない

#### Scenario: Linux は manual download にフォールバックする

- **WHEN** Linux x86_64 上でユーザーが更新を適用する
- **THEN** アプリケーションは壊れた install path を実行しない
- **THEN** ユーザーは release page または matching asset の download 導線へ誘導される
- **THEN** UI は `Install and Restart` のような auto-install を示す文言を表示しない

### Requirement: matching asset が存在しない場合は recoverable に失敗する

システムは、current platform に合う release asset が存在しない場合でも、recoverable error として扱わなければならない（MUST）。

#### Scenario: platform asset が未公開である

- **WHEN** current platform / architecture に一致する release asset が見つからない
- **THEN** アプリケーションは現在のセッションを維持する
- **THEN** ユーザーは actionable な失敗メッセージを受け取る
