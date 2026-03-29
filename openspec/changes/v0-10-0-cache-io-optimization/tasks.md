## 1. Core Platform (Cache I/O)

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 1.1 現行 `DefaultCacheService` の read/write workload を計測し、問題が「JSON の遅さ」ではなく「全量再シリアライズ・全量書き込み・線形探索」にあることを benchmark / tracing で確認する
- [ ] 1.2 `workspace_tabs` / `diagram` などの persistent key namespace を整理し、内部ロジック用の key codec / filename codec を定義する
- [ ] 1.2.1 filename hash を address 用 codec と位置づけ、各 file に `storage_version` と canonical key metadata を含める entry envelope を定義する
- [ ] 1.3 `DefaultCacheService` を改修し、runtime の正規 persistent backend を `cache.json` から per-key file store へ切り替える
- [ ] 1.4 `get_persistent` において、namespace-aware key から安全なファイル名を導出し、必要に応じて `kv` 配下から key 単位で読み込むロジックを実装する
- [ ] 1.5 `set_persistent` において、指定キーのデータのみを `kv` 配下へ同期し、他キーの再シリアライズを行わない処理を実装する
- [ ] 1.6 初期化（`new`）時、旧 `cache.json` が存在する場合はその中身を KVS へ one-shot migration し、完了後は runtime backend として併用しない構成にする
- [ ] 1.6.1 migration の保証対象を `workspace_tabs` などのユーザー状態 namespace に明示し、旧 `cache.json` はその移送成功前に破棄しない
- [ ] 1.6.2 `diagram` cache は再生成可能データとして扱い、migration 失敗時の fallback を定義する
- [ ] 1.7 `PersistentData.entries` / `Vec<(String, String)>` 依存を見直し、オンメモリ探索構造も key 設計に合う形へ整理する

## 2. Testing & Verification

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `cache/default.rs` と関連テストを per-key file store / key codec 構成に合わせて修正し、migration・lookup・write path を網羅する
- [ ] 2.2 `workspace_tabs` の保存復元、diagram cache の保存読込、invalid key / missing file path をそれぞれ namespace 単位で検証する
- [ ] 2.2.1 restart 後に同じ logical key から同じ filename を再計算して復元できること、および file content の metadata から key を検証できることを確認する
- [ ] 2.3 既存のUI操作（タブ切替、起動時の状態復元、画像キャッシュ復元等）による連携テストを実行し、デグレがないかを検証する
- [ ] 2.4 キャッシュ全体クリア（`clear_all_directories_in`）が新しい `kv` ディレクトリ構造でも正常に作動し、孤立したファイルが一切残らないことを確認する
- [ ] 2.5 benchmark / tracing の結果を記録し、改善の中心が JSON 置換ではなく storage unit と key 設計であることを確認する
- [ ] 2.6 upgrade compatibility と downgrade 非保証の境界が design / spec / release-facing note のいずれかで明文化されていることを確認する
