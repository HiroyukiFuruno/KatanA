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
- [ ] エディタ左端の Lint アイコンをホバーしても診断内容がポップ表示されない
- [/] 行番号横の Lint アイコンは、多行診断でも問題 view と同じく診断の開始行だけに表示する
- [ ] Task 2 着手前に、既存の Lint 設定 UI を前提にせず、設定画面全体の情報設計と操作導線を見直す
- [ ] Lint 設定 UI は、通常の操作では設定 JSON をユーザーに意識させない設計思想を維持する
- [ ] 詳しいユーザー向けに、KatanA 管理の共通ルールをワークスペースのルールとして展開する導線を用意する
- [ ] ワークスペースに既存の markdownlint ルールファイルがある場合は、そのワークスペースのルールとして利用する
- [ ] Lint の高度な設定は、アイコン設定の高度な設定と操作パターンを揃えつつ、内容は Lint ルール詳細として最適化する
- [ ] Lint プリセットは、テーマ/アイコンと同じく選べるが、適用後は現在のルールへコピーするテンプレートとして扱う
- [ ] 組み込みプリセットとして `KatanA`、`全て無効`、`厳格`、`すべて警告` を用意する
- [ ] 現在のルールをユーザープリセットとして保存し、他ワークスペースでもテンプレートとして利用できるようにする
- [ ] テーマ、アイコン、Lint は異なる保存仕様のまま拡張せず、同じプリセット保存仕様と同じ UI/UX へ統一する
- [ ] 統一したプリセット操作は、再利用ウィジェット（widget: 再利用できる画面部品）へ落とし込み、テーマ・アイコン・Lint で使い回す
- [/] コードブロック生成時は、何のコードブロックかをプルダウンで選べるようにする
- [/] コードブロック種別のプルダウンは enum と連動させ、`text`、`markdown`、`bash`、`zsh`、`mermaid`、`drawio`、`plantuml`、開発でよく使う言語を選択肢に含める
- [/] `impl-release` 起動時の可視タスク計画は User Review Phase `6.1` まで含め、個別 Task Group 完了で停止しない
- [/] 依存しない調査・実装・検証・ハーネス更新は、補助エージェント（subagent）へ書き込み範囲を分離して移譲する
- [/] Task ごとの通常 PR push は `pre-push` hook を正式な品質ゲートとして通し、push 直前の重い `make check` / `make check-light` 二重実行や `--no-verify` 回避を原則禁止する
- [/] Task 2 のように大きすぎる Task Group は、計画段階で `2A` / `2B` / `2C` のように責務・依存関係・書き込み範囲ごとに分割する

---

## 1. Diagram Fence Support

- [x] 1.1 `crates/katana-core` に、`~~~mermaid` / `~~~plantuml` / `~~~drawio` が現在は図形として抽出されないことを示す回帰テストを追加する
- [x] 1.2 `DiagramSectionOps::try_parse_diagram_fence` を、バッククォートとチルダの両方を扱う設計へ変更する
- [x] 1.3 `DiagramSectionOps::non_diagram_fence_consume_len` を、非図形フェンスのネスト回避がバッククォートとチルダの両方で成立するように変更する
- [x] 1.4 `MarkdownFenceOps::extract_fence_block` と `transform_diagram_blocks` を、HTML エクスポートでも `~~~` 図形ブロックを処理できるように変更する
- [x] 1.5 既存の ````` 図形ブロック、非図形コードブロック、未閉じフェンスの回帰テストが壊れていないことを確認する
- [x] 1.6 コードブロック生成時に、enum と連動したプルダウンで `text` / `markdown` / `bash` / `zsh` / `mermaid` / `drawio` / `plantuml` / 主要な開発言語を選べるようにする

### Definition of Done (DoD)

- [x] ````` と `~~~` の `mermaid` / `plantuml` / `drawio` が、プレビューと HTML エクスポートの両方で図形として扱われること
- [x] `~~~markdown` などの非図形フェンス内にある diagram 例が、図形として誤抽出されないこと
- [x] `crates/katana-core` の対象テストが通過すること
- [x] コードブロック生成 UI で、コード種別をプルダウンから選択して fenced code block を挿入できること
- [x] プルダウンの選択肢が enum の定義と一致し、表示名と挿入される fence info string がずれないこと
- [x] `mermaid` / `drawio` / `plantuml` を選んだ場合、生成後のコードブロックが Task 1 の図形描画プレビュー対象として扱われること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Settings Preset Ownership and Effective Config

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### AsIs / ToBe 差分

> Task 2 は、現状（AsIs）とあるべき状態（ToBe）の乖離だけを実装対象にする。既に ToBe を満たしている挙動は、必要に応じて回帰テストだけを追加する。

