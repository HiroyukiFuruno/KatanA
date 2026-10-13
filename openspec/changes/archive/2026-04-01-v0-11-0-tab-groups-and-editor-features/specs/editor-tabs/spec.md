## ADDED Requirements

### Requirement: ユーザーは名前付きのカラーグループでタブを整理できる

システムは、ユーザーが開いているタブを名前付きかつ色付きのグループに整理できるようにしなければならない（SHALL）。

#### Scenario: タブから新しいグループを作成する

- **WHEN** ユーザーがタブのコンテキストメニューを開き、グループ作成操作を選んだ時
- **THEN** 名前と色を持つ新しいタブグループが作成され、選択中のタブが最初のメンバーになる
- **THEN** 選択中のタブは高々 1 つのグループにのみ所属する

#### Scenario: 既存グループへタブを追加する

- **WHEN** ユーザーがタブのコンテキストメニューから既存グループを選んだ時
- **THEN** 選択中のタブはそのグループのメンバーになる
- **THEN** そのタブは以前所属していたグループから外れる

#### Scenario: pinned タブはグループ化されない

- **WHEN** タブが pinned された時
- **THEN** そのタブはどのタブグループにも所属しない
- **THEN** そのタブに対するグループ追加操作は表示されない、または無効化される

#### Scenario: タブグループを折りたたむ

- **WHEN** ユーザーがタブグループを折りたたんだ時
- **THEN** グループのメンバータブは閉じられずにタブバーから非表示になる
- **THEN** グループヘッダーは表示されたままで、再度展開できる

#### Scenario: 折りたたまれたグループでも active メンバーは見える

- **WHEN** active なタブが折りたたまれたグループに所属している時
- **THEN** グループヘッダーは表示されたまま残る
- **THEN** active なメンバータブは見えたままで、非 active メンバーだけが隠れる

### Requirement: pinned タブは通常の close 操作から保護される

システムは、pinned タブが明示的に unpin されるまで、通常の close 操作から保護しなければならない（SHALL）。

#### Scenario: pinned タブでは close ボタンを隠す

- **WHEN** タブが pinned された時
- **THEN** そのタブの close ボタンはタブバーに表示されない
- **THEN** ただし tooltip など同等の手段でタイトルは確認できる

#### Scenario: 一括 close は pinned タブをスキップする

- **WHEN** ユーザーが close-all、close-others、close-left、close-right のいずれかを実行した時
- **THEN** pinned タブは閉じられない
- **THEN** unpinned タブは指定された close 挙動に従って処理される

#### Scenario: close shortcut では pinned タブを閉じない

- **WHEN** active タブが pinned されており、通常の close shortcut が close action を発行した時
- **THEN** その pinned タブは開いたまま残る
- **THEN** 通常の close を行うには、先にユーザーが unpin しなければならない
