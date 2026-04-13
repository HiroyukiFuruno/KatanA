## ADDED Requirements

### Requirement: user は local image file を active Markdown document に取り込めなければならない

システムは、user が local image file attach を実行したとき、その画像を active Markdown document 親基準の `./asset/img` 配下へ保存し、相対 path の Markdown image reference を挿入できなければならない（SHALL）。

#### Scenario: image file attach を実行する

- **WHEN** user が active Markdown document に対して image file attach を実行する
- **THEN** system は active Markdown document の親ディレクトリ配下に `./asset/img` を解決する
- **THEN** system は画像を保存し、相対 path の Markdown image reference を editor に挿入する

### Requirement: user は clipboard image を同じ ingest rule で貼り付けできなければならない

システムは、clipboard image paste を file attach と同じ保存先、命名、refresh contract で扱わなければならない（SHALL）。

#### Scenario: clipboard image を貼り付ける

- **WHEN** user が image data を含む clipboard から paste image を実行する
- **THEN** system は file attach と同じ保存先ルールで画像を保存する
- **THEN** system は保存した画像への相対 path を editor に挿入する

### Requirement: user は外部 image file を editor へ drag-and-drop で取り込めなければならない

システムは、外部環境から editor へ drop された image file を ingest pipeline へ流し、Markdown image reference を挿入できなければならない（SHALL）。

#### Scenario: cursor 位置へ drag-and-drop image を挿入する

- **WHEN** user が image file を editor 上へ drop し、editor cursor が有効である
- **THEN** system は画像を ingest して cursor 位置へ Markdown image reference を挿入する

#### Scenario: cursor が不明な場合は文書末尾へ挿入する

- **WHEN** user が image file を editor 上へ drop し、cursor または selection が確定していない
- **THEN** system は Markdown image reference を active document の末尾へ追加する

### Requirement: image ingest の保存先と命名挙動は settings で変更できなければならない

システムは、image ingest について保存先ディレクトリ、命名 format、確認ダイアログ表示有無を settings から変更できなければならない（SHALL）。

#### Scenario: ingest settings を変更する

- **WHEN** user が image ingest settings を変更する
- **THEN** system は subsequent file attach、clipboard paste、external image drop にその設定を反映する

### Requirement: image ingest 完了後は preview と explorer の asset state が更新されなければならない

システムは、image ingest が成功した後、active document preview と explorer 上の asset state を再評価しなければならない（SHALL）。

#### Scenario: image ingest 後に asset state を更新する

- **WHEN** system が image ingest を完了する
- **THEN** active Markdown preview は新しい image reference を解決できる
- **THEN** explorer 側の asset / thumbnail state も後続 refresh cycle で更新される
