## ADDED Requirements

### Requirement: Markdown diagnostics を実行できる

システムは、アクティブな Markdown 文書と、必要に応じて関連する workspace 内 Markdown file に対して決定的な diagnostics を評価できなければならない（SHALL）。

#### Scenario: アクティブ文書の diagnostics を実行する

- **WHEN** ユーザーがアクティブ文書に対して Markdown diagnostics を実行した時
- **THEN** システムは、その文書に対して有効な Markdown diagnostic rules を評価する

#### Scenario: 関連する Markdown 対の diagnostics を実行する

- **WHEN** diagnostic rule が `foo.md` と `foo.ja.md` のような関連 Markdown file に依存する時
- **THEN** システムは、その rule に必要な関連 file も評価する

#### Scenario: 初期ルールセットが決定的な文書問題を扱う

- **WHEN** Markdown diagnostics が実行された時
- **THEN** 初期 supported rules には、heading structure check、paired Markdown heading sync check、broken relative links、missing local assets が含まれる

### Requirement: Problems Panel で diagnostics を表示できる

システムは、Markdown diagnostics を専用の Problems Panel に表示できなければならない（SHALL）。

#### Scenario: Diagnostics 一覧を表示する

- **WHEN** 1 件以上の Markdown diagnostic が存在する時
- **THEN** Problems Panel は、各 diagnostic を少なくとも重大度、メッセージ、file、location とともに一覧表示する

#### Scenario: 問題がない場合の empty state を表示する

- **WHEN** Markdown diagnostics が findings なしで完了した時
- **THEN** Problems Panel は古い結果一覧ではなく、明示的な empty state を表示する

### Requirement: Diagnostics から該当箇所へ移動できる

システムは、一覧表示された diagnostic から位置情報へ移動できなければならない（SHALL）。

#### Scenario: Problems Panel から editor へ jump する

- **WHEN** ユーザーが diagnostic item を選択した時
- **THEN** システムは、必要であれば対象文書を開く
- **THEN** システムは、editor を diagnostic location へ移動する

#### Scenario: Problems Panel から preview 導線を得られる

- **WHEN** ユーザーが表示可能な Markdown preview を持つ diagnostic item を選択した時
- **THEN** システムは、preview 上の対応 location を reveal するか、editor jump と preview を同期させる

#### Scenario: location が解決できない場合もクラッシュしない

- **WHEN** diagnostic が、もはや解決できない file または location を指している時
- **THEN** システムは crash しない
- **THEN** ユーザーは回復可能な failure indication を受ける

### Requirement: Diagnostics は明示実行と保存契機で更新できる

システムは、per-keystroke の live analysis を必須にせず、明示実行と保存契機によって Markdown diagnostics を更新できなければならない（SHALL）。

#### Scenario: 手動 refresh で diagnostics を更新する

- **WHEN** ユーザーが diagnostics refresh action を実行した時
- **THEN** システムは Markdown diagnostics を再計算し、Problems Panel を更新する

#### Scenario: Save 後に diagnostics を更新する

- **WHEN** ユーザーが Markdown document を保存した時
- **THEN** システムは、その保存済み文書に関連する diagnostics を更新する
