## ADDED Requirements

### Requirement: shell.rs のロジック分離

`shell.rs` 内の純粋ロジック（egui Context に依存しない関数・メソッド）を別モジュールに分離しなければならない（SHALL）。

#### Scenario: hash_str のテスト可能性

- **WHEN** `hash_str` 関数を単独で呼び出す
- **THEN** egui への依存なくテスト実行でき、FNV-1a ハッシュの期待値と一致する

#### Scenario: relative_full_path のテスト

- **WHEN** `relative_full_path` 関数にパスとワークスペースルートを渡す
- **THEN** 正しい相対パス文字列を返す

#### Scenario: process_action のロジックテスト

- **WHEN** `AppAction::CloseDocument(idx)` を処理する
- **THEN** `open_documents` から指定インデックスのドキュメントが削除され、`active_doc_idx` が適切に更新される

### Requirement: egui_kittest による E2E テスト

`egui_kittest::Harness` を使用し、ユーザーシナリオを自動検証する E2E テストを導入しなければならない（SHALL）。

#### Scenario: ワークスペースを開いてファイルを選択する

- **WHEN** Harness でアプリを起動し、ワークスペースを設定してファイルを選択する操作をシミュレーションする
- **THEN** プレビューペインにファイル内容が表示される

#### Scenario: タブの開閉操作

- **WHEN** 複数ファイルを開いた状態でタブの閉じるボタンをクリックする
- **THEN** タブが閉じられ、残りのタブにフォーカスが移る

#### Scenario: ビューモード切替

- **WHEN** ビューモードバーで「Split」を選択する
- **THEN** エディタとプレビューが左右に分割表示される

### Requirement: UI スナップショット回帰テスト

`egui_kittest` の `snapshot` 機能を使用し、UI レンダリング結果の回帰を検知しなければならない（SHALL）。

#### Scenario: スナップショット比較テスト

- **WHEN** `render_tab_bar` をヘッドレスコンテキストでレンダリングし、スナップショットと比較する
- **THEN** 前回の承認済みスナップショットと差分がない（または許容範囲内）

#### Scenario: スナップショット更新ワークフロー

- **WHEN** 意図的な UI 変更を行い `UPDATE_SNAPSHOTS=true cargo test` を実行する
- **THEN** 新しいスナップショットが生成され、次回以降のテストの基準となる
