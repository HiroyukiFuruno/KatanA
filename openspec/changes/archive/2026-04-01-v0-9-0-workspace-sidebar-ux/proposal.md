# OpenSpec Change Proposal: ワークスペースサイドバー UX 改善 (v0.9.0)

## 背景

現在のワークスペースサイドバーは、タイトル文字列と複数の小型アイコンを同じヘッダー周辺に集約しており、検索・履歴・表示切り替えといった頻出操作の発見性と操作効率が低い。ワークスペースは KatanA の主要導線であるため、`v0.9.0` では左側アクティビティレールへの整理とヘッダーの再配置によって、より迷いにくい UI に更新する。

## 変更内容

- ワークスペースペインから `Workspace` / `ワークスペース` の見出し文言を削除し、表示領域を操作とツリー本体に優先配分する。
- 添付イメージのような左側アクティビティレールを追加し、ワークスペース表示切り替え、ファイル検索、最近のワークスペース履歴をそこから呼び出せるようにする。
- 左側アクティビティレールのアイコンは既存資産をそのまま再利用し、現行ヘッダー内の小型ボタンより一段大きく描画する。
- 現ワークスペースの操作ヘッダーを再構成し、更新を左寄せ、全展開・全閉を右寄せに再配置する。
- ワークスペース一覧の並び順を version-aware sort へ更新し、`v0-9-x` と `v0-11-x` のような数値を含む名前でも自然順で表示する。
- フィルター機能はワークスペースヘッダー内に残し、現行の正規表現入力 UI と挙動を維持する。
- ワークスペースヘッダーに新しい `...` メニューを追加し、`表示 -> フラット表示` から tree / flat の表示モードを切り替えられるようにする。
- flat 表示はディレクトリ概念を持たない file 単位の一覧とし、flat 表示フラグの既定値は `false`、既定表示は tree とする。選択状態は workspace ごとに永続化する。
- 履歴ボタンは履歴 0 件でもレール内に残し、非活性表示でレイアウトの安定性を保つ。
- 既存の collapsed toggle 専用サイドパネルは廃止し、レールが常時その役割を担う。

## ケイパビリティ

### 追加されるケイパビリティ

- `workspace-activity-rail`: 左側アクティビティレールでワークスペース表示切り替えと最近のワークスペース履歴を提供する。

### 変更されるケイパビリティ

- `workspace-shell`: ワークスペースペインのタイトル表示を廃止し、ヘッダー操作、表示モード切り替え、一覧 sort order を再構成する。
- `workspace-file-search`: ファイル検索モーダルを左側アクティビティレールから起動できるようにする。
- `workspace-file-filter`: フィルタートグルと入力 UI を新しいヘッダー配置に合わせて維持する。

## 影響範囲

- 主な影響範囲は `crates/katana-ui/src/views/app_frame.rs`、`crates/katana-ui/src/views/panels/workspace/ui.rs`、`crates/katana-ui/src/views/panels/workspace/logic.rs`、`crates/katana-ui/src/state/workspace.rs`、`crates/katana-platform/src/filesystem/scanner.rs`、`crates/katana-ui/locales/*.json`。
- 最近のワークスペース履歴は既存の `settings.workspace.paths` を再利用し、flat / tree の表示選択は workspace ごとに既存設定へ追記する形で保持する。
- 既存の検索モーダル、フィルター、ワークスペース再読み込み、全展開・全閉のロジックは維持しつつ、sort comparator と表示モード projection を追加する。
- 実装は新しい左レール component、既存 `WorkspacePanel` ヘッダー整理、workspace 一覧の sort / flat projection 追加に分け、検索モーダルや履歴 action 自体は再利用する。
