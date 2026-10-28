## ADDED Requirements

### Requirement: Markdown分割previewはExplorer window背景roleを使う

KatanA MUST horizontal/verticalいずれのsplit layoutでも、分割Markdown previewの背面を既存themeのExplorer/window背景roleで描画する。Markdown専用の背景処理は他document typeが選ぶ背景を置き換えてはならない。

#### Scenario: Markdown split previewがExplorer背景に従う

- **WHEN** Markdown documentをhorizontalまたはvertical split previewで表示する
- **THEN** split preview背景にactive themeのExplorer/window background roleを使う

#### Scenario: 他document typeのpreview背景を保つ

- **WHEN** non-Markdown documentをsplit previewで表示する
- **THEN** Markdown専用のExplorer背景処理を適用しない
- **AND THEN** documentは既存のpreview背景動作を保つ
