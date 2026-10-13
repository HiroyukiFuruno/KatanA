## MODIFIED Requirements

### Requirement: プレビュー出力で GitHub Flavored Markdown をサポートする

システムは、選択された Markdown エンジンが対応する GitHub Flavored Markdown 構文を、ネストした task list を含めて解析および描画し、task 項目に重複した list marker を導入してはならない。

#### Scenario: 一般的な GFM 構造を描画する

- **WHEN** アクティブな文書に Markdown エンジンが対応する見出し、list、fenced code block、table が含まれているとき
- **THEN** preview 出力はそれらの構造を描画結果として維持する
- **THEN** 未対応の内容もアプリケーションを crash させずに穏当に劣化する

#### Scenario: ネストした子要素を持つ task list item を描画する

- **WHEN** アクティブな文書に、行頭が checkbox marker で始まり、その子要素にネストした bullet または ordered list item を含む task list item があるとき
- **THEN** 親行は checkbox を唯一の先頭 marker として描画される
- **THEN** ネストされた子 list は既存の bullet / ordered marker のスタイルと indentation ルールを維持する

## ADDED Requirements

### Requirement: アクティブな Markdown 文書は暗黙保存なしで hash 管理された disk 更新を利用する

システムは、アクティブな Markdown 文書へ最後に取り込んだ disk state の content hash を追跡しなければならず、暗黙保存を行うことなく、その hash をユーザー起点の更新と定期的な自動更新の両方で利用しなければならない。

#### Scenario: 同期成功後に imported disk hash を初期化または更新する

- **WHEN** アクティブな Markdown 文書が disk から load されたとき、save に成功したとき、または reload に成功したとき
- **THEN** 保存されている last imported disk hash は、同期済みの disk 内容と一致する値へ更新される
- **THEN** 以後の更新判定はその更新済み hash と比較される

#### Scenario: hash が変わっていない状態で手動更新する

- **WHEN** ユーザーが共有更新 action を実行し、現在の on-disk content hash が last imported disk hash と一致しているとき
- **THEN** システムは文書再読込を skip する
- **THEN** in-memory buffer は変更されないままである

#### Scenario: 外部編集後に clean な文書を再読込する

- **WHEN** アクティブな Markdown 文書に未保存変更がなく、その source file hash が last imported disk hash と異なるとき
- **THEN** システムは file 内容を in-memory document buffer へ再読込する
- **THEN** 保存されている disk hash は新たに取り込んだ値へ更新される
- **THEN** preview は、対応する diagram block を含めて、再読込した buffer から再描画される
- **THEN** 文書は clean のまま維持される

#### Scenario: 自動更新が clean な文書への外部編集を検出する

- **WHEN** automatic refresh polling が有効であり、アクティブな Markdown 文書が clean なとき
- **THEN** システムは現在の on-disk content hash が last imported disk hash と異なるかを定期的に確認する
- **THEN** hash が変わった場合にだけ reload と再描画を行う

#### Scenario: dirty な文書を更新する

- **WHEN** アクティブな Markdown 文書に未保存の in-memory 変更があり、手動更新または自動更新のいずれかで on-disk content hash の変更を検出したとき
- **THEN** システムは in-memory buffer を on-disk 内容で黙って置き換えてはならない
- **THEN** 代わりに preview は現在の in-memory buffer から更新される
- **THEN** ユーザーには、文書が dirty のため disk reload を skip したことを示す復旧可能な警告が表示される

#### Scenario: 同じ external hash に対して dirty 文書の warning を繰り返さない

- **WHEN** automatic refresh polling が、アクティブな文書が dirty のまま同じ変更済み on-disk content hash を繰り返し観測したとき
- **THEN** システムは、その hash に対して外部変更が pending であることを記録する
- **THEN** すべての polling interval ごとに同じ warning を繰り返し出してはならない
- **THEN** pending 状態は、save 成功、reload 成功、または on-disk hash が保存済み imported hash に戻った後にのみ解消される

#### Scenario: source file を読めずに再読込が失敗する

- **WHEN** アクティブな Markdown 文書が clean であり、更新処理が source file を disk から読めないとき
- **THEN** 現在の in-memory buffer は保持される
- **THEN** 保存されている last imported disk hash は変更されないままである
- **THEN** ユーザーには復旧可能なエラー状態が表示される
