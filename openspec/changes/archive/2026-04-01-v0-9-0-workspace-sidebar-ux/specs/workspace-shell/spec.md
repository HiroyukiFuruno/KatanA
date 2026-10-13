## MODIFIED Requirements

### Requirement: シェルレイアウトは MVP ナビゲーションモデルを維持する

システムは、ワークスペースユーティリティレール、ワークスペースナビゲーション、ドキュメント編集、プレビュー描画の各領域を持つデスクトップシェルを提供しなければならない（SHALL）。

#### Scenario: 既定の MVP レイアウトを表示する

- **WHEN** アクティブなワークスペースを持つ状態でアプリケーションが起動した時
- **THEN** ユーザーにはワークスペースユーティリティレール、ワークスペースペイン、エディタペイン、プレビューペインが表示される
- **THEN** ワークスペースペインが折りたたまれても、ワークスペースユーティリティレールは利用可能なまま残る
- **THEN** シェルは将来の menu や AI panel 拡張のために一貫した配置領域を確保する

## ADDED Requirements

### Requirement: ワークスペースペインはタイトル文字列なしで表示される

ワークスペースペインのヘッダーは、`Workspace` / `ワークスペース` などのタイトル文字列を表示せず、アイコン操作とツリー表示を優先しなければならない（MUST）。

#### Scenario: ワークスペースヘッダーからタイトル文字列を除去する

- **WHEN** ユーザーがワークスペースペインを見る
- **THEN** ヘッダーには `Workspace` / `ワークスペース` の文言が表示されない
- **THEN** ワークスペースツリーの表示領域が優先される

### Requirement: ワークスペースヘッダーの操作ボタン整列

アクティブなワークスペースペインの操作ヘッダーは、更新ボタンを先頭側に、全展開・全閉ボタンと `...` メニューを末尾側に配置し、フィルター機能を同じヘッダー内で継続利用できなければならない（MUST）。

#### Scenario: ヘッダーの操作グループを描画する

- **WHEN** ユーザーがワークスペースペインを表示する
- **THEN** 更新ボタンはヘッダー行の先頭側に表示される
- **THEN** 全展開ボタンと全閉ボタンはヘッダー行の末尾側に表示される
- **THEN** `...` メニューは全展開・全閉と同じ末尾側グループに表示される
- **THEN** フィルター機能はヘッダー内で引き続き利用できる

### Requirement: ワークスペース一覧は version-aware sort で表示される

ワークスペース一覧は、数値 token を含む名前に対して辞書順ではなく version-aware sort を適用しなければならない（MUST）。tree 表示では directory を file より前に表示し、flat 表示では file 一覧を workspace-relative path 基準で同じ sort ルールに従わせなければならない（MUST）。

#### Scenario: 数値を含む名前を自然順で表示する

- **WHEN** 同じ階層に `v0-9-x` と `v0-11-x` のような entry が存在する
- **THEN** ワークスペース一覧では `v0-9-x` が `v0-11-x` より先に表示される

#### Scenario: tree 表示では directory を先に保つ

- **WHEN** 同じ階層に directory と file が混在している
- **THEN** tree 表示では directory が file より先に表示される
- **THEN** そのうえで各 group 内の順序には version-aware sort が適用される

### Requirement: ワークスペース一覧は tree / flat 表示を切り替えられる

ワークスペースペインは、tree 表示と directory 概念を持たない file 単位の flat 表示を切り替えられなければならない（MUST）。flat 表示フラグの既定値は `false` でなければならず（MUST）、その結果として既定表示は tree でなければならない（MUST）。ユーザーが切り替えた表示状態は workspace ごとに永続化されなければならない（MUST）。

#### Scenario: 既定表示は tree である

- **WHEN** ユーザーがワークスペースペインを初めて表示する
- **THEN** flat 表示フラグは `false` で初期化される
- **THEN** ワークスペース一覧は tree 表示で始まる

#### Scenario: `...` メニューから flat 表示を有効にする

- **WHEN** ユーザーが `...` メニューの `表示 -> フラット表示` をクリックする
- **THEN** flat 表示フラグは `true` になる
- **THEN** ワークスペース一覧は directory node を持たない file 単位の flat 表示へ切り替わる
- **THEN** メニュー項目は `✔フラット表示` のように有効状態が分かる表現になる

#### Scenario: workspace ごとの表示選択が復元される

- **WHEN** ユーザーがある workspace で flat 表示を有効にして、その workspace を閉じる
- **AND** その後に同じ workspace を再度開く
- **THEN** 以前選択した flat 表示状態が復元される
- **THEN** 既定値 `false` は未設定の workspace にのみ適用される

#### Scenario: 有効中の flat 表示を解除する

- **WHEN** ユーザーが有効中の `✔フラット表示` を再度クリックする
- **THEN** flat 表示フラグは `false` に戻る
- **THEN** ワークスペース一覧は tree 表示へ戻る

#### Scenario: flat 表示では file 一覧を workspace-relative path で識別できる

- **WHEN** ワークスペース一覧が flat 表示で描画される
- **THEN** row は directory node ではなく file entry のみで構成される
- **THEN** 同名 file があっても workspace-relative path により識別できる

#### Scenario: flat 表示では tree 専用操作を無効化する

- **WHEN** ワークスペース一覧が flat 表示になっている
- **THEN** 全展開と全閉は実行できない
- **THEN** 更新とフィルターは引き続き利用できる
