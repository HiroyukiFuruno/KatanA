## MODIFIED Requirements

### Requirement: ワークスペース単位のタブセッション永続化

システムは、ユーザー設定に従ってワークスペース単位のタブセッション状態を保存し、ワークスペース再オープン時に復元しなければならない（SHALL）。

#### Scenario: ワークスペースのタブセッションを復元する

- **WHEN** ユーザーがワークスペースを再度開き、タブセッション復元が有効になっている時
- **THEN** そのワークスペースで以前開いていたタブが復元される
- **THEN** active タブ、pinned 状態、展開ディレクトリ、タブグループが保存済みのワークスペースセッションから復元される

#### Scenario: 設定により復元が無効化されている

- **WHEN** ユーザーがタブセッション復元を無効にした状態でワークスペースを開いた時
- **THEN** 保存済みのワークスペースタブセッションは自動適用されない
- **THEN** それでもワークスペース自体は通常どおり開ける

### Requirement: ワークスペースセッション payload は versioned で後方互換を保つ

システムは、legacy なワークスペースタブセッション payload を読み込み、新しいセッションモデルへ昇格させても既存ユーザーを壊さないようにしなければならない（SHALL）。

#### Scenario: version を持たない legacy session payload

- **WHEN** システムが tabs と active index だけを持つ旧ワークスペースタブセッション payload を読み込んだ時
- **THEN** その payload は legacy version として解釈される
- **THEN** 不足している pinned と group の項目は既定値で補完される

#### Scenario: 新しい session payload を保存する

- **WHEN** システムが新形式でワークスペースタブセッション状態を保存した時
- **THEN** payload には明示的な version が含まれる
- **THEN** 保存データは、そのワークスペースの grouped / pinned タブを復元するのに十分である
