## ADDED Requirements

### Requirement: Markdown本文を対象に検索できる

システムは、現在の workspace 内にある Markdown 文書の本文を検索できなければならない（SHALL）。

#### Scenario: 本文検索を実行する

- **WHEN** ユーザーが、1つ以上の Markdown 文書本文に一致する検索語を入力した時
- **THEN** システムは、ファイル名ではなく本文内容に基づく一致結果を返す
- **THEN** 各結果は、Markdown 文書内の特定の一致位置に対応する

#### Scenario: 検索対象をMarkdown文書に限定する

- **WHEN** 現在の workspace に非 Markdown file が含まれている時
- **THEN** システムは、それらの file を Markdown content search の結果に含めない

### Requirement: 検索結果に文脈付きスニペットを表示する

システムは、各 Markdown 検索結果に対して、一致箇所を文脈付きで判断できるだけの周辺テキストを表示しなければならない（SHALL）。

#### Scenario: ヒット位置の周辺を表示する

- **WHEN** 検索語に対する一致位置が返された時
- **THEN** システムは、一致テキストとその前後文脈を含む結果スニペットを表示する

#### Scenario: 結果数を制限する

- **WHEN** 検索語の一致数が、設定済みまたは既定の結果上限を超えた時
- **THEN** システムは、上位に順位付けされた範囲の結果のみを表示する

### Requirement: 検索ワード履歴を保持できる

システムは、最近使用した Markdown content search の検索語を保持し、後で再利用できるようにしなければならない（SHALL）。

#### Scenario: 検索語を履歴に保存する

- **WHEN** ユーザーが Markdown content search の検索語を実行した時
- **THEN** システムは、その検索語を検索履歴に保存する

#### Scenario: 履歴から再利用する

- **WHEN** ユーザーが過去に Markdown content search を使った後で search UI を開いた時
- **THEN** システムは、最近の検索語を再利用候補として提示する

### Requirement: 検索履歴を消去できる

システムは、保存済みの Markdown content search 履歴を消去する手段を提供しなければならない（SHALL）。

#### Scenario: 履歴を消去する

- **WHEN** ユーザーが履歴消去 action を実行した時
- **THEN** システムは、保存済みの Markdown content search 検索語を削除する

#### Scenario: 履歴消去後は候補が表示されない

- **WHEN** ユーザーが検索履歴を消去した時
- **THEN** 以後の search UI 起動では、消去済みの検索語を recent history として表示しない

### Requirement: 検索語へ順次ジャンプできる

システムは、アクティブな Markdown 文書内において、現在の検索語に対する次または前の一致箇所へ順次移動できなければならない（SHALL）。

#### Scenario: 次の一致へ移動する

- **WHEN** 現在の Markdown 検索語がアクティブ文書内で複数一致している時
- **THEN** ユーザーが next-match navigation を実行すると、システムは次の一致箇所へ移動する

#### Scenario: 前の一致へ移動する

- **WHEN** 現在の Markdown 検索語がアクティブ文書内で複数一致している時
- **THEN** ユーザーが previous-match navigation を実行すると、システムは前の一致箇所へ移動する

#### Scenario: 現在の検索語がない場合はジャンプを実行しない

- **WHEN** アクティブな Markdown content search の検索語が存在しない時
- **THEN** システムは、next-match または previous-match navigation を実行しない

### Requirement: 検索語を結果表示とプレビューで強調する

システムは、結果スニペット、Markdown editor/code view、Markdown preview pane において、一致した検索語を視覚的に強調表示しなければならない（SHALL）。

#### Scenario: 結果一覧で一致語を強調する

- **WHEN** 検索結果スニペットに一致語が含まれる時
- **THEN** その一致語は結果一覧上で強調表示される

#### Scenario: Preview で一致語を強調する

- **WHEN** ユーザーが検索結果から Markdown 文書を開き、その一致語が preview pane に表示されている時
- **THEN** その一致語は preview pane で強調表示される

#### Scenario: Editor で一致語を強調する

- **WHEN** ユーザーが検索結果から Markdown 文書を開き、その一致語が editor/code view に表示されている時
- **THEN** その一致語は editor/code view で強調表示される

### Requirement: 検索結果から該当箇所へ移動できる

システムは、ユーザーが検索結果から対象の Markdown 文書を開き、その一致位置へ移動できるようにしなければならない（SHALL）。

#### Scenario: 結果選択で該当箇所へ移動する

- **WHEN** ユーザーが検索結果を選択した時
- **THEN** システムは、必要であれば対象の Markdown 文書を開く
- **THEN** システムは、その文書内の一致位置へ移動する

#### Scenario: 対象ファイルが開けない場合は失敗を通知する

- **WHEN** ユーザーが、既に disk 上に存在しない file を指す検索結果を選択した時
- **THEN** システムは crash しない
- **THEN** システムは、その file を開けないことをユーザーに通知する
