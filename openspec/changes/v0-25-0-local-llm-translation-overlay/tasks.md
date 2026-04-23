## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.25.0 の変更 ID とスコープが確認されていること
- [ ] `v0.23.0` におけるローカルLLMプロバイダーの基盤設計が、翻訳処理 (translation) に再利用可能な状態であること
- [ ] リリース時点での、動的に発生する英語の表示物や、外部から持ち込まれる英語テキスト (target inventory) を洗い出す方針が合意・確認されていること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Translation Target Inventory and Eligibility Rules (翻訳ターゲットの棚卸しと翻訳可否の判定ルールの策定)

- [ ] 1.1 問題診断パネル (diagnostics)、LLMへの問い合わせ結果 (AI result)、その他外部テキスト表示等の「動的翻訳対象箇所 (target inventory)」のリストを作成・特定する
- [ ] 1.2 そもそも翻訳を掛けるべきテキストか、あえてそのまま表示すべきテキストか (eligible / ineligible text) を判定する明確なルールを定義する
- [ ] 1.3 オリジナルの英語テキストと、翻訳されて浮かび上がる表示 (translated view) との共存方針を定義する
- [ ] 1.4 LLMへの無駄なAPI呼び出しを減らすため、翻訳結果のキャッシュキー (translation cache key) とキャッシュの破棄ルール (invalidation rule) を定義する
- [ ] 1.5 既にオーバーレイで表示されている生成テキスト、そもそも英語ではないテキスト、あるいは現在翻訳リクエスト中であるテキストに対する、再翻訳要求の「除外ルール」を定義する

### Definition of Done (DoD)

- [ ] LLMによる自動翻訳を掛ける対象範囲（スコープ）が明確に文書化されていること
- [ ] 静的な多言語化 (UI上の静的テキスト翻訳, static i18n) と、動的翻訳オーバーレイ表示 (dynamic translation overlay) との責務の境界線が明確に分離されていること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. Translation Pipeline and Cache (翻訳リクエストのパイプライン化とキャッシュ構築)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 2.1 ローカルLLMプロバイダーの仕組みを、自動翻訳リクエストの通信経路に接続する
- [ ] 2.2 翻訳元の原文、翻訳先の言語、プロバイダー側のコンテキスト情報をキーとした、API呼び出し節約用のキャッシュシステムを実装する
- [ ] 2.3 翻訳の実行失敗 (failure)、タイムアウト処理、不正な応答データ (invalid response) 発生時のエラー回避・フォールバック対応を実装する
- [ ] 2.4 アプリケーションの言語切り替え時、あるいはLLMプロバイダーの切り替え時に、キャッシュが期待通りに破棄/動作するかを確認する
- [ ] 2.5 翻訳で得られたオーバーレイ表示テキストに対して、さらに再翻訳をかけたり、無駄な二重リクエスト (double request) が発生したりしないことをコードから入念に確認する

### Definition of Done (DoD)

- [ ] 自動翻訳機能が、ローカルLLMプロバイダーが有効化 (enabled) されている時にのみ動作すること
- [ ] 万が一翻訳に失敗しても、クラッシュしたり表示が消滅したりせず、オリジナルの英語テキストが安全にそのまま表示されること
- [ ] 翻訳によってオーバーレイ化されたテキストに対し、再帰的な再翻訳が掛からないという保証があること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. UI Overlay Integration and Feedback (翻訳オーバーレイUIの統合とフィードバック対応)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 3.1 診断パネル (diagnostics)、LLMのAI応答、およびその他要翻訳（eligible target）と判定された箇所に対して、実際の翻訳オーバーレイ表現 (translation overlay UI) を追加する
- [ ] 3.2 翻訳後のテキストを表示しつつも、元のオリジナルの英語テキストも参照できる導線 (トグル等) を追加する
- [ ] 3.3 自動翻訳の処理中 (loading) / キャッシュ利用済み (cached) / 失敗に伴うフォールバック (fallback) といった状態の推移を、UI上に視覚的に反映させる

### Definition of Done (DoD)

- [ ] ユーザーが、直感的な操作で自動翻訳されたビュー (translated view) と、元の英語テキストの両方を確認できるようになっていること
- [ ] LLMの応答が遅延中 (loading) または失敗した状態でも、全体のUIレイアウトが破壊されたりエラーになったりしないこと
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. User Review (Pre-Final Phase)

- [ ] 4.1 ユーザーへ実装完了の報告および動作状況（UIの場合はスナップショット画像等）の提示を行う
- [ ] 4.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## 5. Final Verification & Release Work (最終確認とリリース対応)

- [ ] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [ ] 5.2 markdownのフォーマット（format）および Lint修正（lintfix）を実行し、全ドキュメントの体裁を整える
- [ ] 5.3 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 5.4 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [ ] 5.5 `master` に向けて PR（プルリクエスト）を作成する
- [ ] 5.6 `master` へマージする (※ `--admin` の利用は許容される)
- [ ] 5.7 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.25.0` のリリースタグ打ちとリリース作成を実行する
- [ ] 5.8 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