| ID | 現状（AsIs） | あるべき状態（ToBe） | 実装タスク |
| --- | --- | --- | --- |
| D2-1 | 通常設定で保存先 JSON が操作対象に見えやすい | 通常画面では「共通ルール」「このワークスペースのルール」「ルールの詳細」として操作する | 2B.1, 2B.6, 2B.7 |
| D2-2 | 一般設定の重大度と markdownlint の詳細設定が混ざり、設定消失が起き得る | 「無視 / 警告 / エラー」とルール詳細を分離し、詳細設定の履歴を保持する | 2B.2, 2B.4, 2B.5 |
| D2-3 | ワークスペース設定のオン/オフで高度な設定へ勝手に切り替わる | オン/オフしても通常画面に留まり、診断だけ更新する | 2B.3 |
| D2-4 | 共通設定、ワークスペース設定、既定値の優先順位が曖昧 | `workspace > global > default` で effective config を解決する | 2B.1, 2B.8 |
| D2-5 | 既存のワークスペースルールを使うのか、KatanA 管理設定で上書きするのかが曖昧 | 既存のワークスペースルールは現在ルールとして読み込み、明示操作なしに上書きしない | 2B.7, 2B.9 |
| D2-6 | プリセットがなく、共通設定や別ワークスペースへ設定を再利用しにくい | 組み込みプリセットとユーザープリセットをテンプレートとして適用できる | 2A.1, 2C.4, 2C.5, 2C.6 |
| D2-7 | Lint の高度な設定が、アイコン設定と同じ操作導線であるか整理されていない | 操作パターンはアイコン設定に揃え、内容は Lint ルール詳細として最適化する | 2C.7 |
| D2-8 | 行番号横の Lint アイコンをホバーしても診断内容が出ない | ホバー時に対象診断の内容をポップ表示する | 2D.1 |
| D2-9 | 多行診断で行番号横のアイコンが各行に出て主張が強い | アイコンは診断開始行だけに表示し、波線は対象範囲全体に残す | 2D.2 |
| D2-10 | テーマ、アイコン、Lint のプリセット保存仕様と UI/UX が対象ごとに違い、カスタム状態の由来が曖昧 | すべて同じ保存仕様と同じプリセット操作へ統一し、共通ウィジェットで扱う | 2A.1, 2A.2, 2A.3, 2A.4, 2C.1, 2C.2, 2C.3 |

### Task 2 分割方針

Task 2 は大きすぎるため、1ブランチに詰め込まず、以下のサブタスクとして扱う。`2A` と `2B` は並列実装可能、`2C` は `2A` 完了後に着手、`2D` は設定保存と独立して進められる。各サブタスクは個別ブランチ、個別PR、個別の `/openspec-delivery` 対象にする。

| サブタスク | 目的 | 依存関係 | 主な書き込み範囲 |
| --- | --- | --- | --- |
| 2A | テーマ・アイコン・Lint の共通プリセット保存仕様と既存値移行を先に固める | Task 1 完了後に開始可 | `crates/katana-platform/src/settings/**`、設定migrationテスト |
| 2B | Lint の effective config とワークスペース/共通/既定値の優先順位を固める | Task 1 完了後に開始可。2Aとは並列可 | `crates/katana-ui/src/linter_*`、`crates/katana-platform/src/settings/types/linter.rs`、KML連携テスト |
| 2C | 共通プリセットウィジェットを作り、テーマ・アイコン・Lint の設定画面へ接続する | 2A 完了後。Lint接続は2Bの結果に合わせる | `crates/katana-ui/src/settings/**`、設定UIテスト |
| 2D | Lint 診断アイコンの表示とホバー表示を直す | Task 1 完了後に開始可。2A/2B/2Cとは独立 | `crates/katana-ui/src/views/panels/editor/*diagnostic*`、エディタUIテスト |

### 2A. Preset State Model and Migration

- [ ] 2A.1 テーマ、アイコン、Lint で共通利用するプリセット保存仕様を定義し、現在値、元プリセット、変更状態、ユーザープリセット一覧を同じ構造で扱う
- [ ] 2A.2 既存保存値の移行を fixture 付きで実装し、テーマの `preset` / `custom_color_overrides` / `active_custom_theme`、`theme.icon_pack`、アイコンの `active_preset` / `active_overrides` / `custom_presets` が見た目を変えずに統一保存仕様へ移行することをテストする
- [ ] 2A.3 Lint の既存 `enabled` / `use_workspace_local_config` / `rule_severity` が、統一保存仕様導入後も同じ診断状態を保つことをテストする
- [ ] 2A.4 移行後の保存で、元プリセット不明のカスタム状態を根拠なく既存プリセット扱いにしない

### 2B. Lint Effective Config and Workspace Ownership

