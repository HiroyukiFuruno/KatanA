## 着手条件（DoR）

- [ ] `proposal.md`、`design.md`、`specs/vendor-extension-boundary/spec.md` がレビュー済みであること
- [ ] upstream baseline として `lampsitter/egui_commonmark` `master` コミット `9cc31bd725bc417fc9980375357c18bdf7feee37` を記録済みであること
- [ ] 現在進行中の vendor に触る WIP が、merge / stash / branch 分離のいずれかで整理されていること
- [ ] subtree root (`vendor/egui_commonmark_upstream/`) が実装開始時点の基準ブランチへ同期済みであること

## ブランチ運用ルール

本タスクでは、以下のブランチ運用を適用します。

- **基準ブランチ**: `v0-22-7-markdown-vendor-extension-architecture`
- **作業ブランチ**: 標準は `v0-22-7-markdown-vendor-extension-architecture-task-x` (`x` はタスク番号)

実装完了後は `/openspec-delivery` を使用して基準ブランチへ PR を作成・マージしてください。

---

## 1. 乖離棚卸しとガードレールの整備

- [ ] 1.1 upstream baseline (`9cc31bd725bc417fc9980375357c18bdf7feee37`) との差分棚卸しを作成する
- [ ] 1.2 差分ファイルを `拡張ブリッジ`、`upstream へ返せる汎用修正`、`製品固有ロジック`、`運用・同期補助` の 4 区分で棚卸しする
- [ ] 1.3 プレビュー用フィクスチャセットを定義する
  - テーブル
  - タスクリスト
  - インライン絵文字
  - 検索ハイライト
  - ブロックアンカー / TOC 同期
  - コードブロックのコピー画面要素
- [ ] 1.4 `docs/vendor-egui-commonmark.md` の雛形を作成し、upstream URL、baseline コミット、差分監査コマンド、許可一覧の考え方を記録する
- [ ] 1.5 許可一覧外の vendor 変更を将来検出する監査方針を決める

### 完了条件（DoD）

