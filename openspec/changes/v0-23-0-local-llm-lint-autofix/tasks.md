## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.23.0 の変更 ID とスコープが確認されていること
- [ ] `v0.19.0` の markdownlint 検知結果データ構造 (diagnostics payload) が安定利用できる状態であること
- [ ] 初期の local LLM 接続先は Ollama 経由に限定する前提が確認されていること
- [ ] `katana-ui` 内に chat UI を既存 editor / preview / diagnostics と分離して置く前提が確認されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-23-0-local-llm-lint-autofix` またはリリース用統合ブランチ（例: `release/v0.23.0`）
- **作業ブランチ**: 標準は `v0-23-0-local-llm-lint-autofix-task-x`、リリース用は `feature/v0.23.0-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## User Feedback / Open Decisions

> ユーザーから受けた要求・未確定点。対応完了したフィードバックは `[/]`、通常タスクは `[x]`、未決は `[ ]` とする。

- [/] 初期接続先は Ollama 経由の local LLM とし、KatanA 独自の外部 AI IF の一般化は初期スコープに含めない
- [/] 1桁GB級の軽量 local model を推奨する導線を初期スコープへ含める
- [/] `katana-ui` 内に既存機能から分離した chat UI を置く
- [/] 「weget」は widget の typo として扱う
- [/] autofix は single diagnostic ではなく file 単位の一括修正として扱う
- [/] LLM autofix は KML の一括 fix 後 content と残存 diagnostics を context に渡し、全エラー解消を提案させる
- [/] LLM autofix は差分 preview を必須にする。差分 preview 機能は現状未実装なので、この change 内の実装対象に含める
- [/] ユーザー要望の `openapi` は OpenAI / OpenAI-compatible provider の意味として扱う
- [/] 音声入力 MVP は OS dictation 連携寄りとし、アプリ内録音 + speech-to-text + typeless 的ノイズ除去は劣後 task とする
- [/] chat UI は VS Code 風に、画面端のアイコン列から開閉できるサイドパネルとして扱う
- [/] chat UI はアイコン操作で表示・非表示・固定表示を制御できるようにする
- [/] MVP のチャット履歴はアプリ起動中の一時状態に限定し、履歴保存・履歴一覧・履歴管理は後続 task に分離する
- [/] MVP では Ollama モデル選択を必須にし、細かい生成設定は後続 task に分離する
- [/] task ごとの PR は残さず、チェック通過後に `release/v0.23.0` へ merge してから次 task へ進む
- [ ] widget 依存の追加許容範囲を egui 系 crate までとするか決める
- [ ] Vertex AI / Bedrock / OpenAI 系 provider をどの version milestone に切るか決める

## Deferred Expansion Backlog

- [ ] 後続 task としてチャット履歴の永続化、履歴一覧、履歴検索、履歴削除 UI を扱う
- [ ] 後続 task として model ごとの細かい生成パラメータ UI を扱う
- [ ] 後続 OpenSpec change として音声入力を切り出す。MVP は OS dictation 連携寄りとし、音声入力結果を chat composer に入れ、document mutation は confirmation 境界を通す
- [ ] 後続 OpenSpec change または分離 repository として、アプリ内録音、speech-to-text、typeless 的な不要音声・ノイズ・口癖除去を扱う
- [ ] 後続 OpenSpec change として Vertex AI / Bedrock / OpenAI 系 provider を切り出す。API key / secret の保存責務を OS keychain または settings persistence で決める

---

## 1. Ollama Provider Contract and Settings

- [x] 1.1 Ollama endpoint、model、capability、timeout を保持する設定スキーマを追加する
- [x] 1.2 `AiProvider` abstraction に接続する Ollama adapter を追加する
- [x] 1.3 Ollama の availability check と model list 取得を実装する
- [x] 1.4 Ollama モデル選択を必須入力として settings UI に追加する
- [x] 1.5 1桁GB級の lightweight model を選びやすい推奨導線を settings UI に追加する

