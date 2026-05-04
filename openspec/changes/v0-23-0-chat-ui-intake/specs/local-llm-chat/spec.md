## ADDED Requirements

### Requirement: user は独立した local LLM chat UI を開けなければならない

システムは、`katana-ui` 内に editor、preview、diagnostics と責務を分け、画面端のアイコン列から開閉できる local LLM chat サイドパネルを提供しなければならない（SHALL）。

#### Scenario: chat UI を開く

- **WHEN** user が画面端の chat アイコンを選択する
- **THEN** system は専用の chat サイドパネルを表示する
- **THEN** system は既存 editor / preview / diagnostics の表示状態を破壊しない

#### Scenario: chat UI を隠す

- **WHEN** user が chat サイドパネルの非表示操作を行う
- **THEN** system は chat サイドパネルを閉じる
- **THEN** system は editor / preview / diagnostics の表示状態を破壊しない

#### Scenario: chat UI を固定表示する

- **WHEN** user が chat サイドパネルを固定表示にする
- **THEN** system は他の操作中も chat サイドパネルを表示し続ける

### Requirement: chat は Ollama provider を通じて message を送受信できなければならない

システムは、user の chat message を configured Ollama provider に送り、assistant response を app session 内のチャットメッセージに追加できなければならない（SHALL）。

#### Scenario: message を送信する

- **WHEN** user が chat message を送信する
- **THEN** system は active Ollama provider へ generation request を送信する
- **THEN** system は response を assistant message として app session 内のチャットメッセージに追加する

### Requirement: MVP のチャット履歴は app session 内の一時状態でなければならない

システムは、MVP ではチャットメッセージをアプリ起動中の一時状態として保持し、履歴の永続化、一覧、検索、削除管理を提供してはならない（SHALL / MUST NOT）。

#### Scenario: app session 中に chat を続ける

- **WHEN** user が同じ app session 内で chat UI を使う
- **THEN** system はその session の chat messages を表示できる
- **THEN** system は user が続けて request を送信できる

#### Scenario: app を再起動する

- **WHEN** user が app を終了して再起動する
- **THEN** system は過去のチャット履歴を復元しない

#### Scenario: チャット履歴を管理する

- **WHEN** user が MVP で過去 session のチャット履歴一覧、検索、削除管理を探す
- **THEN** system はその管理 UI を提供しない

### Requirement: chat は document mutation と分離されなければならない

システムは、chat response を user の明示 action なしに active document や workspace file へ反映してはならない（MUST NOT）。

#### Scenario: assistant response を受け取る

- **WHEN** system が assistant response を受け取る
- **THEN** system は response を chat UI 上に表示する
- **THEN** system は user が明示的に apply / insert / save を選ぶまで document を変更しない

### Requirement: provider unavailable 時は chat request を送信してはならない

システムは、Ollama provider が未設定または unavailable の場合、chat request を送信してはならず、設定または接続確認への導線を表示しなければならない（MUST NOT / SHALL）。

#### Scenario: provider unavailable で message を送信する

- **WHEN** user が provider 未設定または unavailable 状態で chat message を送信する
- **THEN** system は request を送信しない
- **THEN** system は provider 設定または接続確認へ戻る導線を表示する