- [ ] 差分棚卸しから「何を vendor に残し、何を `katana-ui` に戻すか」が判定可能であること
- [ ] フィクスチャ対象が tasks 2-5 の検証項目に接続されていること
- [ ] `docs/vendor-egui-commonmark.md` に baseline と update 手順の骨子があること
- [ ] `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、自己レビュー、コミット、PR 作成、マージまでの包括的なデリバリー手順を完了すること

---

## 2. KatanA 側ファサードの先行導入

### 着手条件（DoR）

- [ ] 1 つ前のタスクが自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除まで含む完全なデリバリーサイクルを完了していること
- [ ] 基準ブランチが同期済みであり、このタスク用の新しいブランチが明示的に作成されていること

- [ ] 2.1 `crates/katana-ui/src/markdown_viewer/` を新設する
- [ ] 2.2 `bridge.rs`, `mode.rs`, `host.rs`, `session.rs`, `types.rs` を作成し、ビューア構築責務をプレビュー呼び出し側から分離する
- [ ] 2.3 `crates/katana-ui/src/preview_pane/extension_table.rs` を `markdown_viewer/blocks/table.rs` へ移設する
- [ ] 2.4 `crates/katana-ui/src/widgets/markdown_hooks/*` のタスクリスト関連責務を `markdown_viewer/inline/task_list.rs` へ移設する
- [ ] 2.5 `section_show.rs`、changelog、update modal などの呼び出し側をファサード経由へ統一する
- [ ] 2.6 呼び出し側からコールバックの直接合成を排除する

### 完了条件（DoD）

- [ ] `preview_pane` から vendor のコールバック接続詳細が見えなくなっていること
- [ ] テーブル / タスクリスト / 絵文字 / リスト強調 / 検索状態の入口が `markdown_viewer/` 配下へ集約されていること
- [ ] `katana-ui` 側の新規モジュールが lint 対象・テスト対象になっていること
- [ ] `make check` がエラーなしで通過すること
- [ ] `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、自己レビュー、コミット、PR 作成、マージまでの包括的なデリバリー手順を完了すること

---

## 3. 安定した vendor ブリッジの導入

### 着手条件（DoR）

- [ ] 1 つ前のタスクが自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除まで含む完全なデリバリーサイクルを完了していること
- [ ] 基準ブランチが同期済みであり、このタスク用の新しいブランチが明示的に作成されていること

- [ ] 3.1 `vendor/egui_commonmark_upstream/egui_commonmark_backend/src/extension.rs` を追加し、単一の拡張ホスト契約を定義する
- [ ] 3.2 アンカー収集先、検索状態、アクティブ項目情報を扱う型付きレンダリングセッション文脈を定義する
- [ ] 3.3 `CommonMarkOptions` に `extension_host` 相当の接続点を追加する
- [ ] 3.4 `CommonMarkViewer` へ `extension_host(...)` または同等のブリッジ接続 API を追加する
- [ ] 3.5 `pulldown.rs` の委譲点を最小箇所へ限定し、要求とセッション文脈を拡張ホストへ渡す
- [ ] 3.6 標準 upstream 描画経路は空ホストまたは host 未設定で維持する
- [ ] 3.7 既存コールバック群を後方互換レイヤとして残すか、ブリッジ内へ畳み込むかを実装時に確定する

### 完了条件（DoD）

- [ ] 今後の KatanA 機能追加で `CommonMarkViewer` の機能別 field を増やさずに済む契約ができていること
- [ ] vendor 側の新規差分が許可一覧に収まっていること
- [ ] 標準描画モードと KatanA 拡張描画モードの切り替えが同一ファサードから可能であること
- [ ] `make check` がエラーなしで通過すること
- [ ] `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、自己レビュー、コミット、PR 作成、マージまでの包括的なデリバリー手順を完了すること

---

## 4. 機能の KatanA モジュールへの移管

### 着手条件（DoR）

- [ ] 1 つ前のタスクが自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除まで含む完全なデリバリーサイクルを完了していること
- [ ] 基準ブランチが同期済みであり、このタスク用の新しいブランチが明示的に作成されていること

- [ ] 4.1 ブロック系機能を `markdown_viewer/blocks/*` へ移行する
  - テーブル描画
  - コードブロック外観
  - 複合ブロック矩形 / アンカー収集
- [ ] 4.2 インライン系機能を `markdown_viewer/inline/*` へ移行する
  - 絵文字
  - タスクリストの画面要素 / コンテキストメニュー
- [ ] 4.3 装飾系機能を `markdown_viewer/decorations/*` へ移行する
  - リスト項目強調
  - 検索ハイライト / アクティブ一致
  - 見出し / ブロックアンカー収集先
- [ ] 4.4 `bytes://katana-*` のような製品固有命名を vendor から除去する
- [ ] 4.5 主要画面フィクスチャのスクリーンショット確認を行い、差分を記録する
- [ ] 4.6 スクリーンショット確認で判明した微調整を同タスク内で反映する

### 完了条件（DoD）

- [ ] KatanA 固有の描画 / 操作ロジックが `katana-ui` モジュールとして独立していること
- [ ] フィクスチャベースでテーブル / タスクリスト / 絵文字 / 検索 / アンカー / コードブロックが回帰検知できること
- [ ] 標準描画モードとの比較で意図した差分だけが残ること
- [ ] `make check` がエラーなしで通過すること
- [ ] `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、自己レビュー、コミット、PR 作成、マージまでの包括的なデリバリー手順を完了すること

---

## 5. vendor 差分縮小と同期運用手順書の完成

### 着手条件（DoR）

- [ ] 1 つ前のタスクが自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除まで含む完全なデリバリーサイクルを完了していること
- [ ] 基準ブランチが同期済みであり、このタスク用の新しいブランチが明示的に作成されていること

- [ ] 5.1 vendor 差分を許可一覧と照合し、不要な KatanA 固有コードを除去する
- [ ] 5.2 upstream へ返せる汎用修正と製品固有コードを切り分ける
- [ ] 5.3 `docs/vendor-egui-commonmark.md` を完成させる
- [ ] 5.4 差分監査コマンドまたはスクリプトを追加する
- [ ] 5.5 今後の update 時に確認すべき手順（baseline 取得、差分監査、フィクスチャテスト、マージ方針）を運用手順書化する

### 完了条件（DoD）

- [ ] vendor 側の差分理由がファイル単位で説明できること
- [ ] 運用手順書を見れば保守担当者が upstream update を再実行できること
- [ ] 許可一覧外の vendor 変更を検知する仕組みがあること
- [ ] `make check` がエラーなしで通過すること
- [ ] `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、自己レビュー、コミット、PR 作成、マージまでの包括的なデリバリー手順を完了すること

---

## 6. 最終確認とリリース作業

- [ ] 6.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を使って自己レビューを実施する
- [ ] 6.2 `make check` が終了コード 0 で通過することを確認する
- [ ] 6.3 基準ブランチから `master` 向けの PR を作成する
- [ ] 6.4 PR 上で CI（Lint / Coverage / CodeQL）が通過することを確認し、失敗時は merge を止める
- [ ] 6.5 `master` へ merge する (`gh pr merge --merge --delete-branch`)
- [ ] 6.6 `master` から `release/v0.22.7` ブランチを作成する
- [ ] 6.7 `make release VERSION=0.22.7` を実行し、`changelog-writing` skill に従って CHANGELOG を更新する
- [ ] 6.8 `release/v0.22.7` から `master` 向けの PR を作成し、`Release Readiness` CI が通ることを確認する
- [ ] 6.9 release PR を `master` へ merge する (`gh pr merge --merge --delete-branch`)
- [ ] 6.10 GitHub Release 完了を確認し、`/opsx-archive` でこの change を archive する