### Definition of Done (DoD)

- [x] ユーザーが UI から Ollama endpoint と model を設定し、保存、再選択できること
- [x] モデル未選択では chat / autofix request が送信されないこと
- [x] Ollama 未設定の状態でも、アプリケーションの通常編集機能が問題なく維持されること
- [x] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Isolated Chat UI Foundation

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 2.1 app session 内だけで保持する chat message、pending request、error state、selected model を持つ専用 state を追加する
- [x] 2.2 editor / preview / diagnostics と分離した VS Code 風の chat サイドパネルを `katana-ui` に追加する
- [x] 2.3 画面端のアイコン操作で chat サイドパネルを表示・非表示・固定表示できるようにする
- [x] 2.4 user message を Ollama provider に送り、assistant response を app session 内のチャット履歴へ追加する
- [x] 2.5 provider 未設定 / モデル未選択 / unavailable / timeout / invalid response の disabled state と recovery 導線を追加する
- [x] 2.6 チャット履歴の永続化、履歴一覧、履歴検索、履歴削除管理を MVP に含めないことをテストまたは仕様上で確認する
- [x] 2.7 chat response が user confirmation なしに document や workspace file を変更しないことをテストする

### Definition of Done (DoD)

- [x] chat UI が単独で開閉でき、既存 editor / preview / diagnostics の状態を破壊しないこと
- [x] chat UI を画面端アイコンから開閉・固定表示できること
- [x] chat messages は app session 中だけ扱われ、再起動後に履歴復元されないこと
- [x] chat の request lifecycle が専用 state に閉じており、lint autofix や document generation の実装と競合しないこと
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. File-Level Autofix Request and Diff Preview Pipeline

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 元 content、KML 一括 fix 後 content、残存 diagnostics、file path から file-level autofix request を組み立てる
- [ ] 3.2 Ollama からの応答を、アプリケーション内部で扱う normalized file-level fix candidate に変換する
- [ ] 3.3 元 content と proposal content の差分を表示する reusable diff preview surface を実装する
- [ ] 3.4 生成された file-level autofix candidate について、diff preview / confirm / apply flow を実装する
- [ ] 3.5 apply 後に save、re-lint、error recovery が一連の動作として成立するか確認する

### Definition of Done (DoD)

- [ ] autofix が file diagnostics を起点にして一括実行できること
- [ ] KML の deterministic fix 結果と残存 diagnostics が LLM context に含まれること
- [ ] 元 content と LLM proposal content の差分を apply 前に preview できること
- [ ] ユーザーの confirmation 無しに、勝手に Markdown が書き換わらないこと
- [ ] 修正の適用後に再び lint が走り、エラーが解消された事実を確認できること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Settings and Diagnostics UI Integration

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 Ollama provider 設定 UI と接続テスト導線を整える
- [ ] 4.2 diagnostics UI 上に autofix entry point を追加する
- [ ] 4.3 provider unavailable 時の disabled state と recovery 導線を追加する
- [ ] 4.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 4.5 ユーザーからのフィードバックに基づく UI 微調整を行う

### Definition of Done (DoD)

- [ ] provider 設定から chat、diagnostics autofix まで、UI 上で迷わずに辿り着けること
- [ ] provider unavailable の理由と復旧導線が、ユーザーに分かること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 5.1 ユーザーへ実装完了の報告および動作状況（UIの場合はスナップショット画像等）の提示を行う
- [ ] 5.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## 6. Final Verification & Release Work

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `$self-review` skill
- [ ] 6.2 Format and lint-fix all updated markdown documents
- [ ] 6.3 Ensure `make check` passes with exit code 0
- [ ] 6.4 Create PR from Base Feature Branch targeting `master`
- [ ] 6.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 6.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.7 Create `release/v0.23.0` branch from master
- [ ] 6.8 Run `make release VERSION=0.23.0` and update CHANGELOG (`changelog-writing` skill)
- [ ] 6.9 Create PR from `release/v0.23.0` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
