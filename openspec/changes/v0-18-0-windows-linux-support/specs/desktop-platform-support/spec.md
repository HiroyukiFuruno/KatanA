## ADDED Requirements

### Requirement: KatanA は対応 desktop OS で主要フローを起動できる

システムは、macOS、Windows x86_64、Linux x86_64 の各 OS で、workspace、editor、preview を含む主要フローを起動できなければならない（MUST）。

#### Scenario: Windows でシェルを起動する

- **WHEN** ユーザーが Windows x86_64 向けの KatanA binary を起動する
- **THEN** アプリケーションは startup crash せずに desktop shell を表示する
- **THEN** workspace pane、editor pane、preview pane が利用可能になる

#### Scenario: Linux でシェルを起動する

- **WHEN** ユーザーが Linux x86_64 向けの KatanA binary を起動する
- **THEN** アプリケーションは startup crash せずに desktop shell を表示する
- **THEN** workspace pane、editor pane、preview pane が利用可能になる

#### Scenario: macOS の既存シェル起動を維持する

- **WHEN** ユーザーが macOS 上で KatanA を起動する
- **THEN** 既存の desktop shell は回帰せずに表示される
- **THEN** workspace pane、editor pane、preview pane が利用可能なままである

### Requirement: 対応 OS では core Markdown workflow が等価に利用できる

システムは、対応 desktop OS のいずれでも、workspace を開いて Markdown 文書を編集し、preview を確認できなければならない（MUST）。

#### Scenario: Windows で workspace を開いて文書を編集する

- **WHEN** ユーザーが Windows 上でローカル directory を workspace として開き、Markdown file を選択する
- **THEN** 文書は editor に読み込まれる
- **THEN** preview はその active document を表示する

#### Scenario: Linux で workspace を開いて文書を編集する

- **WHEN** ユーザーが Linux 上でローカル directory を workspace として開き、Markdown file を選択する
- **THEN** 文書は editor に読み込まれる
- **THEN** preview はその active document を表示する

### Requirement: platform integration failure は recoverable fallback を持つ

システムは、対応 OS 上で theme、locale、font、menu などの platform integration の一部が取得できなくても、recoverable fallback により起動と編集フローを継続しなければならない（MUST）。

#### Scenario: system theme を取得できない

- **WHEN** 対応 OS 上で system theme 検出が失敗する
- **THEN** アプリケーションは default theme で起動を継続する
- **THEN** theme 検出 failure は startup crash を引き起こさない

#### Scenario: system locale を取得できない

- **WHEN** 対応 OS 上で system locale 検出が失敗する
- **THEN** アプリケーションは default language で起動を継続する
- **THEN** locale 検出 failure は startup crash を引き起こさない
