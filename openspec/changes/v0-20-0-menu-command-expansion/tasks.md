## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.20.0 の変更 ID とスコープが確認されていること
- [ ] 現在のネイティブメニュー、コマンドパレット、および `AppAction` の一覧を再確認していること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Shared Command Inventory (コマンドリストの共通化管理)

- [x] 1.1 ユーザーが利用可能な全コマンド (user-facing commands) を一斉に棚卸しする
- [x] 1.2 「ラベル (label)」「グループ (group)」「可用性や利用状態 (availability)」を一元管理するための、「共通コマンド一覧機能 (shared command inventory)」を導入する
- [x] 1.3 メニュー、コマンドパレット、および将来のショートカットエディタなどが、この inventory を共通して参照・取得できるインターフェース (shape) を定義する

### Definition of Done (DoD)

- [x] 共通のコマンドインベントリ (inventory) が、単一の信頼できる情報源 (source of truth) として機能していること
- [x] `AppAction` 本体の処理と、インベントリに登録される情報管理の責務分担が明確に切り離されていること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. File and View Menu Expansion (ファイル・ビューメニューの拡張)

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 2.1 File メニューに対して、ワークスペースおよびドキュメント関連の各種メニューコマンドを追加する
- [x] 2.2 View メニューに対して、画面ナビゲーションや UI 表示切り替えに関連する操作コマンドを追加する
- [x] 2.3 各処理が実行不可となる状態 (disabled state) を、共通管理しているインベントリ側の `availability` 情報と直結させて同期する
- [x] 2.4 macOS 以外のメニュー領域 (non-macOS command surface) でも、欠如することなく同じレベルのコマンド利用範囲 (coverage) を提供する
- [x] 2.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 2.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] File および View メニューのカバー範囲 (command coverage) が事前の設計どおりに適切に増補されていること
- [x] ネイティブメニューやアプリ内等、表示媒体の違いに関わらず可用性（利用可能・不可の判断）が完全に一致していること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. Help Menu and Palette Alignment (ヘルプメニューとコマンドパレットのすり合わせ)

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 3.1 Help メニュー内に、ドキュメント操作、GitHubリンク、リリースノート表示、アップデート確認などのコマンド群をわかりやすく整理して追加する
- [x] 3.2 コマンドパレットの表示項目やグループ分けが、共通インベントリ (inventory) で定義された情報 (labels, groups) を参照して反映されるように修正する
- [x] 3.3 それらに関連するドキュメント (docs) や、多言語対応のテキスト文言 (i18n copy) を漏れなく最新情報に更新する
- [x] 3.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 3.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] Help メニューの項目とコマンドパレットの表示が、インベントリの定義基準にズレなく準拠していること
- [x] 対応する翻訳テキスト (i18n copy) やドキュメント情報が、新しいメニューコマンド構成の変更内容を正しく反映していること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Final Verification & Release Work (最終確認とリリース対応)

- [ ] 4.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [ ] 4.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 4.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [ ] 4.4 `master` に向けて PR（プルリクエスト）を作成する
- [ ] 4.5 `master` へマージする (※ `--admin` の利用は許容される)
- [ ] 4.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.20.0` のリリースタグ打ちとリリース作成を実行する
- [ ] 4.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
