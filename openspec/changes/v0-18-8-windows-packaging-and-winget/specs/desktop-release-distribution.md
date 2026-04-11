## MODIFIED Requirements

### Requirement: Windows release artifact は追加 runtime セットアップなしで起動できる

システムは、Windows x86_64 向けに配布する KatanA binary / installer を、追加の VC++ runtime セットアップなしで起動可能な形で提供しなければならない（MUST）。

#### Scenario: clean Windows 環境で MSI を導入する

- **WHEN** ユーザーが `KatanA-windows-x86_64.msi` を、`Microsoft.VCRedist.2015+.x64` が事前導入されていない Windows x86_64 環境でインストールする
- **THEN** installer は user scope で正常完了する
- **THEN** `KatanA.exe` は `VCRUNTIME140*.dll` または `api-ms-win-crt-*` 欠落エラーなしで起動できる

### Requirement: winget の初回申請と version update は区別される

システムは、winget package が upstream に未存在の状態と、既存 package への version update とを区別しなければならない（MUST）。

#### Scenario: package が upstream に未存在

- **WHEN** `HiroyukiFuruno.katana-desktop` が `microsoft/winget-pkgs` に存在しない状態で release helper を実行する
- **THEN** helper は `komac update` を盲目的に実行しない
- **THEN** helper は bootstrap path が必要であることを明示するか、non-interactive な初回申請用 submit path を使う

#### Scenario: package が upstream に既に存在

- **WHEN** `HiroyukiFuruno.katana-desktop` が `microsoft/winget-pkgs` に存在する状態で新しい version を配布する
- **THEN** helper は `komac update` ベースの version update flow を利用できる

### Requirement: winget helper は PR 作成可能な token を明示的に要求する

システムは、winget helper / release workflow で `komac` に渡す token source を曖昧にしてはならない（MUST）。

#### Scenario: PR 作成用 token が未設定または不適切

- **WHEN** helper が `komac` の PR 作成要件を満たす classic GitHub PAT を受け取っていない
- **THEN** helper は winget sync を skip し、必要な secret / token 条件を明示する
- **THEN** helper は GitHub Actions の自動 `github.token` へ暗黙 fallback しない

### Requirement: Windows installer は KatanA branding と整合した表示を持つ

システムは、Windows installer の初回導入体験で KatanA 固有の metadata と branding を提示しなければならない（MUST）。

#### Scenario: installer dialog を表示する

- **WHEN** ユーザーが `KatanA-windows-x86_64.msi` を起動して installer dialog を開く
- **THEN** dialog は既定の WiX 文言だけではなく、KatanA 向けの Product 名、説明、icon、banner または dialog bitmap を表示する
- **THEN** その確認証跡を release 作業時に参照できる
