## ADDED Requirements

### Requirement: user は local image file を active Markdown document に添付できなければならない

システムは、user が local image file を選択したとき、その画像を active Markdown document を起点にした `./asset/img` 配下へ保存し、相対 path の Markdown image reference を挿入できなければならない（SHALL）。

#### Scenario: local file を添付する

- **WHEN** user が active Markdown document に対して image file attach を実行する
- **THEN** system は active Markdown document の親ディレクトリ配下に `asset/img` を解決する
- **THEN** system は選択した画像を保存し、相対 path の Markdown image reference を editor に挿入する

### Requirement: user は clipboard image を active Markdown document に貼り付けできなければならない

システムは、user が image data を clipboard に持つ場合、その画像を local asset として保存し、Markdown image reference を挿入できなければならない（SHALL）。

#### Scenario: clipboard image を貼り付ける

- **WHEN** user が image data を含む clipboard から paste image を実行する
- **THEN** system は file attach と同じ保存先ルールで画像を保存する
- **THEN** system は保存した画像への相対 path を editor に挿入する

### Requirement: image ingest の保存先と命名挙動は settings で変更できなければならない

システムは、image ingest について保存先ディレクトリ、timestamp default naming、命名ダイアログ表示有無を settings から変更できなければならない（SHALL）。

#### Scenario: 保存先と命名ポリシーを変更する

- **WHEN** user が image ingest settings を変更する
- **THEN** system は subsequent file attach / paste image にその設定を反映する

### Requirement: local image reference は asset の場所へ辿れなければならない

システムは、active Markdown document から相対解決できる local image reference について、対象ディレクトリまたはファイルへ辿る導線を提供しなければならない（SHALL）。

#### Scenario: local image reference から asset を辿る

- **WHEN** Markdown 内の local image reference が解決可能である
- **THEN** system は user に対象ディレクトリまたはファイルへ辿る導線を提示する

#### Scenario: local image reference が解決できない

- **WHEN** Markdown 内の image reference が remote URL または missing local file である
- **THEN** system は誤った reveal を行わない
- **THEN** system は missing または non-local の状態を区別できる
