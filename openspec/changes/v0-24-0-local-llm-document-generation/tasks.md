## Definition of Ready (DoR)
- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.24.0 の変更 ID とスコープが確認されていること
- [ ] `v0.23.0` で実装したローカルLLMプロバイダー設定と接続可否判定 (availability 判定) が利用可能であること
- [ ] 「現在開いているドキュメントへの追記/置換」「新規ファイルの作成」「テンプレートから足場を作る」という 3 種の出力先を同時に扱うという設計前提が確認されていること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Generation Job Model and Context Assembly (生成ジョブのモデル化とコンテキストの組み立て)

- [ ] 1.1 「現在開いているドキュメント内への挿入 (current document)」「新規ファイルの作成 (new file)」「テンプレートから足場を作る (template scaffold)」の出力方法を表現するためのモデル層 (generation job model) を定義する
- [ ] 1.2 現在編集中のファイル (active document)、選択範囲 (selection)、ワークスペース等から、LLMに渡すコンテキスト情報 (context) を構築する
- [ ] 1.3 LLMへ渡す入力データ (generation input) のサイズ制限制御と、生成対象側のメタデータ情報を整理する
- [ ] 1.4 生成結果を実際のファイルに書き込む前にプレビュー表示するための、統一されたレスポンス形式 (normalized response shape) を定義する

### Definition of Done (DoD)

- [ ] 全く異なる 3 種類の出力方式が、1 つの共通化された仕組み (generation contract) で表現できていること
- [ ] LLMのプロンプトに渡されるコンテキストの範囲が明文化・明示されていること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. Write and Insert Execution Pipeline (ファイル書き込みとテキスト挿入実行パイプラインの構築)

### Definition of Ready (DoR)
- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 2.1 「現在開いているドキュメントへの挿入」における プレビュー表示 / 反映適用 / アンドゥ(Undo)可能な流れ (preview / apply / undo-friendly flow) を実装する
- [ ] 2.2 「新規ファイルの作成」における 生成結果プレビュー / 保存完了までの流れ (preview / save flow) を実装する
- [ ] 2.3 「テンプレートから足場を作る」動作における 事前設定利用 / 保存先指定 / 保存完了までの流れ (preset / destination / save flow) を実装する
- [ ] 2.4 上記の書き込み処理完了後に発生する、画面のリフレッシュ、未保存状態(dirty state)のフラグ管理、既存ファイルとの衝突時(file collision)のハンドリング処理を統一的に管理する

### Definition of Done (DoD)

- [ ] 「既存ドキュメント」「新規ファイル」「テンプレート」の 3 つの方式すべてを通じて、意図した通りのファイル書き込みが成立すること
- [ ] どの方式を利用しても、ユーザーの最終確認 (confirmation) アクションの前に勝手にファイルが書き換わったり上書きされたりしないこと
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. UI Integration and Review Flow (UIの統合とプレビュー・レビュー体験の構築)

### Definition of Ready (DoR)
- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 3.1 3種類の生成アクション（既存ドキュメント / 新規ファイル / テンプレート）を呼び出すための入り口 (entry point) を UI に追加する
- [ ] 3.2 LLMによる生成結果のプレビュー、挿入・保存先の選択、および最終確認ボタン (confirmation) への UI 導線を追加する
- [ ] 3.3 プロバイダー接続エラー (unavailable)、同名ファイルとの衝突 (file collision)、生成結果が空 (empty result) 等の異常状態 (error state) を表現する表示を追加する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] ユーザーが 3 種類の生成・出力フローを UI 上で明確に区別して使い分けられること
- [ ] 生成されたテキストを、反映ボタンを押す前に安全にレビュー(事前確認)してから書き込むという導線が、直感的にユーザーに分かること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Final Verification & Release Work (最終確認とリリース対応)

- [ ] 4.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [ ] 4.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 4.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [ ] 4.4 `master` に向けて PR（プルリクエスト）を作成する
- [ ] 4.5 `master` へマージする (※ `--admin` の利用は許容される)
- [ ] 4.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.24.0` のリリースタグ打ちとリリース作成を実行する
- [ ] 4.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
