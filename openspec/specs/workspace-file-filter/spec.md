## Purpose

This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: ツリーのフィルタートグルと入力UI

サイドバーのワークスペースヘッダー部分（既存の「更新」ボタンの左など）に、フィルターモードのトグルボタンを配置しなければならない。 The system SHALL conform.
トグルボタンを有効にすると、その直下に正規表現を入力するためのテキストフィールドが出現する。

#### Scenario: フィルター入力UIの表示と非表示

- **WHEN** ユーザがワークスペースのフィルターボタンをクリックする
- **THEN** 直下にテキスト入力フィールドがドロップダウン表示される

### Requirement: ワークスペースのファイルフィルタリング

入力フィールドに入力された正規表現パターン（ワークスペースからの相対パスを対象）にマッチしないファイルは、ワークスペースのツリー描画時に非表示とならなければならない。 The system SHALL conform.
フィルタリングの結果、表示すべき子ファイルを持たなくなったディレクトリは、ツリー上に表示してはならない（ノイズ軽減のため隠すか折り畳む等視覚的に消すこと）。
入力された正規表現が不正なパターンの場合は、フィルター処理を行わず要素をそのまま表示し、入力フィールドを赤枠などの視覚表現でエラー状態にしなければならない。

#### Scenario: 正常な正規表現によるフィルタリングが成功する

- **WHEN** 入力フィールドに `\.rs$` などの有効な正規表現を入力する
- **THEN** ツリーから `.rs` 以外の拡張子を持つファイルが非表示になる

#### Scenario: 空になったディレクトリの非表示

- **WHEN** 特定のディレクトリ配下の全てのファイルがフィルターにより非表示になる
- **THEN** 該当する親ディレクトリもツリー上に表示されない（あるいは視覚的に無視される）

#### Scenario: 不正な正規表現での視覚的フィードバック

- **WHEN** 入力フィールドに `[` のようなコンパイルできない正規表現文字列を入力する
- **THEN** ファイル一覧は直前の有効なパターン（または全表示）を維持し、入力UIがエラーであることを視覚的に通知する

### Requirement: HTML files are standard visible and openable workspace files

The system SHALL treat `.html` and `.htm` as standard visible and openable workspace file extensions.

#### Scenario: HTML appears in the workspace tree

- **WHEN** a workspace directory contains `index.html`
- **THEN** the workspace tree includes `index.html` in the visible file list

#### Scenario: HTM appears in the workspace tree

- **WHEN** a workspace directory contains `legacy.htm`
- **THEN** the workspace tree includes `legacy.htm` in the visible file list

#### Scenario: Existing workspace filtering still applies

- **WHEN** the workspace filter is active with a pattern that does not match `index.html`
- **THEN** the workspace tree hides `index.html` according to the existing filtering rules
- **THEN** HTML visibility does not bypass the user's workspace filter

#### Scenario: HTML remains visible with older workspace settings

- **WHEN** persisted workspace settings do not list `html` or `htm` in `visible_extensions`
- **THEN** the effective workspace visible extension set still includes `html` and `htm`
- **THEN** workspace scanning and new-file extension choices do not hide HTML solely because the saved settings predate HTML support

### Requirement: フィルターの入力検証は有効な検索モードと一致する

KatanA MUST 通常検索・正規表現・大小文字・単語一致の有効設定を照合と入力エラー検証の両方へ反映する。不正な正規表現は全表示とエラー通知を維持する。

#### Scenario: 正規表現と通常検索を切り替える

- **WHEN** 同じ検索文字列で正規表現モードを切り替える
- **THEN** 有効なモードに従い相対パスへの照合とエラー状態を再計算する

#### Scenario: 通常検索で正規表現メタ文字を検索する

- **WHEN** 正規表現モードを無効にし `[` を検索する
- **THEN** 文字そのものを照合し正規表現エラーを表示しない

### Requirement: 通常検索のファイル名ワイルドカード

KatanA SHALL 通常検索で `*` を含む入力を1 path segment内の任意文字として照合する。パス区切りを含まない入力は各階層のbasename、含む入力はワークスペース相対path全体に一致させる。区切りはUnixでは `/`、Windowsでは `/` とbackslashとする。Unixのbackslashはfilename文字として保持する。`*` を含まない通常文字列と標準正規表現モードの動作は維持する。

#### Scenario: PNGだけを表示する

- **WHEN** 通常検索のフィルターへ `*.png` を入力する
- **THEN** rootおよび子directoryのPNGと必要な祖先directoryを表示し、txtやpng.bakを除外する

#### Scenario: 除外と大文字小文字

- **WHEN** `!*.png` または大小文字一致を有効にした `*.png` を入力する
- **THEN** 有効な否定・大小文字設定に従ってファイルを選別する

#### Scenario: 正規表現モードを維持する

- **WHEN** 正規表現モードで `.*\.png$` を入力する
- **THEN** 従来どおり正規表現として照合し、`*.png` のような無効な正規表現は入力エラーとする
