## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.22.7 の変更 ID とスコープが確認されていること
- [ ] KML（katana-markdown-linter）の format API と config API の現在仕様を確認していること
- [ ] 既存の `v0-22-7-fix-preview` 削除差分や `v0-22-8-fix-preview` 未追跡差分を混ぜずに、この変更 ID の範囲だけで作業すること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-22-7-markdownlint-workspace-formatting` またはリリース用統合ブランチ（例: `release/v0.22.7`）
- **作業ブランチ**: 標準は `v0-22-7-markdownlint-workspace-formatting-task-x`、リリース用は `feature/v0.22.7-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## User Feedback / Open Decisions

> ユーザーから受けた要求・未確定点。対応完了したフィードバックは `[/]`、通常タスクは `[x]`、未決は `[ ]` とする。

- [/] 図形描画は markdownlint 正式パターンの `~~~` でも対応する
- [/] ワークスペースの `.markdownlint.json` を見る設定をオン/オフしても、高度な設定画面へ勝手に切り替えない
- [/] Lint の一般設定と `.markdownlint.json` の詳細設定の責務を分ける
- [/] 一般設定の「無視」は詳細設定の履歴を消さず、再度「警告/エラー」に戻した時に復元できるようにする
- [/] グローバル設定とワークスペース設定の概念を入れ、ワークスペース設定を優先する
- [/] KML に effective config を渡す。API がパス非対応なら KatanA 側で構造体へ変換して渡す
- [/] 有効な Markdown ファイルのコンテキストメニューに「ファイルをフォーマットする」を追加する
- [/] エクスプローラー空き領域のコンテキストメニューに「ワークスペース内の Markdown を一括フォーマット」を追加する
- [/] 同じ空き領域メニューに「ファイルの新規作成」「フォルダの新規作成」を追加する
- [/] エクスプローラーのフィルター左にファイル追加・フォルダ追加アイコンを配置する
- [/] 追加アイコンは `katana-icon-management` に従い、各 icon pack の native SVG を使う
- [ ] `.markdownlint.json` に KatanA namespace を保存してよいか、KML と外部 markdownlint の互換性を実装時に確認する
- [ ] KML の format API がファイルパス、文字列、設定構造体のどれを受け取るか実装時に確認する

---

## 1. Diagram Fence Support

- [ ] 1.1 `crates/katana-core` に、`~~~mermaid` / `~~~plantuml` / `~~~drawio` が現在は図形として抽出されないことを示す回帰テストを追加する
- [ ] 1.2 `DiagramSectionOps::try_parse_diagram_fence` を、バッククォートとチルダの両方を扱う設計へ変更する
- [ ] 1.3 `DiagramSectionOps::non_diagram_fence_consume_len` を、非図形フェンスのネスト回避がバッククォートとチルダの両方で成立するように変更する
- [ ] 1.4 `MarkdownFenceOps::extract_fence_block` と `transform_diagram_blocks` を、HTML エクスポートでも `~~~` 図形ブロックを処理できるように変更する
- [ ] 1.5 既存の ` ``` ` 図形ブロック、非図形コードブロック、未閉じフェンスの回帰テストが壊れていないことを確認する

### Definition of Done (DoD)

- [ ] ` ``` ` と `~~~` の `mermaid` / `plantuml` / `drawio` が、プレビューと HTML エクスポートの両方で図形として扱われること
- [ ] `~~~markdown` などの非図形フェンス内にある diagram 例が、図形として誤抽出されないこと
- [ ] `crates/katana-core` の対象テストが通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Lint Settings Ownership and Effective Config

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 現在の `LinterSettings`、`MarkdownLinterConfigOps`、`MarkdownLinterOptionsBridgeOps` の責務を整理し、一般設定と `.markdownlint.json` 設定の境界をテストで固定する
- [ ] 2.2 ワークスペース設定のオン/オフ切り替えで `linter_advanced_is_open` が変更されない回帰テストを追加する
- [ ] 2.3 `workspace > global > default` の優先順位で effective config を解決するサービスを追加または整理する
- [ ] 2.4 一般設定の `RuleSeverity::Ignore` が、詳細設定を削除せず KatanA 側の診断抑制として働くことを実装する
- [ ] 2.5 `Ignore` から `Warning` / `Error` に戻した時、保持していた詳細設定が復元されることを実装する
- [ ] 2.6 `.markdownlint.json` に KatanA namespace を保存できるか検証し、不可の場合は既存 workspace state に重大度だけを保存する方針へ確定する
- [ ] 2.7 KML に渡す config を、ファイルパスまたは KML が要求する構造体として確実に渡す
- [ ] 2.8 診断とフォーマットが同じ effective config を使うことをテストする

