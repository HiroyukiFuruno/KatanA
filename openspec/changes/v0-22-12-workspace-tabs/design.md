## Context

KatanA の現在の workspace は、アプリ全体に対して1つだけ読み込まれる作業ルートである。ファイルタブ、エクスプローラー、診断、検索、プレビューは active workspace に結びついている。

一方で、workspace 履歴と保存済み workspace は既に存在するため、ユーザーは複数 workspace を行き来する入口を持っている。今回の変更では、workspace 自体をタブとして保持し、active workspace の切替時に各 workspace のファイルタブ状態を保存・復元する。

## Goals / Non-Goals

**Goals:**

- 複数 workspace を workspace tab として保持し、クリックで active workspace を切り替えられる。
- workspace tab と active workspace を `workspace.json` に保存し、次回起動時に復元する。
- workspace を開く既定動作を設定で切り替えられる。
- 設定が OFF の場合は新規 workspace tab を増やさず、既存状態を壊さない。
- workspace tab 列に閉じるボタン、`+` ボタン、非表示スクロールバーの横スクロールを提供する。

**Non-Goals:**

- 複数 workspace のファイルツリーを同時にメモリ上へ完全ロードすること。
- workspace ごとにエクスプローラーや診断を同時表示すること。
- workspace tab のドラッグ並び替え、グループ化、ピン留め。
- ファイルタブと workspace tab を同一コレクションへ統合すること。

## Decisions

1. `GlobalWorkspaceState` に `open_workspace_tabs: Vec<String>` と `active_workspace: Option<String>` を追加する。

   既存の `persisted` と `histories` は登録・履歴の意味を持つため、開いている workspace tab の状態を混ぜない。`workspace.json` の責務内に新しい明示フィールドを追加することで、復元対象と履歴対象を分ける。

2. `WorkspaceState` は active workspace の読み込み結果だけを持つ。

   workspace tab は root path の一覧として保持し、切替時に現在 workspace のファイルタブ状態を既存の workspace session 保存へ書き出してから、選択先 workspace を読み込む。これにより、既存のエクスプローラー・診断・プレビューの前提を保てる。

3. workspace を開く処理は `open_workspace_in_tabs` 設定で分岐する。

   `true` の場合は未登録の workspace を `open_workspace_tabs` に追加し、既存なら重複追加せず選択する。`false` の場合は workspace tab 数を増やさない。既に複数 tab がある場合は現状の数を維持し、active tab の位置を新しい workspace で置き換える。

4. workspace tab UI はファイルタブ列とは別の横スクロール行として配置する。

   ファイルタブは document model に依存しており、workspace tab と同じ配列へ混ぜると保存・閉じる・選択・ドラッグの責務が崩れる。workspace tab 専用行にすることで、閉じるボタンと `+` ボタンを独立して扱える。

5. `+` ボタンは `PickOpenWorkspace` と同じ action を発火する。

   新規 workspace 選択の入口を増やすだけで、実際の open policy は既存の workspace open pipeline に集約する。

## Risks / Trade-offs

- [Risk] workspace 切替時に未保存ファイルタブの状態が失われる  
  → 切替前に既存の `save_workspace_state` を必ず呼び、active workspace ごとのファイルタブ状態を保存する。

- [Risk] 設定 OFF 時に既存の複数 workspace tab が消える  
  → 設定変更時には既存 tab を変えず、新規 open 操作時だけ増加を抑制する。

- [Risk] 起動時復元で存在しない workspace path を active にしようとする  
  → `workspace.json` の復元時に存在しない path を除外し、active workspace が無効なら残った先頭 workspace を選ぶ。

- [Risk] workspace tab のスクロール位置が新規追加先へ移動しない  
  → 新規 workspace open 時に一時 UI フラグを立て、該当 tab の描画時に `scroll_to_me` を呼ぶ。
