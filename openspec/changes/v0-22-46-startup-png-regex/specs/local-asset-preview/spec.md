## ADDED Requirements

### Requirement: ローカル画像は描画デバイスの上限を超えてuploadしない

KatanA MUST 単独画像とMarkdown内画像のupload寸法を現在の描画上限以内に収め、元ファイル・保存タブ・元の表示寸法と縦横比を保持する。

#### Scenario: 縦長PNGを起動時に復元する

- **WHEN** 保存済みのactive tabが高さ19,067pxのPNGを参照する
- **THEN** textureの各辺を現在の上限以内に収め、クラッシュせず画像を表示する

#### Scenario: 横長画像とMarkdown内画像

- **WHEN** 横幅または高さが現在の上限を超えるローカル画像を表示する
- **THEN** 単独とMarkdown内の両方で元の表示寸法・比率を保ち、安全なtextureを使用する

#### Scenario: 上限以下の画像

- **WHEN** 画像の両辺が現在の上限以内である
- **THEN** 縮小せず元のpixel寸法でuploadする
