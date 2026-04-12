## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.22.0 の変更 ID とスコープが確認されていること
- [ ] `v0.20.0` のコマンドインベントリ設定と、`v0.21.0` のショートカットスキーマ設定が利用可能であること
- [ ] 作業中の Markdown ファイルの位置を起点とした際の、アセット出力先ディレクトリ方針が `./asset/img` で確定していること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Source-First Authoring Commands (マークダウン編集補助コマンドの実装)

- [x] 1.1 見出し、装飾、リスト、表などのドキュメント作成補助コマンド (authoring command) の一覧を確定する
- [x] 1.2 カーソル位置やテキスト選択範囲 (selection) に対して、Markdown 記法への挿入・変換を行うロジックを実装する
- [x] 1.3 搭載した編集補助コマンドが、コマンド一覧 (inventory) やショートカットから容易に呼び出せるようにする
- [x] 1.4 これにより、ファイルの「保存 (save)」や「未保存状態検知 (dirty buffer)」、および「プレビューの同期 (preview sync)」に関する内部管理システム (契約/contract) が破壊されていないことを確認する

### Definition of Done (DoD)

- [x] エディター上で Markdown のソースコード操作を直接的に行う環境 (source-first) を維持しつつ、編集補助用のコマンドを利用できること
- [x] 選択状態の有無にかかわらず、意図しない Markdown 原稿の破壊が生じないこと
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. Image Ingest Pipeline and Settings (画像の取り込みパイプラインと設定の実装)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 2.1 作業中の Markdown ファイル (active Markdown file) の位置を起点として、保存先を `./asset/img` に解決して保存するロジックを実装する
- [ ] 2.2 ローカルファイルの添付操作 (local file attach) を、実装した画像取り込み処理 (image ingest pipeline) に接続して動作させる
- [ ] 2.3 クリップボードからの画像貼り付け操作 (clipboard image paste) も、同様の画像取り込み処理に接続する
- [ ] 2.4 画像の保存先、ファイル命名規則、ダイアログの表示ポリシーに関して、設定画面用のスキーマ (settings schema) を追加拡充する
- [ ] 2.5 テキストへの相対パス挿入処理や、アセット用ディレクトリの自動作成処理が正しく動作することを確認するためのテストを追加する

### Definition of Done (DoD)

- [ ] ファイルの添付操作とクリップボード経由の貼り付け操作の両方が、同じファイル保存ルールに基づいて動作すること
- [ ] デフォルトの設定で、アクティブな Markdown ファイルの位置から見て `./asset/img` に画像が保存されること
- [ ] 設定画面で保存先ポリシーなどを変更した場合、その後の画像取り込み (subsequent ingest) 設定が即座に反映されること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. UI Integration and Asset Navigation (UIへの統合とアセットへのナビゲーション機能連携)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 3.1 画像の添付や貼り付けなどの動作開始操作 (UI導線) を、エディタエリア、メニュー、およびショートカットに追加設定する
- [ ] 3.2 ローカル画像の参照パス文字列から、実際の対象ファイルまたはディレクトリを直接開いて辿れるナビゲーションリンク (導線) を確保・追加する
- [ ] 3.3 UI 上において、「ローカルに存在して行方不明になっている画像 (missing local image)」と「リモートの画像 (remote image)」を明確に区別して表示する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] 画像の挿入操作から、参照先の保管状況を確認する操作まで、UI 上でシームレスに完結していること
- [ ] ファイルが行方不明な場合やローカル画像ではない場合に、事実と異なる誤ったジャンプ導線等が出現しないこと
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Final Verification & Release Work (最終確認とリリース対応)

- [ ] 4.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [ ] 4.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 4.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [ ] 4.4 `master` に向けて PR（プルリクエスト）を作成する
- [ ] 4.5 `master` へマージする (※ `--admin` の利用は許容される)
- [ ] 4.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.22.0` のリリースタグ打ちとリリース作成を実行する
- [ ] 4.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
