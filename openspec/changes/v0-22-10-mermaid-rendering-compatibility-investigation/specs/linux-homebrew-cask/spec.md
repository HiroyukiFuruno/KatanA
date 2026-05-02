## ADDED Requirements

### Requirement: Linux GUI distribution supports Homebrew cask when Homebrew supports the artifact

システムは、Homebrew が Linux binary cask を扱える環境では、Linux 版 KatanA GUI アプリを `brew install --cask` で導入できる配布契約を提供しなければならない（MUST）。

#### Scenario: Install Linux GUI app through cask

- **WHEN** ユーザーが Linux の Homebrew 環境で KatanA の cask を install する
- **THEN** cask は Linux 向け release asset を取得する
- **THEN** cask は取得対象の sha256 を検証する
- **THEN** cask は GUI アプリとして起動できる実行ファイル、desktop entry、icon の配置方針を持つ
- **THEN** cask は macOS 専用 artifact を Linux に導入しない

#### Scenario: Separate formula and cask responsibilities

- **WHEN** Linux 向け Formula と cask が同時に存在する
- **THEN** GUI アプリの導入導線は cask を主経路として扱う
- **THEN** Formula を残す場合は、CLI、互換用途、または移行猶予としての役割を文書化する
- **THEN** release notes と install docs は Linux ユーザーへ cask と Formula の違いを説明する

### Requirement: Release automation updates Linux cask metadata from release assets

システムは、GitHub Release の Linux asset から Homebrew tap の Linux cask metadata を更新できなければならない（MUST）。

#### Scenario: Update URL and checksum during release

- **WHEN** GitHub Release に Linux asset が公開される
- **THEN** release automation は Linux cask の URL を対象 release tag へ更新する
- **THEN** release automation は Linux asset の sha256 を算出して cask へ反映する
- **THEN** release automation は upstream に存在しない tag や asset を参照する cask を作らない

#### Scenario: Verify cask installation in Linux environment

- **WHEN** `make linux-up` の Ubuntu 環境で Linux cask を検証する
- **THEN** 手順は `brew tap`、`brew install --cask`、起動確認、`brew uninstall --cask` を含む
- **THEN** 検証は macOS の `--cask` 動作だけで成功扱いにしない
- **THEN** cask が Homebrew 側制約で成立しない場合は、不成立理由と代替配布方式を記録する
