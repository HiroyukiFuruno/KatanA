## Why

現在の KatanA は、別の workspace を開くと作業対象が置き換わるため、複数プロジェクトを行き来する作業でコンテキストが途切れる。workspace 自体をタブとして扱い、ファイルタブと同じ感覚で切り替えられるようにする。

## What Changes

- workspace 一覧または履歴から、現在と異なる workspace を開いた場合、設定に応じて workspace tab として追加する。
- 開いた workspace tab と active workspace を `workspace.json` に保持し、再起動時に復元する。
- workspace tab はファイルタブと同様にタブ名の右へ閉じるボタンを表示する。
- workspace tab 列の一番右に `+` ボタンを置き、メニューの「ワークスペースを開く」と同じ処理で新規 workspace tab を追加できるようにする。
- workspace tab 列は横スクロール可能にし、スクロールバーと左右移動ボタンは表示しない。
- 新しく workspace を開いた場合は、新しく開いた workspace tab の位置へスクロールする。
- 設定に「workspace をタブで開く」既定動作を追加し、default は `true` とする。
- 設定が `false` の場合、新規に異なる workspace を開いても workspace tab を増やさず、現在の workspace を置き換える。
- 設定が `false` の時点で複数 workspace tab が既に存在する場合、既存タブは維持するが、それ以上タブ数が増えないようにする。

## Capabilities

### New Capabilities

- `workspace-tabs`: workspace をタブとして保持・復元・切替・閉じるための挙動。

### Modified Capabilities

- `workspace-shell`: workspace を開く操作が単一 workspace の置き換えだけでなく、設定に応じて workspace tab 追加または置き換えになる。
- `settings-persistence`: workspace tab の既定動作と `workspace.json` の保存対象が追加される。

## Impact

- `katana-platform/src/workspace`: `workspace.json` の保存形式に workspace tab と active workspace を追加する。
- `katana-platform/src/settings`: workspace をタブで開く既定設定を追加する。
- `katana-ui/src/app/workspace`: workspace を開く、保存する、復元する、閉じる処理を拡張する。
- `katana-ui/src/views/top_bar`: workspace tab 列、閉じるボタン、`+` ボタン、横スクロールを追加する。
- `katana-ui/src/settings`: workspace 設定画面へ既定動作の切替を追加する。