- [ ] 2B.1 現在の `LinterSettings`、`MarkdownLinterConfigOps`、`MarkdownLinterOptionsBridgeOps` の責務を整理し、`workspace > global > default` の優先順位で effective config を解決する単一入口を追加または整理する
- [ ] 2B.2 一般設定の「無視 / 警告 / エラー」と `.markdownlint.json` のルール適用設定の境界をテストで固定する
- [ ] 2B.3 ワークスペース設定のオン/オフ切り替えで `linter_advanced_is_open` が変更されない回帰テストを追加する
- [ ] 2B.4 一般設定の `RuleSeverity::Ignore` が、詳細設定を削除せず KatanA 側の診断抑制として働くことを実装する
- [ ] 2B.5 `Ignore` から `Warning` / `Error` に戻した時、保持していた詳細設定が復元されることを実装する
- [ ] 2B.6 `.markdownlint.json` に KatanA namespace を保存できるか検証し、不可の場合は既存 workspace state に重大度だけを保存する方針へ確定する
- [ ] 2B.7 KatanA 管理の共通ルールを、ワークスペースのルールとして展開する操作を設計・実装する
- [ ] 2B.8 KML に渡す config を、ファイルパスまたは KML が要求する構造体として確実に渡し、診断とフォーマットが同じ effective config を使うことをテストする
- [ ] 2B.9 既存のワークスペースルールがある場合に、それを優先して読み込み、設定画面を開いただけでプリセットや共通ルールに上書きしないことをテストする

### 2C. Shared Preset Widget and Settings UI

- [ ] 2C.1 通常画面で設定 JSON を意識させない前提で、設定画面上の主語を「共通ルール」「このワークスペースのルール」「ルールの詳細」に整理する
- [ ] 2C.2 プリセット一覧、現在状態、保存、元へ戻す、詳細設定入口を持つ再利用ウィジェットを追加する
- [ ] 2C.3 テーマ設定を統一プリセット保存仕様と再利用ウィジェットへ移行し、既存のカスタムテーマをユーザープリセットとして扱う
- [ ] 2C.4 アイコン設定を統一プリセット保存仕様と再利用ウィジェットへ移行し、icon pack と個別上書きを一つの現在値として扱う
- [ ] 2C.5 Lint 設定を統一プリセット保存仕様と再利用ウィジェットへ接続し、プリセット適用を現在の共通ルールまたはワークスペースルールへコピーする導線として実装する
- [ ] 2C.6 `KatanA`、`全て無効`、`厳格`、`すべて警告` の Lint 組み込みプリセットを、統一プリセット保存仕様の組み込みプリセットとして定義し、適用結果をテストする
- [ ] 2C.7 現在の Lint ルールをユーザープリセットとして保存し、別ワークスペースへテンプレート適用できることを実装・テストする
- [ ] 2C.8 Lint の高度な設定導線を、アイコン設定の高度な設定と同じ全高パネル、検索、展開/折りたたみ、閉じる操作に揃える

### 2D. Editor Diagnostics UI

- [ ] 2D.1 エディタ左端の Lint アイコンをホバーした時、対象診断の内容がポップ表示されることを実装・テストする
- [ ] 2D.2 多行診断の Lint アイコンを診断の開始行だけに表示し、波線は対象範囲全体に残す

### Definition of Done (DoD)

- [ ] ワークスペース設定をオンにした場合、ワークスペース直下の `.markdownlint.json` / `.markdownlint.jsonc` が診断に反映されること
- [ ] ワークスペース設定のオン/オフで、高度な設定画面へ勝手に切り替わらないこと
- [ ] 一般設定の「無視 / 警告 / エラー」と `.markdownlint.json` のルール適用設定が混ざって消失しないこと
- [ ] KML に渡される config が、診断とフォーマットで一致していること
- [ ] Lint アイコンのホバーで、診断メッセージが画面上に確認できること
- [ ] 多行診断で、行番号横の Lint アイコンが診断の開始行だけに表示されること
- [ ] 通常の設定画面では JSON ファイル名を主操作対象として表示しないこと
- [ ] ワークスペースへルールを展開した後、そのワークスペースのルールとして診断とフォーマットに反映されること
- [ ] Lint の高度な設定が、アイコン設定と同じ操作パターンで開閉・検索・展開できること
- [ ] プリセット適用後に個別ルールを変更しても、組み込みプリセット自体が変更されないこと
- [ ] ユーザープリセットが他ワークスペースでもテンプレートとして選べること
- [ ] 既存のワークスペースルールが、設定画面表示やプリセット一覧表示だけで上書きされないこと
- [ ] テーマ、アイコン、Lint が同じプリセット保存仕様を使い、元プリセットと変更状態を画面上で確認できること
- [ ] テーマ、アイコン、Lint のプリセット操作が同じ再利用ウィジェットで表示されること
- [ ] 既存のテーマ・アイコン保存値が、根拠なく別プリセット扱いに移行されないこと
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

- [ ] 6.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot/run.sh --request <request.json> --output scripts/screenshot/output/v0-22-7-review` で生成したスクリーンショットまたは動画を提示して確認できる状態にする。シナリオ定義は git 管理対象、生成物は `.gitignore` 対象にする
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
- [ ] 7.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `make check` / `make check-light` を二重実行しない
- [ ] 7.4 Create PR from Base Feature Branch targeting `master`
- [ ] 7.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 7.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.7 Create `release/v0.22.7` branch from master
- [ ] 7.8 Run `make release VERSION=0.22.7` and update CHANGELOG (`changelog-writing` skill)
- [ ] 7.9 Create PR from `release/v0.22.7` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 7.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
