## Definition of Ready (DoR)

- [ ] proposal.md、design.md、specs が揃っていること
- [ ] workspace tab の保存形式、設定、open policy、UI 仕様が明文化されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-22-12-workspace-tabs` またはリリース用統合ブランチ（例: `release/vX.Y.Z`）
- **作業ブランチ**: 標準は `v0-22-12-workspace-tabs-task-x`、リリース用は `feature/v0.22.12-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. 保存モデルと設定

- [ ] 1.1 `GlobalWorkspaceState` に `open_workspace_tabs` と `active_workspace` を追加し、既存 `workspace.json` と serde 互換を保つ
- [ ] 1.2 `WorkspaceSettings` に `open_workspace_in_tabs` を追加し、default を `true` にする
- [ ] 1.3 `workspace.json` が opened workspace tabs と active workspace を保存・復元する回帰テストを追加する
- [ ] 1.4 設定の default / 保存 / 復元の回帰テストを追加する

### Definition of Done (DoD)

- [ ] 旧 `workspace.json` が読めること
- [ ] 新しい `workspace.json` に workspace tab 状態が保存されること
- [ ] `open_workspace_in_tabs` の default が `true` であること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. Workspace Open Policy

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 workspace を開く前に、現在 workspace のファイルタブ状態を保存する
- [ ] 2.2 `open_workspace_in_tabs=true` の場合、異なる workspace を tab として追加し、既存なら重複せず選択する
- [ ] 2.3 `open_workspace_in_tabs=false` かつ workspace tab が1つの場合、現在 tab を新しい workspace で置き換える
- [ ] 2.4 `open_workspace_in_tabs=false` かつ workspace tab が複数ある場合、既存数を維持し active tab を新しい workspace で置き換える
- [ ] 2.5 workspace tab 選択時に、対象 workspace の既存ファイルタブセッションを復元する
- [ ] 2.6 workspace tab close 時に、active / inactive / 最後の1つの挙動を実装する

### Definition of Done (DoD)

- [ ] workspace list / history / menu / plus button が同じ open policy を通ること
- [ ] 設定 OFF 時に workspace tab 数が増えないこと
- [ ] workspace 切替で per-workspace の document tabs が復元されること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. Workspace Tab UI

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 workspace tab 専用の横スクロール行を追加する
- [ ] 3.2 workspace tab に workspace 名と右側 close button を表示する
- [ ] 3.3 workspace tab の右端に `+` button を表示し、`PickOpenWorkspace` を発火する
- [ ] 3.4 workspace tab が幅を超えた場合、スクロールバー非表示で横スクロール可能にする
- [ ] 3.5 workspace tab には左右移動ボタンを表示しない
- [ ] 3.6 新しく開いた workspace tab の位置へスクロールする
- [ ] 3.7 workspace tab を Explorer sidebar より前の最上位 top bar として描画し、ウィンドウ横幅全体を使う

### Definition of Done (DoD)

- [ ] ファイルタブと workspace tab が視覚的に混ざらないこと
- [ ] workspace tab が workspace sidebar の右側に押し込まれず、ウィンドウ横幅全体に表示されること
- [ ] workspace tab の close / select / plus が操作できること
- [ ] workspace tab の横スクロールにスクロールバーと左右移動ボタンが出ないこと
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 4. 設定UIと表示文言

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 Workspace 設定画面に `open_workspace_in_tabs` の toggle を追加する
- [ ] 4.2 日本語・英語 locale を追加し、他 locale には fallback 可能な文言を入れる
- [ ] 4.3 既存の「ワークスペースを開く」menu / history / list の文言と矛盾しないことを確認する

### Definition of Done (DoD)

- [ ] 設定ON/OFFが保存され、再起動後の値として復元されること
- [ ] 設定OFF時に既存の複数 workspace tab が即時削除されないこと
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 5. 検証

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 workspace tab policy の unit test を追加・通過させる
- [ ] 5.2 workspace restore / close / select の app-level test を追加・通過させる
- [ ] 5.3 temporary workspace が workspace tab / active workspace に残らない回帰テストを追加・通過させる
- [ ] 5.4 workspace tab UI の描画ロジックに対する test を追加・通過させる
- [ ] 5.5 `./scripts/openspec validate v0-22-12-workspace-tabs` を通す
- [/] 5.6 `just check-local` を通す

### Definition of Done (DoD)

- [ ] OpenSpec validate が成功すること
- [ ] `just check-local` が成功すること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 6.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする
- [ ] 6.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）
- [/] 6.3 workspace tab が最上位の横幅全体を使うようにする
- [/] 6.4 仮想 workspace が workspace tab / workspace 復元状態に表示されないようにする
- [/] 6.5 workspace tab を `+` ボタン幅を除いた残り幅で等分表示する
- [/] 6.6 workspace 切替時に global 検索結果を workspace 単位でリセットし、前 workspace の結果が残らないようにする
- [/] 6.7 workspace tab に薄い内側 border を描画し、タブ境界を見やすくする
- [/] 6.8 `workspace_tab_bar.rs` は workspace tab 列の組み立て、`workspace_tab_bar_detail.rs` は詳細描画として責務分離する
- [/] 6.9 workspace tab の border がテーマ上透明に近い場合は、視認できる薄い線色へフォールバックする
- [/] 6.10 workspace tab / document tab の border を共通化し、ホバー時はアクセントカラーへ切り替える
- [/] 6.11 workspace tab の閉じるボタンをホバー時のみ表示し、非表示時も幅を確保する
- [/] 6.12 workspace tab の上下中央を揃え、workspace tab のみ 4px radius にする
- [/] 6.13 document tab の border を親側の矩形に描画し、ホバー判定をタブ全体へ広げる
- [/] 6.14 border 対応では active 背景色を追加せず、親矩形へ border のみ描画する
- [/] 6.15 workspace tab / document tab の閉じるボタンがホバー表示後に実際に閉じることを保証する
- [/] 6.16 ショートカットで document tab を移動したとき、選択中タブの位置へ横スクロールが追従することを保証する
- [/] 6.17 最終 document tab から「次」で先頭へ戻るとき、横スクロール位置が初期側へ戻り、その後の移動でも追従が継続することを保証する
- [/] 6.18 document toolbar のパンくず先頭階層が左端へめり込まず、明示的な左余白を持つことを保証する
- [/] 6.19 document toolbar のパンくずからボタン背景を除去し、テキストと区切りだけにする
- [/] 6.20 document tab がホバー時に閉じるボタンの高さへ引っ張られて上下に拡張しないことを保証する
- [/] 6.21 document tab の親矩形は close control の response 矩形に引っ張られず、平時と close hover 時で高さが一致することを保証する
- [/] 6.22 workspace tab を drag and drop で並び替えできることを保証する
- [/] 6.23 workspace tab の close control に hover したとき、close button 独自の border / frame を描画しないことを保証する
- [/] 6.24 workspace tab の drag ghost が viewport 左上ではなく workspace tab 行上に描画されることを保証する
- [/] 6.25 workspace tab の drag 中に document tab と同じ drop indicator を表示することを保証する

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 7. Final Verification & Release Work

- [ ] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `$self-review` skill
- [ ] 7.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 7.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
- [ ] 7.4 Create PR from Base Feature Branch targeting `master`
- [ ] 7.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 7.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.7 Create `release/v0.22.12` branch from master
- [ ] 7.8 Run `just VERSION=0.22.12 release` and update CHANGELOG (`changelog-writing` skill)
- [ ] 7.9 Create PR from `release/v0.22.12` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 7.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
