## Definition of Ready (DoR)

- [x] proposal.md、design.md、specs が揃っていること
- [x] workspace tab の保存形式、設定、open policy、UI 仕様が明文化されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-22-12-workspace-tabs` またはリリース用統合ブランチ（例: `release/vX.Y.Z`）
- **作業ブランチ**: 標準は `v0-22-12-workspace-tabs-task-x`、リリース用は `feature/v0.22.12-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. 保存モデルと設定

- [x] 1.1 `GlobalWorkspaceState` に `open_workspace_tabs` と `active_workspace` を追加し、既存 `workspace.json` と serde 互換を保つ
- [x] 1.2 `WorkspaceSettings` に `open_workspace_in_tabs` を追加し、default を `true` にする
- [x] 1.3 `workspace.json` が opened workspace tabs と active workspace を保存・復元する回帰テストを追加する
- [x] 1.4 設定の default / 保存 / 復元の回帰テストを追加する

### Definition of Done (DoD)

- [x] 旧 `workspace.json` が読めること
- [x] 新しい `workspace.json` に workspace tab 状態が保存されること
- [x] `open_workspace_in_tabs` の default が `true` であること
- [x] release/v0.22.12 に統合し、対象検証と `just check-local` を通過させること。

## 2. Workspace Open Policy

### Definition of Ready (DoR)

- [x] 前タスクの変更が release/v0.22.12 に統合済みであること。
- [x] release/v0.22.12 上で対象差分を分離して作業すること。

- [x] 2.1 workspace を開く前に、現在 workspace のファイルタブ状態を保存する
- [x] 2.2 `open_workspace_in_tabs=true` の場合、異なる workspace を tab として追加し、既存なら重複せず選択する
- [x] 2.3 `open_workspace_in_tabs=false` かつ workspace tab が1つの場合、現在 tab を新しい workspace で置き換える
- [x] 2.4 `open_workspace_in_tabs=false` かつ workspace tab が複数ある場合、既存数を維持し active tab を新しい workspace で置き換える
- [x] 2.5 workspace tab 選択時に、対象 workspace の既存ファイルタブセッションを復元する
- [x] 2.6 workspace tab close 時に、active / inactive / 最後の1つの挙動を実装する

### Definition of Done (DoD)

- [x] workspace list / history / menu / plus button が同じ open policy を通ること
- [x] 設定 OFF 時に workspace tab 数が増えないこと
- [x] workspace 切替で per-workspace の document tabs が復元されること
- [x] release/v0.22.12 に統合し、対象検証と `just check-local` を通過させること。

## 3. Workspace Tab UI

### Definition of Ready (DoR)

- [x] 前タスクの変更が release/v0.22.12 に統合済みであること。
- [x] release/v0.22.12 上で対象差分を分離して作業すること。

- [x] 3.1 workspace tab 専用の横スクロール行を追加する
- [x] 3.2 workspace tab に workspace 名と右側 close button を表示する
- [x] 3.3 workspace tab の右端に `+` button を表示し、`PickOpenWorkspace` を発火する
- [x] 3.4 workspace tab が幅を超えた場合、スクロールバー非表示で横スクロール可能にする
- [x] 3.5 workspace tab には左右移動ボタンを表示しない
- [x] 3.6 新しく開いた workspace tab の位置へスクロールする
- [x] 3.7 workspace tab を Explorer sidebar より前の最上位 top bar として描画し、ウィンドウ横幅全体を使う

### Definition of Done (DoD)

- [x] ファイルタブと workspace tab が視覚的に混ざらないこと
- [x] workspace tab が workspace sidebar の右側に押し込まれず、ウィンドウ横幅全体に表示されること
- [x] workspace tab の close / select / plus が操作できること
- [x] workspace tab の横スクロールにスクロールバーと左右移動ボタンが出ないこと
- [x] release/v0.22.12 に統合し、対象検証と `just check-local` を通過させること。

## 4. 設定UIと表示文言

### Definition of Ready (DoR)

- [x] 前タスクの変更が release/v0.22.12 に統合済みであること。
- [x] release/v0.22.12 上で対象差分を分離して作業すること。

- [x] 4.1 Workspace 設定画面に `open_workspace_in_tabs` の toggle を追加する
- [x] 4.2 日本語・英語 locale を追加し、他 locale には fallback 可能な文言を入れる
- [x] 4.3 既存の「ワークスペースを開く」menu / history / list の文言と矛盾しないことを確認する

### Definition of Done (DoD)

