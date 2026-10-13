## ADDED Requirements

### Requirement: user は editor から Markdown 記法を挿入できなければならない

システムは、user が editor 上の command を通じて見出し、装飾、リスト、表などの Markdown 記法を現在の cursor または selection に挿入できるようにしなければならない（SHALL）。

#### Scenario: selection に装飾を挿入する

- **WHEN** user が selection を持つ状態で装飾 command を実行する
- **THEN** system はその selection を対応する Markdown 記法で囲むか、適切な block 記法へ変換する

#### Scenario: cursor 位置に snippet を挿入する

- **WHEN** user が selection を持たずに Markdown snippet command を実行する
- **THEN** system は current cursor location に対応する記法を挿入する

### Requirement: authoring command は Markdown source-first editing を壊してはならない

システムは、authoring command を追加しても Markdown source を主編集面とする契約、preview sync、save 契約を壊してはならない（MUST NOT）。

#### Scenario: authoring command 実行後に保存する

- **WHEN** user が authoring command を使って buffer を更新し、その後 save する
- **THEN** system は更新後の Markdown source をそのまま file に保存する
- **THEN** preview は save の有無に関係なく current buffer を反映する
