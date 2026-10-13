## ADDED Requirements

### Requirement: Split mode scroll sync は文書全体の scrollable range を同期できる

システムは、Code / Preview の split mode で scroll sync が有効なとき、editor と preview の scrollable range 全体を相互に同期できなければならない（SHALL）。

#### Scenario: Editor から preview の末尾まで同期する

- **WHEN** ユーザーが split mode の editor を最後の scrollable position まで移動した時
- **THEN** preview は最後の scrollable position まで同期する
- **THEN** preview は末尾到達前で止まってはならない

#### Scenario: Preview から editor の末尾まで同期する

- **WHEN** ユーザーが split mode の preview を最後の scrollable position まで移動した時
- **THEN** editor は最後の scrollable position まで同期する
- **THEN** editor は末尾到達前で止まってはならない

#### Scenario: Heading がない文書でも full-range sync できる

- **WHEN** アクティブな Markdown 文書に heading anchor が存在しない時
- **THEN** システムは scrollable range の start と EOF を使って split scroll sync を成立させる

### Requirement: 最後の見出し以降の tail 区間も同期対象に含める

システムは、最後の見出し以降に続く本文、list、code block、table などの tail 区間を split scroll sync の対応表に含めなければならない（SHALL）。

#### Scenario: 最後の見出しの後に長い本文がある

- **WHEN** 最後の heading anchor の後に、複数 screen 分の本文が続いている時
- **THEN** システムはその tail 区間を独立した最終 segment として扱える
- **THEN** tail 区間内の scroll 進捗は対応する pane 側でも連続的に再現される

### Requirement: 同期後の scroll は収束し、逆方向へガタつかない

システムは、片側 pane に同期位置を適用した後、その適用結果を新しいユーザー scroll と誤認して逆方向の corrective scroll を繰り返してはならない（MUST）。

#### Scenario: Editor が source の時に preview が収束する

- **WHEN** ユーザー scroll により editor が split scroll sync の source になった時
- **THEN** preview は対応位置へ移動した後、追加のユーザー入力なしに上下へ往復しない
- **THEN** 同期後は安定した state に収束する

#### Scenario: Preview が source の時に editor が収束する

- **WHEN** ユーザー scroll により preview が split scroll sync の source になった時
- **THEN** editor は対応位置へ移動した後、追加のユーザー入力なしに上下へ往復しない
- **THEN** 同期後は安定した state に収束する

#### Scenario: Geometry 変化後も corrective scroll を連発しない

- **WHEN** preview content height や pane size が変化した後に split scroll sync が再評価される時
- **THEN** システムは geometry 変化を反映した再同期を行えてもよい
- **THEN** その再同期は連続的な corrective scroll loop を引き起こしてはならない
