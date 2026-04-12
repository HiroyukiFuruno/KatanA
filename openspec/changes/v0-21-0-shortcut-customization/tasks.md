## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.21.0 の変更 ID とスコープが確認されていること
- [ ] `v0.20.0` で定義されたコマンドインベントリ (command inventory) が利用可能な状態であること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Shortcut Schema and Defaults (ショートカットのスキーマ定義とデフォルト設定)

- [x] 1.1 コマンドインベントリと紐づくキー情報の定義と、デフォルトのショートカット設定パターン (default shortcut set) を定義する
- [x] 1.2 設定情報スキーマ (settings schema) および永続化データ (persistence) の中に、ショートカット割り当て情報 (shortcut bindings) を追加する
- [x] 1.3 既存のコードに直書き (hard-coded) されていたショートカットから、新たな仕組みへの移行 (migration) を実施する

### Definition of Done (DoD)

- [ ] ショートカット定義のスキーマが永続化可能な形で実装されており、デフォルトの設定群が固定されていること
- [ ] 既存設定からの移行方針 (migration policy) が明確に文書化されていること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. Runtime Shortcut Dispatcher (ランタイムにおけるショートカットディスパッチャの実装)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 2.1 実行時のショートカットディスパッチャ (runtime shortcut dispatcher) を、コマンドインベントリ駆動型 (inventory-driven) へ置き換える
- [ ] 2.2 OS ごとの修飾キーの違い (platform-aware modifier handling) を吸収できるように整理する
- [ ] 2.3 アプリケーション内でのショートカット重複 (duplicate binding) を検出する機能を実装する

### Definition of Done (DoD)

- [ ] ユーザーが独自に設定したショートカット設定 (custom binding) が、実行時に正しく該当の処理へディスパッチされること
- [ ] 重複したショートカット割り当てがあっても、実行時に競合・誤作動 (runtime ambiguity) を起こさないこと
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. Settings UI and Conflict Popup (設定UIと重複・競合時の警告ポップアップ)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 3.1 ショートカットをカスタマイズするための設定画面 (settings UI) を追加する
- [ ] 3.2 割当時に他の機能と競合・重複した際、現在の割当先を警告表示するポップアップを追加する
- [ ] 3.3 ショートカットを「デフォルト設定へ戻す機能 (restore defaults)」を実装する
- [ ] 3.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] ショートカットの確認、変更、およびデフォルトへの復元 (restore defaults) が UI の設定画面上から完結して行えること
- [ ] 競合警告のポップアップが、ユーザーにとってわかりやすい文字列 (user-facing label) で現在の割当先を表示すること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Final Verification & Release Work (最終確認とリリース対応)

- [ ] 4.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [ ] 4.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 4.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [ ] 4.4 `master` に向けて PR（プルリクエスト）を作成する
- [ ] 4.5 `master` へマージする (※ `--admin` の利用は許容される)
- [ ] 4.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.21.0` のリリースタグ打ちとリリース作成を実行する
- [ ] 4.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
