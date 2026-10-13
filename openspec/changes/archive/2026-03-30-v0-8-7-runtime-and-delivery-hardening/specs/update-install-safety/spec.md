## ADDED Requirements

### Requirement: 更新インストールは staged swap で行う

更新インストールは、既存アプリを破壊的に削除する前に置換候補を staging し、失敗時に rollback できなければならない（MUST）。

#### Scenario: 正常な更新アーカイブを適用する

- **WHEN** 有効な更新アーカイブがダウンロードされ、期待する `.app` bundle を含んでいる
- **THEN** システムは target 近傍に置換候補を staging してから swap を行う
- **THEN** 既存アプリ bundle は、置換候補の準備が完了する前に削除されない

#### Scenario: swap 途中で失敗する

- **WHEN** copy、rename、permission、quarantine removal、launch のいずれかが失敗する
- **THEN** 既存アプリ bundle は restore されるか、そのまま保持される
- **THEN** 次回起動不能な中間状態を唯一の結果として残してはならない

### Requirement: 更新失敗は可視かつ回復可能である

更新準備や relaunch の失敗は、現在のセッションを不必要に終了させず、ユーザーが再試行可能な失敗として扱わなければならない（SHALL）。

#### Scenario: 更新準備に失敗する

- **WHEN** 更新 ZIP の取得、展開、bundle 検証のいずれかに失敗する
- **THEN** 現在のアプリセッションは継続する
- **THEN** 更新ダイアログまたはステータス表示に、再試行可能なエラーが表示される

#### Scenario: relaunch command を起動できない

- **WHEN** relaunch script の生成や起動に失敗する
- **THEN** アプリは silent exit しない
- **THEN** ユーザーは原因を確認して再度更新を試せる
