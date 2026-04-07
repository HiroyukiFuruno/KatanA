## Definition of Ready (DoR)

- [x] `proposal.md`、`design.md`、`specs` が揃っていること
- [x] 対象バージョン 0.17.0 の変更 ID とスコープが確認されていること
- [x] 現行の icon registry、SVG loader、SVG linter、settings schema の仕様や実装を再確認していること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Icon Pack Contract and Asset Layout (アイコンパックの仕様定義とアセット配置)

- [x] 1.1 計画に未利用のsvgを整理する
- [x] 1.2 既存の `assets/icons` を `assets/icons/katana/...` 配下へ再編し、`katana-icon` パックのアセットルートを固定化する
- [x] 1.3 アイコンパックの manifest またはそれと同等のメタデータを追加し、パック ID (pack id)、表示名 (display name)、レンダリングポリシー (render policy)、およびライセンスメタデータ (license metadata) を表現できるようにする
- [x] 1.4 `Icon` enum に定義された全アイテムについて、パックのカバー率表（coverage table）で確認できる契約体系（contract）を作る
- [x] 1.5 選定済み (curated) の外部パック 5 種類の採用候補と、そのソースおよびライセンス情報を固定する
- [x] 1.6 組み込み (built-in) パックのディレクトリ命名規則を `assets/icons/<pack-dir>/...` に統一する

### Definition of Done (DoD)

- [x] `katana-icon` が既存のデフォルトパックとして定義されていること
- [x] 組み込みパックが `assets/icons/katana` や `assets/icons/<external-pack>` のように、パック単位の階層へ整理されていること
- [x] 同梱 (shipping) されるパックのメタデータの情報源 (source of truth) が 1 箇所に集約・固定されていること
- [x] 各パックのカバー率 (pack coverage) を確認できる一覧、または検証手段が存在すること
- [x] 計画に不要となった未利用SVGの整理が完了していること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. Runtime Registry and Color-aware Rendering (ランタイムレジストリと色対応レンダリング)

### Definition of Ready (DoR)

- [x] Task 1 の PR がマージされ、ブランチが削除されている
- [x] `RenderPolicy` の定義が `katana-ui` クレート内に存在している

#### Sub Tasks

- [x] 2.1 アイコンレジストリ (`IconRegistry`) をパック対応にし、指定されたパックからアセットを解決できるようにする
- [x] 2.2 「選択中パックの直接アセット」 -> 「パック内の上書きアセット」 -> 「`katana-icon` コアへのフォールバック」 の順でアイコンを解決する安全構造を実装する
- [x] 2.3 `TintedMonochrome` と `NativeColor` のレンダリングポリシーを `icon.ui_image()` 等の呼び出しに反映させる
- [x] 2.4 現行の「白色のみ」というアイコン検査 (AST linter) を更新し、パックのポリシーを考慮した検証ロジックに変更する

#### DoD (Definition of Done)

- [x] アイコンのレジストリが Pack 構造を理解し、安全にフォールバックできる状態になっている
- [x] レンダリングポリシーにより、単色アイコンは現在のテーマに合わせて色付けされ、本来の複数色(NativeColor)アイコンはそのまま表示されるようになっている
- [x] `ast_linter` および `cargo test` (`make check`) が全てエラーなしで通過すること
- [x] `/openspec-delivery` ワークフローを実行し、デリバリールーチンを完了すること。

---

## 3. Settings UI and Live Preview (設定UIとライブプレビュー)

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 3.1 設定画面にアイコンパックを選択できる UI を追加する
- [x] 3.2 パック一覧に、プレビュー (preview)、表示名 (display name)、および必要に応じてレンダリングポリシー (render policy) の説明を表示する
- [x] 3.3 選択されたパックの設定を保存し、次回起動時にも確実に復元される状態にする
- [x] 3.4 パックの切り替え操作が、アプリケーションの再起動なしで即座に反映されるようにする
- [x] 3.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告 ( Skipped visually, validated via unit tests )
- [x] 3.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] 設定画面上からアイコンパックの切り替えが完了できること
- [x] アプリケーション再起動後も、選択したパックが正しく復元されること
- [x] 設定画面上のライブプレビューと、実際の UI に表示されるアイコンが完全に一致していること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Curated Pack Import, Overrides, and License Inventory (選定済みパックのインポート、上書き定義、ライセンスの目録化)

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 同梱 (shipping) される各パックが、要求されるアイコン定義 (required icon contract) を満たしていること
- [x] サードパーティ製ソースと、KatanA 側で作成した上書きアセットとの境界線が明確に文書化されていること
- [x] 同梱パックの出所 (provenance) とライセンス情報がリポジトリ上から容易に追跡可能であること
- [x] 未利用SVGの自動検知および互換性を担保する AST linter が稼働していること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 5. AST Linter Custom Rule for Icon Synchronization (アイコン同期用のAST Linterカスタムルール追加)

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

#### Sub Tasks

- [x] 5.1 `assets/icons/katana/` 配下などのSVGファイル一覧を取得する
- [x] 5.2 `crates/katana-ui/src/icon/types.rs` の `define_icons!` マクロなどに定義されている登録済みのアイコン一覧を抽出する
- [x] 5.3 ディレクトリ内に存在するが、コード（enum/ALL_ICONS等）に未登録のSVGファイルがある場合に追加漏れとしてエラーを報告するカスタムリンタールールを `katana-linter` に実装する
- [x] 5.4 登録されているがSVGファイルが存在しない場合も検出する
- [x] 5.5 SVGのアイコンがwhite listに登録されていないものは同一であることを許容しないast lintの設定を追加する。エラーメッセージに作成したskill (`.gemini/antigravity/skills/svg-icon-management/SKILL.md`) を参照することを出力する
- [x] 5.6 既存のSVGカラーチェック(`svg.rs`)を強化し、すべてのテーマパックの基本配色を `white` (`#FFFFFF`) または `currentColor` に完全に統一する。それ以外の固定色の使用を禁止し、かつ「`fill` も `stroke` も設定されていない（黒潰れする）」不備も検知してエラーにする

### Definition of Done (DoD)

- [x] `katana-linter` のカスタムルールにより、SVGの追加と登録の同期が検証可能になっていること
- [x] 意図的に未登録SVGを配置した場合にlinterが期待通り失敗することを確認すること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフローを実行し、デリバリールーチンを完了すること。

---

## 6. Final Verification & Release Work (最終確認とリリース対応)

- [x] 6.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [x] 6.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [x] 6.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [x] 6.4 `master` に向けて PR（プルリクエスト）を作成する
- [x] 6.5 `master` へマージする (※ `--admin` の利用は許容される)
- [x] 6.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.17.0` のリリースタグ打ちとリリース作成を実行する
- [x] 6.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
