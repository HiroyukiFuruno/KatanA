## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.23.0 の変更 ID とスコープが確認されていること
- [ ] `v0.19.0` の markdownlint 検知結果データ構造 (diagnostics payload) が安定利用できる状態であること
- [ ] ローカル環境の LLM エンドポイント (local endpoint) を使用する前提が確定していること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Provider Settings and Registry Extensions (プロバイダー設定とレジストリ拡張)

- [ ] 1.1 ローカルの LLM プロバイダー情報を設定するスキーマの中に、プロバイダー種別 (provider kind)、エンドポイント URL (endpoint)、モデル名 (model)、能力フラグ (capability) を追加する
- [ ] 1.2 `Ollama`、`LM Studio` などの OpenAI 互換ローカルエンドポイント向けに、設定を吸収・変換するアダプタおよびプリセット設定を追加する
- [ ] 1.3 現在使用中のプロバイダー切り替えと設定の保存を、共通のレジストリ (registry) に接続する
- [ ] 1.4 LLM エンドポイントの接続可否チェック (availability check) や、動作が軽量な推奨モデルへの誘導ステップ（導線）を追加実装する

### Definition of Done (DoD)

- [ ] ユーザーが UI からローカルの LLM プロバイダーを選んで設定し、保存、再選択できること
- [ ] プロバイダーが未設定の状態でも、アプリケーションの通常編集機能が問題なく維持されること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. Autofix Request and Apply Pipeline (自動修復リクエストと適用パイプラインの構築)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 2.1 markdownlint の検知結果データ (diagnostics payload) から、LLM 向けの自動修復要求 (autofix request) データを組み立てる
- [ ] 2.2 ローカルプロバイダーからの応答データを、アプリケーション内部で扱いやすい統一形式 (normalized shape) に変換して扱う
- [ ] 2.3 生成された自動修復候補について、適用前に確認する流れ ( preview / confirm / apply flow ) を実装する
- [ ] 2.4 自動修復の適用後に、ファイルの保存 (save)、再評価 (re-lint)、エラーからの復旧 (error recovery) が一連の動作として破綻なく成立するか確認する

### Definition of Done (DoD)

- [ ] 自動修復 (autofix) が、診断エラー行（diagnostic）を起点にして直接実行できること
- [ ] ユーザーの最終確認 (confirmation) 無しに、勝手にコードが書き換わらないこと
- [ ] 修正の適用後に再び自動で lint が走り、エラーが解消された事実を確認できること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. Settings and Diagnostics UI Integration (設定画面とエラープレビューパネル全体の統合)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 3.1 プロバイダー設定 UI と、接続テスト用ボタン等の導線を追加する
- [ ] 3.2 エラー診断の一覧 (diagnostics UI) 上に、自動修復を実行するための操作入り口 (autofix entry point) を明示的に追加する
- [ ] 3.3 プロバイダーと接続不可 (unavailable) になった際のエラー表現 (disabled state) と、そこからの復旧手順の導線を追加する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] プロバイダーの設定から、診断画面での自動修復の実行完了まで、UI 上で迷わずに辿り着けること
- [ ] プロバイダー接続不可 (unavailable) 状態になった理由と、そこから正常状態へ復旧するための導線が、直感的にユーザーに分かること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Final Verification & Release Work (最終確認とリリース対応)

- [ ] 4.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [ ] 4.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 4.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [ ] 4.4 `master` に向けて PR（プルリクエスト）を作成する
- [ ] 4.5 `master` へマージする (※ `--admin` の利用は許容される)
- [ ] 4.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.23.0` のリリースタグ打ちとリリース作成を実行する
- [ ] 4.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
