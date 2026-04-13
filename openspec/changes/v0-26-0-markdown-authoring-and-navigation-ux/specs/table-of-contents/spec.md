## ADDED Requirements

### Requirement: TOC row は filled background を持たない accordion presentation で表示されなければならない

システムは、TOC の各見出し row を filled background なしの accordion presentation で表示し、default では全見出しを展開した状態にしなければならない（SHALL）。

#### Scenario: TOC を accordion presentation で表示する

- **WHEN** user が TOC panel を開く
- **THEN** system は各見出し row を accordion presentation で表示する
- **THEN** row の active state は text emphasis で示され、不要な filled background は表示されない
- **THEN** 初期状態では各見出し階層が展開されたままである

### Requirement: TOC panel header から全開 / 全閉を実行できなければならない

システムは、TOC panel header 左上から expand all と collapse all を実行できなければならない（SHALL）。

#### Scenario: header icon から全開する

- **WHEN** user が TOC panel header の expand-all icon を実行する
- **THEN** system は TOC の全見出し階層を展開する

#### Scenario: header icon から全閉する

- **WHEN** user が TOC panel header の collapse-all icon を実行する
- **THEN** system は TOC の全見出し階層を折りたたむ

### Requirement: TOC accordion は vertical guide line の表示設定を持たなければならない

システムは、TOC accordion の階層を示す vertical guide line を表示できなければならず、その表示有無を settings に保存しなければならない（SHALL）。

#### Scenario: guide line を表示する

- **WHEN** user の TOC guide line setting が enabled である
- **THEN** system は TOC の階層に対応した vertical guide line を表示する

#### Scenario: guide line setting を永続化する

- **WHEN** user が TOC guide line setting を変更する
- **THEN** system はその設定を保存する
- **THEN** 次回起動時も同じ TOC guide line state を復元する