- [x] 設定ON/OFFが保存され、再起動後の値として復元されること
- [x] 設定OFF時に既存の複数 workspace tab が即時削除されないこと
- [x] release/v0.22.12 に統合し、対象検証と `just check-local` を通過させること。

## 5. 検証

### Definition of Ready (DoR)

- [x] 前タスクの変更が release/v0.22.12 に統合済みであること。
- [x] release/v0.22.12 上で対象差分を分離して作業すること。

- [x] 5.1 workspace tab policy の unit test を追加・通過させる
- [x] 5.2 workspace restore / close / select の app-level test を追加・通過させる
- [x] 5.3 temporary workspace が workspace tab / active workspace に残らない回帰テストを追加・通過させる
- [x] 5.4 workspace tab UI の描画ロジックに対する test を追加・通過させる
- [x] 5.5 `./scripts/openspec validate v0-22-12-workspace-tabs` を通す
- [x] 5.6 `just check-local` を通す

### Definition of Done (DoD)

- [x] OpenSpec validate が成功すること
- [x] `just check-local` が成功すること
- [x] release/v0.22.12 に統合し、対象検証と `just check-local` を通過させること。

---

## 6. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [x] 6.1 ユーザーへ実装完了の報告および動作状況を提示し、起動確認で要件を満たす状態にする
- [x] 6.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する
- [x] 6.3 workspace tab が最上位の横幅全体を使うようにする
- [x] 6.4 仮想 workspace が workspace tab / workspace 復元状態に表示されないようにする
- [x] 6.5 workspace tab を `+` ボタン幅を除いた残り幅で等分表示する
- [x] 6.6 workspace 切替時に global 検索結果を workspace 単位でリセットし、前 workspace の結果が残らないようにする
- [x] 6.7 workspace tab に薄い内側 border を描画し、タブ境界を見やすくする
- [x] 6.8 `workspace_tab_bar.rs` は workspace tab 列の組み立て、`workspace_tab_bar_detail.rs` は詳細描画として責務分離する
- [x] 6.9 workspace tab の border がテーマ上透明に近い場合は、視認できる薄い線色へフォールバックする
- [x] 6.10 workspace tab / document tab の border を共通化し、ホバー時はアクセントカラーへ切り替える
- [x] 6.11 workspace tab の閉じるボタンをホバー時のみ表示し、非表示時も幅を確保する
- [x] 6.12 workspace tab の上下中央を揃え、workspace tab のみ 4px radius にする
- [x] 6.13 document tab の border を親側の矩形に描画し、ホバー判定をタブ全体へ広げる
- [x] 6.14 border 対応では active 背景色を追加せず、親矩形へ border のみ描画する
- [x] 6.15 workspace tab / document tab の閉じるボタンがホバー表示後に実際に閉じることを保証する
- [x] 6.16 ショートカットで document tab を移動したとき、選択中タブの位置へ横スクロールが追従することを保証する
- [x] 6.17 最終 document tab から「次」で先頭へ戻るとき、横スクロール位置が初期側へ戻り、その後の移動でも追従が継続することを保証する
- [x] 6.18 document toolbar のパンくず先頭階層が左端へめり込まず、明示的な左余白を持つことを保証する
- [x] 6.19 document toolbar のパンくずからボタン背景を除去し、テキストと区切りだけにする
- [x] 6.20 document tab がホバー時に閉じるボタンの高さへ引っ張られて上下に拡張しないことを保証する
- [x] 6.21 document tab の親矩形は close control の response 矩形に引っ張られず、平時と close hover 時で高さが一致することを保証する
- [x] 6.22 workspace tab を drag and drop で並び替えできることを保証する
- [x] 6.23 workspace tab の close control に hover したとき、close button 独自の border / frame を描画しないことを保証する
- [x] 6.24 workspace tab の drag ghost が viewport 左上ではなく workspace tab 行上に描画されることを保証する
- [x] 6.25 workspace tab の drag 中に document tab と同じ drop indicator を表示することを保証する

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 7. Final Verification & Release Handoff

- [x] 7.1 `$self-review` の観点で差分と検証結果を確認する
- [x] 7.2 OpenSpec validate と対象テストを通過させる
- [x] 7.3 `just check-local` を通過させる
- [x] 7.4 ユーザー許可に基づき、直前に通した `just check-local` を品質ゲートとして `git commit --no-verify` / `git push --no-verify` を使用する
- [x] 7.5 release/v0.22.12 の PR #284 へ反映する
- [x] 7.6 OpenSpec の delta specs を main specs へ同期し、release PR に含める
- [x] 7.7 OpenSpec 変更を archive へ移動し、release PR に含める