### Definition of Done (DoD)

- [ ] ワークスペース設定をオンにした場合、ワークスペース直下の `.markdownlint.json` / `.markdownlint.jsonc` が診断に反映されること
- [ ] ワークスペース設定のオン/オフで、高度な設定画面へ勝手に切り替わらないこと
- [ ] 一般設定の「無視 / 警告 / エラー」と `.markdownlint.json` のルール適用設定が混ざって消失しないこと
- [ ] KML に渡される config が、診断とフォーマットで一致していること
- [ ] `crates/katana-ui` の対象テストが通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. KML Formatting Actions

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 KML の format API を確認し、必要なら `katana-markdown-linter` の workspace dependency を更新する
- [ ] 3.2 ファイル単位の Markdown フォーマットを行う action とサービスを追加する
- [ ] 3.3 ワークスペース内の Markdown を一括フォーマットする action とサービスを追加する
- [ ] 3.4 一括フォーマット対象から hidden infrastructure directory を除外する
- [ ] 3.5 フォーマット後にエディタ buffer、保存状態、diagnostics が更新されるようにする
- [ ] 3.6 失敗時は対象ファイルと理由をステータス表示または復旧可能なエラーとして示す

### Definition of Done (DoD)

- [ ] 有効な Markdown ファイルをファイル単位でフォーマットできること
- [ ] ワークスペース内の Markdown を一括フォーマットできること
- [ ] フォーマットは effective config を使い、`.markdownlint.json` の指定を無視しないこと
- [ ] 失敗したファイルがある場合、成功分まで隠さず、失敗件数と理由が分かること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Explorer Context Menus and Creation Shortcuts

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 ファイル行のコンテキストメニューで、有効な Markdown ファイルだけに「ファイルをフォーマットする」を表示する
- [ ] 4.2 エクスプローラーの空き領域にコンテキストメニューを追加する
- [ ] 4.3 空き領域メニューに「ワークスペース内の Markdown を一括フォーマット」「ファイルの新規作成」「フォルダの新規作成」を追加する
- [ ] 4.4 空き領域からの新規作成は、既存のファイル/フォルダ作成モーダルをワークスペース root 指定で再利用する
- [ ] 4.5 エクスプローラーのフィルター左にファイル追加・フォルダ追加アイコンボタンを配置する
- [ ] 4.6 ヘッダーアイコンからの新規作成も、同じ既存モーダルをワークスペース root 指定で再利用する
- [ ] 4.7 UI テストまたは integration test で、メニュー項目の表示条件と action 発行を確認する

### Definition of Done (DoD)

- [ ] ファイル右クリック、空き領域右クリック、ヘッダーアイコンの三つの入口が画面上で確認できること
- [ ] 非 Markdown ファイルにフォーマット操作が表示されないこと
- [ ] 新規作成操作が既存の作成モーダルと同じ validation を使うこと
- [ ] UI スナップショットでフィルター左の二つのアイコンが確認できること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Icon Pack Integration

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 `scripts/download_icon.sh` を使い、`files/file_plus` と `files/folder_plus` 相当の SVG を各 icon pack から取得する
- [ ] 5.2 Feather、Heroicons、Lucide、Material Symbols、Tabler Icons で、それぞれ vendor native の icon name を指定する
- [ ] 5.3 既存 SVG のコピーで代用していないことを確認する
- [ ] 5.4 `crates/katana-ui/src/icon/types.rs` に `FilePlus` と `FolderPlus` を追加する
- [ ] 5.5 `cargo check -p katana-ui` または `make check` で全 icon pack の include が成功することを確認する

### Definition of Done (DoD)

- [ ] すべての icon pack でファイル追加・フォルダ追加アイコンが表示できること
- [ ] 画像で示された「ファイル +」「フォルダ +」の意味が画面上で分かること
- [ ] 追加アイコンが `katana-icon-management` の運用に従っていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. User Review (Pre-Final Phase)

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 6.1 ユーザーへ実装完了の報告および動作状況（UIの場合はスナップショット画像等）の提示を行う
- [ ] 6.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

### Definition of Done (DoD)

- [ ] ユーザーの確認が完了し、フィードバックの修正が Base ブランチにマージされていること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 7. Final Verification & Release Work

- [ ] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 7.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 7.3 Ensure `make check` passes with exit code 0
- [ ] 7.4 Create PR from Base Feature Branch targeting `master`
- [ ] 7.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 7.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.7 Create `release/v0.22.7` branch from master
- [ ] 7.8 Run `make release VERSION=0.22.7` and update CHANGELOG (`changelog-writing` skill)
- [ ] 7.9 Create PR from `release/v0.22.7` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 7.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
