## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.17.0 の変更 ID とスコープが確認されていること
- [ ] 現行の icon registry、SVG loader、SVG linter、settings schema の仕様や実装を再確認していること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Icon Pack Contract and Asset Layout (アイコンパックの仕様定義とアセット配置)

- [ ] 1.1 計画に未利用のsvgを整理する
- [ ] 1.2 既存の `assets/icons` を `assets/icons/katana/...` 配下へ再編し、`katana-icon` パックのアセットルートを固定化する
- [ ] 1.3 アイコンパックの manifest またはそれと同等のメタデータを追加し、パック ID (pack id)、表示名 (display name)、レンダリングポリシー (render policy)、およびライセンスメタデータ (license metadata) を表現できるようにする
- [ ] 1.4 `Icon` enum に定義された全アイテムについて、パックのカバー率表（coverage table）で確認できる契約体系（contract）を作る
- [ ] 1.5 選定済み (curated) の外部パック 5 種類の採用候補と、そのソースおよびライセンス情報を固定する
- [ ] 1.6 組み込み (built-in) パックのディレクトリ命名規則を `assets/icons/<pack-dir>/...` に統一する

### Definition of Done (DoD)

- [ ] `katana-icon` が既存のデフォルトパックとして定義されていること
- [ ] 組み込みパックが `assets/icons/katana` や `assets/icons/<external-pack>` のように、パック単位の階層へ整理されていること
- [ ] 同梱 (shipping) されるパックのメタデータの情報源 (source of truth) が 1 箇所に集約・固定されていること
- [ ] 各パックのカバー率 (pack coverage) を確認できる一覧、または検証手段が存在すること
- [ ] 計画に不要となった未利用SVGの整理が完了していること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. Runtime Registry and Color-aware Rendering (ランタイムレジストリと色対応レンダリング)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 2.1 アイコンレジストリ (icon registry) をパック対応 (pack-aware) に変更し、有効なパックからアセットを解決できるようにする
- [ ] 2.2 「選択中パックの直接アセット」 -> 「パック内の上書きアセット (pack override)」 -> 「`katana-icon` コアアセットへのフォールバック」 の順でアイコンを解決する、セーフティネット構造を追加する
- [ ] 2.3 `TintedMonochrome` (色付け単色) と `NativeColor` (本来の複数色) のレンダリングポリシーを追加する
- [ ] 2.4 現行の「白色のみ (white-only)」というアイコン検査 (validation) を更新し、パックのポリシーを考慮した (pack policy aware) 内容に変更する

### Definition of Done (DoD)

- [ ] アクティブなパックを切り替えると、アイコンアセットの解決先が自動的に変わること
- [ ] 複数色を使用する (colorful) パックが、単色の色付け (tint) によって潰れずに正しく表示されること
- [ ] アセットが見つからない欠落時でも、フォールバック (recoverable fallback) が機能し、UI が壊れないこと
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. Settings UI and Live Preview (設定UIとライブプレビュー)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 3.1 設定画面にアイコンパックを選択できる UI を追加する
- [ ] 3.2 パック一覧に、プレビュー (preview)、表示名 (display name)、および必要に応じてレンダリングポリシー (render policy) の説明を表示する
- [ ] 3.3 選択されたパックの設定を保存し、次回起動時にも確実に復元される状態にする
- [ ] 3.4 パックの切り替え操作が、アプリケーションの再起動なしで即座に反映されるようにする
- [ ] 3.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 設定画面上からアイコンパックの切り替えが完了できること
- [ ] アプリケーション再起動後も、選択したパックが正しく復元されること
- [ ] 設定画面上のライブプレビューと、実際の UI に表示されるアイコンが完全に一致していること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Curated Pack Import, Overrides, and License Inventory (選定済みパックのインポート、上書き定義、ライセンスの目録化)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 4.1 選定済み (curated) の外部パック 5 種類の SVG ファイル群をリポジトリへ追加し、パックごとに整理配置する
- [ ] 4.2 `Icon` enum の全件について、「サードパーティ製ソース」 / 「KatanA側で作成した上書きアセット (override)」 / 「フォールバック」 の対応表 (inventory) を作成する
- [ ] 4.3 既存 UI と直接の互換性がないアイコンについては、選択中パックのビジュアル言語等に合わせた KatanA 独自の上書きアセット (override) を作成する
- [ ] 4.4 `docs/licenses/icon-packs.md`、あるいはそれに準ずる文書に、ソース元、ライセンス、上書きの根拠 (override rationale) を記録する
- [ ] 4.5 商用利用の妨げにならない選定済みパックのみが公開用パッケージ (shipping target) に含まれることを入念に確認する
- [ ] 4.6 ast linterで未利用のsvgを検知できる仕組みを構築する
- [ ] 4.7 アイコンのアセットごとのsvgアイコン同士の互換を担保するast linterを構築する

### Definition of Done (DoD)

- [ ] 同梱 (shipping) される各パックが、要求されるアイコン定義 (required icon contract) を満たしていること
- [ ] サードパーティ製ソースと、KatanA 側で作成した上書きアセットとの境界線が明確に文書化されていること
- [ ] 同梱パックの出所 (provenance) とライセンス情報がリポジトリ上から容易に追跡可能であること
- [ ] 未利用SVGの自動検知および互換性を担保する AST linter が稼働していること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 5. Final Verification & Release Work (最終確認とリリース対応)

- [ ] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [ ] 5.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 5.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [ ] 5.4 `master` に向けて PR（プルリクエスト）を作成する
- [ ] 5.5 `master` へマージする (※ `--admin` の利用は許容される)
- [ ] 5.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.17.0` のリリースタグ打ちとリリース作成を実行する
- [ ] 5.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
